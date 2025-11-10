#!/usr/bin/env bash
#
# en.sh - English Messages for CLI Testing Specialist
#
# Format: Key-Value associative array (individual assignments for Bash compatibility)
# Keys: snake_case (e.g., cli_analysis_started)
# Values: English messages
#
# Note: MESSAGES array is declared in utils/i18n-loader.sh

# CLI Analyzer Messages
MESSAGES[cli_analysis_started]="Starting CLI analysis"
MESSAGES[cli_analysis_completed]="CLI analysis completed"
MESSAGES[analyzing_cli_tool]="Analyzing CLI tool: %s"
MESSAGES[input_binary]="Input binary: %s"
MESSAGES[output_file]="Output file: %s"
MESSAGES[binary_validation_failed]="Binary validation failed"

# Help Fetching Messages
MESSAGES[fetching_main_help]="Fetching main help..."
MESSAGES[failed_to_get_help]="Failed to get help (exit code: %s)"
MESSAGES[timeout_getting_help]="Help fetching timed out: %s"
MESSAGES[no_help_output]="No help output available"
MESSAGES[failed_to_get_help_for_subcommand]="Failed to get help for subcommand: %s"

# Subcommand & Option Messages
MESSAGES[extracting_subcommands]="Extracting subcommands..."
MESSAGES[detected_subcommands]="Detected subcommands: %s"
MESSAGES[extracting_options]="Extracting options..."
MESSAGES[detected_options]="Detected options: %s"
MESSAGES[processing_subcommands]="Processing subcommands: %s/%s"
MESSAGES[analyzing_subcommand]="Analyzing subcommand: %s (depth: %s)"
MESSAGES[no_subcommands_empty_tree]="No subcommands, returning empty tree"

# Command Tree Messages
MESSAGES[building_command_tree]="Building command tree (max depth: %s)..."
MESSAGES[command_tree_built]="Command tree built successfully"
MESSAGES[max_depth_reached]="Maximum depth reached, stopping recursion"
MESSAGES[recursively_analyzing]="Recursively analyzing: %s (depth: %s/%s)"

# JSON Generation Messages
MESSAGES[generating_json_analysis]="Generating JSON analysis..."
MESSAGES[analysis_completed_successfully]="Analysis completed successfully"
MESSAGES[analysis_failed]="Analysis failed: %s"

# Logger Messages
MESSAGES[logger_initialized]="Logger initialized: log_file=%s"
MESSAGES[log_level_changed]="Log level changed: %s → %s"
MESSAGES[invalid_log_level]="Invalid log level: %s (valid: DEBUG, INFO, WARN, ERROR)"
MESSAGES[rotating_log_file]="Rotating log file..."

# Dependency Messages
MESSAGES[jq_not_installed]="jq is not installed"
MESSAGES[jq_install_instruction]="Install: brew install jq (macOS) or apt-get install jq (Linux)"
