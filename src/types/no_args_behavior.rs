use serde::{Deserialize, Serialize};

/// Expected behavior when CLI is invoked without arguments
///
/// This classification helps generate accurate basic-005 tests
/// by predicting how the CLI will behave when run with no arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoArgsBehavior {
    /// Show help and exit with code 0
    ///
    /// **Pattern**: CLIs that default to showing help when no args provided
    ///
    /// **Examples**:
    /// - backup-suite
    /// - cmdrun
    /// - ripgrep (rg)
    ///
    /// **Test expectation**:
    /// - Exit code: 0
    /// - Output contains: "Usage:"
    ShowHelp,

    /// Require subcommand - show error and exit with code 1 or 2
    ///
    /// **Pattern**: CLIs that require a subcommand to operate
    ///
    /// **Examples**:
    /// - git
    /// - cldev
    /// - docker
    /// - cargo
    ///
    /// **Test expectation**:
    /// - Exit code: 1 or 2 (any non-zero)
    /// - Output contains: "error" or "required"
    RequireSubcommand,

    /// Enter interactive mode and exit with code 0
    ///
    /// **Pattern**: REPLs and database clients
    ///
    /// **Examples**:
    /// - psql
    /// - mysql
    /// - python
    /// - node
    /// - irb
    ///
    /// **Test expectation**:
    /// - Exit code: 0 (after receiving empty input via pipe)
    /// - No specific output check (varies by tool)
    Interactive,
}

impl NoArgsBehavior {
    /// Get behavior name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ShowHelp => "show_help",
            Self::RequireSubcommand => "require_subcommand",
            Self::Interactive => "interactive",
        }
    }

    /// Get expected exit code (or range)
    pub fn expected_exit_code(&self) -> Option<i32> {
        match self {
            Self::ShowHelp => Some(0),
            Self::RequireSubcommand => None, // Any non-zero
            Self::Interactive => Some(0),
        }
    }

    /// Get expected output pattern (if any)
    pub fn expected_output_pattern(&self) -> Option<&'static str> {
        match self {
            Self::ShowHelp => Some("Usage:"),
            Self::RequireSubcommand => Some("error"),
            Self::Interactive => None,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ShowHelp => "Show Help",
            Self::RequireSubcommand => "Require Subcommand",
            Self::Interactive => "Interactive Mode",
        }
    }

    /// Get description for documentation
    pub fn description(&self) -> &'static str {
        match self {
            Self::ShowHelp => {
                "Displays help text and exits successfully when invoked without arguments"
            }
            Self::RequireSubcommand => {
                "Requires a subcommand and exits with error when invoked without arguments"
            }
            Self::Interactive => "Enters interactive mode (REPL) when invoked without arguments",
        }
    }
}

impl Default for NoArgsBehavior {
    /// Default to ShowHelp (safest assumption)
    fn default() -> Self {
        Self::ShowHelp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_as_str() {
        assert_eq!(NoArgsBehavior::ShowHelp.as_str(), "show_help");
        assert_eq!(
            NoArgsBehavior::RequireSubcommand.as_str(),
            "require_subcommand"
        );
        assert_eq!(NoArgsBehavior::Interactive.as_str(), "interactive");
    }

    #[test]
    fn test_expected_exit_code() {
        assert_eq!(NoArgsBehavior::ShowHelp.expected_exit_code(), Some(0));
        assert_eq!(NoArgsBehavior::RequireSubcommand.expected_exit_code(), None);
        assert_eq!(NoArgsBehavior::Interactive.expected_exit_code(), Some(0));
    }

    #[test]
    fn test_expected_output_pattern() {
        assert_eq!(
            NoArgsBehavior::ShowHelp.expected_output_pattern(),
            Some("Usage:")
        );
        assert_eq!(
            NoArgsBehavior::RequireSubcommand.expected_output_pattern(),
            Some("error")
        );
        assert_eq!(NoArgsBehavior::Interactive.expected_output_pattern(), None);
    }

    #[test]
    fn test_default() {
        let behavior: NoArgsBehavior = Default::default();
        assert_eq!(behavior, NoArgsBehavior::ShowHelp);
    }

    #[test]
    fn test_serialization() {
        let behavior = NoArgsBehavior::RequireSubcommand;
        let json = serde_json::to_string(&behavior).unwrap();
        assert_eq!(json, r#""require_subcommand""#);

        let deserialized: NoArgsBehavior = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, behavior);
    }
}
