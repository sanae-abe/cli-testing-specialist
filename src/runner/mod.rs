//! # Runner Module
//!
//! Executes BATS (Bash Automated Testing System) test suites and collects results.
//!
//! ## Features
//!
//! - Parallel test execution with configurable workers
//! - Timeout management per test file
//! - TAP (Test Anything Protocol) output parsing
//! - Category-based test filtering
//! - Shell compatibility validation
//!
//! ## Example Usage
//!
//! ```no_run
//! use cli_testing_specialist::runner::BatsExecutor;
//! use std::path::Path;
//!
//! let executor = BatsExecutor::with_timeout(
//!     "curl".to_string(),
//!     Some("8.7.1".to_string()),
//!     300, // timeout in seconds
//! );
//!
//! let report = executor.run_tests(Path::new("/path/to/tests"))?;
//! println!("Tests passed: {}/{}", report.total_passed(), report.total_tests());
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Category Filtering
//!
//! ```no_run
//! use cli_testing_specialist::runner::BatsExecutor;
//! use std::path::Path;
//!
//! // Skip resource-intensive tests
//! let executor = BatsExecutor::with_timeout(
//!     "kubectl".to_string(),
//!     Some("1.28.0".to_string()),
//!     300,
//! ).with_skip_categories(vec![
//!     "directory-traversal".to_string(),
//!     "performance".to_string(),
//! ]);
//!
//! let report = executor.run_tests(Path::new("/path/to/tests"))?;
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```

pub mod bats_executor;

// Re-export main executor
pub use bats_executor::BatsExecutor;
