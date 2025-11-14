//! Tool-specific test configuration types
//!
//! This module defines the schema for `.cli-test-config.yml` files that allow
//! CLI tool authors to customize test behavior without modifying their tools.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure for `.cli-test-config.yml`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CliTestConfig {
    /// Schema version (currently "1.0")
    pub version: String,

    /// Tool name (for validation)
    pub tool_name: String,

    /// Tool version (optional)
    pub tool_version: Option<String>,

    /// Test category adjustments
    pub test_adjustments: TestAdjustments,

    /// Global test settings
    #[serde(default)]
    pub global: GlobalSettings,

    /// CI/CD specific settings
    #[serde(default)]
    pub ci: CiSettings,
}

/// Test category adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct TestAdjustments {
    /// Security test customization
    pub security: Option<SecurityAdjustments>,

    /// Directory traversal test customization
    pub directory_traversal: Option<DirectoryTraversalAdjustments>,

    /// Destructive operation test customization
    pub destructive_ops: Option<DestructiveOpsAdjustments>,

    /// Path handling test customization
    pub path: Option<PathAdjustments>,

    /// Multi-shell test customization
    pub multi_shell: Option<MultiShellAdjustments>,

    /// Performance test customization
    pub performance: Option<PerformanceAdjustments>,
}

/// Security test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct SecurityAdjustments {
    /// Options to skip security testing
    #[serde(default)]
    pub skip_options: Vec<SkipOption>,

    /// Custom security tests to add
    #[serde(default)]
    pub custom_tests: Vec<CustomSecurityTest>,
}

/// Option to skip in security tests
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SkipOption {
    /// Option name (e.g., "lang")
    pub name: String,

    /// Reason for skipping (required for documentation)
    pub reason: String,

    /// Optional category classification
    pub category: Option<String>,
}

/// Custom security test definition
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CustomSecurityTest {
    /// Test name
    pub name: String,

    /// Command to execute
    pub command: String,

    /// Expected exit code
    pub expected_exit_code: i32,

    /// Test description
    pub description: String,
}

/// Directory traversal test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct DirectoryTraversalAdjustments {
    /// RECOMMENDED: Declarative test directory definitions
    #[serde(default)]
    pub test_directories: Vec<TestDirectory>,

    /// ALTERNATIVE: Setup commands (requires --allow-setup flag)
    #[serde(default)]
    pub setup_commands: Vec<String>,

    /// Teardown commands
    #[serde(default)]
    pub teardown_commands: Vec<String>,

    /// Skip all directory traversal tests
    #[serde(default)]
    pub skip: bool,

    /// Skip specific tests by name
    #[serde(default)]
    pub skip_tests: Vec<String>,
}

/// Declarative test directory definition (SAFE: no command execution)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TestDirectory {
    /// Directory path
    pub path: String,

    /// Create directory if it doesn't exist
    #[serde(default)]
    pub create: bool,

    /// Number of files to create (optional)
    pub file_count: Option<usize>,

    /// Directory depth to create (optional)
    pub depth: Option<usize>,

    /// Clean up after tests
    #[serde(default = "default_true")]
    pub cleanup: bool,
}

/// Destructive operation test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct DestructiveOpsAdjustments {
    /// Environment variables for CI/CD
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Expected exit code when operation is cancelled
    #[serde(default = "default_exit_code_1")]
    pub cancel_exit_code: i32,

    /// Commands requiring special handling
    #[serde(default)]
    pub special_commands: Vec<SpecialCommand>,
}

/// Special command configuration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SpecialCommand {
    /// Command name
    pub command: String,

    /// Whether command requires TTY
    #[serde(default)]
    pub requires_tty: bool,

    /// Flag for auto-confirmation (e.g., "--yes")
    pub confirm_flag: Option<String>,
}

/// Path handling test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct PathAdjustments {
    /// Skip Unicode path tests
    #[serde(default)]
    pub skip_unicode: bool,

    /// Custom path separator
    pub path_separator: Option<String>,
}

/// Multi-shell test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct MultiShellAdjustments {
    /// Shells to test
    #[serde(default = "default_shells")]
    pub shells: Vec<String>,

    /// Bash-specific environment variables
    #[serde(default)]
    pub bash_env: HashMap<String, String>,

    /// Zsh-specific environment variables
    #[serde(default)]
    pub zsh_env: HashMap<String, String>,
}

/// Performance test adjustments
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct PerformanceAdjustments {
    /// Maximum startup time in milliseconds
    pub max_startup_time: Option<u64>,

    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,

    /// Skip performance tests in CI
    #[serde(default)]
    pub skip_in_ci: bool,
}

/// Global test settings
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GlobalSettings {
    /// Timeout for all tests in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Retry count for failed tests
    #[serde(default)]
    pub retry_count: u32,

    /// Verbose output
    #[serde(default)]
    pub verbose: bool,

    /// Environment variables for all tests
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            retry_count: 0,
            verbose: false,
            env_vars: HashMap::new(),
        }
    }
}

/// CI/CD specific settings
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CiSettings {
    /// Auto-detect CI environment
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    /// Skip TTY-requiring tests in CI
    #[serde(default = "default_true")]
    pub skip_tty_tests: bool,

    /// Skip intensive tests in CI
    #[serde(default)]
    pub skip_intensive_tests: bool,
}

impl Default for CiSettings {
    fn default() -> Self {
        Self {
            auto_detect: true,
            skip_tty_tests: true,
            skip_intensive_tests: false,
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_exit_code_1() -> i32 {
    1
}

fn default_timeout() -> u64 {
    30
}

fn default_shells() -> Vec<String> {
    vec!["bash".to_string(), "zsh".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_config() {
        let yaml = r#"
version: "1.0"
tool_name: "test-tool"
test_adjustments: {}
"#;

        let config: CliTestConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.tool_name, "test-tool");
    }

    #[test]
    fn test_deserialize_full_config() {
        let yaml = r#"
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"
test_adjustments:
  security:
    skip_options:
      - name: "lang"
        reason: "Enum with safe fallback"
  directory_traversal:
    test_directories:
      - path: "/tmp/test-dir"
        create: true
        file_count: 100
        cleanup: true
  destructive_ops:
    env_vars:
      BACKUP_SUITE_YES: "true"
    cancel_exit_code: 2
global:
  timeout: 60
"#;

        let config: CliTestConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.global.timeout, 60);

        let security = config.test_adjustments.security.unwrap();
        assert_eq!(security.skip_options.len(), 1);
        assert_eq!(security.skip_options[0].name, "lang");

        let dir_traversal = config.test_adjustments.directory_traversal.unwrap();
        assert_eq!(dir_traversal.test_directories.len(), 1);
        assert_eq!(dir_traversal.test_directories[0].path, "/tmp/test-dir");

        let destructive = config.test_adjustments.destructive_ops.unwrap();
        assert_eq!(destructive.cancel_exit_code, 2);
        assert_eq!(
            destructive.env_vars.get("BACKUP_SUITE_YES"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_default_values() {
        let config = CliTestConfig {
            version: "1.0".to_string(),
            tool_name: "test".to_string(),
            tool_version: None,
            test_adjustments: TestAdjustments::default(),
            global: GlobalSettings::default(),
            ci: CiSettings::default(),
        };

        assert_eq!(config.global.timeout, 30);
        assert_eq!(config.global.retry_count, 0);
        assert!(!config.global.verbose);
        assert!(config.ci.auto_detect);
        assert!(config.ci.skip_tty_tests);
    }

    #[test]
    fn test_test_directory_defaults() {
        let yaml = r#"
path: "/tmp/test"
create: true
"#;

        let dir: TestDirectory = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(dir.path, "/tmp/test");
        assert!(dir.create);
        assert!(dir.cleanup); // default true
        assert!(dir.file_count.is_none());
        assert!(dir.depth.is_none());
    }
}
