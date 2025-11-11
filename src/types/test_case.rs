use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

/// Test case definition for BATS generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique test identifier (e.g., "basic-001")
    pub id: String,

    /// Human-readable test name
    pub name: String,

    /// Test category
    pub category: TestCategory,

    /// Command to execute
    pub command: String,

    /// Expected exit code
    pub expected_exit: i32,

    /// Assertions to verify
    pub assertions: Vec<Assertion>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Test category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    /// Basic validation tests (help, version, exit codes)
    Basic,

    /// Help display tests
    Help,

    /// Security vulnerability tests
    Security,

    /// Path handling tests
    Path,

    /// Multi-shell compatibility tests
    MultiShell,

    /// Input validation tests
    InputValidation,

    /// Destructive operation tests
    DestructiveOps,

    /// Directory traversal prevention tests
    DirectoryTraversal,

    /// Performance tests
    Performance,
}

/// Assertion types for test validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assertion {
    /// Assert exact exit code
    ExitCode(i32),

    /// Assert output contains string
    OutputContains(String),

    /// Assert output matches regex pattern
    OutputMatches(String),

    /// Assert output does not contain string
    OutputNotContains(String),

    /// Assert file exists at path
    FileExists(PathBuf),

    /// Assert file does not exist at path
    FileNotExists(PathBuf),
}

impl TestCase {
    /// Create a new test case
    pub fn new(id: String, name: String, category: TestCategory, command: String) -> Self {
        Self {
            id,
            name,
            category,
            command,
            expected_exit: 0, // Default to success
            assertions: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add an assertion to this test case
    pub fn with_assertion(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Add a tag to this test case
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set expected exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.expected_exit = code;
        self
    }
}

impl TestCategory {
    /// Get category name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Help => "help",
            Self::Security => "security",
            Self::Path => "path",
            Self::MultiShell => "multi-shell",
            Self::InputValidation => "input-validation",
            Self::DestructiveOps => "destructive-ops",
            Self::DirectoryTraversal => "directory-traversal",
            Self::Performance => "performance",
        }
    }

    /// Get all available categories
    pub fn all() -> Vec<TestCategory> {
        vec![
            Self::Basic,
            Self::Help,
            Self::Security,
            Self::Path,
            Self::MultiShell,
            Self::InputValidation,
            Self::DestructiveOps,
            Self::DirectoryTraversal,
            Self::Performance,
        ]
    }

    /// Get default categories (excludes resource-intensive tests)
    ///
    /// Excludes:
    /// - DirectoryTraversal: Requires significant /tmp space (100MB+) and creates many files
    ///
    /// Use `--include-intensive` flag to include these categories
    pub fn default() -> Vec<TestCategory> {
        vec![
            Self::Basic,
            Self::Help,
            Self::Security,
            Self::Path,
            Self::MultiShell,
            Self::InputValidation,
            Self::DestructiveOps,
            Self::Performance,
        ]
    }

    /// Get resource-intensive test categories
    ///
    /// These tests require:
    /// - Significant /tmp disk space (100MB+)
    /// - Higher memory limits (2GB+)
    /// - More execution time
    pub fn intensive() -> Vec<TestCategory> {
        vec![Self::DirectoryTraversal]
    }
}

/// Error type for parsing TestCategory from string
#[derive(Debug)]
pub struct ParseCategoryError;

impl std::fmt::Display for ParseCategoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid test category")
    }
}

impl std::error::Error for ParseCategoryError {}

/// Implement FromStr trait for TestCategory
impl FromStr for TestCategory {
    type Err = ParseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "help" => Ok(Self::Help),
            "security" => Ok(Self::Security),
            "path" => Ok(Self::Path),
            "multi-shell" | "multishell" => Ok(Self::MultiShell),
            "input-validation" | "inputvalidation" => Ok(Self::InputValidation),
            "destructive-ops" | "destructiveops" => Ok(Self::DestructiveOps),
            "directory-traversal" | "directorytraversal" => Ok(Self::DirectoryTraversal),
            "performance" => Ok(Self::Performance),
            _ => Err(ParseCategoryError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_builder() {
        let test = TestCase::new(
            "basic-001".to_string(),
            "Help display test".to_string(),
            TestCategory::Basic,
            "cli-test --help".to_string(),
        )
        .with_exit_code(0)
        .with_assertion(Assertion::OutputContains("Usage:".to_string()))
        .with_tag("basic".to_string());

        assert_eq!(test.id, "basic-001");
        assert_eq!(test.expected_exit, 0);
        assert_eq!(test.assertions.len(), 1);
        assert_eq!(test.tags.len(), 1);
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(TestCategory::Security.as_str(), "security");
        assert_eq!(
            TestCategory::DirectoryTraversal.as_str(),
            "directory-traversal"
        );
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(
            "security".parse::<TestCategory>().unwrap(),
            TestCategory::Security
        );
        assert_eq!(
            "multi-shell".parse::<TestCategory>().unwrap(),
            TestCategory::MultiShell
        );
        assert_eq!(
            "multishell".parse::<TestCategory>().unwrap(),
            TestCategory::MultiShell
        );
        assert!("invalid".parse::<TestCategory>().is_err());
    }

    #[test]
    fn test_category_all() {
        let categories = TestCategory::all();
        assert_eq!(categories.len(), 9);
        assert!(categories.contains(&TestCategory::Security));
    }

    #[test]
    fn test_assertion_serialization() {
        let assertion = Assertion::OutputContains("test".to_string());
        let json = serde_json::to_string(&assertion).unwrap();
        let deserialized: Assertion = serde_json::from_str(&json).unwrap();

        match deserialized {
            Assertion::OutputContains(s) => assert_eq!(s, "test"),
            _ => panic!("Wrong assertion type"),
        }
    }
}
