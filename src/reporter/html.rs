use crate::error::Result;
use crate::types::{TestReport, TestStatus};
use std::fs;
use std::path::Path;

/// HTML report generator with embedded Bootstrap 5
pub struct HtmlReporter;

impl HtmlReporter {
    /// Generate HTML report from test results
    pub fn generate(report: &TestReport, output_path: &Path) -> Result<()> {
        let html = Self::render_html(report);
        fs::write(output_path, html)?;
        Ok(())
    }

    /// Render complete HTML document
    fn render_html(report: &TestReport) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Report - {}</title>
    {}
    {}
</head>
<body>
    <div class="container py-5">
        {}
        {}
        {}
        {}
        {}
    </div>
    {}
</body>
</html>"#,
            report.binary_name,
            Self::embedded_css(),
            Self::embedded_bootstrap_css(),
            Self::render_header(report),
            Self::render_summary(report),
            Self::render_suite_overview(report),
            Self::render_detailed_results(report),
            Self::render_environment(report),
            Self::embedded_javascript(),
        )
    }

    /// Render page header
    fn render_header(report: &TestReport) -> String {
        let version_badge = if let Some(version) = &report.binary_version {
            format!(r#"<span class="badge bg-secondary">{}</span>"#, version)
        } else {
            String::new()
        };

        format!(
            r#"<header class="mb-5">
            <h1 class="display-4">
                Test Report: {} {}
            </h1>
            <p class="text-muted">
                Generated: {}
            </p>
        </header>"#,
            report.binary_name,
            version_badge,
            report.finished_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    /// Render summary section
    fn render_summary(report: &TestReport) -> String {
        let success_rate = (report.success_rate() * 100.0) as u32;
        let alert_class = if report.all_passed() {
            "alert-success"
        } else if success_rate >= 80 {
            "alert-warning"
        } else {
            "alert-danger"
        };

        let status_icon = if report.all_passed() { "✅" } else { "❌" };

        format!(
            r#"<section class="mb-5">
            <h2>Summary</h2>
            <div class="alert {} d-flex align-items-center" role="alert">
                <div class="me-3" style="font-size: 2rem;">{}</div>
                <div>
                    <h4 class="alert-heading mb-1">Overall Status: {}% passed</h4>
                    <p class="mb-0">{} of {} tests passed</p>
                </div>
            </div>

            <div class="row g-3 mb-4">
                <div class="col-md-3">
                    <div class="card border-success">
                        <div class="card-body text-center">
                            <h3 class="text-success">{}</h3>
                            <p class="card-text text-muted">Passed</p>
                        </div>
                    </div>
                </div>
                <div class="col-md-3">
                    <div class="card border-danger">
                        <div class="card-body text-center">
                            <h3 class="text-danger">{}</h3>
                            <p class="card-text text-muted">Failed</p>
                        </div>
                    </div>
                </div>
                <div class="col-md-3">
                    <div class="card border-secondary">
                        <div class="card-body text-center">
                            <h3 class="text-secondary">{}</h3>
                            <p class="card-text text-muted">Skipped</p>
                        </div>
                    </div>
                </div>
                <div class="col-md-3">
                    <div class="card border-info">
                        <div class="card-body text-center">
                            <h3 class="text-info">{:.2}s</h3>
                            <p class="card-text text-muted">Duration</p>
                        </div>
                    </div>
                </div>
            </div>

            <div class="progress" style="height: 30px;">
                <div class="progress-bar bg-success" role="progressbar" style="width: {}%;" aria-valuenow="{}" aria-valuemin="0" aria-valuemax="100">
                    {}%
                </div>
                <div class="progress-bar bg-danger" role="progressbar" style="width: {}%;" aria-valuenow="{}" aria-valuemin="0" aria-valuemax="100">
                </div>
            </div>
        </section>"#,
            alert_class,
            status_icon,
            success_rate,
            report.total_passed(),
            report.total_tests(),
            report.total_passed(),
            report.total_failed(),
            report.total_skipped(),
            report.total_duration.as_secs_f64(),
            success_rate,
            success_rate,
            success_rate,
            100 - success_rate,
            100 - success_rate,
        )
    }

    /// Render suite overview
    fn render_suite_overview(report: &TestReport) -> String {
        let mut suites_html = String::new();

        for suite in &report.suites {
            let success_rate = (suite.success_rate() * 100.0) as u32;
            let status_badge = if suite.failed_count() == 0 {
                String::from(r#"<span class="badge bg-success">✅ All Passed</span>"#)
            } else {
                format!(
                    r#"<span class="badge bg-danger">❌ {} Failed</span>"#,
                    suite.failed_count()
                )
            };

            suites_html.push_str(&format!(
                r#"
            <div class="card mb-3">
                <div class="card-header d-flex justify-content-between align-items-center">
                    <h5 class="mb-0">{}</h5>
                    {}
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-md-8">
                            <p class="mb-1"><strong>File:</strong> <code>{}</code></p>
                            <p class="mb-1"><strong>Duration:</strong> {:.2}s</p>
                            <p class="mb-0">
                                <span class="badge bg-success">{} passed</span>
                                <span class="badge bg-danger">{} failed</span>
                                <span class="badge bg-secondary">{} skipped</span>
                            </p>
                        </div>
                        <div class="col-md-4">
                            <div class="progress" style="height: 20px;">
                                <div class="progress-bar bg-success" role="progressbar" style="width: {}%;">{}%</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>"#,
                suite.name,
                status_badge,
                suite.file_path,
                suite.duration.as_secs_f64(),
                suite.passed_count(),
                suite.failed_count(),
                suite.skipped_count(),
                success_rate,
                success_rate,
            ));
        }

        format!(
            r#"<section class="mb-5">
            <h2>Test Suites</h2>
            {}
        </section>"#,
            suites_html
        )
    }

    /// Render detailed results
    fn render_detailed_results(report: &TestReport) -> String {
        let mut details_html = String::new();

        for suite in &report.suites {
            let mut tests_html = String::new();

            for test in &suite.tests {
                let (status_class, status_icon, status_text) = match test.status {
                    TestStatus::Passed => ("table-success", "✅", "Passed"),
                    TestStatus::Failed => ("table-danger", "❌", "Failed"),
                    TestStatus::Skipped => ("table-secondary", "⏭️", "Skipped"),
                    TestStatus::Timeout => ("table-warning", "⏱️", "Timeout"),
                };

                let error_row = if let Some(error) = &test.error_message {
                    format!(
                        r#"<tr><td colspan="4" class="bg-light"><small class="text-danger">Error: {}</small></td></tr>"#,
                        Self::html_escape(error)
                    )
                } else {
                    String::new()
                };

                tests_html.push_str(&format!(
                    r#"<tr class="{}">
                        <td>{} {}</td>
                        <td>{}</td>
                        <td>{:.0}ms</td>
                        <td><span class="badge bg-{}">{}</span></td>
                    </tr>{}"#,
                    status_class,
                    status_icon,
                    Self::html_escape(&test.name),
                    suite.name,
                    test.duration.as_millis(),
                    if test.status.is_success() {
                        "success"
                    } else if test.status.is_failure() {
                        "danger"
                    } else {
                        "secondary"
                    },
                    status_text,
                    error_row,
                ));
            }

            details_html.push_str(&tests_html);
        }

        format!(
            r#"<section class="mb-5">
            <h2>Detailed Results</h2>
            <div class="mb-3">
                <input type="text" id="searchInput" class="form-control" placeholder="Search tests...">
            </div>
            <div class="btn-group mb-3" role="group">
                <button type="button" class="btn btn-outline-primary" onclick="filterTests('all')">All</button>
                <button type="button" class="btn btn-outline-success" onclick="filterTests('passed')">Passed</button>
                <button type="button" class="btn btn-outline-danger" onclick="filterTests('failed')">Failed</button>
                <button type="button" class="btn btn-outline-secondary" onclick="filterTests('skipped')">Skipped</button>
            </div>
            <div class="table-responsive">
                <table class="table table-striped table-hover" id="resultsTable">
                    <thead class="table-dark">
                        <tr>
                            <th>Test Name</th>
                            <th>Suite</th>
                            <th>Duration</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
        </section>"#,
            details_html
        )
    }

    /// Render environment information
    fn render_environment(report: &TestReport) -> String {
        format!(
            r#"<section class="mb-5">
            <h2>Environment</h2>
            <div class="table-responsive">
                <table class="table table-bordered">
                    <tbody>
                        <tr>
                            <th style="width: 30%;">Operating System</th>
                            <td>{} {}</td>
                        </tr>
                        <tr>
                            <th>Shell</th>
                            <td>{}</td>
                        </tr>
                        <tr>
                            <th>BATS Version</th>
                            <td>{}</td>
                        </tr>
                        <tr>
                            <th>Hostname</th>
                            <td>{}</td>
                        </tr>
                        <tr>
                            <th>User</th>
                            <td>{}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </section>"#,
            report.environment.os,
            report.environment.os_version,
            report.environment.shell_version,
            report.environment.bats_version,
            report.environment.hostname,
            report.environment.user,
        )
    }

    /// Escape HTML special characters
    fn html_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// Embedded Bootstrap 5 CSS (minimal subset)
    fn embedded_bootstrap_css() -> &'static str {
        r#"<style>
        /* Bootstrap 5 minimal subset - embedded to avoid CDN dependency */
        *,*::before,*::after{box-sizing:border-box}
        body{margin:0;font-family:system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif;font-size:1rem;font-weight:400;line-height:1.5;color:#212529;background-color:#fff}
        h1,h2,h3,h4,h5{margin-top:0;margin-bottom:.5rem;font-weight:500;line-height:1.2}
        h1{font-size:calc(1.375rem + 1.5vw)}h2{font-size:calc(1.325rem + .9vw)}h3{font-size:calc(1.3rem + .6vw)}h4{font-size:calc(1.275rem + .3vw)}h5{font-size:1.25rem}
        p{margin-top:0;margin-bottom:1rem}
        .container{width:100%;padding-right:.75rem;padding-left:.75rem;margin-right:auto;margin-left:auto}
        @media (min-width:1200px){.container{max-width:1140px}}
        .row{display:flex;flex-wrap:wrap;margin-right:-.75rem;margin-left:-.75rem}
        .col-md-3,.col-md-4,.col-md-8{position:relative;width:100%;padding-right:.75rem;padding-left:.75rem}
        @media (min-width:768px){.col-md-3{flex:0 0 25%;max-width:25%}.col-md-4{flex:0 0 33.333%;max-width:33.333%}.col-md-8{flex:0 0 66.666%;max-width:66.666%}}
        .g-3{margin-right:-0.75rem;margin-left:-0.75rem}.g-3>*{padding-right:0.75rem;padding-left:0.75rem;margin-bottom:1rem}
        .d-flex{display:flex}.align-items-center{align-items:center}.justify-content-between{justify-content:space-between}
        .mb-0{margin-bottom:0}.mb-1{margin-bottom:.25rem}.mb-3{margin-bottom:1rem}.mb-4{margin-bottom:1.5rem}.mb-5{margin-bottom:3rem}.me-3{margin-right:1rem}.py-5{padding-top:3rem;padding-bottom:3rem}
        .card{position:relative;display:flex;flex-direction:column;min-width:0;word-wrap:break-word;background-color:#fff;border:1px solid rgba(0,0,0,.125);border-radius:.25rem}
        .card-body{flex:1 1 auto;padding:1rem}
        .card-header{padding:.5rem 1rem;margin-bottom:0;background-color:rgba(0,0,0,.03);border-bottom:1px solid rgba(0,0,0,.125)}
        .card-header h5{margin:0}
        .card-text{margin-bottom:0}
        .border-success{border-color:#198754!important}.border-danger{border-color:#dc3545!important}.border-secondary{border-color:#6c757d!important}.border-info{border-color:#0dcaf0!important}
        .text-success{color:#198754}.text-danger{color:#dc3545}.text-secondary{color:#6c757d}.text-info{color:#0dcaf0}.text-muted{color:#6c757d}.text-center{text-align:center}
        .badge{display:inline-block;padding:.35em .65em;font-size:.75em;font-weight:700;line-height:1;text-align:center;white-space:nowrap;vertical-align:baseline;border-radius:.25rem}
        .bg-success{background-color:#198754!important;color:#fff}.bg-danger{background-color:#dc3545!important;color:#fff}.bg-secondary{background-color:#6c757d!important;color:#fff}.bg-info{background-color:#0dcaf0!important}.bg-light{background-color:#f8f9fa!important}
        .alert{position:relative;padding:1rem;margin-bottom:1rem;border:1px solid transparent;border-radius:.25rem}
        .alert-success{color:#0f5132;background-color:#d1e7dd;border-color:#badbcc}.alert-warning{color:#664d03;background-color:#fff3cd;border-color:#ffecb5}.alert-danger{color:#842029;background-color:#f8d7da;border-color:#f5c2c7}
        .alert-heading{color:inherit}
        .progress{display:flex;height:1rem;overflow:hidden;font-size:.75rem;background-color:#e9ecef;border-radius:.25rem}
        .progress-bar{display:flex;flex-direction:column;justify-content:center;overflow:hidden;color:#fff;text-align:center;white-space:nowrap;background-color:#0d6efd;transition:width .6s ease}
        .table{width:100%;margin-bottom:1rem;color:#212529;border-collapse:collapse}
        .table th,.table td{padding:.5rem;border-bottom:1px solid #dee2e6}
        .table-responsive{overflow-x:auto}
        .table-striped tbody tr:nth-of-type(odd){background-color:rgba(0,0,0,.05)}
        .table-hover tbody tr:hover{background-color:rgba(0,0,0,.075)}
        .table-bordered{border:1px solid #dee2e6}.table-bordered th,.table-bordered td{border:1px solid #dee2e6}
        .table-dark{color:#fff;background-color:#212529}
        .table-success{background-color:#d1e7dd}.table-danger{background-color:#f8d7da}.table-secondary{background-color:#e2e3e5}.table-warning{background-color:#fff3cd}
        .btn{display:inline-block;font-weight:400;line-height:1.5;text-align:center;text-decoration:none;vertical-align:middle;cursor:pointer;user-select:none;border:1px solid transparent;padding:.375rem .75rem;font-size:1rem;border-radius:.25rem;transition:color .15s ease-in-out}
        .btn-group{position:relative;display:inline-flex;vertical-align:middle}.btn-group>.btn{position:relative;flex:1 1 auto}
        .btn-outline-primary{color:#0d6efd;border-color:#0d6efd}.btn-outline-primary:hover{color:#fff;background-color:#0d6efd}
        .btn-outline-success{color:#198754;border-color:#198754}.btn-outline-success:hover{color:#fff;background-color:#198754}
        .btn-outline-danger{color:#dc3545;border-color:#dc3545}.btn-outline-danger:hover{color:#fff;background-color:#dc3545}
        .btn-outline-secondary{color:#6c757d;border-color:#6c757d}.btn-outline-secondary:hover{color:#fff;background-color:#6c757d}
        .form-control{display:block;width:100%;padding:.375rem .75rem;font-size:1rem;line-height:1.5;color:#212529;background-color:#fff;border:1px solid #ced4da;border-radius:.25rem}
        .display-4{font-size:3.5rem;font-weight:300;line-height:1.2}
        code{font-family:SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:.875em;color:#d63384;word-wrap:break-word}
        </style>"#
    }

    /// Custom CSS for additional styling
    fn embedded_css() -> &'static str {
        r#"<style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        .card h3 { font-size: 2.5rem; margin: 0; }
        .progress { box-shadow: inset 0 1px 2px rgba(0,0,0,.1); }
        .table-responsive { max-height: 600px; overflow-y: auto; }
        .filter-hidden { display: none !important; }
        </style>"#
    }

    /// Embedded JavaScript for interactive features
    fn embedded_javascript() -> &'static str {
        r#"<script>
        // Search functionality
        document.getElementById('searchInput').addEventListener('keyup', function() {
            const searchTerm = this.value.toLowerCase();
            const rows = document.querySelectorAll('#resultsTable tbody tr');

            rows.forEach(row => {
                const text = row.textContent.toLowerCase();
                row.style.display = text.includes(searchTerm) ? '' : 'none';
            });
        });

        // Filter functionality
        function filterTests(status) {
            const rows = document.querySelectorAll('#resultsTable tbody tr');

            rows.forEach(row => {
                if (status === 'all') {
                    row.style.display = '';
                } else {
                    const statusBadge = row.querySelector('.badge');
                    if (statusBadge) {
                        const badgeText = statusBadge.textContent.toLowerCase();
                        row.style.display = badgeText.includes(status) ? '' : 'none';
                    }
                }
            });

            // Update active button
            document.querySelectorAll('.btn-group .btn').forEach(btn => {
                btn.classList.remove('active');
            });
            event.target.classList.add('active');
        }
        </script>"#
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
    fn test_html_generation() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        HtmlReporter::generate(&report, temp_file.path()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify HTML structure
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("<html lang=\"en\">"));
        assert!(content.contains("</html>"));

        // Verify title
        assert!(content.contains("<title>Test Report - test-cli</title>"));

        // Verify header
        assert!(content.contains("Test Report: test-cli"));
        assert!(content.contains("1.0.0"));

        // Verify summary
        assert!(content.contains("Summary"));
        assert!(content.contains("Overall Status"));

        // Verify test results
        assert!(content.contains("successful test"));
        assert!(content.contains("failed test"));

        // Verify environment
        assert!(content.contains("Environment"));
        assert!(content.contains("Operating System"));

        // Verify JavaScript is embedded
        assert!(content.contains("<script>"));
        assert!(content.contains("searchInput"));
        assert!(content.contains("filterTests"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(
            HtmlReporter::html_escape("Test <script>alert('xss')</script>"),
            "Test &lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"
        );
        assert_eq!(HtmlReporter::html_escape("A & B"), "A &amp; B");
        assert_eq!(
            HtmlReporter::html_escape("\"quoted\""),
            "&quot;quoted&quot;"
        );
    }
}
