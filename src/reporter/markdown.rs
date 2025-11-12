use crate::error::Result;
use crate::types::{TestReport, TestStatus};
use std::fs;
use std::path::Path;

/// Markdown report generator
pub struct MarkdownReporter;

impl MarkdownReporter {
    /// Generate Markdown report from test results
    pub fn generate(report: &TestReport, output_path: &Path) -> Result<()> {
        let mut content = String::new();

        // Header
        content.push_str(&format!("# Test Report: {}\n\n", report.binary_name));

        if let Some(version) = &report.binary_version {
            content.push_str(&format!("**Version:** {}\n\n", version));
        }

        content.push_str(&format!(
            "**Generated:** {}\n\n",
            report.finished_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary section
        content.push_str("## Summary\n\n");

        let success_rate = (report.success_rate() * 100.0) as u32;
        let status_emoji = if report.all_passed() {
            "✅"
        } else if success_rate >= 80 {
            "⚠️"
        } else {
            "❌"
        };

        content.push_str(&format!(
            "**Overall Status:** {} {}% passed\n\n",
            status_emoji, success_rate
        ));

        content.push_str("| Metric | Value |\n");
        content.push_str("|--------|-------|\n");
        content.push_str(&format!("| Total Tests | {} |\n", report.total_tests()));
        content.push_str(&format!("| Passed | ✅ {} |\n", report.total_passed()));
        content.push_str(&format!("| Failed | ❌ {} |\n", report.total_failed()));
        content.push_str(&format!("| Skipped | ⏭️ {} |\n", report.total_skipped()));
        content.push_str(&format!(
            "| Duration | {:.2}s |\n",
            report.total_duration.as_secs_f64()
        ));
        content.push_str(&format!("| Suites | {} |\n\n", report.suites.len()));

        // Test Suites section
        content.push_str("## Test Suites\n\n");

        for suite in &report.suites {
            let suite_success_rate = (suite.success_rate() * 100.0) as u32;
            let suite_status = if suite.failed_count() == 0 {
                "✅"
            } else {
                "❌"
            };

            content.push_str(&format!(
                "### {} {} ({}%)\n\n",
                suite_status, suite.name, suite_success_rate
            ));

            content.push_str(&format!("**File:** `{}`\n\n", suite.file_path));
            content.push_str(&format!(
                "**Duration:** {:.2}s\n\n",
                suite.duration.as_secs_f64()
            ));

            // Suite summary table
            content.push_str("| Status | Count |\n");
            content.push_str("|--------|-------|\n");
            content.push_str(&format!("| Passed | {} |\n", suite.passed_count()));
            content.push_str(&format!("| Failed | {} |\n", suite.failed_count()));
            content.push_str(&format!("| Skipped | {} |\n", suite.skipped_count()));
            content.push_str(&format!("| Total | {} |\n\n", suite.total_count()));

            // Show failed tests if any
            let failed_tests: Vec<_> = suite
                .tests
                .iter()
                .filter(|t| t.status.is_failure())
                .collect();

            if !failed_tests.is_empty() {
                content.push_str("#### Failed Tests\n\n");

                for test in failed_tests {
                    content.push_str(&format!("- ❌ **{}**\n", test.name));
                    if let Some(error) = &test.error_message {
                        content.push_str(&format!("  - Error: `{}`\n", error));
                    }
                    if !test.output.is_empty() {
                        content.push_str(&format!(
                            "  - Output:\n    ```\n    {}\n    ```\n",
                            test.output
                        ));
                    }
                }
                content.push('\n');
            }
        }

        // Environment section
        content.push_str("## Environment\n\n");
        content.push_str("| Property | Value |\n");
        content.push_str("|----------|-------|\n");
        content.push_str(&format!(
            "| OS | {} {} |\n",
            report.environment.os, report.environment.os_version
        ));
        content.push_str(&format!(
            "| Shell | {} |\n",
            report.environment.shell_version
        ));
        content.push_str(&format!("| BATS | {} |\n", report.environment.bats_version));
        content.push_str(&format!("| Hostname | {} |\n", report.environment.hostname));
        content.push_str(&format!("| User | {} |\n", report.environment.user));

        // Detailed Results section
        content.push_str("\n## Detailed Results\n\n");

        for suite in &report.suites {
            content.push_str(&format!("### {}\n\n", suite.name));

            content.push_str("| # | Test Name | Status | Duration |\n");
            content.push_str("|---|-----------|--------|----------|\n");

            for (idx, test) in suite.tests.iter().enumerate() {
                let status_str = match test.status {
                    TestStatus::Passed => "✅ Passed",
                    TestStatus::Failed => "❌ Failed",
                    TestStatus::Skipped => "⏭️ Skipped",
                    TestStatus::Timeout => "⏱️ Timeout",
                };

                content.push_str(&format!(
                    "| {} | {} | {} | {:.0}ms |\n",
                    idx + 1,
                    test.name,
                    status_str,
                    test.duration.as_millis()
                ));
            }
            content.push('\n');
        }

        // Write to file
        fs::write(output_path, content)?;

        Ok(())
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
    fn test_markdown_generation() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        MarkdownReporter::generate(&report, output_path).unwrap();

        let content = fs::read_to_string(output_path).unwrap();

        // Verify header
        assert!(content.contains("# Test Report: test-cli"));
        assert!(content.contains("**Version:** 1.0.0"));

        // Verify summary
        assert!(content.contains("## Summary"));
        assert!(content.contains("Total Tests"));
        assert!(content.contains("| 2 |"));

        // Verify suite information
        assert!(content.contains("## Test Suites"));
        assert!(content.contains("test_suite"));

        // Verify detailed results
        assert!(content.contains("## Detailed Results"));
        assert!(content.contains("successful test"));
        assert!(content.contains("failed test"));
        assert!(content.contains("✅ Passed"));
        assert!(content.contains("❌ Failed"));

        // Verify environment section
        assert!(content.contains("## Environment"));
    }

    #[test]
    fn test_markdown_all_passed() {
        let suite = TestSuite {
            name: "all_pass".to_string(),
            file_path: "/test.bats".to_string(),
            tests: vec![TestResult {
                name: "test".to_string(),
                status: TestStatus::Passed,
                duration: Duration::from_millis(100),
                output: String::new(),
                error_message: None,
                file_path: "/test.bats".to_string(),
                line_number: None,
                tags: vec![],
                priority: crate::types::TestPriority::Important,
            }],
            duration: Duration::from_millis(100),
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };

        let report = TestReport {
            binary_name: "cli".to_string(),
            binary_version: None,
            suites: vec![suite],
            total_duration: Duration::from_millis(100),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            environment: EnvironmentInfo::default(),
            security_findings: vec![],
        };

        let temp_file = NamedTempFile::new().unwrap();
        MarkdownReporter::generate(&report, temp_file.path()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("✅ 100% passed"));
    }
}
