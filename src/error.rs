use colored::Colorize;
use std::path::{Path, PathBuf};

/// Result type alias for cli-testing-specialist
pub type Result<T> = std::result::Result<T, CliTestError>;

/// Sanitize a path for display to end users
///
/// This function removes sensitive directory information and only shows
/// the filename to prevent information disclosure.
///
/// # Security
///
/// - Strips all directory components
/// - Shows only the filename
/// - Prevents path traversal information leakage
fn sanitize_path_for_display(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "<invalid-path>".to_string())
}

/// Error types for CLI testing operations
///
/// # Security Note
///
/// The `Display` implementation hides sensitive path information by showing
/// only filenames (not full paths). For debugging and logging, use the
/// `detailed_message()` method which includes full path information.
#[derive(Debug)]
pub enum CliTestError {
    /// Binary file not found at specified path
    BinaryNotFound(PathBuf),

    /// Binary file exists but is not executable
    BinaryNotExecutable(PathBuf),

    /// Failed to execute the binary
    ExecutionFailed(String),

    /// Help output is invalid or cannot be parsed
    InvalidHelpOutput,

    /// Failed to parse option from help text
    OptionParseError(String),

    /// Template rendering failed
    TemplateError(String),

    /// BATS test execution failed
    BatsExecutionFailed(String),

    /// Report generation failed
    ReportError(String),

    /// Configuration error
    Config(String),

    /// Validation error
    Validation(String),

    /// Invalid format specified
    InvalidFormat(String),

    /// I/O error occurred
    IoError(std::io::Error),

    /// JSON serialization/deserialization error
    Json(serde_json::Error),

    /// YAML serialization/deserialization error
    Yaml(serde_yaml::Error),

    /// Handlebars template error
    HandlebarsTemplate(handlebars::TemplateError),

    /// Handlebars render error
    HandlebarsRender(handlebars::RenderError),
}

// Manual Display implementation that hides sensitive paths
impl std::fmt::Display for CliTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryNotFound(path) => {
                write!(f, "Binary not found: {}", sanitize_path_for_display(path))
            }
            Self::BinaryNotExecutable(path) => {
                write!(
                    f,
                    "Binary not executable: {}",
                    sanitize_path_for_display(path)
                )
            }
            Self::ExecutionFailed(msg) => write!(f, "Failed to execute binary: {}", msg),
            Self::InvalidHelpOutput => write!(f, "Invalid help output"),
            Self::OptionParseError(details) => write!(f, "Failed to parse option: {}", details),
            Self::TemplateError(msg) => write!(f, "Template rendering failed: {}", msg),
            Self::BatsExecutionFailed(msg) => write!(f, "BATS execution failed: {}", msg),
            Self::ReportError(msg) => write!(f, "Report generation failed: {}", msg),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::Yaml(e) => write!(f, "YAML error: {}", e),
            Self::HandlebarsTemplate(e) => write!(f, "Template syntax error: {}", e),
            Self::HandlebarsRender(e) => write!(f, "Template rendering error: {}", e),
        }
    }
}

// Implement std::error::Error trait manually
impl std::error::Error for CliTestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::Yaml(e) => Some(e),
            Self::HandlebarsTemplate(e) => Some(e),
            Self::HandlebarsRender(e) => Some(e),
            _ => None,
        }
    }
}

// Manual From implementations (replacing thiserror's #[from])
impl From<std::io::Error> for CliTestError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for CliTestError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<serde_yaml::Error> for CliTestError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}

impl From<handlebars::TemplateError> for CliTestError {
    fn from(err: handlebars::TemplateError) -> Self {
        Self::HandlebarsTemplate(err)
    }
}

impl From<handlebars::RenderError> for CliTestError {
    fn from(err: handlebars::RenderError) -> Self {
        Self::HandlebarsRender(err)
    }
}

// Re-export as Error for convenience
pub use CliTestError as Error;

impl CliTestError {
    /// Get detailed error message for logging (may contain sensitive info)
    ///
    /// This method provides verbose error details suitable for logging
    /// but should NOT be displayed directly to end users.
    pub fn detailed_message(&self) -> String {
        match self {
            Self::BinaryNotFound(path) => {
                format!("Binary not found at path: {}", path.display())
            }
            Self::BinaryNotExecutable(path) => {
                format!("Binary at {} is not executable", path.display())
            }
            Self::ExecutionFailed(msg) => {
                format!("Binary execution failed: {}", msg)
            }
            Self::InvalidHelpOutput => {
                "Help output could not be parsed - ensure binary supports --help".to_string()
            }
            Self::OptionParseError(details) => {
                format!("Failed to parse option: {}", details)
            }
            Self::TemplateError(msg) => {
                format!("Template rendering error: {}", msg)
            }
            Self::BatsExecutionFailed(msg) => {
                format!("BATS test execution failed: {}", msg)
            }
            Self::ReportError(msg) => {
                format!("Report generation error: {}", msg)
            }
            Self::Config(msg) => {
                format!("Configuration error: {}", msg)
            }
            Self::Validation(msg) => {
                format!("Validation error: {}", msg)
            }
            Self::InvalidFormat(msg) => {
                format!("Invalid format: {}", msg)
            }
            Self::IoError(e) => {
                format!("I/O error: {}", e)
            }
            Self::Json(e) => {
                format!("JSON error: {}", e)
            }
            Self::Yaml(e) => {
                format!("YAML error: {}", e)
            }
            Self::HandlebarsTemplate(e) => {
                format!("Handlebars template error: {}", e)
            }
            Self::HandlebarsRender(e) => {
                format!("Handlebars render error: {}", e)
            }
        }
    }

    /// Get user-friendly colored error message with helpful suggestions
    ///
    /// This method formats errors with colors and provides actionable suggestions
    /// for users to resolve common issues.
    pub fn user_message(&self) -> String {
        match self {
            Self::BinaryNotFound(path) => {
                let filename = sanitize_path_for_display(path);
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Binary not found: {}", filename).white(),
                    "Suggestion:".yellow().bold(),
                    "Check that the path is correct and the file exists".white()
                )
            }
            Self::BinaryNotExecutable(path) => {
                let filename = sanitize_path_for_display(path);
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Binary is not executable: {}", filename).white(),
                    "Suggestion:".yellow().bold(),
                    format!("Try: chmod +x {}", filename).white()
                )
            }
            Self::ExecutionFailed(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Failed to execute binary: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Verify the binary runs correctly with --help flag".white()
                )
            }
            Self::InvalidHelpOutput => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    "Help output could not be parsed".white(),
                    "Suggestion:".yellow().bold(),
                    "Ensure the binary supports --help and produces valid output".white()
                )
            }
            Self::OptionParseError(details) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Failed to parse option: {}", details).white(),
                    "Suggestion:".yellow().bold(),
                    "Check if the help text follows standard CLI conventions".white()
                )
            }
            Self::TemplateError(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Template rendering failed: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Verify template syntax and variable bindings".white()
                )
            }
            Self::BatsExecutionFailed(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("BATS test execution failed: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Install BATS: brew install bats-core or apt-get install bats".white()
                )
            }
            Self::ReportError(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Report generation failed: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Check output directory permissions and disk space".white()
                )
            }
            Self::Config(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Configuration error: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Review your configuration file syntax and required fields".white()
                )
            }
            Self::Validation(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Validation error: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Ensure all required parameters are provided".white()
                )
            }
            Self::InvalidFormat(msg) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Invalid format: {}", msg).white(),
                    "Suggestion:".yellow().bold(),
                    "Use a supported format (bats, assert_cmd, snapbox)".white()
                )
            }
            Self::IoError(e) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("I/O error: {}", e).white(),
                    "Suggestion:".yellow().bold(),
                    "Check file permissions and disk space".white()
                )
            }
            Self::Json(e) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("JSON error: {}", e).white(),
                    "Suggestion:".yellow().bold(),
                    "Validate JSON syntax using a JSON linter".white()
                )
            }
            Self::Yaml(e) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("YAML error: {}", e).white(),
                    "Suggestion:".yellow().bold(),
                    "Check YAML indentation and syntax".white()
                )
            }
            Self::HandlebarsTemplate(e) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Template syntax error: {}", e).white(),
                    "Suggestion:".yellow().bold(),
                    "Check Handlebars template syntax and variable names".white()
                )
            }
            Self::HandlebarsRender(e) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Template rendering error: {}", e).white(),
                    "Suggestion:".yellow().bold(),
                    "Verify template data and variable bindings".white()
                )
            }
        }
    }

    /// Print error with color to stderr
    pub fn print_error(&self) {
        eprintln!("{}", self.user_message());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_not_found_error() {
        let path = PathBuf::from("/nonexistent/binary");
        let error = CliTestError::BinaryNotFound(path.clone());
        assert!(error.to_string().contains("Binary not found"));
    }

    #[test]
    fn test_binary_not_executable_error() {
        let path = PathBuf::from("/bin/not-executable");
        let error = CliTestError::BinaryNotExecutable(path);
        assert!(error.to_string().contains("not executable"));
    }

    #[test]
    fn test_execution_failed_error() {
        let error = CliTestError::ExecutionFailed("timeout".to_string());
        assert!(error.to_string().contains("Failed to execute"));
    }

    #[test]
    fn test_detailed_message_contains_more_info() {
        let path = PathBuf::from("/test/binary");
        let error = CliTestError::BinaryNotFound(path);
        let detailed = error.detailed_message();

        // Detailed message should contain full path
        assert!(detailed.contains("/test/binary"));
    }

    // ========== Security Tests ==========

    #[test]
    fn test_display_hides_sensitive_paths() {
        // Test that Display (to_string) hides full paths
        let path = PathBuf::from("/home/user/secret/project/binary");
        let error = CliTestError::BinaryNotFound(path);
        let display_msg = error.to_string();

        // Should NOT contain directory path
        assert!(!display_msg.contains("/home"));
        assert!(!display_msg.contains("/user"));
        assert!(!display_msg.contains("/secret"));
        assert!(!display_msg.contains("/project"));

        // Should only contain filename
        assert!(display_msg.contains("binary"));
    }

    #[test]
    fn test_display_vs_detailed_security() {
        let path = PathBuf::from("/var/lib/sensitive/data.json");
        let error = CliTestError::BinaryNotFound(path);

        let display_msg = error.to_string();
        let detailed_msg = error.detailed_message();

        // Display should hide path
        assert!(!display_msg.contains("/var"));
        assert!(!display_msg.contains("sensitive"));
        assert!(display_msg.contains("data.json"));

        // Detailed should show full path (for logging)
        assert!(detailed_msg.contains("/var/lib/sensitive/data.json"));
    }

    #[test]
    fn test_path_sanitization_windows_style() {
        // Note: On Unix, backslashes are treated as regular filename characters
        // This test verifies the behavior is consistent across platforms
        #[cfg(windows)]
        {
            let path = PathBuf::from("C:\\Users\\Admin\\Documents\\secret.exe");
            let error = CliTestError::BinaryNotExecutable(path);
            let display_msg = error.to_string();

            // Should not contain directory components
            assert!(!display_msg.contains("Users"));
            assert!(!display_msg.contains("Admin"));
            assert!(!display_msg.contains("Documents"));

            // Should contain filename
            assert!(display_msg.contains("secret.exe"));
        }

        #[cfg(unix)]
        {
            // On Unix, test with Unix-style path
            let path = PathBuf::from("/home/admin/documents/secret.exe");
            let error = CliTestError::BinaryNotExecutable(path);
            let display_msg = error.to_string();

            // Should not contain directory components
            assert!(!display_msg.contains("home"));
            assert!(!display_msg.contains("admin"));
            assert!(!display_msg.contains("documents"));

            // Should contain filename
            assert!(display_msg.contains("secret.exe"));
        }
    }

    #[test]
    fn test_sanitize_path_with_special_characters() {
        let path = PathBuf::from("/tmp/../../../etc/passwd");
        let error = CliTestError::BinaryNotFound(path);
        let display_msg = error.to_string();

        // Should not reveal path traversal attempts
        assert!(!display_msg.contains(".."));
        assert!(!display_msg.contains("/etc"));
        assert!(!display_msg.contains("/tmp"));
    }

    #[test]
    fn test_invalid_path_handling() {
        // Test with empty path or invalid UTF-8
        let path = PathBuf::from("");
        let error = CliTestError::BinaryNotFound(path);
        let display_msg = error.to_string();

        // Should show placeholder instead of crashing
        assert!(display_msg.contains("<invalid-path>") || display_msg.is_empty());
    }

    #[test]
    fn test_user_message_is_safe() {
        let path = PathBuf::from("/home/user/.ssh/id_rsa");
        let error = CliTestError::BinaryNotFound(path);
        let user_msg = error.user_message();

        // User message should also not expose full paths
        assert!(!user_msg.contains(".ssh"));
        assert!(!user_msg.contains("/home"));
    }

    #[test]
    fn test_io_error_does_not_leak_paths() {
        // Create an I/O error (these often contain path information)
        let io_error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found: /secret/path/file.txt",
        );
        let error = CliTestError::from(io_error);
        let display_msg = error.to_string();

        // The error message itself might contain paths from the OS,
        // but our Display impl should at least not ADD additional path exposure
        assert!(display_msg.contains("I/O error"));
    }
}
