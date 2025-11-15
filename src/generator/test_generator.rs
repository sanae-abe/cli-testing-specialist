use crate::analyzer::BehaviorInferrer;
use crate::config::load_config;
use crate::error::Result;
use crate::types::{
    Assertion, CliAnalysis, CliOption, CliTestConfig, NoArgsBehavior, OptionType, TestCase,
    TestCategory, TestPriority,
};
use crate::utils::{choose_strategy, ParallelStrategy, Workload};
use rayon::prelude::*;
use std::path::Path;

/// Test generator for creating test cases from CLI analysis
pub struct TestGenerator {
    /// CLI analysis to generate tests from
    analysis: CliAnalysis,

    /// Categories to generate tests for
    categories: Vec<TestCategory>,

    /// Optional configuration for test adjustments
    config: Option<CliTestConfig>,
}

impl TestGenerator {
    /// Create a new test generator
    pub fn new(analysis: CliAnalysis, categories: Vec<TestCategory>) -> Self {
        Self {
            analysis,
            categories,
            config: None,
        }
    }

    /// Create a new test generator with configuration file
    pub fn with_config(
        analysis: CliAnalysis,
        categories: Vec<TestCategory>,
        config_path: Option<&Path>,
    ) -> Result<Self> {
        let config = load_config(config_path)?;
        Ok(Self {
            analysis,
            categories,
            config,
        })
    }

    /// Generate all test cases based on selected categories
    pub fn generate(&self) -> Result<Vec<TestCase>> {
        log::info!("Generating tests for {} categories", self.categories.len());

        let mut all_tests = Vec::new();

        for category in &self.categories {
            let tests = match category {
                TestCategory::Basic => self.generate_basic_tests()?,
                TestCategory::Help => self.generate_help_tests()?,
                TestCategory::Security => self.generate_security_tests()?,
                TestCategory::Path => self.generate_path_tests()?,
                TestCategory::InputValidation => self.generate_input_validation_tests()?,
                TestCategory::DestructiveOps => self.generate_destructive_ops_tests()?,
                TestCategory::DirectoryTraversal => self.generate_directory_traversal_tests()?,
                TestCategory::Performance => self.generate_performance_tests()?,
                TestCategory::MultiShell => self.generate_multi_shell_tests()?,
            };

            log::info!("Generated {} tests for {:?}", tests.len(), category);
            all_tests.extend(tests);
        }

        log::info!("Total tests generated: {}", all_tests.len());
        Ok(all_tests)
    }

    /// Generate tests in parallel using rayon
    pub fn generate_parallel(&self) -> Result<Vec<TestCase>> {
        log::info!(
            "Generating tests in parallel for {} categories",
            self.categories.len()
        );

        let results: Result<Vec<Vec<TestCase>>> = self
            .categories
            .par_iter()
            .map(|category| match category {
                TestCategory::Basic => self.generate_basic_tests(),
                TestCategory::Help => self.generate_help_tests(),
                TestCategory::Security => self.generate_security_tests(),
                TestCategory::Path => self.generate_path_tests(),
                TestCategory::InputValidation => self.generate_input_validation_tests(),
                TestCategory::DestructiveOps => self.generate_destructive_ops_tests(),
                TestCategory::DirectoryTraversal => self.generate_directory_traversal_tests(),
                TestCategory::Performance => self.generate_performance_tests(),
                TestCategory::MultiShell => self.generate_multi_shell_tests(),
            })
            .collect();

        let all_tests: Vec<TestCase> = results?.into_iter().flatten().collect();

        log::info!("Total tests generated (parallel): {}", all_tests.len());
        Ok(all_tests)
    }

    /// Generate tests with automatic strategy selection
    ///
    /// This is the recommended method for test generation. It automatically
    /// chooses the optimal parallel processing strategy based on:
    /// - Number of test categories
    /// - CLI complexity (options and subcommands)
    /// - Available CPU cores
    ///
    /// # Strategy Selection
    ///
    /// - **Sequential**: Small workloads (<20 tests, 1 category)
    /// - **CategoryLevel**: Medium workloads (20-100 tests, multiple categories)
    /// - **TestLevel**: Large workloads (100+ tests, 4+ CPU cores)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let generator = TestGenerator::new(analysis, categories);
    /// let tests = generator.generate_with_strategy()?;
    /// ```
    pub fn generate_with_strategy(&self) -> Result<Vec<TestCase>> {
        // Create workload descriptor
        let workload = Workload::new(
            &self.categories,
            self.analysis.global_options.len(),
            self.analysis.subcommands.len(),
        );

        // Choose optimal strategy
        let strategy = choose_strategy(&workload);

        log::info!(
            "Auto-selected strategy: {:?} (categories={}, global_options={}, subcommands={}, estimated_tests={})",
            strategy,
            workload.num_categories,
            self.analysis.global_options.len(),
            self.analysis.subcommands.len(),
            workload.total_estimated_tests()
        );

        // Execute based on strategy
        match strategy {
            ParallelStrategy::Sequential => {
                log::debug!("Using sequential generation");
                self.generate()
            }
            ParallelStrategy::CategoryLevel => {
                log::debug!("Using category-level parallel generation");
                self.generate_parallel()
            }
            ParallelStrategy::TestLevel => {
                log::debug!("Using test-level parallel generation");
                // Use category-level parallelism as base
                // Individual categories (e.g., Help with 10+ subcommands) automatically
                // use test-level parallelism internally
                self.generate_parallel()
            }
        }
    }

    /// Generate basic validation tests (help, version, exit codes)
    fn generate_basic_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Test 1: Help flag
        tests.push(
            TestCase::new(
                "basic-001".to_string(),
                "Display help with --help flag".to_string(),
                TestCategory::Basic,
                "\"$CLI_BINARY\" --help".to_string(),
            )
            .with_exit_code(0)
            .with_assertion(Assertion::OutputContains("Usage:".to_string()))
            .with_tag("help".to_string()),
        );

        // Test 2: Short help flag
        tests.push(
            TestCase::new(
                "basic-002".to_string(),
                "Display help with -h flag".to_string(),
                TestCategory::Basic,
                "\"$CLI_BINARY\" -h".to_string(),
            )
            .with_exit_code(0)
            .with_assertion(Assertion::OutputContains("Usage:".to_string()))
            .with_tag("help".to_string()),
        );

        // Test 3: Version flag (if version detected)
        if self.analysis.version.is_some() {
            tests.push(
                TestCase::new(
                    "basic-003".to_string(),
                    "Display version with --version flag".to_string(),
                    TestCategory::Basic,
                    "\"$CLI_BINARY\" --version".to_string(),
                )
                .with_exit_code(0)
                .with_tag("version".to_string()),
            );
        }

        // Test 4: Invalid option
        // Note: Different CLI frameworks use different exit codes for invalid options:
        // - Rust (clap): exit code 2 (Unix standard for usage errors)
        // - Node.js (commander.js): exit code 1
        // - Python (argparse): exit code 2
        // Accept any non-zero exit code to support all frameworks
        tests.push(
            TestCase::new(
                "basic-004".to_string(),
                "Reject invalid option".to_string(),
                TestCategory::Basic,
                "\"$CLI_BINARY\" --invalid-option-xyz".to_string(),
            )
            .expect_nonzero_exit() // Accept exit 1 or 2 (framework-agnostic)
            // Match common error message patterns across different CLIs:
            // - error, Error (most CLIs)
            // - unknown, unrecognized (curl, many CLIs)
            // - invalid (common in validation errors)
            .with_assertion(Assertion::OutputMatches(
                "(error|Error|unknown|unrecognized|invalid)".to_string(),
            ))
            .with_tag("error-handling".to_string()),
        );

        // Test 5: No arguments (behavior depends on CLI type)
        let inferrer = BehaviorInferrer::new();
        let no_args_behavior = inferrer.infer_no_args_behavior(&self.analysis);

        match no_args_behavior {
            NoArgsBehavior::ShowHelp => {
                tests.push(
                    TestCase::new(
                        "basic-005".to_string(),
                        "Show help when invoked without arguments".to_string(),
                        TestCategory::Basic,
                        "\"$CLI_BINARY\"".to_string(),
                    )
                    .with_exit_code(0)
                    // Output check removed: Some CLIs output nothing
                    // (e.g., backup-suite exits silently with code 0)
                    .with_priority(TestPriority::Important)
                    .with_tag("no-args".to_string())
                    .with_tag("show-help".to_string()),
                );
            }

            NoArgsBehavior::RequireSubcommand => {
                tests.push(
                    TestCase::new(
                        "basic-005".to_string(),
                        "Require subcommand when invoked without arguments".to_string(),
                        TestCategory::Basic,
                        "\"$CLI_BINARY\"".to_string(),
                    )
                    .expect_nonzero_exit() // Accept exit 1 or 2
                    // Output check removed: CLIs show different error formats
                    // (short error message vs full help text)
                    .with_priority(TestPriority::Important)
                    .with_tag("no-args".to_string())
                    .with_tag("require-subcommand".to_string()),
                );
            }

            NoArgsBehavior::Interactive => {
                tests.push(
                    TestCase::new(
                        "basic-005".to_string(),
                        "Enter interactive mode when invoked without arguments".to_string(),
                        TestCategory::Basic,
                        "echo '' | \"$CLI_BINARY\"".to_string(), // Pipe empty input to exit immediately
                    )
                    .with_exit_code(0)
                    .with_priority(TestPriority::Important)
                    .with_tag("no-args".to_string())
                    .with_tag("interactive".to_string()),
                );
            }
        }

        Ok(tests)
    }

    /// Generate help display tests
    fn generate_help_tests(&self) -> Result<Vec<TestCase>> {
        // For small number of subcommands, use simple sequential generation
        // Parallel overhead not worth it for <10 subcommands
        if self.analysis.subcommands.len() < 10 {
            let mut tests = Vec::new();

            for (idx, subcommand) in self.analysis.subcommands.iter().enumerate() {
                // Skip 'help' meta-command
                if subcommand.name.to_lowercase() == "help" {
                    log::debug!("Skipping help test for meta-command 'help'");
                    continue;
                }

                tests.push(
                    TestCase::new(
                        format!("help-{:03}", idx + 1),
                        format!("Display help for subcommand '{}'", subcommand.name),
                        TestCategory::Help,
                        format!("\"$CLI_BINARY\" {} --help", subcommand.name),
                    )
                    .with_exit_code(0)
                    .with_assertion(Assertion::OutputContains("Usage:".to_string()))
                    .with_tag("subcommand-help".to_string())
                    .with_tag(subcommand.name.clone()),
                );
            }

            return Ok(tests);
        }

        // For large number of subcommands (10+), use parallel generation
        let tests: Vec<TestCase> = self
            .analysis
            .subcommands
            .par_iter()
            .enumerate()
            .filter_map(|(idx, subcommand)| {
                // Skip 'help' meta-command
                if subcommand.name.to_lowercase() == "help" {
                    log::debug!("Skipping help test for meta-command 'help'");
                    return None;
                }

                Some(
                    TestCase::new(
                        format!("help-{:03}", idx + 1),
                        format!("Display help for subcommand '{}'", subcommand.name),
                        TestCategory::Help,
                        format!("\"$CLI_BINARY\" {} --help", subcommand.name),
                    )
                    .with_exit_code(0)
                    .with_assertion(Assertion::OutputContains("Usage:".to_string()))
                    .with_tag("subcommand-help".to_string())
                    .with_tag(subcommand.name.clone()),
                )
            })
            .collect();

        Ok(tests)
    }

    /// Generate security scanner tests
    ///
    /// **IMPORTANT**: Security tests MUST expect non-zero exit codes for malicious inputs.
    /// A tool that accepts malicious input (exit code 0) is vulnerable.
    ///
    /// # Security Test Philosophy
    ///
    /// - **Injection attacks**: Tool MUST reject with non-zero exit code (1 or 2)
    /// - **Null bytes**: Tool MUST reject with non-zero exit code (1 or 2)
    /// - **Path traversal**: Tool MUST reject with non-zero exit code (1 or 2)
    /// - **Buffer overflow**: Tool MUST handle gracefully (may succeed if sanitized)
    ///
    /// # Unix Exit Code Convention
    ///
    /// - **0**: Success
    /// - **1**: General error (runtime error, validation failure)
    /// - **2**: Command-line usage error (invalid option, clap/argparse default)
    ///
    /// Modern CLI parsers (clap, commander, argparse) return exit code 2 for invalid options,
    /// which is the correct Unix convention. Security tests accept both 1 and 2 as valid rejection.
    fn generate_security_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Get skip_options from config if available
        let skip_options: Vec<String> = self
            .config
            .as_ref()
            .and_then(|c| {
                c.test_adjustments
                    .security
                    .as_ref()
                    .map(|s| s.skip_options.iter().map(|opt| opt.name.clone()).collect())
            })
            .unwrap_or_default();

        // Find a string option for testing (prefer --config, --file, or first string option)
        // Skip options that are in skip_options list
        let string_option = self
            .analysis
            .global_options
            .iter()
            .find(|opt| {
                matches!(opt.option_type, OptionType::String | OptionType::Path)
                    && opt.long.is_some()
                    && !skip_options.iter().any(|skip_name| {
                        opt.long
                            .as_ref()
                            .is_some_and(|long| long.trim_start_matches("--") == skip_name)
                    })
            })
            .and_then(|opt| opt.long.as_ref())
            .unwrap_or(&"--invalid-option".to_string())
            .clone();

        // Test 1: Command injection via option
        // MUST reject malicious input (any non-zero exit code)
        tests.push(
            TestCase::new(
                "security-001".to_string(),
                "Reject command injection in option value".to_string(),
                TestCategory::Security,
                format!("\"$CLI_BINARY\" {} 'test; rm -rf /'", string_option),
            )
            .expect_nonzero_exit() // Accept exit code 1, 2, or any non-zero
            .with_priority(TestPriority::SecurityCheck)
            .with_tag("injection".to_string())
            .with_tag("critical".to_string()),
        );

        // Test 2: Null byte injection
        // MUST reject malicious input (any non-zero exit code)
        tests.push(
            TestCase::new(
                "security-002".to_string(),
                "Reject null byte in option value".to_string(),
                TestCategory::Security,
                format!(
                    r#""$CLI_BINARY" {} $'/tmp/test\x00malicious'"#,
                    string_option
                ),
            )
            .expect_nonzero_exit() // Accept exit code 1, 2, or any non-zero
            .with_priority(TestPriority::SecurityCheck)
            .with_tag("injection".to_string())
            .with_tag("critical".to_string()),
        );

        // Test 3: Path traversal
        // MUST reject path traversal attempt (any non-zero exit code)
        tests.push(
            TestCase::new(
                "security-003".to_string(),
                "Reject path traversal attempt".to_string(),
                TestCategory::Security,
                format!("\"$CLI_BINARY\" {} ../../../etc/passwd", string_option),
            )
            .expect_nonzero_exit() // Accept exit code 1, 2, or any non-zero
            .with_priority(TestPriority::SecurityCheck)
            .with_tag("path-traversal".to_string())
            .with_tag("critical".to_string()),
        );

        // Test 4: Long input (buffer overflow test)
        // NOTE: Disabled by default due to platform-dependent behavior
        // - Node.js: May fail with E2BIG (Argument list too long) - OS limit
        // - Shell: May fail with ARG_MAX exceeded - OS limit (typically 128KB-2MB)
        // - Different platforms have different limits (macOS: 256KB, Linux: 2MB)
        //
        // This test is informational and should only be enabled for:
        // - Low-level languages (C/C++, Rust with unsafe code)
        // - Tools handling binary data or parsing untrusted input
        //
        // For most CLI tools (especially Node.js), this test is not meaningful
        // and will fail due to OS argument length limits, not application bugs.
        //
        // Uncomment to enable (not recommended for Node.js CLIs):
        // let long_input = "A".repeat(10000);
        // tests.push(
        //     TestCase::new(
        //         "security-004".to_string(),
        //         "Handle extremely long input without crashing".to_string(),
        //         TestCategory::Security,
        //         format!("\"$CLI_BINARY\" {} '{}'", string_option, long_input),
        //     )
        //     .expect_nonzero_exit() // Expect rejection (OS limit or input validation)
        //     .with_priority(TestPriority::Important) // Informational test
        //     .with_tag("buffer-overflow".to_string())
        //     .with_tag("dos-protection".to_string())
        //     .with_tag("informational".to_string()),
        // );

        // Add custom security tests from config
        if let Some(config) = &self.config {
            if let Some(security_config) = &config.test_adjustments.security {
                for (idx, custom_test) in security_config.custom_tests.iter().enumerate() {
                    tests.push(
                        TestCase::new(
                            format!("security-custom-{:03}", idx + 1),
                            custom_test.description.clone(),
                            TestCategory::Security,
                            custom_test.command.clone(),
                        )
                        .with_exit_code(custom_test.expected_exit_code)
                        .with_priority(TestPriority::SecurityCheck)
                        .with_tag("custom".to_string())
                        .with_tag(custom_test.name.clone()),
                    );
                }
            }
        }

        Ok(tests)
    }

    /// Generate path handling tests
    fn generate_path_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Find path options
        let path_options: Vec<&CliOption> = self
            .analysis
            .global_options
            .iter()
            .filter(|opt| matches!(opt.option_type, OptionType::Path))
            .collect();

        if path_options.is_empty() {
            log::debug!("No path options found, generating generic path tests");
            return Ok(tests);
        }

        for (idx, option) in path_options.iter().enumerate() {
            let flag = option.long.as_ref().or(option.short.as_ref()).unwrap();

            // Test 1: Path with spaces
            tests.push(
                TestCase::new(
                    format!("path-{:03}-spaces", idx + 1),
                    format!("Handle path with spaces for {}", flag),
                    TestCategory::Path,
                    format!("\"$CLI_BINARY\" {} '/tmp/test dir/file.txt'", flag),
                )
                .with_tag("spaces".to_string()),
            );

            // Test 2: Unicode path
            tests.push(
                TestCase::new(
                    format!("path-{:03}-unicode", idx + 1),
                    format!("Handle Unicode in path for {}", flag),
                    TestCategory::Path,
                    format!("\"$CLI_BINARY\" {} '/tmp/テスト/file.txt'", flag),
                )
                .with_tag("unicode".to_string()),
            );

            // Test 3: Symbolic links
            tests.push(
                TestCase::new(
                    format!("path-{:03}-symlink", idx + 1),
                    format!("Handle symbolic links for {}", flag),
                    TestCategory::Path,
                    format!("\"$CLI_BINARY\" {} '/tmp/test-symlink'", flag),
                )
                .with_tag("symlink".to_string()),
            );
        }

        Ok(tests)
    }

    /// Generate input validation tests
    fn generate_input_validation_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Find numeric options
        let numeric_options: Vec<&CliOption> = self
            .analysis
            .global_options
            .iter()
            .filter(|opt| matches!(opt.option_type, OptionType::Numeric { .. }))
            .collect();

        for (idx, option) in numeric_options.iter().enumerate() {
            let flag = option.long.as_ref().or(option.short.as_ref()).unwrap();

            // Test 1: Valid value
            tests.push(
                TestCase::new(
                    format!("input-{:03}-valid", idx + 1),
                    format!("Accept valid numeric value for {}", flag),
                    TestCategory::InputValidation,
                    format!("\"$CLI_BINARY\" {} 10", flag),
                )
                .with_tag("numeric".to_string()),
            );

            // Test 2: Non-numeric value
            tests.push(
                TestCase::new(
                    format!("input-{:03}-invalid", idx + 1),
                    format!("Reject non-numeric value for {}", flag),
                    TestCategory::InputValidation,
                    format!("\"$CLI_BINARY\" {} 'not-a-number'", flag),
                )
                .with_exit_code(1)
                .with_tag("numeric".to_string())
                .with_tag("validation".to_string()),
            );

            // Test 3: Negative value (if min >= 0)
            if let OptionType::Numeric {
                min: Some(min_val), ..
            } = &option.option_type
            {
                if *min_val >= 0 {
                    tests.push(
                        TestCase::new(
                            format!("input-{:03}-negative", idx + 1),
                            format!("Reject negative value for {}", flag),
                            TestCategory::InputValidation,
                            format!("\"$CLI_BINARY\" {} -1", flag),
                        )
                        .with_exit_code(1)
                        .with_tag("numeric".to_string())
                        .with_tag("validation".to_string()),
                    );
                }
            }
        }

        // Find enum options
        let enum_options: Vec<&CliOption> = self
            .analysis
            .global_options
            .iter()
            .filter(|opt| matches!(opt.option_type, OptionType::Enum { .. }))
            .collect();

        for (idx, option) in enum_options.iter().enumerate() {
            let flag = option.long.as_ref().or(option.short.as_ref()).unwrap();

            if let OptionType::Enum { values } = &option.option_type {
                if let Some(first_value) = values.first() {
                    // Test valid enum value
                    tests.push(
                        TestCase::new(
                            format!("enum-{:03}-valid", idx + 1),
                            format!("Accept valid enum value for {}", flag),
                            TestCategory::InputValidation,
                            format!("\"$CLI_BINARY\" {} {}", flag, first_value),
                        )
                        .with_tag("enum".to_string()),
                    );
                }

                // Test invalid enum value
                tests.push(
                    TestCase::new(
                        format!("enum-{:03}-invalid", idx + 1),
                        format!("Reject invalid enum value for {}", flag),
                        TestCategory::InputValidation,
                        format!("\"$CLI_BINARY\" {} 'invalid-value-xyz'", flag),
                    )
                    .with_exit_code(1)
                    .with_tag("enum".to_string())
                    .with_tag("validation".to_string()),
                );
            }
        }

        Ok(tests)
    }

    /// Generate destructive operations tests
    fn generate_destructive_ops_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Get env_vars and cancel_exit_code from config
        let env_vars = self
            .config
            .as_ref()
            .and_then(|c| c.test_adjustments.destructive_ops.as_ref())
            .map(|d| d.env_vars.clone())
            .unwrap_or_default();

        let cancel_exit_code = self
            .config
            .as_ref()
            .and_then(|c| c.test_adjustments.destructive_ops.as_ref())
            .map(|d| d.cancel_exit_code)
            .unwrap_or(1); // Default to 1 if not specified

        // Look for destructive subcommands (delete, remove, clean, destroy, etc.)
        let destructive_keywords = ["delete", "remove", "clean", "destroy", "purge", "drop"];

        for subcommand in &self.analysis.subcommands {
            let is_destructive = destructive_keywords
                .iter()
                .any(|keyword| subcommand.name.to_lowercase().contains(keyword));

            if is_destructive {
                // Generate dummy values for required arguments
                let dummy_args = subcommand
                    .required_args
                    .iter()
                    .map(|arg| {
                        // Generate appropriate dummy value based on argument name
                        match arg.to_lowercase().as_str() {
                            "id" | "name" => "test-id",
                            "file" | "path" => "/tmp/test-file",
                            "dir" | "directory" => "/tmp/test-dir",
                            _ => "test-value",
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let args_part = if dummy_args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", dummy_args)
                };

                // Build env vars prefix for commands
                let env_prefix = if env_vars.is_empty() {
                    String::new()
                } else {
                    env_vars
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect::<Vec<_>>()
                        .join(" ")
                        + " "
                };

                // Test 1: Check for confirmation prompt (or skip if env vars set)
                let test_command = if env_vars.is_empty() {
                    // No env vars: test cancellation with 'n' input
                    format!(
                        "echo 'n' | \"$CLI_BINARY\" {}{}",
                        subcommand.name, args_part
                    )
                } else {
                    // With env vars: test auto-confirmation
                    format!(
                        "{}\"$CLI_BINARY\" {}{}",
                        env_prefix, subcommand.name, args_part
                    )
                };

                let mut test = TestCase::new(
                    format!("destructive-{}-001", subcommand.name),
                    if env_vars.is_empty() {
                        format!("Subcommand '{}' requires confirmation", subcommand.name)
                    } else {
                        format!(
                            "Subcommand '{}' auto-confirms with env vars",
                            subcommand.name
                        )
                    },
                    TestCategory::DestructiveOps,
                    test_command,
                )
                .with_tag("confirmation".to_string())
                .with_tag(subcommand.name.clone());

                // Set expected exit code based on whether we're testing cancellation or execution
                if env_vars.is_empty() {
                    // Test cancellation: expect cancel_exit_code
                    test = test.with_exit_code(cancel_exit_code);
                }
                // else: execution test, exit code depends on implementation (don't set)

                tests.push(test);

                // Test 2: Check for --yes or --force flag
                let has_yes_flag = subcommand.options.iter().any(|opt| {
                    opt.long
                        .as_ref()
                        .is_some_and(|l| l.contains("yes") || l.contains("force"))
                });

                if has_yes_flag {
                    tests.push(
                        TestCase::new(
                            format!("destructive-{}-002", subcommand.name),
                            format!("Subcommand '{}' accepts --yes flag", subcommand.name),
                            TestCategory::DestructiveOps,
                            format!("\"$CLI_BINARY\" {}{} --yes", subcommand.name, args_part),
                        )
                        .with_tag("force".to_string())
                        .with_tag(subcommand.name.clone()),
                    );
                }
            }
        }

        // If no destructive subcommands found, generate generic test
        if tests.is_empty() {
            log::debug!("No destructive subcommands detected");
        }

        Ok(tests)
    }

    /// Generate directory traversal tests
    fn generate_directory_traversal_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Get test_directories from config or use defaults
        let test_directories = self
            .config
            .as_ref()
            .and_then(|c| c.test_adjustments.directory_traversal.as_ref())
            .and_then(|dt| {
                if dt.test_directories.is_empty() {
                    None
                } else {
                    Some(dt.test_directories.clone())
                }
            });

        if let Some(test_dirs) = test_directories {
            // Use configured test directories
            for (idx, test_dir) in test_dirs.iter().enumerate() {
                let description = if let Some(file_count) = test_dir.file_count {
                    format!(
                        "Handle directory with {} files at {}",
                        file_count, test_dir.path
                    )
                } else if let Some(depth) = test_dir.depth {
                    format!(
                        "Handle deeply nested directory (depth {}) at {}",
                        depth, test_dir.path
                    )
                } else {
                    format!("Handle directory at {}", test_dir.path)
                };

                tests.push(
                    TestCase::new(
                        format!("dir-traversal-{:03}", idx + 1),
                        description,
                        TestCategory::DirectoryTraversal,
                        format!("\"$CLI_BINARY\" {}", test_dir.path),
                    )
                    .with_tag("configured".to_string())
                    .with_tag(if test_dir.file_count.is_some() {
                        "large-dir".to_string()
                    } else if test_dir.depth.is_some() {
                        "deep-nesting".to_string()
                    } else {
                        "basic".to_string()
                    }),
                );
            }
        } else {
            // Use default tests
            tests = vec![
                // Test 1: Large directory (1000 files)
                TestCase::new(
                    "dir-traversal-001".to_string(),
                    "Handle directory with 1000 files".to_string(),
                    TestCategory::DirectoryTraversal,
                    "\"$CLI_BINARY\" /tmp/test-large-dir".to_string(),
                )
                .with_tag("performance".to_string())
                .with_tag("large-dir".to_string()),
                // Test 2: Deep directory nesting (50 levels)
                TestCase::new(
                    "dir-traversal-002".to_string(),
                    "Handle deeply nested directory (50 levels)".to_string(),
                    TestCategory::DirectoryTraversal,
                    "\"$CLI_BINARY\" /tmp/test-deep-dir".to_string(),
                )
                .with_tag("performance".to_string())
                .with_tag("deep-nesting".to_string()),
                // Test 3: Symlink loops
                TestCase::new(
                    "dir-traversal-003".to_string(),
                    "Detect and handle symlink loops".to_string(),
                    TestCategory::DirectoryTraversal,
                    "\"$CLI_BINARY\" /tmp/test-symlink-loop".to_string(),
                )
                .with_tag("symlink".to_string())
                .with_tag("loop-detection".to_string()),
            ];
        }

        Ok(tests)
    }

    /// Generate performance tests
    fn generate_performance_tests(&self) -> Result<Vec<TestCase>> {
        let tests = vec![
            // Test 1: Startup time (help should be fast)
            TestCase::new(
                "perf-001".to_string(),
                "Startup time for --help < 100ms".to_string(),
                TestCategory::Performance,
                "\"$CLI_BINARY\" --help".to_string(),
            )
            .with_exit_code(0)
            .with_tag("startup".to_string())
            .with_tag("benchmark".to_string()),
            // Test 2: Memory usage
            TestCase::new(
                "perf-002".to_string(),
                "Memory usage stays within reasonable limits".to_string(),
                TestCategory::Performance,
                "\"$CLI_BINARY\" --help".to_string(),
            )
            .with_exit_code(0)
            .with_tag("memory".to_string())
            .with_tag("benchmark".to_string()),
        ];

        Ok(tests)
    }

    /// Generate multi-shell compatibility tests
    fn generate_multi_shell_tests(&self) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Test basic command in different shells (bash, zsh, sh)
        // Note: Use double quotes to allow variable expansion, escape inner quotes
        for shell in &["bash", "zsh", "sh"] {
            tests.push(
                TestCase::new(
                    format!("multi-shell-{}", shell),
                    format!("Run --help in {}", shell),
                    TestCategory::MultiShell,
                    format!("{} -c \"\\\"$CLI_BINARY\\\" --help\"", shell),
                )
                .with_exit_code(0)
                .with_tag(shell.to_string()),
            );
        }

        Ok(tests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Subcommand;
    use std::path::PathBuf;

    fn create_test_analysis() -> CliAnalysis {
        let mut analysis = CliAnalysis::new(
            PathBuf::from("/usr/bin/test-cli"),
            "test-cli".to_string(),
            "Test CLI help output".to_string(),
        );

        analysis.version = Some("1.0.0".to_string());

        // Add a numeric option
        analysis.global_options.push(CliOption {
            short: Some("-t".to_string()),
            long: Some("--timeout".to_string()),
            description: Some("Timeout in seconds".to_string()),
            option_type: OptionType::Numeric {
                min: Some(0),
                max: Some(3600),
            },
            required: false,
            default_value: Some("30".to_string()),
        });

        // Add a path option
        analysis.global_options.push(CliOption {
            short: Some("-f".to_string()),
            long: Some("--file".to_string()),
            description: Some("Input file".to_string()),
            option_type: OptionType::Path,
            required: false,
            default_value: None,
        });

        // Add an enum option
        analysis.global_options.push(CliOption {
            short: None,
            long: Some("--format".to_string()),
            description: Some("Output format".to_string()),
            option_type: OptionType::Enum {
                values: vec!["json".to_string(), "yaml".to_string(), "text".to_string()],
            },
            required: false,
            default_value: Some("text".to_string()),
        });

        // Add a subcommand
        analysis.subcommands.push(Subcommand {
            name: "delete".to_string(),
            description: Some("Delete resources".to_string()),
            options: vec![CliOption {
                short: None,
                long: Some("--yes".to_string()),
                description: Some("Skip confirmation".to_string()),
                option_type: OptionType::Flag,
                required: false,
                default_value: None,
            }],
            required_args: vec![],
            subcommands: vec![],
            depth: 0,
        });

        analysis
    }

    #[test]
    fn test_generator_creation() {
        let analysis = create_test_analysis();
        let categories = vec![TestCategory::Basic];
        let generator = TestGenerator::new(analysis, categories);

        assert_eq!(generator.categories.len(), 1);
    }

    #[test]
    fn test_generate_basic_tests() {
        let analysis = create_test_analysis();
        let generator = TestGenerator::new(analysis, vec![]);

        let tests = generator.generate_basic_tests().unwrap();

        assert!(!tests.is_empty());
        assert!(tests.iter().any(|t| t.command.contains("--help")));
        assert!(tests.iter().any(|t| t.command.contains("--version")));
    }

    #[test]
    fn test_generate_security_tests() {
        let analysis = create_test_analysis();
        let generator = TestGenerator::new(analysis, vec![]);

        let tests = generator.generate_security_tests().unwrap();

        assert!(!tests.is_empty());
        assert!(tests
            .iter()
            .any(|t| t.tags.contains(&"injection".to_string())));
    }

    #[test]
    fn test_generate_input_validation_tests() {
        let analysis = create_test_analysis();
        let generator = TestGenerator::new(analysis, vec![]);

        let tests = generator.generate_input_validation_tests().unwrap();

        assert!(!tests.is_empty());
        // Should have tests for numeric, path, and enum options
        assert!(tests
            .iter()
            .any(|t| t.tags.contains(&"numeric".to_string())));
        assert!(tests.iter().any(|t| t.tags.contains(&"enum".to_string())));
    }

    #[test]
    fn test_generate_destructive_ops_tests() {
        let analysis = create_test_analysis();
        let generator = TestGenerator::new(analysis, vec![]);

        let tests = generator.generate_destructive_ops_tests().unwrap();

        assert!(!tests.is_empty());
        assert!(tests.iter().any(|t| t.command.contains("delete")));
    }

    #[test]
    fn test_generate_all_categories() {
        let analysis = create_test_analysis();
        let categories = vec![
            TestCategory::Basic,
            TestCategory::Security,
            TestCategory::InputValidation,
        ];
        let generator = TestGenerator::new(analysis, categories);

        let tests = generator.generate().unwrap();

        assert!(!tests.is_empty());
        assert!(tests.iter().any(|t| t.category == TestCategory::Basic));
        assert!(tests.iter().any(|t| t.category == TestCategory::Security));
        assert!(tests
            .iter()
            .any(|t| t.category == TestCategory::InputValidation));
    }

    #[test]
    fn test_generate_parallel() {
        let analysis = create_test_analysis();
        let categories = vec![
            TestCategory::Basic,
            TestCategory::Security,
            TestCategory::Performance,
        ];
        let generator = TestGenerator::new(analysis, categories);

        let tests = generator.generate_parallel().unwrap();

        assert!(!tests.is_empty());
    }
}
