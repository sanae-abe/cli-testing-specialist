use crate::error::Result;
use crate::types::TestReport;
use std::fs;
use std::path::Path;

/// JSON report generator
pub struct JsonReporter;

impl JsonReporter {
    /// Generate JSON report from test results
    pub fn generate(report: &TestReport, output_path: &Path) -> Result<()> {
        // Serialize to pretty JSON
        let json = serde_json::to_string_pretty(report)?;

        // Write to file
        fs::write(output_path, json)?;

        Ok(())
    }

    /// Generate compact JSON report (minified)
    pub fn generate_compact(report: &TestReport, output_path: &Path) -> Result<()> {
        // Serialize to compact JSON
        let json = serde_json::to_string(report)?;

        // Write to file
        fs::write(output_path, json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EnvironmentInfo, TestResult, TestStatus, TestSuite};
    use chrono::Utc;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    fn create_test_report() -> TestReport {
        let suite = TestSuite {
            name: "test_suite".to_string(),
            file_path: "/path/to/test.bats".to_string(),
            tests: vec![
                TestResult {
                    name: "successful test".to_string(),
                    status: TestStatus::Passed,
                    duration: Duration::from_millis(150),
                    output: String::new(),
                    error_message: None,
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(5),
                    tags: vec![],
                    priority: crate::types::TestPriority::Important,
                },
                TestResult {
                    name: "failed test".to_string(),
                    status: TestStatus::Failed,
                    duration: Duration::from_millis(200),
                    output: "error output".to_string(),
                    error_message: Some("assertion failed".to_string()),
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(10),
                    tags: vec![],
                    priority: crate::types::TestPriority::Important,
                },
            ],
            duration: Duration::from_millis(350),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };

        TestReport {
            binary_name: "test-cli".to_string(),
            binary_version: Some("1.0.0".to_string()),
            suites: vec![suite],
            total_duration: Duration::from_millis(350),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            environment: EnvironmentInfo::default(),
            security_findings: vec![],
        }
    }

    #[test]
    fn test_json_generation() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        JsonReporter::generate(&report, output_path).unwrap();

        // Read and parse JSON
        let content = fs::read_to_string(output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify structure
        assert_eq!(parsed["binary_name"], "test-cli");
        assert_eq!(parsed["binary_version"], "1.0.0");
        assert!(parsed["suites"].is_array());
        assert_eq!(parsed["suites"].as_array().unwrap().len(), 1);

        // Verify suite data
        let suite = &parsed["suites"][0];
        assert_eq!(suite["name"], "test_suite");
        assert_eq!(suite["tests"].as_array().unwrap().len(), 2);

        // Verify test data
        let test1 = &suite["tests"][0];
        assert_eq!(test1["name"], "successful test");
        assert_eq!(test1["status"], "passed");

        let test2 = &suite["tests"][1];
        assert_eq!(test2["name"], "failed test");
        assert_eq!(test2["status"], "failed");
        assert_eq!(test2["error_message"], "assertion failed");
    }

    #[test]
    fn test_json_compact_generation() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        JsonReporter::generate_compact(&report, output_path).unwrap();

        // Read JSON
        let content = fs::read_to_string(output_path).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["binary_name"], "test-cli");

        // Verify it's compact (no pretty formatting)
        assert!(!content.contains("  ")); // No indentation
    }

    #[test]
    fn test_json_roundtrip() {
        let original = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        // Write
        JsonReporter::generate(&original, output_path).unwrap();

        // Read back
        let content = fs::read_to_string(output_path).unwrap();
        let deserialized: TestReport = serde_json::from_str(&content).unwrap();

        // Verify key fields
        assert_eq!(deserialized.binary_name, original.binary_name);
        assert_eq!(deserialized.binary_version, original.binary_version);
        assert_eq!(deserialized.suites.len(), original.suites.len());
        assert_eq!(
            deserialized.suites[0].tests.len(),
            original.suites[0].tests.len()
        );
    }
}
