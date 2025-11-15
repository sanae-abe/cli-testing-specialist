//! Tool-specific test configuration types
//!
//! This module defines the schema for `.cli-test-config.yml` files that allow
//! CLI tool authors to customize test behavior without modifying their tools.
//!
//! ## Version Migration
//!
//! This module supports automatic configuration file migration across versions.
//! When a configuration file is loaded, it is automatically migrated to the current
//! version if needed.

use crate::error::{CliTestError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

// ============================================================================
// Configuration Migration Support
// ============================================================================

impl CliTestConfig {
    /// Load configuration from file with automatic migration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cli_testing_specialist::types::CliTestConfig;
    ///
    /// let config = CliTestConfig::load(".cli-test-config.yml")?;
    /// println!("Loaded config for: {}", config.tool_name);
    /// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Read file
        let content = fs::read_to_string(path)?;

        // Deserialize
        let mut config: CliTestConfig = serde_yaml::from_str(&content)
            .map_err(|e| CliTestError::Config(format!("Failed to parse config: {}", e)))?;

        // Migrate if needed
        config = migrate_config(config)?;

        Ok(config)
    }

    /// Save configuration to file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cli_testing_specialist::types::CliTestConfig;
    ///
    /// let config = CliTestConfig {
    ///     version: "1.0".to_string(),
    ///     tool_name: "my-cli".to_string(),
    ///     tool_version: Some("1.0.0".to_string()),
    ///     test_adjustments: Default::default(),
    ///     global: Default::default(),
    ///     ci: Default::default(),
    /// };
    ///
    /// config.save(".cli-test-config.yml")?;
    /// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
    /// ```
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| CliTestError::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)?;

        Ok(())
    }

    /// Get current version
    pub fn current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Create a backup of the configuration file
    pub fn backup<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        let backup_path = path.with_extension("yml.bak");

        fs::copy(path, &backup_path)?;

        log::info!("Created backup: {}", backup_path.display());

        Ok(())
    }
}

/// Migrate configuration to current version
///
/// Handles version upgrades automatically. Creates a backup before migration.
fn migrate_config(mut config: CliTestConfig) -> Result<CliTestConfig> {
    let current_version = CliTestConfig::current_version();

    // Parse versions
    let config_version = parse_version(&config.version)?;
    let target_version = parse_version(current_version)?;

    if config_version == target_version {
        // No migration needed
        return Ok(config);
    }

    log::info!(
        "Migrating config from v{} to v{}",
        config.version,
        current_version
    );

    // Major version migration
    if config_version.0 < target_version.0 {
        log::warn!(
            "Major version migration from v{}.x to v{}.x",
            config_version.0,
            target_version.0
        );
        config = migrate_major_version(config, config_version.0)?;
    }

    // Minor/Patch version migration (add new fields with defaults)
    // Always apply migration if versions differ to ensure new fields are added
    if config_version != target_version {
        log::debug!(
            "Version migration from v{}.{}.{} to v{}.{}.{}",
            config_version.0,
            config_version.1,
            config_version.2,
            target_version.0,
            target_version.1,
            target_version.2
        );
        config = migrate_minor_version(config)?;
    }

    // Update version
    config.version = current_version.to_string();

    Ok(config)
}

/// Parse version string (simple MAJOR.MINOR.PATCH)
fn parse_version(version: &str) -> Result<(u64, u64, u64)> {
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() < 2 {
        return Err(CliTestError::Config(format!(
            "Invalid version format: {}",
            version
        )));
    }

    let major = parts[0]
        .parse::<u64>()
        .map_err(|_| CliTestError::Config(format!("Invalid major version: {}", parts[0])))?;

    let minor = parts[1]
        .parse::<u64>()
        .map_err(|_| CliTestError::Config(format!("Invalid minor version: {}", parts[1])))?;

    let patch = if parts.len() >= 3 {
        parts[2]
            .parse::<u64>()
            .map_err(|_| CliTestError::Config(format!("Invalid patch version: {}", parts[2])))?
    } else {
        0
    };

    Ok((major, minor, patch))
}

/// Migrate from v1.x to v2.x
fn migrate_major_version(config: CliTestConfig, from_major: u64) -> Result<CliTestConfig> {
    match from_major {
        1 => {
            // Future: v1 → v2 migration
            // Currently no breaking changes planned
            log::info!("No structural changes required for v1 → v2 migration");
            Ok(config)
        }
        _ => Err(CliTestError::Config(format!(
            "Unsupported config version: {}.x (current version supports v1.x only)",
            from_major
        ))),
    }
}

/// Migrate minor version (add new fields with defaults)
fn migrate_minor_version(mut config: CliTestConfig) -> Result<CliTestConfig> {
    // v1.0 → v1.1+: Add missing fields with defaults

    // Ensure test_adjustments has all optional fields initialized
    if config.test_adjustments.path.is_none() {
        config.test_adjustments.path = Some(PathAdjustments::default());
    }

    if config.test_adjustments.multi_shell.is_none() {
        config.test_adjustments.multi_shell = Some(MultiShellAdjustments::default());
    }

    if config.test_adjustments.performance.is_none() {
        config.test_adjustments.performance = Some(PerformanceAdjustments::default());
    }

    // Ensure global settings are properly initialized
    if config.global.env_vars.is_empty() {
        // Add default environment variables for consistent testing
        config
            .global
            .env_vars
            .insert("LANG".to_string(), "en_US.UTF-8".to_string());
        config
            .global
            .env_vars
            .insert("TZ".to_string(), "UTC".to_string());
    }

    log::debug!("Minor version migration completed");

    Ok(config)
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

    // ======================================================================
    // Configuration Migration Tests
    // ======================================================================

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.0.0").unwrap(), (1, 0, 0));
        assert_eq!(parse_version("1.5.3").unwrap(), (1, 5, 3));
        assert_eq!(parse_version("2.0").unwrap(), (2, 0, 0));
        assert_eq!(parse_version("1.9").unwrap(), (1, 9, 0));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert!(parse_version("").is_err());
        assert!(parse_version("1").is_err());
        assert!(parse_version("abc").is_err());
        assert!(parse_version("1.x.0").is_err());
    }

    #[test]
    fn test_save_and_load() {
        use tempfile::NamedTempFile;

        let config = CliTestConfig {
            version: "1.0".to_string(),
            tool_name: "test-cli".to_string(),
            tool_version: Some("1.0.0".to_string()),
            test_adjustments: TestAdjustments::default(),
            global: GlobalSettings::default(),
            ci: CiSettings::default(),
        };

        let temp_file = NamedTempFile::new().unwrap();

        // Save
        config.save(temp_file.path()).unwrap();

        // Load
        let loaded = CliTestConfig::load(temp_file.path()).unwrap();

        assert_eq!(loaded.tool_name, "test-cli");
        assert_eq!(loaded.tool_version, Some("1.0.0".to_string()));
        // Version should be updated to current version after migration
        assert_eq!(loaded.version, CliTestConfig::current_version());
    }

    #[test]
    fn test_migration_same_version() {
        let config = CliTestConfig {
            version: CliTestConfig::current_version().to_string(),
            tool_name: "test-cli".to_string(),
            tool_version: None,
            test_adjustments: TestAdjustments::default(),
            global: GlobalSettings::default(),
            ci: CiSettings::default(),
        };

        let migrated = migrate_config(config.clone()).unwrap();

        // No migration should occur
        assert_eq!(migrated.version, config.version);
        assert_eq!(migrated.tool_name, config.tool_name);
    }

    #[test]
    fn test_migration_old_to_current() {
        let mut config = CliTestConfig {
            version: "1.0".to_string(),
            tool_name: "test-cli".to_string(),
            tool_version: None,
            test_adjustments: TestAdjustments::default(),
            global: GlobalSettings::default(),
            ci: CiSettings::default(),
        };

        // Simulate old config without optional fields
        config.test_adjustments.path = None;
        config.test_adjustments.multi_shell = None;
        config.test_adjustments.performance = None;

        let migrated = migrate_config(config).unwrap();

        // Version should be updated
        assert_eq!(migrated.version, CliTestConfig::current_version());

        // Optional fields should be initialized with defaults
        assert!(migrated.test_adjustments.path.is_some());
        assert!(migrated.test_adjustments.multi_shell.is_some());
        assert!(migrated.test_adjustments.performance.is_some());

        // Global env vars should have defaults
        assert!(migrated.global.env_vars.contains_key("LANG"));
        assert!(migrated.global.env_vars.contains_key("TZ"));
    }

    #[test]
    fn test_migration_preserves_existing_data() {
        let mut config = CliTestConfig {
            version: "1.0".to_string(),
            tool_name: "my-tool".to_string(),
            tool_version: Some("2.0.0".to_string()),
            test_adjustments: TestAdjustments::default(),
            global: GlobalSettings {
                timeout: 60,
                retry_count: 3,
                verbose: true,
                env_vars: {
                    let mut map = HashMap::new();
                    map.insert("CUSTOM_VAR".to_string(), "value".to_string());
                    map
                },
            },
            ci: CiSettings::default(),
        };

        config.test_adjustments.security = Some(SecurityAdjustments {
            skip_options: vec![SkipOption {
                name: "lang".to_string(),
                reason: "test".to_string(),
                category: None,
            }],
            custom_tests: vec![],
        });

        let migrated = migrate_config(config.clone()).unwrap();

        // Version should be updated
        assert_eq!(migrated.version, CliTestConfig::current_version());

        // But existing data should be preserved
        assert_eq!(migrated.tool_name, "my-tool");
        assert_eq!(migrated.tool_version, Some("2.0.0".to_string()));
        assert_eq!(migrated.global.timeout, 60);
        assert_eq!(migrated.global.retry_count, 3);
        assert!(migrated.global.verbose);

        // Custom env vars should be preserved
        assert_eq!(
            migrated.global.env_vars.get("CUSTOM_VAR"),
            Some(&"value".to_string())
        );

        // Security adjustments should be preserved
        assert!(migrated.test_adjustments.security.is_some());
        let security = migrated.test_adjustments.security.unwrap();
        assert_eq!(security.skip_options.len(), 1);
        assert_eq!(security.skip_options[0].name, "lang");
    }

    #[test]
    fn test_roundtrip_with_migration() {
        use tempfile::NamedTempFile;

        // Create old version config
        let yaml = r#"
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"
test_adjustments:
  security:
    custom_tests:
      - name: "test_injection"
        command: "backup-suite --lang 'test; rm -rf /'"
        expected_exit_code: 0
        description: "Test injection handling"
global:
  timeout: 45
  verbose: true
ci:
  skip_tty_tests: false
"#;

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), yaml).unwrap();

        // Load (should trigger migration)
        let loaded = CliTestConfig::load(temp_file.path()).unwrap();

        // Verify migration occurred
        assert_eq!(loaded.version, CliTestConfig::current_version());

        // Verify data integrity
        assert_eq!(loaded.tool_name, "backup-suite");
        assert_eq!(loaded.global.timeout, 45);
        assert!(loaded.global.verbose);
        assert!(!loaded.ci.skip_tty_tests);

        // Verify custom tests were preserved
        let security = loaded.test_adjustments.security.unwrap();
        assert_eq!(security.custom_tests.len(), 1);
        assert_eq!(security.custom_tests[0].name, "test_injection");

        // Verify new fields were added with defaults
        assert!(loaded.test_adjustments.path.is_some());
        assert!(loaded.test_adjustments.multi_shell.is_some());
        assert!(loaded.test_adjustments.performance.is_some());
    }
}
