//! # Configuration Module
//!
//! Loads and validates YAML configuration files for the CLI testing framework.
//!
//! ## Configuration Files
//!
//! - `option-patterns.yaml`: Pattern matching rules for option type inference
//! - `numeric-constraints.yaml`: Min/max constraints for numeric options
//! - `enum-definitions.yaml`: Enum value definitions for specific option patterns
//!
//! ## Example Usage
//!
//! ```no_run
//! use cli_testing_specialist::config::load_config;
//! use std::path::Path;
//!
//! // Load from specific path
//! let config = load_config(Some(Path::new(".cli-test-config.yml")))?;
//! if let Some(cfg) = config {
//!     println!("Loaded configuration version: {}", cfg.version);
//! }
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Validation
//!
//! ```no_run
//! use cli_testing_specialist::config::{load_config, validate_config};
//! use std::path::Path;
//!
//! if let Some(config) = load_config(Some(Path::new(".cli-test-config.yml")))? {
//!     validate_config(&config)?; // Ensures valid configuration
//!     println!("Configuration validated successfully");
//! }
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```

pub mod loader;
pub mod validator;

pub use loader::load_config;
pub use validator::validate_config;
