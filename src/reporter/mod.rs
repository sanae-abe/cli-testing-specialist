//! # Reporter Module
//!
//! Generates test reports in multiple formats from test execution results.
//!
//! ## Supported Formats
//!
//! - **Markdown**: Human-readable summary with tables
//! - **JSON**: Machine-readable structured data
//! - **HTML**: Interactive web-based reports with filtering
//! - **JUnit**: CI/CD compatible XML format
//!
//! ## Example Usage
//!
//! ```no_run
//! use cli_testing_specialist::reporter::MarkdownReporter;
//! use cli_testing_specialist::types::{TestReport, TestSuite, TestResult};
//! use std::path::Path;
//!
//! let report = TestReport {
//!     binary_name: "curl".to_string(),
//!     test_suites: vec![],
//!     total_tests: 42,
//!     total_passed: 40,
//!     total_failed: 2,
//!     total_skipped: 0,
//!     duration_secs: 5.2,
//! };
//!
//! let reporter = MarkdownReporter::new();
//! reporter.generate(&report, Path::new("report.md"))?;
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Multi-Format Generation
//!
//! ```no_run
//! use cli_testing_specialist::reporter::{MarkdownReporter, JsonReporter, HtmlReporter};
//! use cli_testing_specialist::types::TestReport;
//! use std::path::Path;
//! # let report = TestReport {
//! #     binary_name: "curl".to_string(),
//! #     test_suites: vec![],
//! #     total_tests: 0,
//! #     total_passed: 0,
//! #     total_failed: 0,
//! #     total_skipped: 0,
//! #     duration_secs: 0.0,
//! # };
//!
//! // Generate all formats
//! MarkdownReporter::new().generate(&report, Path::new("report.md"))?;
//! JsonReporter::new().generate(&report, Path::new("report.json"))?;
//! HtmlReporter::new().generate(&report, Path::new("report.html"))?;
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```

pub mod html;
pub mod json;
pub mod junit;
pub mod markdown;

// Re-export reporters
pub use html::HtmlReporter;
pub use json::JsonReporter;
pub use junit::JunitReporter;
pub use markdown::MarkdownReporter;
