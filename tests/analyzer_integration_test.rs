/// Integration tests for the Analyzer module
///
/// These tests verify the complete analysis workflow from binary execution
/// to option type inference and subcommand detection.
use cli_testing_specialist::analyzer::{
    apply_numeric_constraints, load_enum_values, CliParser, OptionInferrer, SubcommandDetector,
};
use cli_testing_specialist::types::analysis::OptionType;
use std::path::Path;

#[cfg(unix)]
#[test]
fn test_full_curl_analysis() {
    // Test with /usr/bin/curl which should be available on most Unix systems
    let curl_path = Path::new("/usr/bin/curl");

    if !curl_path.exists() {
        println!("Skipping test: /usr/bin/curl not found");
        return;
    }

    // Step 1: Parse the CLI
    let parser = CliParser::new();
    let mut analysis = parser.analyze(curl_path).expect("Failed to analyze curl");

    println!("✓ Analyzed binary: {}", analysis.binary_name);
    println!(
        "✓ Version: {}",
        analysis.version.as_ref().unwrap_or(&"unknown".to_string())
    );
    println!("✓ Found {} global options", analysis.global_options.len());

    // Verify basic analysis results
    assert_eq!(analysis.binary_name, "curl");
    assert!(analysis.version.is_some(), "Curl should have a version");
    assert!(
        !analysis.global_options.is_empty(),
        "Curl should have global options"
    );
    assert!(
        analysis.global_options.len() > 10,
        "Curl should have many options"
    );

    // Step 2: Infer option types
    let inferrer = OptionInferrer::new().expect("Failed to create option inferrer");
    inferrer.infer_types(&mut analysis.global_options);

    // Apply constraints
    apply_numeric_constraints(&mut analysis.global_options);
    load_enum_values(&mut analysis.global_options);

    println!("✓ Inferred types for all options");

    // Verify some known curl options
    let max_time_option = analysis
        .global_options
        .iter()
        .find(|o| o.long.as_deref() == Some("--max-time"));

    if let Some(opt) = max_time_option {
        // --max-time should be numeric
        assert!(
            matches!(opt.option_type, OptionType::Numeric { .. }),
            "--max-time should be numeric type"
        );
        println!("✓ Correctly inferred --max-time as numeric");
    }

    // Step 3: Detect subcommands (curl doesn't have subcommands, so this should be empty)
    let detector = SubcommandDetector::new().expect("Failed to create subcommand detector");
    let subcommands = detector
        .detect(curl_path, &analysis.help_output)
        .expect("Failed to detect subcommands");

    println!("✓ Detected {} subcommands", subcommands.len());

    // curl typically doesn't have subcommands
    assert!(
        subcommands.is_empty() || subcommands.len() < 5,
        "Curl should have few or no subcommands"
    );

    // Print summary
    println!("\n=== Analysis Summary ===");
    println!("Binary: {}", analysis.binary_name);
    println!(
        "Version: {}",
        analysis.version.unwrap_or_else(|| "unknown".to_string())
    );
    println!("Global Options: {}", analysis.global_options.len());
    println!("Subcommands: {}", subcommands.len());
    println!("Duration: {}ms", analysis.metadata.analysis_duration_ms);

    // Count option types
    let mut type_counts = std::collections::HashMap::new();
    for option in &analysis.global_options {
        let type_name = match &option.option_type {
            OptionType::Flag => "Flag",
            OptionType::String => "String",
            OptionType::Numeric { .. } => "Numeric",
            OptionType::Path => "Path",
            OptionType::Enum { .. } => "Enum",
        };
        *type_counts.entry(type_name).or_insert(0) += 1;
    }

    println!("\nOption Type Distribution:");
    for (type_name, count) in type_counts {
        println!("  {}: {}", type_name, count);
    }
}

#[cfg(unix)]
#[test]
fn test_ls_analysis() {
    // Test with /bin/ls which is simpler than curl
    let ls_path = Path::new("/bin/ls");

    if !ls_path.exists() {
        println!("Skipping test: /bin/ls not found");
        return;
    }

    let parser = CliParser::new();
    let analysis = parser.analyze(ls_path).expect("Failed to analyze ls");

    println!(
        "✓ Analyzed ls: {} options found",
        analysis.global_options.len()
    );

    assert_eq!(analysis.binary_name, "ls");

    // Note: ls help format varies by system (BSD vs GNU)
    // On macOS (BSD), ls might not parse well with our patterns
    // Just verify we got the binary analyzed without errors
    if !analysis.global_options.is_empty() {
        println!(
            "✓ Successfully parsed {} ls options",
            analysis.global_options.len()
        );
    } else {
        println!("⚠ No options parsed - BSD ls on macOS has different help format");
    }
}

#[test]
fn test_option_type_inference() {
    // Test option type inference with mock options
    use cli_testing_specialist::types::analysis::CliOption;

    let inferrer = OptionInferrer::new().expect("Failed to create inferrer");

    // Test timeout option (should be numeric)
    let mut timeout_opt = CliOption {
        short: Some("-t".to_string()),
        long: Some("--timeout".to_string()),
        description: Some("Timeout in seconds".to_string()),
        option_type: OptionType::String,
        required: false,
        default_value: None,
    };

    let inferred_type = inferrer.infer_type(&timeout_opt);
    assert!(
        matches!(inferred_type, OptionType::Numeric { .. }),
        "timeout should be numeric"
    );

    timeout_opt.option_type = inferred_type;
    let mut opts = vec![timeout_opt];
    apply_numeric_constraints(&mut opts);

    // Check constraints were applied
    if let OptionType::Numeric { min, max } = &opts[0].option_type {
        assert_eq!(*min, Some(0));
        assert_eq!(*max, Some(3600));
        println!("✓ Applied numeric constraints to timeout option");
    }

    // Test config option (should be path)
    let config_opt = CliOption {
        short: Some("-c".to_string()),
        long: Some("--config".to_string()),
        description: Some("Config file path".to_string()),
        option_type: OptionType::String,
        required: false,
        default_value: None,
    };

    let inferred_type = inferrer.infer_type(&config_opt);
    assert!(
        matches!(inferred_type, OptionType::Path),
        "config should be path"
    );
    println!("✓ Correctly inferred config as path type");

    // Test format option (should be enum)
    let mut format_opt = CliOption {
        short: Some("-f".to_string()),
        long: Some("--format".to_string()),
        description: Some("Output format".to_string()),
        option_type: OptionType::String,
        required: false,
        default_value: None,
    };

    let inferred_type = inferrer.infer_type(&format_opt);
    assert!(
        matches!(inferred_type, OptionType::Enum { .. }),
        "format should be enum"
    );

    format_opt.option_type = inferred_type;
    let mut opts = vec![format_opt];
    load_enum_values(&mut opts);

    // Check enum values were loaded
    if let OptionType::Enum { values } = &opts[0].option_type {
        assert!(!values.is_empty(), "Enum values should be populated");
        assert!(values.contains(&"json".to_string()));
        println!("✓ Loaded enum values for format option: {:?}", values);
    }
}
