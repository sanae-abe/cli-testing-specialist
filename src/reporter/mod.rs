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
//! use cli_testing_specialist::types::TestReport;
//! use std::path::Path;
//! use std::time::Duration;
//! use chrono::Utc;
//!
//! let report = TestReport {
//!     binary_name: "curl".to_string(),
//!     binary_version: Some("8.7.1".to_string()),
//!     suites: vec![],
//!     total_duration: Duration::from_secs(5),
//!     started_at: Utc::now(),
//!     finished_at: Utc::now(),
//!     environment: Default::default(),
//!     security_findings: vec![],
//! };
//!
//! MarkdownReporter::generate(&report, Path::new("report.md"))?;
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Multi-Format Generation
//!
//! ```no_run
//! use cli_testing_specialist::reporter::{MarkdownReporter, JsonReporter, HtmlReporter};
//! use cli_testing_specialist::types::TestReport;
//! use std::path::Path;
//! use std::time::Duration;
//! use chrono::Utc;
//! # let report = TestReport {
//! #     binary_name: "curl".to_string(),
//! #     binary_version: Some("8.7.1".to_string()),
//! #     suites: vec![],
//! #     total_duration: Duration::from_secs(0),
//! #     started_at: Utc::now(),
//! #     finished_at: Utc::now(),
//! #     environment: Default::default(),
//! #     security_findings: vec![],
//! # };
//!
//! // Generate all formats
//! MarkdownReporter::generate(&report, Path::new("report.md"))?;
//! JsonReporter::generate(&report, Path::new("report.json"))?;
//! HtmlReporter::generate(&report, Path::new("report.html"))?;
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
