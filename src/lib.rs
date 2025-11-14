//! CLI Testing Specialist - Comprehensive testing framework for CLI tools
//!
//! This library provides automated analysis, test generation, and security
//! validation for command-line interface (CLI) tools.
//!
//! ## Features
//!
//! - Automatic CLI analysis and option detection
//! - Comprehensive BATS test suite generation
//! - Security vulnerability testing
//! - Multi-format report generation (Markdown, JSON, HTML, JUnit)
//! - Parallel processing for large CLI tools
//!
//! ## Usage
//!
//! ```no_run
//! use cli_testing_specialist::cli::Cli;
//! use clap::Parser;
//!
//! let cli = Cli::parse();
//! // Process commands...
//! ```

// Public modules
pub mod analyzer;
pub mod cli;
pub mod config;
pub mod error;
pub mod generator;
pub mod reporter;
pub mod runner;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use error::{CliTestError, Result};
pub use types::{
    AnalysisMetadata, Assertion, CliAnalysis, CliOption, OptionType, Subcommand, TestCase,
    TestCategory, TestReport, TestResult, TestStatus, TestSuite,
};
