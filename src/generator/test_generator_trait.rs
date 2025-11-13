use crate::error::Result;
use crate::types::analysis::CliAnalysis;
use crate::types::test_case::TestCategory;

/// Common interface for all test generators
///
/// This trait provides a unified interface for generating tests in different formats:
/// - BatsGenerator: Generates BATS shell test scripts
/// - AssertCmdGenerator: Generates Rust tests using assert_cmd crate
/// - SnapboxGenerator: Generates snapshot tests (future implementation)
///
/// Design rationale:
/// - Maintainability: Single source of truth for test generation logic
/// - Extensibility: Easy to add new test formats (e.g., pytest, Go tests)
/// - Consistency: All generators follow the same pattern
pub trait TestGenerator {
    /// Generate tests for a specific category
    ///
    /// # Arguments
    ///
    /// * `analysis` - CLI analysis results containing subcommands, options, etc.
    /// * `category` - Test category (basic, security, help, etc.)
    ///
    /// # Returns
    ///
    /// Generated test code as a String
    ///
    /// # Example
    ///
    /// ```ignore
    /// let generator = AssertCmdGenerator::new()?;
    /// let test_code = generator.generate(&analysis, TestCategory::Basic)?;
    /// ```
    fn generate(&self, analysis: &CliAnalysis, category: TestCategory) -> Result<String>;

    /// Generate all test categories
    ///
    /// Convenience method to generate tests for all standard categories.
    ///
    /// # Arguments
    ///
    /// * `analysis` - CLI analysis results
    ///
    /// # Returns
    ///
    /// Map of category name to generated test code
    fn generate_all(&self, analysis: &CliAnalysis) -> Result<Vec<(TestCategory, String)>> {
        let categories = TestCategory::standard_categories();
        let mut results = Vec::new();

        for category in categories {
            let test_code = self.generate(analysis, category)?;
            results.push((category, test_code));
        }

        Ok(results)
    }

    /// Get the file extension for generated test files
    ///
    /// # Returns
    ///
    /// File extension (e.g., "bats", "rs", "snap")
    fn file_extension(&self) -> &str;

    /// Get the generator name
    ///
    /// # Returns
    ///
    /// Human-readable name (e.g., "BATS", "assert_cmd", "snapbox")
    fn name(&self) -> &str;
}

/// Factory function to create a test generator by name
///
/// # Arguments
///
/// * `format` - Generator format name ("bats", "assert_cmd", "snapbox")
///
/// # Returns
///
/// Boxed TestGenerator instance
///
/// # Example
///
/// ```ignore
/// let generator = create_generator("assert_cmd")?;
/// let test_code = generator.generate(&analysis, TestCategory::Security)?;
/// ```
pub fn create_generator(format: &str) -> Result<Box<dyn TestGenerator>> {
    match format.to_lowercase().as_str() {
        "bats" => {
            // Use existing BatsGenerator (to be refactored to implement TestGenerator)
            Err(crate::error::CliTestError::InvalidFormat(
                "BATS generator not yet refactored to use TestGenerator trait".to_string(),
            ))
        }
        "assert_cmd" | "assert-cmd" => {
            // Create AssertCmdGenerator
            Err(crate::error::CliTestError::InvalidFormat(
                "AssertCmdGenerator not yet implemented".to_string(),
            ))
        }
        "snapbox" => {
            // Future implementation
            Err(crate::error::CliTestError::InvalidFormat(
                "Snapbox generator not yet implemented".to_string(),
            ))
        }
        _ => Err(crate::error::CliTestError::InvalidFormat(format!(
            "Unknown generator format: {}",
            format
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator_invalid_format() {
        let result = create_generator("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_generator_not_implemented() {
        // These should fail until implemented
        assert!(create_generator("bats").is_err());
        assert!(create_generator("assert_cmd").is_err());
        assert!(create_generator("snapbox").is_err());
    }
}
