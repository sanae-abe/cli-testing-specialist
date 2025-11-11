use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

/// CLI Testing Specialist - Comprehensive testing framework for CLI tools
#[derive(Parser, Debug)]
#[command(
    name = "cli-test",
    version,
    about = "Comprehensive CLI testing framework",
    long_about = "Analyzes CLI tools, generates BATS test suites, and produces detailed security reports"
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze a CLI tool and extract its structure
    #[command(about = "Analyze CLI tool structure and options")]
    Analyze {
        /// Path to the CLI binary to analyze
        #[arg(value_name = "BINARY")]
        binary: PathBuf,

        /// Output JSON file path
        #[arg(short, long, default_value = "cli-analysis.json")]
        output: PathBuf,

        /// Maximum recursion depth for subcommands
        #[arg(short, long, default_value = "3")]
        depth: u8,

        /// Enable parallel processing
        #[arg(long)]
        parallel: bool,
    },

    /// Generate test cases from analysis results
    #[command(about = "Generate BATS test suites from analysis")]
    Generate {
        /// Analysis JSON file path
        #[arg(value_name = "ANALYSIS")]
        analysis: PathBuf,

        /// Output directory for test files
        #[arg(short, long, default_value = "test-output")]
        output: PathBuf,

        /// Test categories to generate (comma-separated or "all")
        #[arg(short, long, default_value = "all")]
        categories: String,

        /// Include resource-intensive tests (directory-traversal, large-scale performance)
        /// These tests may require significant /tmp space and memory
        #[arg(long)]
        include_intensive: bool,
    },

    /// Run BATS tests and generate reports
    #[command(about = "Execute BATS tests and generate reports")]
    Run {
        /// Test directory containing BATS files
        #[arg(value_name = "TEST_DIR")]
        test_dir: PathBuf,

        /// Report format to generate
        #[arg(short, long, default_value = "markdown")]
        format: ReportFormat,

        /// Output directory for reports
        #[arg(short, long, default_value = "reports")]
        output: PathBuf,

        /// Timeout per test suite in seconds (default: 300)
        #[arg(short = 't', long, default_value = "300")]
        timeout: u64,

        /// Skip specific test categories (comma-separated)
        #[arg(short = 's', long)]
        skip: Option<String>,
    },

    /// Validate analysis JSON file
    #[command(about = "Validate analysis JSON file structure")]
    Validate {
        /// Analysis JSON file to validate
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Generate shell completion scripts
    #[command(about = "Generate shell completion scripts")]
    Completion {
        /// Shell type to generate completion for
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
}

/// Report output format
#[derive(ValueEnum, Clone, Debug)]
pub enum ReportFormat {
    /// Markdown format
    Markdown,

    /// JSON format
    Json,

    /// HTML format
    Html,

    /// JUnit XML format
    Junit,

    /// All formats
    All,
}

impl ReportFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Html => "html",
            Self::Junit => "xml",
            Self::All => "all",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format_extension() {
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Html.extension(), "html");
        assert_eq!(ReportFormat::Junit.extension(), "xml");
    }
}
