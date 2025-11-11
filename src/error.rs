use colored::Colorize;
use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for cli-testing-specialist
pub type Result<T> = std::result::Result<T, CliTestError>;

/// Error types for CLI testing operations
#[derive(Error, Debug)]
pub enum CliTestError {
    /// Binary file not found at specified path
    #[error("Binary not found: {0}")]
    BinaryNotFound(PathBuf),

    /// Binary file exists but is not executable
    #[error("Binary not executable: {0}")]
    BinaryNotExecutable(PathBuf),

    /// Failed to execute the binary
    #[error("Failed to execute binary: {0}")]
    ExecutionFailed(String),

    /// Help output is invalid or cannot be parsed
    #[error("Invalid help output")]
    InvalidHelpOutput,

    /// Failed to parse option from help text
    #[error("Failed to parse option: {0}")]
    OptionParseError(String),

    /// Template rendering failed
    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    /// BATS test execution failed
    #[error("BATS execution failed: {0}")]
    BatsExecutionFailed(String),

    /// Report generation failed
    #[error("Report generation failed: {0}")]
    ReportError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O error occurred
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error
    #[error(transparent)]
    Yaml(#[from] serde_yml::Error),
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
            Self::IoError(e) => {
                format!("I/O error: {}", e)
            }
            Self::Json(e) => {
                format!("JSON error: {}", e)
            }
            Self::Yaml(e) => {
                format!("YAML error: {}", e)
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
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Binary not found: {}", path.display()).white(),
                    "Suggestion:".yellow().bold(),
                    "Check that the path is correct and the file exists".white()
                )
            }
            Self::BinaryNotExecutable(path) => {
                format!(
                    "{} {}\n{} {}",
                    "Error:".red().bold(),
                    format!("Binary is not executable: {}", path.display()).white(),
                    "Suggestion:".yellow().bold(),
                    format!("Try: chmod +x {}", path.display()).white()
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
}
