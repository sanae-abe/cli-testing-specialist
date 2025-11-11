use crate::error::{Error, Result};
use crate::types::{EnvironmentInfo, TestReport, TestResult, TestStatus, TestSuite};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// BATS test executor with TAP (Test Anything Protocol) parser
pub struct BatsExecutor {
    /// Timeout per test suite in seconds
    timeout: u64,

    /// Binary name being tested
    binary_name: String,

    /// Binary version (if available)
    binary_version: Option<String>,

    /// Categories to skip (optional)
    skip_categories: Option<Vec<String>>,
}

impl BatsExecutor {
    /// Create new BATS executor with default timeout (300 seconds)
    pub fn new(binary_name: String, binary_version: Option<String>) -> Self {
        Self {
            timeout: 300,
            binary_name,
            binary_version,
            skip_categories: None,
        }
    }

    /// Create new BATS executor with custom timeout
    pub fn with_timeout(binary_name: String, binary_version: Option<String>, timeout: u64) -> Self {
        Self {
            timeout,
            binary_name,
            binary_version,
            skip_categories: None,
        }
    }

    /// Set categories to skip
    pub fn with_skip_categories(mut self, skip: Vec<String>) -> Self {
        self.skip_categories = Some(skip);
        self
    }

    /// Verify BATS is installed and available
    pub fn verify_bats_installed() -> Result<String> {
        let output = Command::new("bats")
            .arg("--version")
            .output()
            .map_err(|e| {
                Error::BatsExecutionFailed(format!(
                    "BATS not found. Please install BATS: https://github.com/bats-core/bats-core\nError: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(Error::BatsExecutionFailed(
                "BATS is installed but --version failed".to_string(),
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        let version_str = version
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();

        info!("BATS version: {}", version_str);
        Ok(version_str)
    }

    /// Find all BATS files in a directory
    pub fn find_bats_files(test_dir: &Path) -> Result<Vec<PathBuf>> {
        if !test_dir.exists() {
            return Err(Error::Config(format!(
                "Test directory not found: {}",
                test_dir.display()
            )));
        }

        let mut bats_files = Vec::new();

        for entry in fs::read_dir(test_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "bats" {
                        bats_files.push(path);
                    }
                }
            }
        }

        if bats_files.is_empty() {
            return Err(Error::Config(format!(
                "No BATS files found in directory: {}",
                test_dir.display()
            )));
        }

        // Sort for consistent ordering
        bats_files.sort();
        info!("Found {} BATS files", bats_files.len());

        Ok(bats_files)
    }

    /// Execute all BATS files and generate report
    pub fn run_tests(&self, test_dir: &Path) -> Result<TestReport> {
        let start_time = Instant::now();
        let started_at = Utc::now();

        // Verify BATS is installed
        let bats_version = Self::verify_bats_installed()?;

        // Find all BATS files
        let mut bats_files = Self::find_bats_files(test_dir)?;

        // Filter out skipped categories if specified
        if let Some(ref skip_cats) = self.skip_categories {
            let original_count = bats_files.len();
            bats_files.retain(|path| {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    !skip_cats
                        .iter()
                        .any(|skip_cat| file_stem.contains(skip_cat))
                } else {
                    true
                }
            });
            let skipped_count = original_count - bats_files.len();
            if skipped_count > 0 {
                info!(
                    "Skipped {} test suite(s) based on skip categories",
                    skipped_count
                );
            }
        }

        info!("Executing {} test suites", bats_files.len());

        // Create single tokio runtime for all test executions
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            Error::BatsExecutionFailed(format!("Failed to create async runtime: {}", e))
        })?;

        // Create progress bar
        let pb = ProgressBar::new(bats_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        // Execute each BATS file
        let mut suites = Vec::new();
        for bats_file in bats_files.iter() {
            let suite_name = bats_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let suite_start_time = Instant::now();
            pb.set_message(format!(
                "Running {} (timeout: {}s)",
                suite_name, self.timeout
            ));

            match self.execute_suite(bats_file, &runtime) {
                Ok(suite) => {
                    let passed = suite.passed_count();
                    let total = suite.total_count();
                    let elapsed = suite_start_time.elapsed();

                    info!(
                        "Suite '{}': {}/{} tests passed in {:.1}s",
                        suite.name,
                        passed,
                        total,
                        elapsed.as_secs_f64()
                    );

                    pb.set_message(format!(
                        "{} ✓ ({}/{}) {:.1}s",
                        suite_name,
                        passed,
                        total,
                        elapsed.as_secs_f64()
                    ));
                    suites.push(suite);
                }
                Err(e) => {
                    let elapsed = suite_start_time.elapsed();
                    warn!(
                        "Failed to execute suite '{}' after {:.1}s: {}",
                        suite_name,
                        elapsed.as_secs_f64(),
                        e
                    );
                    pb.set_message(format!(
                        "{} ✗ (timeout after {:.0}s)",
                        suite_name,
                        elapsed.as_secs_f64()
                    ));

                    // Print user-friendly error message
                    eprintln!("\n⚠️  Warning: {}", e);
                    eprintln!("    Continuing with remaining test suites...\n");

                    // Continue with other suites
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("All test suites completed");

        let total_duration = start_time.elapsed();
        let finished_at = Utc::now();

        // Gather environment information
        let environment = self.gather_environment_info(bats_version);

        Ok(TestReport {
            binary_name: self.binary_name.clone(),
            binary_version: self.binary_version.clone(),
            suites,
            total_duration,
            started_at,
            finished_at,
            environment,
        })
    }

    /// Execute a single BATS suite with timeout
    fn execute_suite(
        &self,
        bats_file: &Path,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<TestSuite> {
        let suite_start = Instant::now();
        let started_at = Utc::now();

        let suite_name = bats_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        debug!("Executing BATS file: {}", bats_file.display());

        // Execute BATS with TAP output and timeout with periodic progress updates
        let timeout_duration = std::time::Duration::from_secs(self.timeout);
        let bats_file_path = bats_file.to_path_buf();
        let suite_name_clone = suite_name.to_string();

        let output = runtime
            .block_on(async move {
                // Wrap execution in timeout
                tokio::time::timeout(timeout_duration, async move {
                    let mut execution = tokio::task::spawn_blocking(move || {
                        Command::new("bats")
                            .arg("--formatter")
                            .arg("tap")
                            .arg(&bats_file_path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output()
                    });

                    // Progress ticker that prints every 30 seconds
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                    let mut elapsed_secs = 0u64;
                    let timeout_secs = timeout_duration.as_secs();

                    loop {
                        tokio::select! {
                            result = &mut execution => {
                                // result is Result<Result<Output, io::Error>, JoinError>
                                return result.map_err(|e| std::io::Error::other(
                                    format!("Task join error: {}", e)
                                ))?;
                            }
                            _ = interval.tick() => {
                                elapsed_secs += 30;
                                if elapsed_secs < timeout_secs {
                                    eprintln!("  ⏳ Still running '{}' ({}/{}s elapsed)...",
                                        suite_name_clone, elapsed_secs, timeout_secs);
                                }
                            }
                        }
                    }
                })
                .await
            })
            .map_err(|_| {
                // Timeout error from tokio::time::timeout
                Error::BatsExecutionFailed(format!(
                    "Test suite '{}' timed out after {} seconds. \
                     This may indicate a hanging test (e.g., waiting for user input). \
                     Check the test file: {}",
                    suite_name,
                    self.timeout,
                    bats_file.display()
                ))
            })?
            .map_err(|e| Error::BatsExecutionFailed(format!("Failed to execute BATS: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        debug!("BATS stdout:\n{}", stdout);
        if !stderr.is_empty() {
            debug!("BATS stderr:\n{}", stderr);
        }

        // Parse TAP output
        let tests = self.parse_tap_output(&stdout, bats_file)?;

        let duration = suite_start.elapsed();
        let finished_at = Utc::now();

        let suite_name = bats_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(TestSuite {
            name: suite_name,
            file_path: bats_file.to_string_lossy().to_string(),
            tests,
            duration,
            started_at,
            finished_at,
        })
    }

    /// Parse TAP (Test Anything Protocol) output from BATS
    fn parse_tap_output(&self, output: &str, bats_file: &Path) -> Result<Vec<TestResult>> {
        let mut tests = Vec::new();
        let lines: Vec<&str> = output.lines().collect();

        // TAP format:
        // 1..N (plan)
        // ok 1 test name
        // not ok 2 test name
        // # (comments/diagnostics)

        let test_line_re = Regex::new(r"^(ok|not ok)\s+(\d+)\s+(.+)$").unwrap();
        let skip_re = Regex::new(r"#\s*skip").unwrap();

        for line in lines {
            if let Some(caps) = test_line_re.captures(line) {
                let status_str = &caps[1];
                let test_num = &caps[2];
                let test_name = caps[3].trim();

                // Check if test was skipped
                let is_skipped = skip_re.is_match(test_name);

                let status = if is_skipped {
                    TestStatus::Skipped
                } else if status_str == "ok" {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                };

                // Extract clean test name (remove skip directive)
                let clean_name = skip_re.replace(test_name, "").trim().to_string();

                tests.push(TestResult {
                    name: clean_name,
                    status,
                    duration: Duration::from_millis(100), // Default duration, BATS doesn't provide timing
                    output: String::new(),
                    error_message: if status == TestStatus::Failed {
                        Some(format!("Test {} failed", test_num))
                    } else {
                        None
                    },
                    file_path: bats_file.to_string_lossy().to_string(),
                    line_number: None,
                });

                debug!("Parsed test: {} - {:?}", test_name, status);
            }
        }

        if tests.is_empty() {
            warn!("No tests found in TAP output");
        }

        Ok(tests)
    }

    /// Gather environment information
    fn gather_environment_info(&self, bats_version: String) -> EnvironmentInfo {
        let shell_version = Command::new("bash")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let os_version = if cfg!(target_os = "macos") {
            Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else if cfg!(target_os = "linux") {
            Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        EnvironmentInfo {
            os_version,
            shell_version,
            bats_version,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tap_output_success() {
        let executor = BatsExecutor::new("test-cli".to_string(), None);
        let tap_output = r#"
1..3
ok 1 test one
ok 2 test two
ok 3 test three
"#;

        let bats_file = Path::new("/tmp/test.bats");
        let results = executor.parse_tap_output(tap_output, bats_file).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "test one");
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[1].name, "test two");
        assert_eq!(results[2].name, "test three");
    }

    #[test]
    fn test_parse_tap_output_failures() {
        let executor = BatsExecutor::new("test-cli".to_string(), None);
        let tap_output = r#"
1..3
ok 1 test one
not ok 2 test two
ok 3 test three
"#;

        let bats_file = Path::new("/tmp/test.bats");
        let results = executor.parse_tap_output(tap_output, bats_file).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[1].status, TestStatus::Failed);
        assert!(results[1].error_message.is_some());
        assert_eq!(results[2].status, TestStatus::Passed);
    }

    #[test]
    fn test_parse_tap_output_skipped() {
        let executor = BatsExecutor::new("test-cli".to_string(), None);
        let tap_output = r#"
1..2
ok 1 test one # skip
ok 2 test two
"#;

        let bats_file = Path::new("/tmp/test.bats");
        let results = executor.parse_tap_output(tap_output, bats_file).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, TestStatus::Skipped);
        assert_eq!(results[1].status, TestStatus::Passed);
    }

    #[test]
    fn test_executor_creation() {
        let executor = BatsExecutor::new("test-cli".to_string(), Some("1.0.0".to_string()));
        assert_eq!(executor.binary_name, "test-cli");
        assert_eq!(executor.binary_version, Some("1.0.0".to_string()));
        assert_eq!(executor.timeout, 300);

        let custom = BatsExecutor::with_timeout("cli".to_string(), None, 600);
        assert_eq!(custom.timeout, 600);
    }
}
