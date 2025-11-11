use crate::error::Result;
use crate::types::{TestReport, TestStatus};
use std::fs;
use std::path::Path;

/// JUnit XML report generator
pub struct JunitReporter;

impl JunitReporter {
    /// Generate JUnit XML report from test results
    pub fn generate(report: &TestReport, output_path: &Path) -> Result<()> {
        let xml = Self::render_xml(report);
        fs::write(output_path, xml)?;
        Ok(())
    }

    /// Render complete JUnit XML document
    fn render_xml(report: &TestReport) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        // Root testsuites element
        xml.push_str(&format!(
            r#"<testsuites name="{}" tests="{}" failures="{}" errors="0" skipped="{}" time="{:.3}" timestamp="{}">"#,
            Self::xml_escape(&report.binary_name),
            report.total_tests(),
            report.total_failed(),
            report.total_skipped(),
            report.total_duration.as_secs_f64(),
            report.started_at.to_rfc3339(),
        ));
        xml.push('\n');

        // Add properties for environment
        xml.push_str("  <properties>\n");
        xml.push_str(&format!(
            r#"    <property name="os" value="{}"/>"#,
            Self::xml_escape(&format!(
                "{} {}",
                report.environment.os, report.environment.os_version
            ))
        ));
        xml.push('\n');
        xml.push_str(&format!(
            r#"    <property name="shell" value="{}"/>"#,
            Self::xml_escape(&report.environment.shell_version)
        ));
        xml.push('\n');
        xml.push_str(&format!(
            r#"    <property name="bats_version" value="{}"/>"#,
            Self::xml_escape(&report.environment.bats_version)
        ));
        xml.push('\n');
        xml.push_str(&format!(
            r#"    <property name="hostname" value="{}"/>"#,
            Self::xml_escape(&report.environment.hostname)
        ));
        xml.push('\n');
        if let Some(version) = &report.binary_version {
            xml.push_str(&format!(
                r#"    <property name="binary_version" value="{}"/>"#,
                Self::xml_escape(version)
            ));
            xml.push('\n');
        }
        xml.push_str("  </properties>\n");

        // Add each test suite
        for suite in &report.suites {
            xml.push_str(&Self::render_suite(suite));
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    /// Render a single test suite
    fn render_suite(suite: &crate::types::TestSuite) -> String {
        let mut xml = String::new();

        xml.push_str(&format!(
            r#"  <testsuite name="{}" tests="{}" failures="{}" errors="0" skipped="{}" time="{:.3}" timestamp="{}" file="{}">"#,
            Self::xml_escape(&suite.name),
            suite.total_count(),
            suite.failed_count(),
            suite.skipped_count(),
            suite.duration.as_secs_f64(),
            suite.started_at.to_rfc3339(),
            Self::xml_escape(&suite.file_path),
        ));
        xml.push('\n');

        // Add each test case
        for test in &suite.tests {
            xml.push_str(&Self::render_test(test, &suite.name));
        }

        xml.push_str("  </testsuite>\n");
        xml
    }

    /// Render a single test case
    fn render_test(test: &crate::types::TestResult, suite_name: &str) -> String {
        let mut xml = String::new();

        xml.push_str(&format!(
            r#"    <testcase name="{}" classname="{}" time="{:.3}""#,
            Self::xml_escape(&test.name),
            Self::xml_escape(suite_name),
            test.duration.as_secs_f64(),
        ));

        match test.status {
            TestStatus::Passed => {
                xml.push_str("/>\n");
            }
            TestStatus::Failed => {
                xml.push_str(">\n");
                let error_msg = test
                    .error_message
                    .as_deref()
                    .unwrap_or("Test failed without error message");
                xml.push_str(&format!(
                    r#"      <failure message="{}" type="AssertionError">"#,
                    Self::xml_escape(error_msg)
                ));
                xml.push('\n');
                if !test.output.is_empty() {
                    xml.push_str(&Self::xml_escape(&test.output));
                    xml.push('\n');
                }
                xml.push_str("      </failure>\n");
                xml.push_str("    </testcase>\n");
            }
            TestStatus::Skipped => {
                xml.push_str(">\n");
                xml.push_str(r#"      <skipped/>"#);
                xml.push('\n');
                xml.push_str("    </testcase>\n");
            }
            TestStatus::Timeout => {
                xml.push_str(">\n");
                xml.push_str(r#"      <error message="Test timed out" type="TimeoutError"/>"#);
                xml.push('\n');
                xml.push_str("    </testcase>\n");
            }
        }

        xml
    }

    /// Escape XML special characters
    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EnvironmentInfo, TestResult, TestSuite};
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
                },
                TestResult {
                    name: "failed test".to_string(),
                    status: TestStatus::Failed,
                    duration: Duration::from_millis(200),
                    output: "error output".to_string(),
                    error_message: Some("assertion failed".to_string()),
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(10),
                },
                TestResult {
                    name: "skipped test".to_string(),
                    status: TestStatus::Skipped,
                    duration: Duration::from_millis(0),
                    output: String::new(),
                    error_message: None,
                    file_path: "/path/to/test.bats".to_string(),
                    line_number: Some(15),
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
        }
    }

    #[test]
    fn test_junit_generation() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        JunitReporter::generate(&report, temp_file.path()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify XML structure
        assert!(content.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(content.contains("<testsuites"));
        assert!(content.contains("</testsuites>"));

        // Verify test suite
        assert!(content.contains(r#"<testsuite name="test_suite""#));
        assert!(content.contains(r#"tests="3""#));
        assert!(content.contains(r#"failures="1""#));
        assert!(content.contains(r#"skipped="1""#));

        // Verify test cases
        assert!(content.contains(r#"<testcase name="successful test""#));
        assert!(content.contains(r#"<testcase name="failed test""#));
        assert!(content.contains(r#"<testcase name="skipped test""#));

        // Verify failure element
        assert!(content.contains("<failure"));
        assert!(content.contains("assertion failed"));

        // Verify skipped element
        assert!(content.contains("<skipped/>"));

        // Verify properties
        assert!(content.contains("<properties>"));
        assert!(content.contains(r#"<property name="os""#));
        assert!(content.contains(r#"<property name="bats_version""#));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(
            JunitReporter::xml_escape("Test <xml> & \"quotes\""),
            "Test &lt;xml&gt; &amp; &quot;quotes&quot;"
        );
        assert_eq!(JunitReporter::xml_escape("A & B"), "A &amp; B");
        assert_eq!(JunitReporter::xml_escape("'single'"), "&apos;single&apos;");
    }

    #[test]
    fn test_junit_valid_xml() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        JunitReporter::generate(&report, temp_file.path()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();

        // Basic XML well-formedness checks
        assert_eq!(content.matches("<testsuites").count(), 1);
        assert_eq!(content.matches("</testsuites>").count(), 1);
        assert_eq!(content.matches("<testsuite ").count(), 1); // Space ensures we match opening tag only
        assert_eq!(content.matches("</testsuite>").count(), 1);

        // Count testcase elements (3 tests)
        assert_eq!(content.matches("<testcase").count(), 3);

        // Verify properties are well-formed
        let property_count = content.matches("<property").count();
        assert!(property_count >= 4); // At least os, shell, bats_version, hostname
    }
}
