use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI analysis result containing all discovered information about a binary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliAnalysis {
    /// Path to the analyzed binary
    pub binary_path: PathBuf,

    /// Binary name (extracted from path)
    pub binary_name: String,

    /// Version string (if detected from --version)
    pub version: Option<String>,

    /// Raw help output from --help command
    pub help_output: String,

    /// Detected subcommands (recursive)
    pub subcommands: Vec<Subcommand>,

    /// Global options available for all subcommands
    pub global_options: Vec<CliOption>,

    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Subcommand definition with recursive structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subcommand {
    /// Subcommand name
    pub name: String,

    /// Description text (if available)
    pub description: Option<String>,

    /// Options specific to this subcommand
    pub options: Vec<CliOption>,

    /// Nested subcommands (recursive structure)
    pub subcommands: Vec<Subcommand>,

    /// Recursion depth level (0 = top-level, max 3)
    pub depth: u8,
}

/// CLI option definition with type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliOption {
    /// Short flag (e.g., "-h")
    pub short: Option<String>,

    /// Long flag (e.g., "--help")
    pub long: Option<String>,

    /// Description text (if available)
    pub description: Option<String>,

    /// Inferred option type
    pub option_type: OptionType,

    /// Whether this option is required
    pub required: bool,

    /// Default value (if specified)
    pub default_value: Option<String>,
}

/// Option type with inferred constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    /// Boolean flag (no value, e.g., --verbose)
    Flag,

    /// String value (e.g., --name VALUE)
    String,

    /// Numeric value with optional min/max constraints
    Numeric { min: Option<i64>, max: Option<i64> },

    /// File or directory path
    Path,

    /// Enum value with allowed choices
    Enum { values: Vec<String> },
}

/// Analysis metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisMetadata {
    /// Timestamp when analysis was performed (ISO 8601)
    pub analyzed_at: String,

    /// Version of the analyzer tool
    pub analyzer_version: String,

    /// Total number of subcommands discovered (including nested)
    pub total_subcommands: usize,

    /// Total number of options discovered (including global)
    pub total_options: usize,

    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

impl CliAnalysis {
    /// Create a new CLI analysis result
    pub fn new(binary_path: PathBuf, binary_name: String, help_output: String) -> Self {
        Self {
            binary_path,
            binary_name,
            version: None,
            help_output,
            subcommands: Vec::new(),
            global_options: Vec::new(),
            metadata: AnalysisMetadata {
                analyzed_at: chrono::Utc::now().to_rfc3339(),
                analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
                total_subcommands: 0,
                total_options: 0,
                analysis_duration_ms: 0,
            },
        }
    }

    /// Update metadata statistics
    pub fn update_metadata(&mut self, duration_ms: u64) {
        self.metadata.total_subcommands = count_subcommands(&self.subcommands);
        self.metadata.total_options = self.global_options.len() + count_options(&self.subcommands);
        self.metadata.analysis_duration_ms = duration_ms;
    }
}

/// Count total subcommands recursively
fn count_subcommands(subcommands: &[Subcommand]) -> usize {
    subcommands.len()
        + subcommands
            .iter()
            .map(|s| count_subcommands(&s.subcommands))
            .sum::<usize>()
}

/// Count total options recursively
fn count_options(subcommands: &[Subcommand]) -> usize {
    subcommands
        .iter()
        .map(|s| s.options.len() + count_options(&s.subcommands))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_analysis_creation() {
        let analysis = CliAnalysis::new(
            PathBuf::from("/usr/bin/test"),
            "test".to_string(),
            "Help output".to_string(),
        );

        assert_eq!(analysis.binary_name, "test");
        assert_eq!(analysis.help_output, "Help output");
        assert!(analysis.subcommands.is_empty());
    }

    #[test]
    fn test_option_type_serialization() {
        let option = CliOption {
            short: Some("-t".to_string()),
            long: Some("--timeout".to_string()),
            description: Some("Timeout in seconds".to_string()),
            option_type: OptionType::Numeric {
                min: Some(0),
                max: Some(3600),
            },
            required: false,
            default_value: Some("30".to_string()),
        };

        let json = serde_json::to_string(&option).unwrap();
        let deserialized: CliOption = serde_json::from_str(&json).unwrap();

        assert_eq!(option, deserialized);
    }

    #[test]
    fn test_subcommand_recursion() {
        let nested = Subcommand {
            name: "nested".to_string(),
            description: None,
            options: vec![],
            subcommands: vec![],
            depth: 2,
        };

        let parent = Subcommand {
            name: "parent".to_string(),
            description: None,
            options: vec![],
            subcommands: vec![nested],
            depth: 1,
        };

        assert_eq!(parent.subcommands.len(), 1);
        assert_eq!(parent.subcommands[0].depth, 2);
    }

    #[test]
    fn test_count_subcommands_recursive() {
        let subcommands = vec![
            Subcommand {
                name: "cmd1".to_string(),
                description: None,
                options: vec![],
                subcommands: vec![Subcommand {
                    name: "subcmd1".to_string(),
                    description: None,
                    options: vec![],
                    subcommands: vec![],
                    depth: 1,
                }],
                depth: 0,
            },
            Subcommand {
                name: "cmd2".to_string(),
                description: None,
                options: vec![],
                subcommands: vec![],
                depth: 0,
            },
        ];

        assert_eq!(count_subcommands(&subcommands), 3); // 2 top-level + 1 nested
    }
}
