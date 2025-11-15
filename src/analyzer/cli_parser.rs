use crate::analyzer::SubcommandDetector;
use crate::error::{CliTestError, Result};
use crate::types::analysis::{CliAnalysis, CliOption, OptionType};
use crate::utils::{execute_with_timeout, validate_binary_path, ResourceLimits};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use std::time::Instant;

lazy_static! {
    /// Regex pattern for short options: -h, -v, etc.
    static ref SHORT_OPTION: Regex = Regex::new(r"-([a-zA-Z])(?:\s|,|$)").unwrap();

    /// Regex pattern for long options: --help, --verbose, etc.
    static ref LONG_OPTION: Regex = Regex::new(r"--([a-z][a-z0-9-]+)").unwrap();

    /// Regex pattern for version strings: v1.0.0, 2.5.3, etc.
    static ref VERSION_PATTERN: Regex = Regex::new(r"\b\d+\.\d+(?:\.\d+)?(?:-[a-z0-9.]+)?\b").unwrap();

    /// Regex pattern for option with value: --name <value>, --file <path>
    static ref OPTION_WITH_VALUE: Regex = Regex::new(r"--([a-z][a-z0-9-]+)\s+<([^>]+)>").unwrap();

    /// Regex pattern for option description (tries to capture text after option)
    static ref OPTION_DESCRIPTION: Regex = Regex::new(r"(?:--[a-z][a-z0-9-]+)(?:\s+<[^>]+>)?\s+(.+)").unwrap();
}

/// CLI Parser - Executes binaries and parses help output
pub struct CliParser {
    resource_limits: ResourceLimits,
}

impl CliParser {
    /// Create a new CLI parser with default resource limits
    pub fn new() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Create a new CLI parser with custom resource limits
    pub fn with_limits(resource_limits: ResourceLimits) -> Self {
        Self { resource_limits }
    }

    /// Analyze a CLI binary and extract its structure
    ///
    /// This performs the following steps:
    /// 1. Validate binary path
    /// 2. Execute with --help to get help output
    /// 3. Execute with --version to get version string
    /// 4. Parse help output to extract options
    /// 5. Detect subcommands recursively
    /// 6. Build CliAnalysis structure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cli_testing_specialist::analyzer::CliParser;
    /// use std::path::Path;
    ///
    /// let parser = CliParser::new();
    /// let analysis = parser.analyze(Path::new("/usr/bin/curl"))?;
    ///
    /// println!("Binary: {}", analysis.binary_name);
    /// println!("Version: {:?}", analysis.version);
    /// println!("Options: {}", analysis.metadata.total_options);
    /// println!("Subcommands: {}", analysis.subcommands.len());
    /// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
    /// ```
    ///
    /// # With Custom Resource Limits
    ///
    /// ```no_run
    /// use cli_testing_specialist::analyzer::CliParser;
    /// use cli_testing_specialist::utils::ResourceLimits;
    /// use std::path::Path;
    /// use std::time::Duration;
    ///
    /// let limits = ResourceLimits::new(
    ///     1024 * 1024 * 1024, // 1GB memory
    ///     1024,               // file descriptors
    ///     100,                // max processes
    ///     Duration::from_secs(60), // timeout
    /// );
    ///
    /// let parser = CliParser::with_limits(limits);
    /// let analysis = parser.analyze(Path::new("/usr/bin/kubectl"))?;
    /// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
    /// ```
    pub fn analyze(&self, binary_path: &Path) -> Result<CliAnalysis> {
        let start_time = Instant::now();

        // Step 1: Validate binary
        let canonical_path = validate_binary_path(binary_path)?;
        log::info!("Analyzing binary: {}", canonical_path.display());

        // Extract binary name
        let binary_name = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CliTestError::BinaryNotFound(canonical_path.clone()))?
            .to_string();

        // Step 2: Execute with --help
        let help_output = self.execute_help(&canonical_path)?;

        if help_output.trim().is_empty() {
            return Err(CliTestError::InvalidHelpOutput);
        }

        // Step 3: Try to get version
        let version = self.try_get_version(&canonical_path);

        // Step 4: Parse options from help output
        let global_options = self.parse_options(&help_output);

        // Step 5: Detect subcommands recursively
        let subcommand_detector = SubcommandDetector::default();
        let subcommands = subcommand_detector
            .detect(&canonical_path, &help_output)
            .unwrap_or_default();

        // Step 6: Build analysis result
        let mut analysis = CliAnalysis::new(canonical_path, binary_name, help_output);
        analysis.version = version;
        analysis.global_options = global_options;
        analysis.subcommands = subcommands;

        // Update metadata
        let duration_ms = start_time.elapsed().as_millis() as u64;
        analysis.update_metadata(duration_ms);

        log::info!(
            "Analysis complete: {} options, {} subcommands found in {}ms",
            analysis.metadata.total_options,
            analysis.subcommands.len(),
            duration_ms
        );

        Ok(analysis)
    }

    /// Execute binary with --help flag
    fn execute_help(&self, binary: &Path) -> Result<String> {
        log::debug!("Executing {} --help", binary.display());

        // Try --help first (most common)
        match execute_with_timeout(binary, &["--help"], self.resource_limits.timeout()) {
            Ok(output) => Ok(output),
            Err(_) => {
                // Try -h as fallback
                log::debug!("--help failed, trying -h");
                match execute_with_timeout(binary, &["-h"], self.resource_limits.timeout()) {
                    Ok(output) => Ok(output),
                    Err(_) => {
                        // Try 'help' subcommand as last resort
                        log::debug!("-h failed, trying 'help' subcommand");
                        execute_with_timeout(binary, &["help"], self.resource_limits.timeout())
                    }
                }
            }
        }
    }

    /// Try to get version string from binary
    fn try_get_version(&self, binary: &Path) -> Option<String> {
        log::debug!("Attempting to get version for {}", binary.display());

        // Try --version
        if let Ok(output) =
            execute_with_timeout(binary, &["--version"], self.resource_limits.timeout())
        {
            if let Some(version) = self.extract_version(&output) {
                return Some(version);
            }
        }

        // Try -v
        if let Ok(output) = execute_with_timeout(binary, &["-v"], self.resource_limits.timeout()) {
            if let Some(version) = self.extract_version(&output) {
                return Some(version);
            }
        }

        // Try 'version' subcommand
        if let Ok(output) =
            execute_with_timeout(binary, &["version"], self.resource_limits.timeout())
        {
            if let Some(version) = self.extract_version(&output) {
                return Some(version);
            }
        }

        None
    }

    /// Extract version string from output
    fn extract_version(&self, output: &str) -> Option<String> {
        VERSION_PATTERN.find(output).map(|m| m.as_str().to_string())
    }

    /// Parse CLI options from help output
    pub fn parse_options(&self, help_output: &str) -> Vec<CliOption> {
        let mut options = Vec::new();
        let mut seen_options = std::collections::HashSet::new();

        for line in help_output.lines() {
            let trimmed = line.trim();

            // Skip empty lines and headers
            if trimmed.is_empty() || !trimmed.contains('-') {
                continue;
            }

            // Extract short and long options from the line
            let short = SHORT_OPTION
                .captures(trimmed)
                .and_then(|cap| cap.get(1))
                .map(|m| format!("-{}", m.as_str()));

            let long = LONG_OPTION
                .captures(trimmed)
                .and_then(|cap| cap.get(1))
                .map(|m| format!("--{}", m.as_str()));

            // Skip if no option found or already processed
            if short.is_none() && long.is_none() {
                continue;
            }

            let option_key = format!("{:?}:{:?}", short, long);
            if seen_options.contains(&option_key) {
                continue;
            }
            seen_options.insert(option_key);

            // Extract description
            let description = OPTION_DESCRIPTION
                .captures(trimmed)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().trim().to_string());

            // Determine option type (basic inference, will be enhanced by option_inferrer)
            let option_type = if OPTION_WITH_VALUE.is_match(trimmed) {
                OptionType::String
            } else {
                OptionType::Flag
            };

            options.push(CliOption {
                short,
                long,
                description,
                option_type,
                required: false, // Default to optional
                default_value: None,
            });
        }

        options
    }

    /// Parse required positional arguments from help output
    ///
    /// Looks for Usage line and extracts `<ARG>` patterns:
    /// - "Usage: cmd \[OPTIONS\] `<ID>`" → \["ID"\]
    /// - "Usage: cmd `<FILE>` `<OUTPUT>`" → \["FILE", "OUTPUT"\]
    pub fn parse_required_args(&self, help_output: &str) -> Vec<String> {
        lazy_static! {
            static ref USAGE_LINE: Regex = Regex::new(r"(?i)^\s*usage:\s+").unwrap();
            static ref REQUIRED_ARG: Regex = Regex::new(r"<([^>]+)>").unwrap();
        }

        let mut required_args = Vec::new();

        for line in help_output.lines() {
            if USAGE_LINE.is_match(line) {
                // Extract all <ARG> patterns from the usage line
                for cap in REQUIRED_ARG.captures_iter(line) {
                    if let Some(arg_match) = cap.get(1) {
                        let arg_name = arg_match.as_str().to_string();
                        required_args.push(arg_name);
                    }
                }
                break; // Only process the first Usage line
            }
        }

        log::debug!("Detected {} required arguments", required_args.len());
        required_args
    }
}

impl Default for CliParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_option_regex() {
        assert!(SHORT_OPTION.is_match("-h"));
        assert!(SHORT_OPTION.is_match("-v "));
        assert!(SHORT_OPTION.is_match("-f,"));
        assert!(!SHORT_OPTION.is_match("--help"));
    }

    #[test]
    fn test_long_option_regex() {
        assert!(LONG_OPTION.is_match("--help"));
        assert!(LONG_OPTION.is_match("--verbose"));
        assert!(LONG_OPTION.is_match("--max-size"));
        assert!(!LONG_OPTION.is_match("-h"));
    }

    #[test]
    fn test_version_pattern_regex() {
        assert!(VERSION_PATTERN.is_match("1.0.0"));
        assert!(VERSION_PATTERN.is_match("2.5.3"));
        assert!(VERSION_PATTERN.is_match("1.0.0-alpha.1"));
        assert!(VERSION_PATTERN.is_match("curl 7.64.1"));
    }

    #[test]
    fn test_option_with_value_regex() {
        assert!(OPTION_WITH_VALUE.is_match("--name <value>"));
        assert!(OPTION_WITH_VALUE.is_match("--file <path>"));
        assert!(!OPTION_WITH_VALUE.is_match("--verbose"));
    }

    #[test]
    fn test_extract_version() {
        let parser = CliParser::new();

        assert_eq!(
            parser.extract_version("curl 7.64.1"),
            Some("7.64.1".to_string())
        );
        assert_eq!(
            parser.extract_version("version 1.0.0"),
            Some("1.0.0".to_string())
        );
        assert_eq!(parser.extract_version("no version here"), None);
    }

    #[test]
    fn test_parse_options_basic() {
        let parser = CliParser::new();
        let help_output = r#"
Usage: test [OPTIONS]

Options:
  -h, --help       Print help information
  -v, --verbose    Enable verbose output
      --name <VALUE>  Set name value
"#;

        let options = parser.parse_options(help_output);

        assert_eq!(options.len(), 3);

        // Check --help option
        assert!(options.iter().any(|o| o.long == Some("--help".to_string())));
        assert!(options.iter().any(|o| o.short == Some("-h".to_string())));

        // Check --verbose option
        assert!(options
            .iter()
            .any(|o| o.long == Some("--verbose".to_string())));
    }

    #[test]
    fn test_parse_options_deduplication() {
        let parser = CliParser::new();
        let help_output = r#"
  -h, --help    Help text
  -h, --help    Duplicate help text
"#;

        let options = parser.parse_options(help_output);

        // Should only have one option despite duplicate
        assert_eq!(options.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn test_analyze_ls() {
        // Test with /bin/ls which should be available on all Unix systems
        let ls_path = Path::new("/bin/ls");
        if !ls_path.exists() {
            return; // Skip if ls not available
        }

        let parser = CliParser::new();
        let result = parser.analyze(ls_path);

        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.binary_name, "ls");
        assert!(!analysis.help_output.is_empty());
        assert!(!analysis.global_options.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_analyze_curl() {
        // Test with curl if available
        let curl_path = Path::new("/usr/bin/curl");
        if !curl_path.exists() {
            return; // Skip if curl not available
        }

        let parser = CliParser::new();
        let result = parser.analyze(curl_path);

        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.binary_name, "curl");
        assert!(analysis.version.is_some());
        assert!(!analysis.global_options.is_empty());

        // Curl should have many options
        assert!(analysis.global_options.len() > 10);
    }
}
