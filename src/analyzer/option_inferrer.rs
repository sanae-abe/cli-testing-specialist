use crate::error::Result;
use crate::types::analysis::{CliOption, OptionType};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

/// Pattern configuration loaded from YAML
#[derive(Debug, Clone, Deserialize)]
struct OptionPattern {
    #[serde(rename = "type")]
    pattern_type: String,
    priority: u8,
    keywords: Vec<String>,
    #[allow(dead_code)]
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OptionPatternsConfig {
    patterns: Vec<OptionPattern>,
    default_type: String,
    settings: PatternSettings,
}

#[derive(Debug, Clone, Deserialize)]
struct PatternSettings {
    case_sensitive: bool,
    partial_match: bool,
    min_keyword_length: usize,
}

/// Numeric constraint definition from YAML
#[derive(Debug, Clone, Deserialize)]
struct NumericConstraint {
    aliases: Vec<String>,
    min: i64,
    max: i64,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    constraint_type: String,
    #[allow(dead_code)]
    unit: Option<String>,
    #[allow(dead_code)]
    description: String,
}

/// Numeric constraints configuration from YAML
#[derive(Debug, Clone, Deserialize)]
struct NumericConstraintsConfig {
    constraints: HashMap<String, NumericConstraint>,
    default_constraints: DefaultNumericConstraints,
}

#[derive(Debug, Clone, Deserialize)]
struct DefaultNumericConstraints {
    min: i64,
    max: i64,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    constraint_type: String,
}

/// Enum definition from YAML
#[derive(Debug, Clone, Deserialize)]
struct EnumDefinition {
    aliases: Vec<String>,
    values: Vec<String>,
    #[allow(dead_code)]
    case_sensitive: bool,
    #[allow(dead_code)]
    description: String,
}

/// Enum definitions configuration from YAML
#[derive(Debug, Clone, Deserialize)]
struct EnumDefinitionsConfig {
    enums: HashMap<String, EnumDefinition>,
    #[allow(dead_code)]
    default_enum: DefaultEnumConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DefaultEnumConfig {
    case_sensitive: bool,
    allow_partial_match: bool,
}

lazy_static! {
    /// Global cache for option patterns loaded from YAML
    static ref PATTERN_CACHE: Mutex<Option<OptionPatternsConfig>> = Mutex::new(None);

    /// Global cache for numeric constraints loaded from YAML
    static ref NUMERIC_CONSTRAINTS_CACHE: Mutex<Option<NumericConstraintsConfig>> = Mutex::new(None);

    /// Global cache for enum definitions loaded from YAML
    static ref ENUM_DEFINITIONS_CACHE: Mutex<Option<EnumDefinitionsConfig>> = Mutex::new(None);
}

/// Option Type Inferrer - Infers option types from names and patterns
pub struct OptionInferrer {
    patterns: Vec<OptionPattern>,
    settings: PatternSettings,
    default_type: String,
}

impl OptionInferrer {
    /// Create a new option inferrer by loading patterns from YAML
    pub fn new() -> Result<Self> {
        Self::from_config_path("config/option-patterns.yaml")
    }

    /// Create option inferrer from a specific config file
    pub fn from_config_path(config_path: &str) -> Result<Self> {
        // Check cache first
        let mut cache = PATTERN_CACHE.lock().unwrap();

        if cache.is_none() {
            // Load and parse YAML config (with safe deserialization)
            let config_content = std::fs::read_to_string(config_path)?;
            let config: OptionPatternsConfig =
                crate::utils::deserialize_yaml_safe(&config_content)?;
            *cache = Some(config);
        }

        // Clone from cache
        let config = cache.as_ref().unwrap().clone();

        Ok(Self {
            patterns: config.patterns,
            settings: config.settings,
            default_type: config.default_type,
        })
    }

    /// Infer option types for a list of options
    pub fn infer_types(&self, options: &mut [CliOption]) {
        for option in options.iter_mut() {
            option.option_type = self.infer_type(option);
        }
    }

    /// Infer the type of a single option
    pub fn infer_type(&self, option: &CliOption) -> OptionType {
        // If it's already flagged as having a value (from parser), start with that
        if matches!(option.option_type, OptionType::Flag) {
            // True flag - no value expected
            return OptionType::Flag;
        }

        // Extract option name for pattern matching
        let option_name = self.extract_option_name(option);

        // Sort patterns by priority (higher first)
        let mut sorted_patterns = self.patterns.clone();
        sorted_patterns.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Try to match against patterns
        for pattern in &sorted_patterns {
            if self.matches_pattern(&option_name, pattern) {
                return self.pattern_type_to_option_type(&pattern.pattern_type);
            }
        }

        // Fallback to default type
        self.pattern_type_to_option_type(&self.default_type)
    }

    /// Extract option name from CliOption (prefer long, fallback to short)
    fn extract_option_name(&self, option: &CliOption) -> String {
        if let Some(long) = &option.long {
            // Remove leading dashes: --timeout -> timeout
            long.trim_start_matches('-').to_string()
        } else if let Some(short) = &option.short {
            // Remove leading dash: -t -> t
            short.trim_start_matches('-').to_string()
        } else {
            String::new()
        }
    }

    /// Check if option name matches a pattern
    fn matches_pattern(&self, option_name: &str, pattern: &OptionPattern) -> bool {
        let name = if self.settings.case_sensitive {
            option_name.to_string()
        } else {
            option_name.to_lowercase()
        };

        for keyword in &pattern.keywords {
            let keyword_normalized = if self.settings.case_sensitive {
                keyword.clone()
            } else {
                keyword.to_lowercase()
            };

            if self.settings.partial_match {
                // Partial match: "timeout" matches "connect-timeout"
                if keyword_normalized.len() >= self.settings.min_keyword_length
                    && name.contains(&keyword_normalized)
                {
                    return true;
                }
            } else {
                // Exact match
                if name == keyword_normalized {
                    return true;
                }
            }
        }

        false
    }

    /// Convert pattern type string to OptionType enum
    fn pattern_type_to_option_type(&self, pattern_type: &str) -> OptionType {
        match pattern_type {
            "numeric" => OptionType::Numeric {
                min: None,
                max: None,
            },
            "path" => OptionType::Path,
            "enum" => OptionType::Enum { values: vec![] },
            "boolean" => OptionType::Flag,
            _ => OptionType::String,
        }
    }
}

impl Default for OptionInferrer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to empty patterns if loading fails
            Self {
                patterns: vec![],
                settings: PatternSettings {
                    case_sensitive: false,
                    partial_match: true,
                    min_keyword_length: 3,
                },
                default_type: "string".to_string(),
            }
        })
    }
}

/// Load numeric constraints configuration from YAML (with caching)
fn load_numeric_constraints_config() -> Result<NumericConstraintsConfig> {
    let mut cache = NUMERIC_CONSTRAINTS_CACHE.lock().unwrap();

    if cache.is_none() {
        // Load and parse YAML config
        let config_content = std::fs::read_to_string("config/numeric-constraints.yaml")?;
        let config: NumericConstraintsConfig =
            crate::utils::deserialize_yaml_safe(&config_content)?;
        *cache = Some(config);
    }

    Ok(cache.as_ref().unwrap().clone())
}

/// Apply numeric constraints from numeric-constraints.yaml
///
/// Loads constraints like:
/// - Port numbers: 1-65535
/// - Timeouts: 0-3600
/// - Percentages: 0-100
///
/// Uses global cache for performance (loaded once, reused for all subsequent calls).
pub fn apply_numeric_constraints(options: &mut [CliOption]) {
    // Load config from cache (or file if not cached)
    let config = match load_numeric_constraints_config() {
        Ok(config) => config,
        Err(e) => {
            log::warn!("Failed to load numeric constraints: {}", e);
            return; // Silently skip if config not available
        }
    };

    for option in options.iter_mut() {
        if let OptionType::Numeric {
            ref mut min,
            ref mut max,
        } = option.option_type
        {
            let option_name = option
                .long
                .as_ref()
                .or(option.short.as_ref())
                .map(|s| s.trim_start_matches('-').to_lowercase())
                .unwrap_or_default();

            // Try to match against constraint aliases
            let mut matched = false;
            for constraint in config.constraints.values() {
                if constraint
                    .aliases
                    .iter()
                    .any(|alias| option_name.contains(&alias.to_lowercase()))
                {
                    *min = Some(constraint.min);
                    *max = Some(constraint.max);
                    matched = true;
                    break;
                }
            }

            // Apply default constraints if no match found
            if !matched {
                *min = Some(config.default_constraints.min);
                *max = Some(config.default_constraints.max);
            }
        }
    }
}

/// Load enum definitions configuration from YAML (with caching)
fn load_enum_definitions_config() -> Result<EnumDefinitionsConfig> {
    let mut cache = ENUM_DEFINITIONS_CACHE.lock().unwrap();

    if cache.is_none() {
        // Load and parse YAML config
        let config_content = std::fs::read_to_string("config/enum-definitions.yaml")?;
        let config: EnumDefinitionsConfig = crate::utils::deserialize_yaml_safe(&config_content)?;
        *cache = Some(config);
    }

    Ok(cache.as_ref().unwrap().clone())
}

/// Load enum values from enum-definitions.yaml
///
/// Loads known enum values like:
/// - format: json, yaml, xml, toml
/// - log-level: debug, info, warn, error
/// - protocol: http, https, ftp, ssh
///
/// Uses global cache for performance (loaded once, reused for all subsequent calls).
pub fn load_enum_values(options: &mut [CliOption]) {
    // Load config from cache (or file if not cached)
    let config = match load_enum_definitions_config() {
        Ok(config) => config,
        Err(e) => {
            log::warn!("Failed to load enum definitions: {}", e);
            return; // Silently skip if config not available
        }
    };

    for option in options.iter_mut() {
        if let OptionType::Enum { ref mut values } = option.option_type {
            let option_name = option
                .long
                .as_ref()
                .or(option.short.as_ref())
                .map(|s| s.trim_start_matches('-').to_lowercase())
                .unwrap_or_default();

            // Try to match against enum aliases
            for enum_def in config.enums.values() {
                if enum_def
                    .aliases
                    .iter()
                    .any(|alias| option_name.contains(&alias.to_lowercase()))
                {
                    *values = enum_def.values.clone();
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_option_name() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: None,
            long: Some("--timeout".to_string()),
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        assert_eq!(inferrer.extract_option_name(&option), "timeout");
    }

    #[test]
    fn test_extract_option_name_short() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: Some("-p".to_string()),
            long: None,
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        assert_eq!(inferrer.extract_option_name(&option), "p");
    }

    #[test]
    fn test_infer_type_timeout() {
        let inferrer = OptionInferrer::default();

        let mut option = CliOption {
            short: None,
            long: Some("--timeout".to_string()),
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        let inferred_type = inferrer.infer_type(&option);

        // Should be numeric due to "timeout" keyword
        assert!(matches!(inferred_type, OptionType::Numeric { .. }));

        option.option_type = inferred_type;
        let mut options = vec![option];
        apply_numeric_constraints(&mut options);

        // Check constraints were applied
        if let OptionType::Numeric { min, max } = &options[0].option_type {
            assert_eq!(*min, Some(0));
            assert_eq!(*max, Some(3600));
        }
    }

    #[test]
    fn test_infer_type_path() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: None,
            long: Some("--config".to_string()),
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        let inferred_type = inferrer.infer_type(&option);

        // Should be path due to "config" keyword
        assert!(matches!(inferred_type, OptionType::Path));
    }

    #[test]
    fn test_infer_type_enum() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: None,
            long: Some("--format".to_string()),
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        let inferred_type = inferrer.infer_type(&option);

        // Should be enum due to "format" keyword
        assert!(matches!(inferred_type, OptionType::Enum { .. }));
    }

    #[test]
    fn test_infer_type_flag() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: None,
            long: Some("--verbose".to_string()),
            description: None,
            option_type: OptionType::Flag,
            required: false,
            default_value: None,
        };

        let inferred_type = inferrer.infer_type(&option);

        // Should remain flag
        assert!(matches!(inferred_type, OptionType::Flag));
    }

    #[test]
    fn test_apply_numeric_constraints_port() {
        let mut options = vec![CliOption {
            short: None,
            long: Some("--port".to_string()),
            description: None,
            option_type: OptionType::Numeric {
                min: None,
                max: None,
            },
            required: false,
            default_value: None,
        }];

        apply_numeric_constraints(&mut options);

        if let OptionType::Numeric { min, max } = &options[0].option_type {
            assert_eq!(*min, Some(1));
            assert_eq!(*max, Some(65535));
        } else {
            panic!("Expected Numeric type");
        }
    }

    #[test]
    fn test_load_enum_values_format() {
        let mut options = vec![CliOption {
            short: None,
            long: Some("--format".to_string()),
            description: None,
            option_type: OptionType::Enum { values: vec![] },
            required: false,
            default_value: None,
        }];

        load_enum_values(&mut options);

        if let OptionType::Enum { values } = &options[0].option_type {
            assert!(!values.is_empty());
            assert!(values.contains(&"json".to_string()));
            assert!(values.contains(&"yaml".to_string()));
        } else {
            panic!("Expected Enum type");
        }
    }

    #[test]
    fn test_partial_match() {
        let inferrer = OptionInferrer::default();

        let option = CliOption {
            short: None,
            long: Some("--connect-timeout".to_string()),
            description: None,
            option_type: OptionType::String,
            required: false,
            default_value: None,
        };

        let inferred_type = inferrer.infer_type(&option);

        // Should match "timeout" keyword via partial match
        assert!(matches!(inferred_type, OptionType::Numeric { .. }));
    }
}
