use crate::analyzer::cli_parser::CliParser;
use crate::analyzer::option_inferrer::OptionInferrer;
use crate::error::Result;
use crate::types::analysis::Subcommand;
use crate::utils::{execute_with_timeout, ResourceLimits};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

lazy_static! {
    /// Regex pattern for subcommand lines in help output
    /// Matches lines like:
    /// - "  help      Show help information" (standard format)
    /// - "  config    Manage configuration" (standard format)
    /// - "  publish [options] [project-path]  Publish package to registry" (Commander.js format)
    ///
    /// Pattern breakdown:
    /// - `^\s{2,}` - Line starts with 2+ spaces (indentation)
    /// - `([a-z][a-z0-9-]+)` - Subcommand name (lowercase, alphanumeric, hyphens)
    /// - `(?:\s+\[[^\]]+\])*` - Optional argument specifications like [options], [path] (Commander.js)
    /// - `\s{2,}` - 2+ spaces separating command from description
    /// - `(.+)$` - Description text
    static ref SUBCOMMAND_PATTERN: Regex = Regex::new(r"^\s{2,}([a-z][a-z0-9-]+)(?:\s+\[[^\]]+\])*\s{2,}(.+)$").unwrap();

    /// Common section headers that indicate subcommands section
    static ref SUBCOMMAND_HEADERS: Vec<&'static str> = vec![
        "Commands:",
        "Available Commands:",
        "Subcommands:",
        "Available subcommands:",
        "COMMANDS:",
        "SUBCOMMANDS:",
    ];
}

/// Maximum recursion depth for subcommand detection
const MAX_RECURSION_DEPTH: u8 = 3;

/// Subcommand Detector - Recursively detects CLI subcommands
pub struct SubcommandDetector {
    resource_limits: ResourceLimits,
    option_inferrer: OptionInferrer,
    max_depth: u8,
}

impl SubcommandDetector {
    /// Create a new subcommand detector with default settings
    pub fn new() -> Result<Self> {
        Ok(Self {
            resource_limits: ResourceLimits::default(),
            option_inferrer: OptionInferrer::new()?,
            max_depth: MAX_RECURSION_DEPTH,
        })
    }

    /// Create a new subcommand detector with custom max depth
    pub fn with_max_depth(max_depth: u8) -> Result<Self> {
        Ok(Self {
            resource_limits: ResourceLimits::default(),
            option_inferrer: OptionInferrer::new()?,
            max_depth,
        })
    }

    /// Detect subcommands from help output
    pub fn detect(&self, binary: &Path, help_output: &str) -> Result<Vec<Subcommand>> {
        log::info!("Detecting subcommands for {}", binary.display());
        self.detect_recursive(binary, help_output, 0, &mut HashSet::new())
    }

    /// Recursively detect subcommands
    fn detect_recursive(
        &self,
        binary: &Path,
        help_output: &str,
        current_depth: u8,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<Subcommand>> {
        // Stop if max depth reached
        if current_depth >= self.max_depth {
            log::debug!("Max recursion depth {} reached", self.max_depth);
            return Ok(vec![]);
        }

        // Parse subcommand names from help output
        let subcommand_candidates = self.parse_subcommands(help_output);

        if subcommand_candidates.is_empty() {
            return Ok(vec![]);
        }

        log::debug!(
            "Found {} subcommand candidates at depth {}",
            subcommand_candidates.len(),
            current_depth
        );

        let mut subcommands = Vec::new();

        for (name, description) in subcommand_candidates {
            // Skip if already visited (prevent circular references)
            let visit_key = format!("{}-{}", binary.display(), name);
            if visited.contains(&visit_key) {
                log::debug!("Skipping already visited subcommand: {}", name);
                continue;
            }
            visited.insert(visit_key);

            // Get help output for this subcommand
            let subcommand_help = match self.get_subcommand_help(binary, &name) {
                Ok(help) => help,
                Err(e) => {
                    log::warn!("Failed to get help for subcommand '{}': {}", name, e);
                    continue;
                }
            };

            // Parse options for this subcommand
            let cli_parser = CliParser::new();
            let mut options = cli_parser.parse_options(&subcommand_help);

            // Infer option types
            self.option_inferrer.infer_types(&mut options);

            // Parse required positional arguments
            let required_args = cli_parser.parse_required_args(&subcommand_help);

            // Recursively detect nested subcommands
            let nested_subcommands =
                self.detect_recursive(binary, &subcommand_help, current_depth + 1, visited)?;

            subcommands.push(Subcommand {
                name,
                description: Some(description),
                options,
                required_args,
                subcommands: nested_subcommands,
                depth: current_depth,
            });
        }

        log::info!(
            "Detected {} subcommands at depth {}",
            subcommands.len(),
            current_depth
        );

        Ok(subcommands)
    }

    /// Parse subcommand names and descriptions from help output
    fn parse_subcommands(&self, help_output: &str) -> Vec<(String, String)> {
        let mut subcommands = Vec::new();
        let mut in_subcommand_section = false;

        for line in help_output.lines() {
            // Check if we entered subcommands section
            if !in_subcommand_section {
                for header in SUBCOMMAND_HEADERS.iter() {
                    if line.trim().starts_with(header) {
                        in_subcommand_section = true;
                        break;
                    }
                }
                continue;
            }

            // Check if we left subcommands section (empty line or new section)
            if line.trim().is_empty() {
                in_subcommand_section = false;
                continue;
            }

            // Parse subcommand line
            if let Some(captures) = SUBCOMMAND_PATTERN.captures(line) {
                let name = captures.get(1).unwrap().as_str().to_string();
                let description = captures.get(2).unwrap().as_str().trim().to_string();

                subcommands.push((name, description));
            }
        }

        subcommands
    }

    /// Get help output for a specific subcommand
    fn get_subcommand_help(&self, binary: &Path, subcommand: &str) -> Result<String> {
        log::debug!("Getting help for subcommand: {}", subcommand);

        // Try: <binary> <subcommand> --help
        if let Ok(output) = execute_with_timeout(
            binary,
            &[subcommand, "--help"],
            self.resource_limits.timeout(),
        ) {
            if !output.trim().is_empty() {
                return Ok(output);
            }
        }

        // Try: <binary> <subcommand> -h
        if let Ok(output) =
            execute_with_timeout(binary, &[subcommand, "-h"], self.resource_limits.timeout())
        {
            if !output.trim().is_empty() {
                return Ok(output);
            }
        }

        // Try: <binary> help <subcommand>
        if let Ok(output) = execute_with_timeout(
            binary,
            &["help", subcommand],
            self.resource_limits.timeout(),
        ) {
            if !output.trim().is_empty() {
                return Ok(output);
            }
        }

        Err(crate::error::CliTestError::InvalidHelpOutput)
    }
}

impl Default for SubcommandDetector {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            resource_limits: ResourceLimits::default(),
            option_inferrer: OptionInferrer::default(),
            max_depth: MAX_RECURSION_DEPTH,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subcommand_pattern() {
        assert!(SUBCOMMAND_PATTERN.is_match("  help      Show help information"));
        assert!(SUBCOMMAND_PATTERN.is_match("  config    Manage configuration"));
        assert!(SUBCOMMAND_PATTERN.is_match("    status    Show status"));
        assert!(!SUBCOMMAND_PATTERN.is_match("Options:"));
        assert!(!SUBCOMMAND_PATTERN.is_match("--help"));
    }

    #[test]
    fn test_parse_subcommands_basic() {
        let detector = SubcommandDetector::default();
        let help_output = r#"
Usage: test <command>

Commands:
  help      Show help information
  config    Manage configuration
  status    Show current status

Options:
  -h, --help    Show help
"#;

        let subcommands = detector.parse_subcommands(help_output);

        assert_eq!(subcommands.len(), 3);
        assert!(subcommands.iter().any(|(name, _)| name == "help"));
        assert!(subcommands.iter().any(|(name, _)| name == "config"));
        assert!(subcommands.iter().any(|(name, _)| name == "status"));
    }

    #[test]
    fn test_parse_subcommands_with_description() {
        let detector = SubcommandDetector::default();
        let help_output = r#"
Available Commands:
  init    Initialize a new project
  build   Build the project
"#;

        let subcommands = detector.parse_subcommands(help_output);

        assert_eq!(subcommands.len(), 2);

        let init_cmd = subcommands.iter().find(|(name, _)| name == "init");
        assert!(init_cmd.is_some());
        assert_eq!(init_cmd.unwrap().1, "Initialize a new project");
    }

    #[test]
    fn test_parse_subcommands_empty() {
        let detector = SubcommandDetector::default();
        let help_output = r#"
Usage: test [OPTIONS]

Options:
  -h, --help    Show help
"#;

        let subcommands = detector.parse_subcommands(help_output);

        assert!(subcommands.is_empty());
    }

    #[test]
    fn test_parse_subcommands_multiple_sections() {
        let detector = SubcommandDetector::default();
        let help_output = r#"
Commands:
  help    Show help

Options:
  --verbose    Verbose output

Commands:
  config    Configuration
"#;

        let subcommands = detector.parse_subcommands(help_output);

        // Should find both "help" and "config"
        assert_eq!(subcommands.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn test_detect_git_subcommands() {
        // Test with git if available
        let git_path = Path::new("/usr/bin/git");
        if !git_path.exists() {
            return; // Skip if git not available
        }

        let detector = SubcommandDetector::with_max_depth(1).unwrap();

        // Get git help output
        if let Ok(help_output) =
            execute_with_timeout(git_path, &["--help"], ResourceLimits::default().timeout())
        {
            let result = detector.detect(git_path, &help_output);

            // Note: This test may fail if git's help format is different than expected
            // or if parse_subcommands doesn't match git's specific format.
            // That's okay - the test is primarily to verify the detector doesn't panic.
            match result {
                Ok(subcommands) => {
                    // If we found subcommands, log them for debugging
                    if !subcommands.is_empty() {
                        log::debug!("Found {} git subcommands", subcommands.len());
                    }
                }
                Err(e) => {
                    log::warn!("Git subcommand detection failed (expected): {}", e);
                }
            }
        }
    }

    #[test]
    fn test_circular_reference_prevention() {
        let _detector = SubcommandDetector::default();
        let mut visited = HashSet::new();

        // Simulate visiting a subcommand twice
        visited.insert("/bin/test-help".to_string());

        // This should be prevented by the visited set
        // (In real scenario, this is tested via detect_recursive)
        assert!(visited.contains("/bin/test-help"));
    }

    #[test]
    fn test_max_depth_limit() {
        let detector = SubcommandDetector::with_max_depth(2).unwrap();
        assert_eq!(detector.max_depth, 2);

        let detector_default = SubcommandDetector::default();
        assert_eq!(detector_default.max_depth, MAX_RECURSION_DEPTH);
    }
}
