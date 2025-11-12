use serde::{Deserialize, Serialize};

/// Test priority classification
///
/// Defines the importance level of a test case, which affects:
/// - Success rate calculation (SecurityCheck failures don't lower template quality)
/// - Report presentation (SecurityCheck failures shown as security findings)
/// - Filtering and categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestPriority {
    /// Critical test - must pass for production use
    ///
    /// Examples:
    /// - Basic functionality tests (help, version)
    /// - Core feature validation
    ///
    /// **Failure impact**: Indicates testing framework issue or critical CLI bug
    Critical,

    /// Important test - should pass but not blocking
    ///
    /// Examples:
    /// - Advanced features
    /// - Edge cases
    /// - Optional functionality
    ///
    /// **Failure impact**: Suggests improvement areas but not critical
    Important,

    /// Security vulnerability check - failure indicates target tool issue
    ///
    /// Examples:
    /// - Input validation tests (injection, null bytes)
    /// - Path traversal prevention
    /// - Buffer overflow handling
    ///
    /// **Failure impact**: Target CLI has security vulnerabilities (not framework issue)
    ///
    /// **Special handling**:
    /// - Failures don't reduce "Template Quality" metric
    /// - Displayed separately in "Security Findings" section
    /// - Treated as successful vulnerability detection
    SecurityCheck,
}

impl TestPriority {
    /// Get priority name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Important => "important",
            Self::SecurityCheck => "security_check",
        }
    }

    /// Check if this is a security check
    pub fn is_security_check(&self) -> bool {
        matches!(self, Self::SecurityCheck)
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Important => "Important",
            Self::SecurityCheck => "Security Check",
        }
    }

    /// Get badge color for HTML reports
    pub fn badge_color(&self) -> &'static str {
        match self {
            Self::Critical => "danger",
            Self::Important => "warning",
            Self::SecurityCheck => "info",
        }
    }
}

impl Default for TestPriority {
    fn default() -> Self {
        Self::Important
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_as_str() {
        assert_eq!(TestPriority::Critical.as_str(), "critical");
        assert_eq!(TestPriority::Important.as_str(), "important");
        assert_eq!(TestPriority::SecurityCheck.as_str(), "security_check");
    }

    #[test]
    fn test_is_security_check() {
        assert!(!TestPriority::Critical.is_security_check());
        assert!(!TestPriority::Important.is_security_check());
        assert!(TestPriority::SecurityCheck.is_security_check());
    }

    #[test]
    fn test_default() {
        let priority: TestPriority = Default::default();
        assert_eq!(priority, TestPriority::Important);
    }

    #[test]
    fn test_serialization() {
        let priority = TestPriority::SecurityCheck;
        let json = serde_json::to_string(&priority).unwrap();
        assert_eq!(json, r#""security_check""#);

        let deserialized: TestPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, priority);
    }
}
