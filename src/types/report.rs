use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::test_priority::TestPriority;

/// Test execution result for a single test case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestResult {
    /// Test name from BATS
    pub name: String,

    /// Test status
    pub status: TestStatus,

    /// Duration of test execution
    pub duration: Duration,

    /// Output from the test (stdout + stderr)
    pub output: String,

    /// Error message if test failed
    pub error_message: Option<String>,

    /// File path where test is defined
    pub file_path: String,

    /// Line number in BATS file
    pub line_number: Option<usize>,

    /// Test tags extracted from BATS comments
    #[serde(default)]
    pub tags: Vec<String>,

    /// Test priority (extracted from tags or metadata)
    #[serde(default)]
    pub priority: TestPriority,
}

/// Test execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// Test passed
    Passed,

    /// Test failed
    Failed,

    /// Test was skipped
    Skipped,

    /// Test timed out
    Timeout,
}

impl TestStatus {
    /// Check if status represents a failure
    pub fn is_failure(&self) -> bool {
        matches!(self, TestStatus::Failed | TestStatus::Timeout)
    }

    /// Check if status represents success
    pub fn is_success(&self) -> bool {
        matches!(self, TestStatus::Passed)
    }
}

/// Security vulnerability finding from security check tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityFinding {
    /// Test name that detected this vulnerability
    pub test_name: String,

    /// Vulnerability category (e.g., "injection", "path-traversal")
    pub category: String,

    /// Severity level
    pub severity: SecuritySeverity,

    /// Description of the vulnerability
    pub description: String,

    /// Test output showing the vulnerability
    pub evidence: String,

    /// File path where the security test is defined
    pub test_file: String,
}

/// Security severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySeverity {
    /// Critical security issue
    Critical,

    /// High severity issue
    High,

    /// Medium severity issue
    Medium,

    /// Low severity issue
    Low,

    /// Informational finding
    Info,
}

impl SecuritySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Info => "info",
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self {
            Self::Critical => "danger",
            Self::High => "warning",
            Self::Medium => "warning",
            Self::Low => "info",
            Self::Info => "secondary",
        }
    }
}

/// Test suite representing a single BATS file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Name of the test suite (usually BATS filename)
    pub name: String,

    /// File path to BATS file
    pub file_path: String,

    /// All test results in this suite
    pub tests: Vec<TestResult>,

    /// Total duration of suite execution
    pub duration: Duration,

    /// Timestamp when suite started
    pub started_at: DateTime<Utc>,

    /// Timestamp when suite finished
    pub finished_at: DateTime<Utc>,
}

impl TestSuite {
    /// Count passed tests
    pub fn passed_count(&self) -> usize {
        self.tests.iter().filter(|t| t.status.is_success()).count()
    }

    /// Count failed tests
    pub fn failed_count(&self) -> usize {
        self.tests.iter().filter(|t| t.status.is_failure()).count()
    }

    /// Count skipped tests
    pub fn skipped_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|t| t.status == TestStatus::Skipped)
            .count()
    }

    /// Total number of tests
    pub fn total_count(&self) -> usize {
        self.tests.len()
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_count() == 0 {
            0.0
        } else {
            self.passed_count() as f64 / self.total_count() as f64
        }
    }
}

/// Complete test report aggregating all suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Binary name being tested
    pub binary_name: String,

    /// Binary version (if available)
    pub binary_version: Option<String>,

    /// All test suites executed
    pub suites: Vec<TestSuite>,

    /// Total execution time
    pub total_duration: Duration,

    /// Timestamp when tests started
    pub started_at: DateTime<Utc>,

    /// Timestamp when tests finished
    pub finished_at: DateTime<Utc>,

    /// Environment information
    pub environment: EnvironmentInfo,

    /// Security vulnerabilities detected (extracted from SecurityCheck tests)
    #[serde(default)]
    pub security_findings: Vec<SecurityFinding>,
}

impl TestReport {
    /// Total number of tests across all suites
    pub fn total_tests(&self) -> usize {
        self.suites.iter().map(|s| s.total_count()).sum()
    }

    /// Total passed tests
    pub fn total_passed(&self) -> usize {
        self.suites.iter().map(|s| s.passed_count()).sum()
    }

    /// Total failed tests
    pub fn total_failed(&self) -> usize {
        self.suites.iter().map(|s| s.failed_count()).sum()
    }

    /// Total skipped tests
    pub fn total_skipped(&self) -> usize {
        self.suites.iter().map(|s| s.skipped_count()).sum()
    }

    /// Overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tests() == 0 {
            0.0
        } else {
            self.total_passed() as f64 / self.total_tests() as f64
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.total_failed() == 0
    }

    /// Count tests by priority level
    pub fn tests_by_priority(&self, priority: TestPriority) -> Vec<&TestResult> {
        self.suites
            .iter()
            .flat_map(|suite| &suite.tests)
            .filter(|test| test.priority == priority)
            .collect()
    }

    /// Count security check tests
    pub fn security_check_tests(&self) -> Vec<&TestResult> {
        self.tests_by_priority(TestPriority::SecurityCheck)
    }

    /// Count passed security checks
    pub fn passed_security_checks(&self) -> usize {
        self.security_check_tests()
            .iter()
            .filter(|test| test.status.is_success())
            .count()
    }

    /// Count failed security checks (potential vulnerabilities)
    pub fn failed_security_checks(&self) -> usize {
        self.security_check_tests()
            .iter()
            .filter(|test| test.status.is_failure())
            .count()
    }

    /// Count template quality tests (non-security)
    pub fn template_quality_tests(&self) -> Vec<&TestResult> {
        self.suites
            .iter()
            .flat_map(|suite| &suite.tests)
            .filter(|test| !test.priority.is_security_check())
            .collect()
    }

    /// Count passed template quality tests
    pub fn passed_template_quality(&self) -> usize {
        self.template_quality_tests()
            .iter()
            .filter(|test| test.status.is_success())
            .count()
    }

    /// Count failed template quality tests
    pub fn failed_template_quality(&self) -> usize {
        self.template_quality_tests()
            .iter()
            .filter(|test| test.status.is_failure())
            .count()
    }

    /// Template quality success rate (excludes security checks)
    pub fn template_quality_rate(&self) -> f64 {
        let total = self.template_quality_tests().len();
        if total == 0 {
            0.0
        } else {
            self.passed_template_quality() as f64 / total as f64
        }
    }

    /// Security check success rate
    pub fn security_check_rate(&self) -> f64 {
        let total = self.security_check_tests().len();
        if total == 0 {
            0.0
        } else {
            self.passed_security_checks() as f64 / total as f64
        }
    }

    /// Number of detected vulnerabilities (failed security checks)
    pub fn vulnerability_count(&self) -> usize {
        self.security_findings.len()
    }
}

/// Environment information for the test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system name
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Shell used for testing
    pub shell: String,

    /// Shell version
    pub shell_version: String,

    /// BATS version
    pub bats_version: String,

    /// Hostname where tests ran
    pub hostname: String,

    /// User who ran tests
    pub user: String,
}

impl Default for EnvironmentInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            os_version: "unknown".to_string(),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
            shell_version: "unknown".to_string(),
            bats_version: "unknown".to_string(),
            hostname: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_failure() {
        assert!(TestStatus::Failed.is_failure());
        assert!(TestStatus::Timeout.is_failure());
        assert!(!TestStatus::Passed.is_failure());
        assert!(!TestStatus::Skipped.is_failure());
    }

    #[test]
    fn test_status_is_success() {
        assert!(TestStatus::Passed.is_success());
        assert!(!TestStatus::Failed.is_success());
        assert!(!TestStatus::Skipped.is_success());
        assert!(!TestStatus::Timeout.is_success());
    }

    #[test]
    fn test_suite_counts() {
        let suite = TestSuite {
            name: "test_suite".to_string(),
            file_path: "/path/to/test.bats".to_string(),
            tests: vec![
                TestResult {
                    name: "test1".to_string(),
                    status: TestStatus::Passed,
                    duration: Duration::from_millis(100),
                    output: "".to_string(),
                    error_message: None,
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(5),
                    tags: vec![],
                    priority: TestPriority::Important,
                },
                TestResult {
                    name: "test2".to_string(),
                    status: TestStatus::Failed,
                    duration: Duration::from_millis(200),
                    output: "error output".to_string(),
                    error_message: Some("assertion failed".to_string()),
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(10),
                    tags: vec![],
                    priority: TestPriority::Important,
                },
                TestResult {
                    name: "test3".to_string(),
                    status: TestStatus::Skipped,
                    duration: Duration::from_millis(0),
                    output: "".to_string(),
                    error_message: None,
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(15),
                    tags: vec![],
                    priority: TestPriority::Important,
                },
            ],
            duration: Duration::from_millis(300),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };

        assert_eq!(suite.total_count(), 3);
        assert_eq!(suite.passed_count(), 1);
        assert_eq!(suite.failed_count(), 1);
        assert_eq!(suite.skipped_count(), 1);
        assert!((suite.success_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_report_aggregation() {
        let suite1 = TestSuite {
            name: "suite1".to_string(),
            file_path: "/path/to/suite1.bats".to_string(),
            tests: vec![
                TestResult {
                    name: "test1".to_string(),
                    status: TestStatus::Passed,
                    duration: Duration::from_millis(100),
                    output: "".to_string(),
                    error_message: None,
                    file_path: "/path/to/suite1.bats".to_string(),
                    line_number: Some(5),
                    tags: vec![],
                    priority: TestPriority::Important,
                },
                TestResult {
                    name: "test2".to_string(),
                    status: TestStatus::Failed,
                    duration: Duration::from_millis(100),
                    output: "".to_string(),
                    error_message: Some("error".to_string()),
                    file_path: "/path/to/suite1.bats".to_string(),
                    line_number: Some(10),
                    tags: vec![],
                    priority: TestPriority::Important,
                },
            ],
            duration: Duration::from_millis(200),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };

        let suite2 = TestSuite {
            name: "suite2".to_string(),
            file_path: "/path/to/suite2.bats".to_string(),
            tests: vec![TestResult {
                name: "test3".to_string(),
                status: TestStatus::Passed,
                duration: Duration::from_millis(150),
                output: "".to_string(),
                error_message: None,
                file_path: "/path/to/suite2.bats".to_string(),
                line_number: Some(5),
                tags: vec![],
                priority: TestPriority::Important,
            }],
            duration: Duration::from_millis(150),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };

        let report = TestReport {
            binary_name: "test-cli".to_string(),
            binary_version: Some("1.0.0".to_string()),
            suites: vec![suite1, suite2],
            total_duration: Duration::from_millis(350),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            environment: EnvironmentInfo::default(),
            security_findings: vec![],
        };

        assert_eq!(report.total_tests(), 3);
        assert_eq!(report.total_passed(), 2);
        assert_eq!(report.total_failed(), 1);
        assert_eq!(report.total_skipped(), 0);
        assert!(!report.all_passed());
        assert!((report.success_rate() - 0.666).abs() < 0.01);
    }
}
