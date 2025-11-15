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
//! let config = load_config(Path::new("config/option-patterns.yaml"))?;
//! println!("Loaded {} pattern rules", config.patterns.len());
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```
//!
//! ## Validation
//!
//! ```no_run
//! use cli_testing_specialist::config::{load_config, validate_config};
//! use std::path::Path;
//!
//! let config = load_config(Path::new("config/option-patterns.yaml"))?;
//! validate_config(&config)?; // Ensures no duplicate patterns, valid regex, etc.
//! # Ok::<(), cli_testing_specialist::error::CliTestError>(())
//! ```

pub mod loader;
pub mod validator;

pub use loader::load_config;
pub use validator::validate_config;
