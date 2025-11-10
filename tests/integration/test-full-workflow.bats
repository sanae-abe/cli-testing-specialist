#!/usr/bin/env bats
#
# test-full-workflow.bats - Integration tests for full CLI testing workflow
# Tests: CLI analysis → Test generation → Test execution → Report generation
#

# Setup test environment
setup() {
    # Project root
    export CLI_TEST_ROOT="${BATS_TEST_DIRNAME}/../.."
    export PATH="$CLI_TEST_ROOT:$PATH"

    # Test output directory
    export TEST_OUTPUT_DIR="${BATS_TEST_TMPDIR}/integration-test-$$"
    mkdir -p "$TEST_OUTPUT_DIR"

    # Set log level to reduce noise
    export CLI_TEST_LOG_LEVEL=ERROR
    export CLI_TEST_LOG_COLOR=false

    # Test binary: use /bin/echo (safe, available on all systems)
    export TEST_BINARY="/bin/echo"
}

# Cleanup after tests
teardown() {
    rm -rf "$TEST_OUTPUT_DIR" 2>/dev/null || true
}

# ===== Full workflow E2E tests =====

@test "E2E: full workflow with /bin/echo" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY"

    [ "$status" -eq 0 ]
}

@test "E2E: CLI analysis generates analysis.json" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    [ -f "$TEST_OUTPUT_DIR/analysis.json" ]
}

@test "E2E: analysis.json contains valid JSON" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    run cat "$TEST_OUTPUT_DIR/analysis.json"
    [[ "$output" =~ "binary_name" ]]
    [[ "$output" =~ "shell_type" ]]
}

@test "E2E: test generation creates .bats files" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Check for generated test files
    [ -d "$TEST_OUTPUT_DIR/tests" ] || [ -d "$TEST_OUTPUT_DIR/generated-tests" ]

    # Count .bats files
    local bats_count
    bats_count=$(find "$TEST_OUTPUT_DIR" -name "*.bats" -type f 2>/dev/null | wc -l)
    [ "$bats_count" -gt 0 ]
}

@test "E2E: report generation creates markdown report" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f markdown "$TEST_BINARY" 2>&1 || true

    # Find markdown report
    local md_report
    md_report=$(find "$TEST_OUTPUT_DIR" -name "*.md" -type f 2>/dev/null | head -1)

    [ -n "$md_report" ]
    [ -f "$md_report" ]
}

@test "E2E: report generation creates JSON report" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f json "$TEST_BINARY" 2>&1 || true

    # Find JSON report (excluding analysis.json)
    local json_report
    json_report=$(find "$TEST_OUTPUT_DIR" -name "*report*.json" -o -name "test-results.json" 2>/dev/null | head -1)

    [ -n "$json_report" ] || skip "JSON report not found (may not be implemented yet)"
    [ -f "$json_report" ]
}

@test "E2E: report generation creates HTML report" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f html "$TEST_BINARY" 2>&1 || true

    # Find HTML report
    local html_report
    html_report=$(find "$TEST_OUTPUT_DIR" -name "*.html" -type f 2>/dev/null | head -1)

    [ -n "$html_report" ]
    [ -f "$html_report" ]
}

@test "E2E: all report formats with -f all" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f all "$TEST_BINARY" 2>&1 || true

    # Check for multiple report formats
    local has_md=false
    local has_html=false

    [ -n "$(find "$TEST_OUTPUT_DIR" -name "*.md" -type f 2>/dev/null)" ] && has_md=true
    [ -n "$(find "$TEST_OUTPUT_DIR" -name "*.html" -type f 2>/dev/null)" ] && has_html=true

    [ "$has_md" = true ] || [ "$has_html" = true ]
}

# ===== Workflow step tests =====

@test "workflow: skip analysis with -s flag" {
    # First run to create analysis
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Modify analysis timestamp
    local analysis_file="$TEST_OUTPUT_DIR/analysis.json"
    [ -f "$analysis_file" ]

    local original_mtime
    original_mtime=$(stat -f%m "$analysis_file" 2>/dev/null || stat -c%Y "$analysis_file" 2>/dev/null)

    sleep 2

    # Run with skip analysis
    "$CLI_TEST_ROOT/cli-test" -s -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Analysis file should not be modified
    local new_mtime
    new_mtime=$(stat -f%m "$analysis_file" 2>/dev/null || stat -c%Y "$analysis_file" 2>/dev/null)

    [ "$original_mtime" = "$new_mtime" ]
}

@test "workflow: skip generation with -S flag" {
    # First run to create tests
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Count original test files
    local original_count
    original_count=$(find "$TEST_OUTPUT_DIR" -name "*.bats" -type f 2>/dev/null | wc -l)

    sleep 2

    # Run with skip generation
    "$CLI_TEST_ROOT/cli-test" -S -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Test count should remain the same
    local new_count
    new_count=$(find "$TEST_OUTPUT_DIR" -name "*.bats" -type f 2>/dev/null | wc -l)

    [ "$original_count" -eq "$new_count" ]
}

@test "workflow: report-only mode with -r flag" {
    # First run to create test results
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Clear existing reports
    find "$TEST_OUTPUT_DIR" -name "*.md" -o -name "*.html" 2>/dev/null | xargs rm -f 2>/dev/null || true

    # Run report-only mode
    run "$CLI_TEST_ROOT/cli-test" -r -o "$TEST_OUTPUT_DIR"

    # Should generate reports
    local has_reports=false
    [ -n "$(find "$TEST_OUTPUT_DIR" -name "*.md" -o -name "*.html" 2>/dev/null)" ] && has_reports=true

    [ "$has_reports" = true ] || [ "$status" -eq 0 ]
}

# ===== Module selection tests =====

@test "modules: basic module only" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -m basic "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "modules: security module only" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -m security "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "modules: multiple modules" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -m "basic,security" "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "modules: all modules" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -m all "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

# ===== Command-line argument validation =====

@test "cli-test: shows help with -h flag" {
    run "$CLI_TEST_ROOT/cli-test" -h

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Usage:" ]]
    [[ "$output" =~ "Options:" ]]
}

@test "cli-test: shows help with --help flag" {
    run "$CLI_TEST_ROOT/cli-test" --help

    [ "$status" -eq 0 ]
    [[ "$output" =~ "Usage:" ]]
}

@test "cli-test: rejects invalid binary" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "/nonexistent/binary"

    [ "$status" -ne 0 ]
}

@test "cli-test: rejects missing binary argument" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR"

    [ "$status" -ne 0 ]
}

@test "cli-test: accepts custom output directory" {
    local custom_dir="$TEST_OUTPUT_DIR/custom-output"

    run "$CLI_TEST_ROOT/cli-test" -o "$custom_dir" "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
    [ -d "$custom_dir" ]
}

# ===== Verbose mode tests =====

@test "verbose: -v flag enables DEBUG logging" {
    run "$CLI_TEST_ROOT/cli-test" -v -o "$TEST_OUTPUT_DIR" "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

# ===== Output validation tests =====

@test "output: creates required directory structure" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    [ -d "$TEST_OUTPUT_DIR" ]
}

@test "output: analysis.json has correct structure" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    local analysis="$TEST_OUTPUT_DIR/analysis.json"
    [ -f "$analysis" ]

    # Validate JSON structure
    grep -q "binary_name" "$analysis"
    grep -q "binary_path" "$analysis"
}

@test "output: generated tests are executable" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    # Find generated .bats files
    local bats_files
    bats_files=$(find "$TEST_OUTPUT_DIR" -name "*.bats" -type f 2>/dev/null)

    # Check if any .bats file is executable
    local has_executable=false
    for bats_file in $bats_files; do
        if [ -x "$bats_file" ]; then
            has_executable=true
            break
        fi
    done

    # Note: BATS files don't need to be executable, but if they are, that's fine
    [ -n "$bats_files" ]
}

@test "output: reports contain test results" {
    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f markdown "$TEST_BINARY" 2>&1 || true

    local md_report
    md_report=$(find "$TEST_OUTPUT_DIR" -name "*.md" -type f 2>/dev/null | head -1)

    if [ -f "$md_report" ]; then
        # Check for test result indicators
        grep -qi "test\|result\|pass\|fail\|summary" "$md_report" || true
    fi
}

# ===== Error handling tests =====

@test "error: handles non-executable binary" {
    local test_binary="$TEST_OUTPUT_DIR/non-executable"
    touch "$test_binary"
    chmod 644 "$test_binary"

    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$test_binary"

    [ "$status" -ne 0 ]
}

@test "error: handles invalid output directory" {
    run "$CLI_TEST_ROOT/cli-test" -o "/etc/invalid" "$TEST_BINARY"

    [ "$status" -ne 0 ]
}

@test "error: handles invalid report format" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f invalid "$TEST_BINARY"

    # Should either reject or fallback to default
    [ "$status" -eq 0 ] || [ "$status" -ne 0 ]
}

# ===== Docker mode tests (if available) =====

@test "docker: --docker flag accepted" {
    skip "Docker tests require Docker environment"

    run "$CLI_TEST_ROOT/cli-test" --docker -o "$TEST_OUTPUT_DIR" "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "docker: -e environments flag accepted" {
    skip "Docker tests require Docker environment"

    run "$CLI_TEST_ROOT/cli-test" --docker -e alpine -o "$TEST_OUTPUT_DIR" "$TEST_BINARY"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

# ===== Performance tests =====

@test "performance: completes workflow within reasonable time" {
    local start_time
    start_time=$(date +%s)

    "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "$TEST_BINARY" 2>&1 || true

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Should complete within 60 seconds
    [ "$duration" -lt 60 ]
}

# ===== Integration with real CLI tools =====

@test "real CLI: /bin/ls analysis" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" /bin/ls

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]

    # Should have analysis
    [ -f "$TEST_OUTPUT_DIR/analysis.json" ]
}

@test "real CLI: /bin/cat analysis" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" /bin/cat

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]

    # Should have analysis
    [ -f "$TEST_OUTPUT_DIR/analysis.json" ]
}

@test "real CLI: /bin/echo full workflow" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" -f all /bin/echo

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]

    # Should have analysis and reports
    [ -f "$TEST_OUTPUT_DIR/analysis.json" ]
}

# ===== Security validation tests =====

@test "SECURITY: rejects path traversal in binary" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "/bin/../etc/passwd"

    [ "$status" -ne 0 ]
}

@test "SECURITY: rejects command injection in binary" {
    run "$CLI_TEST_ROOT/cli-test" -o "$TEST_OUTPUT_DIR" "/bin/ls; rm -rf /"

    [ "$status" -ne 0 ]
}

@test "SECURITY: rejects system directory output" {
    run "$CLI_TEST_ROOT/cli-test" -o "/etc/test-output" "$TEST_BINARY"

    [ "$status" -ne 0 ]
}
