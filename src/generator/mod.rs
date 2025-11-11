//! # Generator Module
//!
//! Generates comprehensive BATS (Bash Automated Testing System) test suites from CLI analysis.
//!
//! ## Test Categories
//!
//! The generator produces tests across 9 categories:
//!
//! - **Basic**: Help, version, exit codes
//! - **Help**: Help text validation and formatting
//! - **Security**: Injection attacks, path traversal, command injection
//! - **Path**: File/directory handling, special characters
//! - **InputValidation**: Invalid inputs, boundary conditions
//! - **DestructiveOps**: Operations requiring confirmation
//! - **DirectoryTraversal**: Path traversal prevention
//! - **Performance**: Response time validation
//! - **MultiShell**: Cross-shell compatibility (bash, zsh, fish)
//!
//! ## Example Usage
//!
//! ```no_run
//! use cli_testing_specialist::analyzer::CliParser;
//! use cli_testing_specialist::generator::TestGenerator;
//! use cli_testing_specialist::types::TestCategory;
//! use std::path::Path;
//!
//! let parser = CliParser::new();
//! let analysis = parser.analyze(Path::new("/usr/bin/curl"))?;
//!
//! let generator = TestGenerator::new(
//!     analysis,
//!     vec![TestCategory::Basic, TestCategory::Security]
//! );
//! let tests = generator.generate()?;
//!
//! println!("Generated {} tests", tests.len());
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Performance
//!
//! - **Sequential**: 50-100ms for 50 tests
//! - **Parallel**: 20-40ms for 50 tests (2-3x faster)
//!
//! ## Template System
//!
//! Uses embedded templates with variable substitution:
//! - `{{binary_name}}`: Target CLI tool name
//! - `{{command}}`: Test command to execute
//! - `{{expected_output}}`: Expected output pattern
//!
//! Templates are validated at compile time for correctness.

pub mod bats_writer;
pub mod templates;
pub mod test_generator;

// Re-export commonly used types
pub use bats_writer::BatsWriter;
pub use templates::TemplateEngine;
pub use test_generator::TestGenerator;
