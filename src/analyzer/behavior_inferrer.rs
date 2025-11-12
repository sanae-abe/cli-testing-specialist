use crate::error::Result;
use crate::types::{CliAnalysis, NoArgsBehavior};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

lazy_static! {
    /// Regex pattern for Usage line
    static ref USAGE_LINE: Regex = Regex::new(r"(?i)^\s*usage:\s+(.+)$").unwrap();

    /// Known interactive tools (REPLs, database clients)
    static ref INTERACTIVE_TOOLS: Vec<&'static str> = vec![
        // Database clients
        "psql", "mysql", "redis-cli", "mongo", "mongosh", "sqlite3",
        // Programming language REPLs
        "python", "python3", "node", "irb", "php", "R", "julia",
        // Other interactive tools
        "gdb", "lldb", "ghci", "erl", "iex",
    ];
}

/// Behavior Inferrer - Infers CLI behavior patterns
pub struct BehaviorInferrer;

impl BehaviorInferrer {
    /// Create a new behavior inferrer
    pub fn new() -> Self {
        Self
    }

    /// Infer CLI behavior when invoked without arguments
    ///
    /// Uses multiple strategies in order of preference:
    /// 0. Check for known interactive tools (highest priority - must avoid execution)
    /// 1. Execute binary and measure exit code (most accurate for non-interactive tools)
    /// 2. Parse Usage line pattern for subcommand requirements
    /// 3. Check for subcommands presence
    /// 4. Default to ShowHelp (safest assumption)
    pub fn infer_no_args_behavior(&self, analysis: &CliAnalysis) -> NoArgsBehavior {
        // Strategy 0: Check for interactive tools FIRST (must not execute)
        // Interactive tools (psql, python) may exit immediately with stdin=null
        // which would give false ShowHelp result
        if self.is_interactive_tool(&analysis.binary_name) {
            log::info!(
                "Inferred no-args behavior: Interactive (known REPL: {})",
                analysis.binary_name
            );
            return NoArgsBehavior::Interactive;
        }

        // Strategy 1: Execute and measure exit code (most accurate)
        // This directly observes the actual behavior instead of guessing from Usage line
        if let Ok(Some(exit_code)) = self.execute_and_measure(&analysis.binary_path) {
            let behavior = match exit_code {
                0 => NoArgsBehavior::ShowHelp,
                1 | 2 => NoArgsBehavior::RequireSubcommand,
                _ => NoArgsBehavior::ShowHelp, // Unknown code, assume safe default
            };
            log::info!(
                "Inferred no-args behavior: {:?} (from execution: exit {})",
                behavior,
                exit_code
            );
            return behavior;
        }

        // Strategy 2: Parse Usage line pattern (fallback)
        if let Some(pattern) = self.extract_usage_pattern(&analysis.help_output) {
            log::debug!("Extracted usage pattern: {}", pattern);

            // Check for subcommand requirement patterns
            if self.requires_subcommand_from_usage(&pattern) {
                log::info!(
                    "Inferred no-args behavior: RequireSubcommand (from Usage pattern)"
                );
                return NoArgsBehavior::RequireSubcommand;
            }

            // Check for optional-only pattern (indicates ShowHelp)
            if self.is_optional_only_from_usage(&pattern) {
                log::info!("Inferred no-args behavior: ShowHelp (from Usage pattern)");
                return NoArgsBehavior::ShowHelp;
            }
        }

        // Strategy 3: Check if has subcommands (fallback)
        if !analysis.subcommands.is_empty() {
            log::info!(
                "Inferred no-args behavior: RequireSubcommand (has {} subcommands)",
                analysis.subcommands.len()
            );
            return NoArgsBehavior::RequireSubcommand;
        }

        // Default: Show help (safest assumption)
        log::info!("Inferred no-args behavior: ShowHelp (default)");
        NoArgsBehavior::ShowHelp
    }

    /// Execute binary without arguments and measure exit code
    ///
    /// Safety measures:
    /// - 1 second timeout (prevents hanging on interactive tools)
    /// - Discard all output (stdout/stderr) to avoid log pollution
    /// - No user interaction (stdin=null, non-TTY mode)
    /// - Environment variables to disable colors and interactivity
    ///
    /// Returns:
    /// - Ok(Some(exit_code)) - Successfully executed and got exit code
    /// - Ok(None) - Timeout (likely interactive tool)
    /// - Err(_) - Execution failed (permission denied, not found, etc.)
    fn execute_and_measure(&self, binary_path: &Path) -> Result<Option<i32>> {
        log::debug!(
            "Executing binary to measure no-args behavior: {:?}",
            binary_path
        );

        let mut child = Command::new(binary_path)
            .stdin(Stdio::null()) // No user input
            .stdout(Stdio::null()) // Discard stdout
            .stderr(Stdio::null()) // Discard stderr
            .env("NO_COLOR", "1") // Disable colors
            .env("TERM", "dumb") // Non-interactive terminal
            .spawn()?;

        // Wait with timeout (1 second)
        use wait_timeout::ChildExt;
        match child.wait_timeout(Duration::from_secs(1))? {
            Some(status) => {
                let exit_code = status.code().unwrap_or(0);
                log::debug!("Binary exited with code: {}", exit_code);
                Ok(Some(exit_code))
            }
            None => {
                // Timeout - likely an interactive tool
                log::debug!("Binary execution timed out (likely interactive tool)");
                let _ = child.kill();
                let _ = child.wait();
                Ok(None)
            }
        }
    }

    /// Extract Usage line from help output
    fn extract_usage_pattern(&self, help_output: &str) -> Option<String> {
        for line in help_output.lines() {
            if let Some(cap) = USAGE_LINE.captures(line.trim()) {
                return Some(cap[1].to_string());
            }
        }
        None
    }

    /// Check if Usage pattern indicates subcommand requirement
    ///
    /// Patterns that indicate RequireSubcommand:
    /// - "Usage: cmd <SUBCOMMAND>"
    /// - "Usage: cmd <COMMAND>"
    /// - "Usage: cmd COMMAND"
    fn requires_subcommand_from_usage(&self, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();

        // Check for <SUBCOMMAND> or <COMMAND> pattern
        if pattern_lower.contains("<subcommand>") || pattern_lower.contains("<command>") {
            return true;
        }

        // Check for unbracketed COMMAND/SUBCOMMAND (e.g., "git COMMAND")
        if pattern_lower.contains(" command") || pattern_lower.contains(" subcommand") {
            // Make sure it's not in brackets (which would be optional)
            if !pattern.contains("[command]") && !pattern.contains("[subcommand]") {
                return true;
            }
        }

        false
    }

    /// Check if Usage pattern indicates optional-only (ShowHelp)
    ///
    /// Patterns that indicate ShowHelp:
    /// - "Usage: cmd [OPTIONS]"
    /// - "Usage: cmd [options]"
    /// - Everything in brackets
    fn is_optional_only_from_usage(&self, pattern: &str) -> bool {
        // Remove the binary name from pattern
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        if parts.len() <= 1 {
            return true; // No arguments at all
        }

        // Check if all arguments are optional (in brackets)
        let args = &parts[1..].join(" ");

        // Simple heuristic: if it starts with '[', it's likely optional-only
        args.trim_start().starts_with('[')
    }

    /// Check if tool is known to be interactive
    fn is_interactive_tool(&self, binary_name: &str) -> bool {
        INTERACTIVE_TOOLS
            .iter()
            .any(|&name| binary_name.contains(name))
    }
}

impl Default for BehaviorInferrer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Subcommand;
    use std::path::PathBuf;

    fn create_mock_analysis(
        binary_name: &str,
        help_output: &str,
        subcommands: Vec<&str>,
    ) -> CliAnalysis {
        let mut analysis = CliAnalysis::new(
            PathBuf::from(format!("/usr/bin/{}", binary_name)),
            binary_name.to_string(),
            help_output.to_string(),
        );

        analysis.subcommands = subcommands
            .iter()
            .map(|name| Subcommand {
                name: name.to_string(),
                description: None,
                options: vec![],
                required_args: vec![],
                subcommands: vec![],
                depth: 0,
            })
            .collect();

        analysis
    }

    #[test]
    fn test_infer_require_subcommand_from_usage() {
        let inferrer = BehaviorInferrer::new();

        // Test with <SUBCOMMAND> pattern
        let help_output = "Usage: git <SUBCOMMAND>\n\nAvailable commands:\n  clone\n  pull";
        let analysis = create_mock_analysis("git", help_output, vec!["clone", "pull"]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::RequireSubcommand);
    }

    #[test]
    fn test_infer_require_subcommand_from_command_pattern() {
        let inferrer = BehaviorInferrer::new();

        // Test with COMMAND pattern (no brackets)
        let help_output = "Usage: docker COMMAND\n\nCommands:\n  run\n  build";
        let analysis = create_mock_analysis("docker", help_output, vec!["run", "build"]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::RequireSubcommand);
    }

    #[test]
    fn test_infer_show_help_from_usage() {
        let inferrer = BehaviorInferrer::new();

        // Test with [OPTIONS] pattern
        let help_output = "Usage: backup-suite [OPTIONS]\n\nOptions:\n  --help";
        let analysis = create_mock_analysis("backup-suite", help_output, vec![]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::ShowHelp);
    }

    #[test]
    fn test_infer_require_subcommand_from_subcommands() {
        let inferrer = BehaviorInferrer::new();

        // Test with no clear Usage pattern but has subcommands
        let help_output = "A CLI tool\n\nCommands:\n  start\n  stop";
        let analysis = create_mock_analysis("service", help_output, vec!["start", "stop"]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::RequireSubcommand);
    }

    #[test]
    fn test_infer_interactive_psql() {
        let inferrer = BehaviorInferrer::new();

        let help_output = "Usage: psql [OPTIONS]\n\nOptions:\n  --help";
        let analysis = create_mock_analysis("psql", help_output, vec![]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::Interactive);
    }

    #[test]
    fn test_infer_interactive_python() {
        let inferrer = BehaviorInferrer::new();

        let help_output = "Usage: python [OPTIONS]\n\nOptions:\n  -h";
        let analysis = create_mock_analysis("python3", help_output, vec![]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::Interactive);
    }

    #[test]
    fn test_default_to_show_help() {
        let inferrer = BehaviorInferrer::new();

        // No clear pattern → default to ShowHelp
        let help_output = "A simple tool\n\nOptions:\n  --verbose";
        let analysis = create_mock_analysis("unknown-tool", help_output, vec![]);

        let behavior = inferrer.infer_no_args_behavior(&analysis);
        assert_eq!(behavior, NoArgsBehavior::ShowHelp);
    }

    #[test]
    fn test_extract_usage_pattern() {
        let inferrer = BehaviorInferrer::new();

        let help = "Usage: git <SUBCOMMAND>\n\nOptions:";
        let pattern = inferrer.extract_usage_pattern(help);
        assert_eq!(pattern, Some("git <SUBCOMMAND>".to_string()));

        let help2 = "usage: backup-suite [OPTIONS]";
        let pattern2 = inferrer.extract_usage_pattern(help2);
        assert_eq!(pattern2, Some("backup-suite [OPTIONS]".to_string()));
    }

    #[test]
    fn test_requires_subcommand_from_usage() {
        let inferrer = BehaviorInferrer::new();

        assert!(inferrer.requires_subcommand_from_usage("git <SUBCOMMAND>"));
        assert!(inferrer.requires_subcommand_from_usage("docker <COMMAND>"));
        assert!(inferrer.requires_subcommand_from_usage("cli COMMAND"));
        assert!(!inferrer.requires_subcommand_from_usage("cli [OPTIONS]"));
        assert!(!inferrer.requires_subcommand_from_usage("cli [command]"));
    }

    #[test]
    fn test_is_optional_only_from_usage() {
        let inferrer = BehaviorInferrer::new();

        assert!(inferrer.is_optional_only_from_usage("backup-suite [OPTIONS]"));
        assert!(inferrer.is_optional_only_from_usage("tool [options] [file]"));
        assert!(!inferrer.is_optional_only_from_usage("tool <FILE> [OPTIONS]"));
        assert!(!inferrer.is_optional_only_from_usage("tool COMMAND"));
    }

    #[test]
    fn test_is_interactive_tool() {
        let inferrer = BehaviorInferrer::new();

        assert!(inferrer.is_interactive_tool("psql"));
        assert!(inferrer.is_interactive_tool("python3"));
        assert!(inferrer.is_interactive_tool("node"));
        assert!(inferrer.is_interactive_tool("mysql"));
        assert!(!inferrer.is_interactive_tool("git"));
        assert!(!inferrer.is_interactive_tool("backup-suite"));
    }
}
