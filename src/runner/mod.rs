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
//! use cli_testing_specialist::types::TestCategory;
//! use std::path::Path;
//!
//! let executor = BatsExecutor::new(
//!     Path::new("/path/to/tests"),
//!     300, // timeout in seconds
//!     vec![TestCategory::Basic, TestCategory::Security],
//! );
//!
//! let report = executor.run()?;
//! println!("Tests passed: {}/{}", report.total_passed, report.total_tests);
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Category Filtering
//!
//! ```no_run
//! use cli_testing_specialist::runner::BatsExecutor;
//! use cli_testing_specialist::types::TestCategory;
//! use std::path::Path;
//!
//! // Run only security-related tests
//! let executor = BatsExecutor::new(
//!     Path::new("/path/to/tests"),
//!     300,
//!     vec![
//!         TestCategory::Security,
//!         TestCategory::DirectoryTraversal,
//!         TestCategory::InputValidation,
//!     ],
//! );
//!
//! let report = executor.run()?;
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```

pub mod bats_executor;

// Re-export main executor
pub use bats_executor::BatsExecutor;
