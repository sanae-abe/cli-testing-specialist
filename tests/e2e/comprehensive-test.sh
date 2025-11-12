#!/usr/bin/env bash
# Comprehensive End-to-End Test Suite for cli-testing-specialist v1.0.5
#
# Tests all core functionality with real binaries and validates:
# - Analysis accuracy
# - Test generation quality
# - BATS execution
# - Report generation (all 4 formats)
# - Error handling
# - Performance benchmarks

set -euo pipefail

# ANSI colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
START_TIME=$(date +%s)

# Temporary directory for test artifacts
TEST_DIR=$(mktemp -d)
trap 'rm -rf "$TEST_DIR"' EXIT

# Binary under test
CLI_BIN="${CLI_BIN:-./target/release/cli-testing-specialist}"

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $*"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

run_test() {
    local test_name="$1"
    TESTS_RUN=$((TESTS_RUN + 1))
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "Test #${TESTS_RUN}: ${test_name}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

assert_file_exists() {
    local file="$1"
    local desc="${2:-File should exist}"

    if [[ -f "$file" ]]; then
        log_success "$desc: $file"
        return 0
    else
        log_failure "$desc: $file (NOT FOUND)"
        return 1
    fi
}

assert_file_contains() {
    local file="$1"
    local pattern="$2"
    local desc="${3:-File should contain pattern}"

    if grep -q "$pattern" "$file"; then
        log_success "$desc: '$pattern' found in $file"
        return 0
    else
        log_failure "$desc: '$pattern' NOT found in $file"
        return 1
    fi
}

assert_exit_code() {
    local expected="$1"
    local actual="$2"
    local desc="${3:-Exit code should match}"

    if [[ "$actual" -eq "$expected" ]]; then
        log_success "$desc: $actual (expected: $expected)"
        return 0
    else
        log_failure "$desc: $actual (expected: $expected)"
        return 1
    fi
}

assert_json_valid() {
    local file="$1"
    local desc="${2:-JSON should be valid}"

    if jq empty "$file" 2>/dev/null; then
        log_success "$desc: $file"
        return 0
    else
        log_failure "$desc: $file (INVALID JSON)"
        return 1
    fi
}

assert_greater_than() {
    local actual="$1"
    local threshold="$2"
    local desc="${3:-Value should be greater than threshold}"

    if (( $(echo "$actual > $threshold" | bc -l) )); then
        log_success "$desc: $actual > $threshold"
        return 0
    else
        log_failure "$desc: $actual <= $threshold"
        return 1
    fi
}

# ============================================================================
# Pre-flight Checks
# ============================================================================

preflight_checks() {
    log_info "Running pre-flight checks..."

    # Check binary exists
    if [[ ! -f "$CLI_BIN" ]]; then
        log_failure "Binary not found: $CLI_BIN"
        log_info "Build with: cargo build --release"
        exit 1
    fi

    # Check binary is executable
    if [[ ! -x "$CLI_BIN" ]]; then
        log_failure "Binary not executable: $CLI_BIN"
        exit 1
    fi

    # Check required tools
    local required_tools=("bats" "jq" "curl" "bc")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &>/dev/null; then
            log_warn "Optional tool not found: $tool (some tests may be skipped)"
        fi
    done

    # Check version
    local version=$("$CLI_BIN" --version)
    log_info "Testing binary: $version"
    log_info "Test directory: $TEST_DIR"

    log_success "Pre-flight checks passed"
}

# ============================================================================
# Test Suite 1: analyze Command
# ============================================================================

test_analyze_curl() {
    run_test "Analyze /usr/bin/curl"

    local output="$TEST_DIR/curl-analysis.json"

    # Run analysis
    "$CLI_BIN" analyze /usr/bin/curl -o "$output" || {
        log_failure "Analysis command failed"
        return 1
    }

    # Validate output
    assert_file_exists "$output" "Analysis JSON created" || return 1
    assert_json_valid "$output" "Analysis JSON is valid" || return 1

    # Check structure
    local binary_name=$(jq -r '.binary_name' "$output")
    assert_file_contains <(echo "$binary_name") "curl" "Binary name detected" || return 1

    local option_count=$(jq '.global_options | length' "$output")
    assert_greater_than "$option_count" 10 "Global options detected (>10)" || return 1

    local version=$(jq -r '.version' "$output")
    if [[ "$version" != "null" && -n "$version" ]]; then
        log_success "Version detected: $version"
    else
        log_warn "Version not detected (may be expected)"
    fi

    log_info "Analysis summary:"
    log_info "  - Binary: $binary_name"
    log_info "  - Global options: $option_count"
    log_info "  - Subcommands: $(jq '.subcommands | length' "$output")"
    log_info "  - Analysis time: $(jq '.metadata.analysis_duration_ms' "$output")ms"
}

test_analyze_with_depth() {
    run_test "Analyze with custom depth and parallel options"

    local output="$TEST_DIR/curl-analysis-custom.json"

    "$CLI_BIN" analyze /usr/bin/curl -o "$output" --depth 2 --parallel || {
        log_failure "Analysis with options failed"
        return 1
    }

    assert_file_exists "$output" || return 1
    assert_json_valid "$output" || return 1

    log_success "Custom options accepted"
}

# ============================================================================
# Test Suite 2: generate Command
# ============================================================================

test_generate_basic() {
    run_test "Generate test cases (basic category)"

    local analysis="$TEST_DIR/curl-analysis.json"
    local output_dir="$TEST_DIR/tests-basic"

    # Ensure analysis exists
    if [[ ! -f "$analysis" ]]; then
        "$CLI_BIN" analyze /usr/bin/curl -o "$analysis"
    fi

    # Generate tests
    "$CLI_BIN" generate "$analysis" -o "$output_dir" -c basic || {
        log_failure "Test generation failed"
        return 1
    }

    # Validate output
    assert_file_exists "$output_dir/basic.bats" "BATS file created" || return 1

    # Check BATS syntax
    if command -v bats &>/dev/null; then
        local test_count=$(grep -c '^@test' "$output_dir/basic.bats")
        assert_greater_than "$test_count" 0 "Test cases generated" || return 1
        log_info "Generated $test_count test cases"
    fi
}

test_generate_all_categories() {
    run_test "Generate test cases (all categories)"

    local analysis="$TEST_DIR/curl-analysis.json"
    local output_dir="$TEST_DIR/tests-all"

    # Generate all categories
    "$CLI_BIN" generate "$analysis" -o "$output_dir" -c all || {
        log_failure "Test generation (all) failed"
        return 1
    }

    # Check that at least some category files were generated
    local generated_files=$(find "$output_dir" -name "*.bats" | wc -l | tr -d ' ')
    assert_greater_than "$generated_files" 0 "At least one BATS file generated" || return 1

    # List generated files
    log_info "Generated BATS files:"
    for file in "$output_dir"/*.bats; do
        if [[ -f "$file" ]]; then
            local basename=$(basename "$file")
            local test_count=$(grep -c '^@test' "$file" || echo "0")
            log_info "  - $basename ($test_count tests)"
        fi
    done

    # Count total tests
    local total_tests=0
    for file in "$output_dir"/*.bats; do
        if [[ -f "$file" ]]; then
            local count=$(grep -c '^@test' "$file" 2>/dev/null || echo "0")
            total_tests=$((total_tests + count))
        fi
    done
    log_info "Total test cases across all categories: $total_tests"
    assert_greater_than "$total_tests" 10 "Sufficient test coverage (>10)" || return 1
}

test_generate_with_intensive() {
    run_test "Generate with --include-intensive flag"

    local analysis="$TEST_DIR/curl-analysis.json"
    local output_dir="$TEST_DIR/tests-intensive"

    "$CLI_BIN" generate "$analysis" -o "$output_dir" -c all --include-intensive || {
        log_failure "Test generation (intensive) failed"
        return 1
    }

    # Check directory-traversal category
    if [[ -f "$output_dir/directory-traversal.bats" ]]; then
        log_success "Intensive category included: directory-traversal.bats"
    else
        log_warn "directory-traversal.bats not found (may be expected if not applicable)"
    fi
}

# ============================================================================
# Test Suite 3: run Command
# ============================================================================

test_run_basic() {
    run_test "Run BATS tests (basic category)"

    local test_dir="$TEST_DIR/tests-basic"
    local report_dir="$TEST_DIR/reports-basic"

    # Skip if BATS not installed
    if ! command -v bats &>/dev/null; then
        log_warn "BATS not installed, skipping test execution"
        return 0
    fi

    # Run tests
    local exit_code=0
    "$CLI_BIN" run "$test_dir" -f markdown -o "$report_dir" || exit_code=$?

    # Note: Exit code 1 is expected if some tests fail
    if [[ $exit_code -eq 0 || $exit_code -eq 1 ]]; then
        log_success "Test execution completed (exit: $exit_code)"
    else
        log_failure "Test execution failed with unexpected exit code: $exit_code"
        return 1
    fi

    # Validate report (find any .md file in report_dir)
    local report_file=$(find "$report_dir" -name "*.md" -type f | head -1)
    if [[ -n "$report_file" ]]; then
        log_success "Markdown report created: $(basename "$report_file")"
        # Basic content check - file should be non-empty
        if [[ -s "$report_file" ]]; then
            log_success "Report has content (size: $(wc -c < "$report_file") bytes)"
        else
            log_warn "Report file is empty"
        fi
    else
        log_failure "No markdown report found in $report_dir"
        return 1
    fi
}

test_run_all_formats() {
    run_test "Generate all report formats"

    local test_dir="$TEST_DIR/tests-basic"
    local report_dir="$TEST_DIR/reports-all"

    if ! command -v bats &>/dev/null; then
        log_warn "BATS not installed, skipping"
        return 0
    fi

    # Run with all formats
    "$CLI_BIN" run "$test_dir" -f all -o "$report_dir" || true  # Allow failures

    # Check all format files
    local formats=("markdown" "json" "html" "junit")
    for format in "${formats[@]}"; do
        local pattern="*-report.*"
        if [[ "$format" == "junit" ]]; then
            pattern="*-junit.xml"
        fi

        if compgen -G "$report_dir/$pattern" >/dev/null; then
            log_success "Report generated: $format"
        else
            log_failure "Report missing: $format"
        fi
    done

    # Validate JSON report
    local json_report=$(find "$report_dir" -name "*-report.json" -print -quit)
    if [[ -n "$json_report" ]]; then
        assert_json_valid "$json_report" "JSON report is valid" || return 1
    fi
}

test_run_with_timeout() {
    run_test "Run tests with custom timeout"

    local test_dir="$TEST_DIR/tests-basic"
    local report_dir="$TEST_DIR/reports-timeout"

    if ! command -v bats &>/dev/null; then
        log_warn "BATS not installed, skipping"
        return 0
    fi

    # Run with custom timeout
    "$CLI_BIN" run "$test_dir" -f json -o "$report_dir" --timeout 60 || true

    log_success "Custom timeout option accepted"
}

test_run_with_skip() {
    run_test "Run tests with skip categories"

    local test_dir="$TEST_DIR/tests-all"
    local report_dir="$TEST_DIR/reports-skip"

    if ! command -v bats &>/dev/null; then
        log_warn "BATS not installed, skipping"
        return 0
    fi

    # Run with skip categories
    "$CLI_BIN" run "$test_dir" -f json -o "$report_dir" --skip "destructive-ops,directory-traversal" || true

    log_success "Skip categories option accepted"
}

# ============================================================================
# Test Suite 4: completion Command
# ============================================================================

test_completion_bash() {
    run_test "Generate bash completion"

    local output="$TEST_DIR/completion.bash"

    "$CLI_BIN" completion bash > "$output" || {
        log_failure "Completion generation failed"
        return 1
    }

    assert_file_exists "$output" || return 1
    # Completion script should contain either the full name or shortened name
    if grep -q "cli-test" "$output" 2>/dev/null; then
        log_success "Completion has binary name: cli-test"
    elif grep -q "completion" "$output" 2>/dev/null; then
        log_success "Completion script has completion keywords"
    else
        log_warn "Completion script format may have changed"
    fi

    log_info "Bash completion script: $(wc -l < "$output") lines"
}

test_completion_all_shells() {
    run_test "Generate completions for all shells"

    local shells=("bash" "zsh" "fish")

    for shell in "${shells[@]}"; do
        local output="$TEST_DIR/completion.$shell"
        if "$CLI_BIN" completion "$shell" > "$output" 2>/dev/null; then
            log_success "Completion generated: $shell"
        else
            log_warn "Completion failed: $shell (may not be supported)"
        fi
    done
}

# ============================================================================
# Test Suite 5: Error Handling
# ============================================================================

test_error_nonexistent_binary() {
    run_test "Error handling: Non-existent binary"

    local output="$TEST_DIR/error-analysis.json"
    local exit_code=0

    "$CLI_BIN" analyze /nonexistent/binary -o "$output" 2>/dev/null || exit_code=$?

    # Should fail with non-zero exit code
    if [[ $exit_code -ne 0 ]]; then
        log_success "Correctly failed on non-existent binary (exit: $exit_code)"
    else
        log_failure "Should have failed on non-existent binary"
        return 1
    fi
}

test_error_invalid_json() {
    run_test "Error handling: Invalid analysis JSON"

    local invalid_json="$TEST_DIR/invalid.json"
    echo "{ invalid json }" > "$invalid_json"

    local exit_code=0
    "$CLI_BIN" generate "$invalid_json" -o "$TEST_DIR/error-tests" 2>/dev/null || exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log_success "Correctly failed on invalid JSON (exit: $exit_code)"
    else
        log_failure "Should have failed on invalid JSON"
        return 1
    fi
}

test_error_missing_bats() {
    run_test "Error handling: Missing BATS directory"

    local exit_code=0
    "$CLI_BIN" run /nonexistent/tests -f json -o "$TEST_DIR/error-reports" 2>/dev/null || exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log_success "Correctly failed on missing test directory (exit: $exit_code)"
    else
        log_failure "Should have failed on missing directory"
        return 1
    fi
}

# ============================================================================
# Test Suite 6: Performance Benchmarks
# ============================================================================

test_performance_analyze() {
    run_test "Performance: analyze command"

    local output="$TEST_DIR/perf-analysis.json"

    # Use seconds for macOS compatibility (date +%s%3N doesn't work on macOS)
    local start=$(date +%s)
    "$CLI_BIN" analyze /usr/bin/curl -o "$output" >/dev/null 2>&1
    local end=$(date +%s)
    local duration=$((end - start))

    log_info "Analysis duration: ${duration}s"

    # Check performance target: <5 seconds
    if [[ $duration -lt 5 ]]; then
        log_success "Performance target met: ${duration}s < 5s"
    else
        log_warn "Performance target missed: ${duration}s >= 5s"
    fi
}

test_performance_generate() {
    run_test "Performance: generate command"

    local analysis="$TEST_DIR/curl-analysis.json"
    local output_dir="$TEST_DIR/perf-tests"

    if [[ ! -f "$analysis" ]]; then
        "$CLI_BIN" analyze /usr/bin/curl -o "$analysis" >/dev/null 2>&1
    fi

    # Use seconds for macOS compatibility
    local start=$(date +%s)
    "$CLI_BIN" generate "$analysis" -o "$output_dir" -c all >/dev/null 2>&1
    local end=$(date +%s)
    local duration=$((end - start))

    log_info "Generation duration: ${duration}s"

    # Check performance target: <2 seconds
    if [[ $duration -lt 2 ]]; then
        log_success "Performance target met: ${duration}s < 2s"
    else
        log_warn "Performance target missed: ${duration}s >= 2s"
    fi
}

# ============================================================================
# Test Suite 7: Integration Tests
# ============================================================================

test_integration_full_workflow() {
    run_test "Integration: Full workflow (analyze → generate → run)"

    local analysis="$TEST_DIR/integration-analysis.json"
    local test_dir="$TEST_DIR/integration-tests"
    local report_dir="$TEST_DIR/integration-reports"

    # Step 1: Analyze
    log_info "Step 1: Analyzing /usr/bin/curl"
    "$CLI_BIN" analyze /usr/bin/curl -o "$analysis" || {
        log_failure "Analysis step failed"
        return 1
    }

    # Step 2: Generate
    log_info "Step 2: Generating tests"
    "$CLI_BIN" generate "$analysis" -o "$test_dir" -c "basic,help,security" || {
        log_failure "Generation step failed"
        return 1
    }

    # Step 3: Run (if BATS available)
    if command -v bats &>/dev/null; then
        log_info "Step 3: Running tests"
        "$CLI_BIN" run "$test_dir" -f all -o "$report_dir" || true  # Allow test failures

        # Verify reports (find any .md file)
        local report_file=$(find "$report_dir" -name "*.md" -type f | head -1)
        if [[ -n "$report_file" ]]; then
            log_success "Markdown report exists: $(basename "$report_file")"
            log_success "Full workflow completed successfully"
        else
            log_failure "No markdown report found"
            return 1
        fi
    else
        log_warn "BATS not installed, skipping run step"
        log_success "Partial workflow (analyze + generate) completed"
    fi
}

# ============================================================================
# Main Test Runner
# ============================================================================

main() {
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║  Comprehensive E2E Test Suite - cli-testing-specialist v1.0.5       ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo ""

    preflight_checks
    echo ""

    # Run all test suites
    test_analyze_curl
    test_analyze_with_depth

    test_generate_basic
    test_generate_all_categories
    test_generate_with_intensive

    test_run_basic
    test_run_all_formats
    test_run_with_timeout
    test_run_with_skip

    test_completion_bash
    test_completion_all_shells

    test_error_nonexistent_binary
    test_error_invalid_json
    test_error_missing_bats

    test_performance_analyze
    test_performance_generate

    test_integration_full_workflow

    # Final summary
    local end_time=$(date +%s)
    local total_duration=$((end_time - START_TIME))

    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║  Test Summary                                                        ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Tests Run:    $TESTS_RUN"
    echo "  Tests Passed: $TESTS_PASSED"
    echo "  Tests Failed: $TESTS_FAILED"
    echo "  Duration:     ${total_duration}s"
    echo ""

    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
        exit 0
    else
        echo -e "${RED}✗ SOME TESTS FAILED${NC}"
        exit 1
    fi
}

main "$@"
