use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use cli_testing_specialist::analyzer::CliParser;
use cli_testing_specialist::cli::{Cli, Commands, ReportFormat, TestFormat};
use cli_testing_specialist::error::Result;
use cli_testing_specialist::generator::{
    AssertCmdGenerator, BatsWriter, TestGenerator, TestGeneratorTrait,
};
use cli_testing_specialist::reporter::{
    HtmlReporter, JsonReporter, JunitReporter, MarkdownReporter,
};
use cli_testing_specialist::runner::BatsExecutor;
use cli_testing_specialist::types::{CliAnalysis, TestCategory};
use cli_testing_specialist::utils::validate_binary_path;
use std::fs;
use std::io;

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Setup logging based on verbosity
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Route to appropriate command handler
    match cli.command {
        Commands::Analyze {
            binary,
            output,
            depth: _,
            parallel: _,
        } => {
            // 1. Validate binary path
            let binary_path = validate_binary_path(&binary)?;
            log::info!("Analyzing binary: {}", binary_path.display());

            // 2. Execute analysis with CliParser
            let parser = CliParser::new();
            let analysis = parser.analyze(&binary_path)?;

            log::info!(
                "Analysis complete: {} global options, {} subcommands",
                analysis.global_options.len(),
                analysis.subcommands.len()
            );

            // 3. Serialize to JSON
            let json_output = serde_json::to_string_pretty(&analysis)?;

            // 4. Write to output file
            let output_path = output;

            fs::write(&output_path, json_output)?;

            // 5. Success message
            println!("✓ Analysis complete: {}", output_path.display());
            println!("  Binary: {}", analysis.binary_name);
            if let Some(version) = &analysis.version {
                println!("  Version: {}", version);
            }
            println!("  Global options: {}", analysis.global_options.len());
            println!("  Subcommands: {}", analysis.subcommands.len());
            println!(
                "  Analysis time: {}ms",
                analysis.metadata.analysis_duration_ms
            );

            Ok(())
        }

        Commands::Generate {
            analysis,
            output,
            categories,
            format,
            include_intensive,
        } => {
            log::info!("Generating tests from: {}", analysis.display());

            // 1. Load analysis JSON (with safe deserialization)
            let analysis_json = fs::read_to_string(&analysis)?;
            let cli_analysis: CliAnalysis =
                cli_testing_specialist::utils::deserialize_json_safe(&analysis_json)?;

            log::info!(
                "Loaded analysis for binary: {} (version: {})",
                cli_analysis.binary_name,
                cli_analysis.version.as_deref().unwrap_or("unknown")
            );

            // 2. Parse categories
            let selected_categories = parse_categories(&categories, include_intensive)?;
            let num_categories = selected_categories.len();
            log::info!("Selected {} test categories", num_categories);

            if !include_intensive {
                log::info!(
                    "Excluding resource-intensive tests (use --include-intensive to enable)"
                );
            }

            match format {
                TestFormat::Bats => {
                    // 3. Generate test cases (BATS) with config support and automatic strategy selection
                    let generator = TestGenerator::with_config(
                        cli_analysis.clone(),
                        selected_categories,
                        None, // Auto-detect .cli-test-config.yml
                    )?;

                    // Use automatic strategy selection based on workload
                    let test_cases = generator.generate_with_strategy()?;

                    log::info!("Generated {} test cases", test_cases.len());

                    // 4. Write BATS files
                    let writer = BatsWriter::new(
                        output.clone(),
                        cli_analysis.binary_name.clone(),
                        cli_analysis.binary_path.clone(),
                    )?;

                    let output_files = writer.write_tests(&test_cases)?;

                    // 5. Validate generated files
                    for file in &output_files {
                        writer.validate_bats_file(file)?;
                        log::debug!("Validated: {}", file.display());
                    }

                    // 6. Success message
                    println!("✓ Test generation complete: {} files", output_files.len());
                    println!("  Output directory: {}", output.display());
                    println!("  Total test cases: {}", test_cases.len());
                    println!("\nGenerated files:");
                    for file in &output_files {
                        let file_name = file.file_name().unwrap().to_string_lossy();
                        let test_count = test_cases
                            .iter()
                            .filter(|t| file_name.starts_with(t.category.as_str()))
                            .count();
                        println!("  - {} ({} tests)", file_name, test_count);
                    }

                    println!("\nRun tests with: bats {}", output.display());
                }

                TestFormat::AssertCmd => {
                    // 3. Generate assert_cmd tests
                    log::info!("Generating assert_cmd Rust tests");
                    let generator = AssertCmdGenerator::new(&cli_analysis)?;

                    // Create output directory
                    fs::create_dir_all(&output)?;

                    // Generate tests for each category
                    let mut output_files = Vec::new();
                    for category in &selected_categories {
                        let test_code = generator.generate(&cli_analysis, *category)?;
                        let file_name = format!("{}.rs", category.as_str());
                        let file_path = output.join(&file_name);

                        fs::write(&file_path, test_code)?;
                        output_files.push(file_path);
                        log::info!("Generated: {}", file_name);
                    }

                    // 4. Success message
                    println!(
                        "✓ assert_cmd test generation complete: {} files",
                        output_files.len()
                    );
                    println!("  Output directory: {}", output.display());
                    println!("  Format: Rust (assert_cmd)");
                    println!("\nGenerated files:");
                    for file in &output_files {
                        let file_name = file.file_name().unwrap().to_string_lossy();
                        println!("  - {}", file_name);
                    }

                    println!("\nNext steps:");
                    println!("  1. Add to your Cargo.toml:");
                    println!("     [dev-dependencies]");
                    println!("     assert_cmd = \"2.0\"");
                    println!("     predicates = \"3.0\"");
                    println!("     tempfile = \"3.0\"");
                    println!("\n  2. Copy generated tests to tests/ directory:");
                    println!("     cp {}/*.rs tests/", output.display());
                    println!("\n  3. Run tests:");
                    println!("     cargo test");
                }

                TestFormat::Snapbox => {
                    return Err(cli_testing_specialist::error::Error::InvalidFormat(
                        "snapbox format not yet implemented".to_string(),
                    ));
                }
            }

            Ok(())
        }

        Commands::Run {
            test_dir,
            format,
            output,
            timeout,
            skip,
        } => {
            log::info!("Running tests from: {}", test_dir.display());

            // 1. Determine binary name from test directory or use default
            let binary_name = test_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // 2. Create BATS executor with custom timeout
            log::info!("Using timeout: {}s per test suite", timeout);
            let mut executor = BatsExecutor::with_timeout(binary_name.clone(), None, timeout);

            // 3. Apply skip categories if specified
            if let Some(skip_categories) = skip {
                let skip_list: Vec<String> = skip_categories
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if !skip_list.is_empty() {
                    log::info!("Skipping test categories: {}", skip_list.join(", "));
                    println!("Skipping categories: {}", skip_list.join(", "));
                    executor = executor.with_skip_categories(skip_list);
                }
            }

            // 3. Run tests and collect results
            println!("Running BATS tests from: {}", test_dir.display());
            let report = executor.run_tests(&test_dir)?;

            // 4. Display summary with priority-based breakdown
            println!("\n=== Test Results ===");

            // Template Quality (non-security tests)
            let template_total = report.template_quality_tests().len();
            let template_passed = report.passed_template_quality();
            println!(
                "Template Quality: {}/{} tests passed ({:.1}%)",
                template_passed,
                template_total,
                report.template_quality_rate() * 100.0
            );

            // Security Checks
            let security_total = report.security_check_tests().len();
            let security_passed = report.passed_security_checks();
            let security_failed = report.failed_security_checks();
            if security_total > 0 {
                println!(
                    "Security Checks: {}/{} passed ({} vulnerabilities detected)",
                    security_passed, security_total, security_failed
                );

                // Display vulnerability warnings
                if security_failed > 0 {
                    println!("\n⚠️  Security Vulnerabilities Found:");
                    for test in report.security_check_tests() {
                        if test.status.is_failure() {
                            // Extract vulnerability type from test name or tags
                            let vuln_type = if test.tags.contains(&"injection".to_string()) {
                                "Command Injection"
                            } else if test.tags.contains(&"path-traversal".to_string()) {
                                "Path Traversal"
                            } else {
                                "Security Issue"
                            };
                            println!("  • {} ({})", vuln_type, test.name);
                        }
                    }
                }
            }

            // Overall summary
            println!(
                "\nOverall: {}/{} tests executed in {:.2}s",
                report.total_passed() + report.total_failed(),
                report.total_tests(),
                report.total_duration.as_secs_f64()
            );

            // 5. Ensure output directory exists
            fs::create_dir_all(&output)?;

            // 6. Generate reports in requested format(s)
            let formats = match format {
                ReportFormat::All => vec![
                    ReportFormat::Markdown,
                    ReportFormat::Json,
                    ReportFormat::Html,
                    ReportFormat::Junit,
                ],
                _ => vec![format],
            };

            println!("\nGenerating reports:");
            for fmt in formats {
                match fmt {
                    ReportFormat::Markdown => {
                        let path = output.join(std::format!("{}-report.md", binary_name));
                        MarkdownReporter::generate(&report, &path)?;
                        println!("  ✓ Markdown: {}", path.display());
                    }
                    ReportFormat::Json => {
                        let path = output.join(std::format!("{}-report.json", binary_name));
                        JsonReporter::generate(&report, &path)?;
                        println!("  ✓ JSON: {}", path.display());
                    }
                    ReportFormat::Html => {
                        let path = output.join(std::format!("{}-report.html", binary_name));
                        HtmlReporter::generate(&report, &path)?;
                        println!("  ✓ HTML: {}", path.display());
                    }
                    ReportFormat::Junit => {
                        let path = output.join(std::format!("{}-junit.xml", binary_name));
                        JunitReporter::generate(&report, &path)?;
                        println!("  ✓ JUnit XML: {}", path.display());
                    }
                    ReportFormat::All => {
                        // Already expanded above
                        unreachable!()
                    }
                }
            }

            println!("\n✓ Test execution complete");
            println!("  Reports directory: {}", output.display());

            // 7. Exit with appropriate code
            if report.all_passed() {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }

        Commands::Validate { file } => {
            log::info!("Validating: {:?}", file);
            println!("Validate command - Implementation planned for Phase 2");
            Ok(())
        }

        Commands::Completion { shell } => {
            log::info!("Generating completion for shell: {:?}", shell);

            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();

            generate(shell, &mut cmd, bin_name, &mut io::stdout());

            eprintln!("\n# Completion script generated for {}", shell);
            eprintln!("# To install:");
            match shell {
                Shell::Bash => {
                    eprintln!("# Add to ~/.bashrc:");
                    eprintln!("#   eval \"$(cli-testing-specialist completion bash)\"");
                }
                Shell::Zsh => {
                    eprintln!("# Add to ~/.zshrc:");
                    eprintln!("#   eval \"$(cli-testing-specialist completion zsh)\"");
                }
                Shell::Fish => {
                    eprintln!("# Save to ~/.config/fish/completions/cli-testing-specialist.fish:");
                    eprintln!("#   cli-testing-specialist completion fish > ~/.config/fish/completions/cli-testing-specialist.fish");
                }
                Shell::PowerShell => {
                    eprintln!("# Add to PowerShell profile:");
                    eprintln!("#   cli-testing-specialist completion powershell | Out-String | Invoke-Expression");
                }
                Shell::Elvish => {
                    eprintln!("# Add to ~/.elvish/rc.elv:");
                    eprintln!("#   eval (cli-testing-specialist completion elvish)");
                }
                _ => {}
            }

            Ok(())
        }
    }
}

/// Parse test categories from comma-separated string or "all"
fn parse_categories(categories_str: &str, include_intensive: bool) -> Result<Vec<TestCategory>> {
    if categories_str.trim().to_lowercase() == "all" {
        // "all" respects the include_intensive flag
        return if include_intensive {
            Ok(TestCategory::all())
        } else {
            Ok(TestCategory::standard_categories())
        };
    }

    let mut categories = Vec::new();

    for part in categories_str.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            let category = trimmed.parse::<TestCategory>().map_err(|_| {
                cli_testing_specialist::error::Error::Config(format!(
                    "Invalid test category: '{}'. Valid categories: {}",
                    trimmed,
                    TestCategory::all()
                        .iter()
                        .map(|c| c.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })?;

            // Filter out intensive tests if not explicitly included
            if !include_intensive && TestCategory::intensive().contains(&category) {
                log::warn!(
                    "Skipping resource-intensive category '{}' (use --include-intensive to enable)",
                    category.as_str()
                );
                continue;
            }

            categories.push(category);
        }
    }

    if categories.is_empty() {
        return Err(cli_testing_specialist::error::Error::Config(
            "No valid test categories specified".to_string(),
        ));
    }

    Ok(categories)
}
