//! # Analyzer Module
//!
//! Provides comprehensive CLI tool analysis capabilities including:
//!
//! - **CLI Parsing**: Executes binaries with `--help` and extracts structured information
//! - **Option Inference**: Automatically detects option types (flags, paths, numbers, etc.)
//! - **Subcommand Detection**: Recursively discovers subcommands and their options
//!
//! ## Architecture
//!
//! The analyzer module follows a pipeline pattern:
//!
//! ```text
//! Binary Path → CliParser → Option Detection → Type Inference → CliAnalysis
//!                              ↓
//!                    SubcommandDetector (recursive)
//! ```
//!
//! ## Example Usage
//!
//! ```no_run
//! use cli_testing_specialist::analyzer::CliParser;
//! use std::path::Path;
//!
//! let parser = CliParser::new();
//! let analysis = parser.analyze(Path::new("/usr/bin/curl"))?;
//!
//! println!("Found {} options", analysis.metadata.total_options);
//! println!("Found {} subcommands", analysis.subcommands.len());
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Small CLI (~50 options)**: 100-200ms
//! - **Medium CLI (~100 options)**: 300-500ms
//! - **Large CLI (100+ subcommands)**: 1-3s
//!
//! ## Safety & Resource Limits
//!
//! All binary executions are protected with:
//! - Timeout limits (default: 30 seconds)
//! - Memory limits (configurable)
//! - Recursion depth limits for subcommands (max: 5 levels)

pub mod behavior_inferrer;
pub mod cli_parser;
pub mod option_inferrer;
pub mod subcommand_detector;

pub use behavior_inferrer::BehaviorInferrer;
pub use cli_parser::CliParser;
pub use option_inferrer::{apply_numeric_constraints, load_enum_values, OptionInferrer};
pub use subcommand_detector::SubcommandDetector;
