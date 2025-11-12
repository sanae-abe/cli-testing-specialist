# QA Manual Test Checklist - v1.0.5

**Version**: 1.0.5
**Last Updated**: 2025-11-12
**Test Environment**: macOS / Linux / Windows

---

## 📋 Pre-Test Setup

### Build Binary

```bash
# Release build (required for accurate performance testing)
cargo build --release

# Verify version
./target/release/cli-testing-specialist --version
# Expected: cli-testing-specialist 1.0.5
```

### Install Dependencies

```bash
# BATS (required for test execution)
brew install bats-core      # macOS
apt install bats            # Ubuntu/Debian
pacman -S bats              # Arch Linux

# jq (for JSON validation)
brew install jq             # macOS
apt install jq              # Ubuntu/Debian

# curl (usually pre-installed)
which curl
```

### Environment Variables

```bash
export CLI_BIN="./target/release/cli-testing-specialist"
export TEST_WORKSPACE="$HOME/cli-test-workspace"
mkdir -p "$TEST_WORKSPACE"
cd "$TEST_WORKSPACE"
```

---

## ✅ Test Category 1: analyze Command

### Test 1.1: Basic Analysis

**Objective**: Verify analyze command works with common binaries

**Steps**:
1. Run analysis on /usr/bin/curl
   ```bash
   $CLI_BIN analyze /usr/bin/curl -o curl-analysis.json
   ```

2. Verify output
   ```bash
   cat curl-analysis.json | jq '.binary_name'
   # Expected: "curl"

   cat curl-analysis.json | jq '.global_options | length'
   # Expected: > 50 (curl has many options)

   cat curl-analysis.json | jq '.metadata.analysis_duration_ms'
   # Expected: < 5000ms
   ```

**Pass Criteria**:
- [ ] JSON file created successfully
- [ ] binary_name is "curl"
- [ ] global_options count > 50
- [ ] Analysis completes in < 5 seconds
- [ ] No errors in console output

**Notes**:
_______________________________________________________________

---

### Test 1.2: Analysis with Options

**Objective**: Verify --depth and --parallel options work

**Steps**:
1. Run with custom depth
   ```bash
   $CLI_BIN analyze /usr/bin/curl -o curl-depth2.json --depth 2
   ```

2. Run with parallel flag
   ```bash
   $CLI_BIN analyze /usr/bin/curl -o curl-parallel.json --parallel
   ```

**Pass Criteria**:
- [ ] Both commands complete successfully
- [ ] JSON output is valid
- [ ] No performance degradation with parallel flag

**Notes**:
_______________________________________________________________

---

### Test 1.3: Error Handling - Invalid Binary

**Objective**: Verify graceful error handling

**Steps**:
1. Run analysis on non-existent file
   ```bash
   $CLI_BIN analyze /nonexistent/binary -o error.json
   ```

2. Check exit code and error message
   ```bash
   echo $?  # Should be non-zero (1)
   ```

**Pass Criteria**:
- [ ] Command exits with non-zero code
- [ ] Error message is clear and helpful
- [ ] Suggests possible solutions (e.g., "check file path")

**Error Message**:
_______________________________________________________________

---

## ✅ Test Category 2: generate Command

### Test 2.1: Generate Basic Tests

**Objective**: Verify test generation for basic category

**Steps**:
1. Generate basic tests
   ```bash
   $CLI_BIN generate curl-analysis.json -o tests/ -c basic
   ```

2. Inspect generated file
   ```bash
   ls -la tests/
   # Expected: basic.bats

   head -20 tests/basic.bats
   # Expected: BATS setup, @test blocks

   grep -c '^@test' tests/basic.bats
   # Expected: > 5 test cases
   ```

**Pass Criteria**:
- [ ] basic.bats file created
- [ ] File contains valid BATS syntax
- [ ] At least 5 test cases generated
- [ ] Test names are descriptive

**Test Count**: _____ tests

---

### Test 2.2: Generate All Categories

**Objective**: Verify all test categories generate correctly

**Steps**:
1. Generate all categories
   ```bash
   $CLI_BIN generate curl-analysis.json -o tests-all/ -c all
   ```

2. List generated files
   ```bash
   ls tests-all/*.bats
   # Expected files:
   # - basic.bats
   # - help.bats
   # - security.bats
   # - path.bats
   # - multi-shell.bats
   # - input-validation.bats
   # - destructive-ops.bats
   # - performance.bats
   ```

**Pass Criteria**:
- [ ] At least 7 BATS files created (excluding directory-traversal)
- [ ] Each file has valid BATS syntax
- [ ] Total test count > 30

**Files Generated**:
- [ ] basic.bats (_____ tests)
- [ ] help.bats (_____ tests)
- [ ] security.bats (_____ tests)
- [ ] path.bats (_____ tests)
- [ ] multi-shell.bats (_____ tests)
- [ ] input-validation.bats (_____ tests)
- [ ] destructive-ops.bats (_____ tests)
- [ ] performance.bats (_____ tests)

**Total Tests**: _____

---

### Test 2.3: Intensive Tests (Optional)

**Objective**: Verify --include-intensive flag

**Steps**:
1. Generate with intensive flag
   ```bash
   $CLI_BIN generate curl-analysis.json -o tests-intensive/ -c all --include-intensive
   ```

2. Check for directory-traversal tests
   ```bash
   ls tests-intensive/directory-traversal.bats
   ```

**Pass Criteria**:
- [ ] directory-traversal.bats exists (if applicable)
- [ ] Warning shown about resource requirements

**Notes**:
_______________________________________________________________

---

## ✅ Test Category 3: run Command

### Test 3.1: Run Basic Tests

**Objective**: Execute BATS tests and verify results

**Steps**:
1. Run basic tests
   ```bash
   $CLI_BIN run tests/ -f markdown -o reports/
   ```

2. Inspect report
   ```bash
   cat reports/curl-report.md
   ```

**Pass Criteria**:
- [ ] Tests execute successfully
- [ ] Markdown report created
- [ ] Report shows test statistics (passed/failed/skipped)
- [ ] Success rate > 50%

**Test Results**:
- Total: _____
- Passed: _____
- Failed: _____
- Skipped: _____
- Success Rate: _____%

---

### Test 3.2: All Report Formats

**Objective**: Verify all 4 report formats generate

**Steps**:
1. Run with all formats
   ```bash
   $CLI_BIN run tests-all/ -f all -o reports-all/
   ```

2. Check generated files
   ```bash
   ls -la reports-all/
   ```

**Pass Criteria**:
- [ ] Markdown report (*.md)
- [ ] JSON report (*.json) - valid JSON
- [ ] HTML report (*.html) - opens in browser
- [ ] JUnit XML (*-junit.xml) - valid XML

**File Sizes**:
- Markdown: _____ bytes
- JSON: _____ bytes
- HTML: _____ bytes
- JUnit XML: _____ bytes

---

### Test 3.3: Custom Timeout

**Objective**: Verify --timeout option works

**Steps**:
1. Run with 60s timeout
   ```bash
   $CLI_BIN run tests/ -f json -o reports-timeout/ --timeout 60
   ```

**Pass Criteria**:
- [ ] Command accepts --timeout flag
- [ ] Tests complete within timeout
- [ ] No timeout errors

**Notes**:
_______________________________________________________________

---

### Test 3.4: Skip Categories

**Objective**: Verify --skip option works

**Steps**:
1. Run with skip categories
   ```bash
   $CLI_BIN run tests-all/ -f json -o reports-skip/ --skip "destructive-ops,performance"
   ```

2. Check which tests ran
   ```bash
   cat reports-skip/curl-report.json | jq '.suites[].name'
   ```

**Pass Criteria**:
- [ ] destructive-ops tests not executed
- [ ] performance tests not executed
- [ ] Other categories executed normally

**Categories Executed**:
_______________________________________________________________

---

## ✅ Test Category 4: completion Command

### Test 4.1: Bash Completion

**Objective**: Verify bash completion generation

**Steps**:
1. Generate bash completion
   ```bash
   $CLI_BIN completion bash > cli-completion.bash
   ```

2. Inspect output
   ```bash
   head -10 cli-completion.bash
   ```

**Pass Criteria**:
- [ ] File created successfully
- [ ] Contains bash completion syntax
- [ ] Mentions "cli-testing-specialist"

**Notes**:
_______________________________________________________________

---

### Test 4.2: Multiple Shells

**Objective**: Verify all shell completions

**Steps**:
1. Generate for all shells
   ```bash
   $CLI_BIN completion bash > completion.bash
   $CLI_BIN completion zsh > completion.zsh
   $CLI_BIN completion fish > completion.fish
   ```

**Pass Criteria**:
- [ ] Bash completion generates
- [ ] Zsh completion generates
- [ ] Fish completion generates

**Notes**:
_______________________________________________________________

---

## ✅ Test Category 5: Help & Documentation

### Test 5.1: Help Text

**Objective**: Verify help output is complete

**Steps**:
1. Check main help
   ```bash
   $CLI_BIN --help
   ```

2. Check subcommand help
   ```bash
   $CLI_BIN analyze --help
   $CLI_BIN generate --help
   $CLI_BIN run --help
   ```

**Pass Criteria**:
- [ ] Main help shows all subcommands
- [ ] analyze help shows all options
- [ ] generate help shows all categories
- [ ] run help shows all formats

**Missing Documentation**:
_______________________________________________________________

---

### Test 5.2: Version Information

**Objective**: Verify version output

**Steps**:
1. Check version
   ```bash
   $CLI_BIN --version
   ```

**Pass Criteria**:
- [ ] Shows "cli-testing-specialist 1.0.5"
- [ ] No errors

**Actual Output**:
_______________________________________________________________

---

## ✅ Test Category 6: Performance Benchmarks

### Test 6.1: Analysis Performance

**Objective**: Measure analysis speed

**Steps**:
1. Time curl analysis
   ```bash
   time $CLI_BIN analyze /usr/bin/curl -o perf-curl.json
   ```

2. Time kubectl analysis (if available)
   ```bash
   time $CLI_BIN analyze /usr/local/bin/kubectl -o perf-kubectl.json
   ```

**Pass Criteria**:
- [ ] curl analysis < 5 seconds
- [ ] kubectl analysis < 10 seconds (if 100+ subcommands)

**Results**:
- curl: _____ seconds
- kubectl: _____ seconds

---

### Test 6.2: Generation Performance

**Objective**: Measure test generation speed

**Steps**:
1. Time test generation
   ```bash
   time $CLI_BIN generate curl-analysis.json -o perf-tests/ -c all
   ```

**Pass Criteria**:
- [ ] Generation < 2 seconds

**Result**: _____ seconds

---

### Test 6.3: Memory Usage

**Objective**: Verify memory footprint

**Steps**:
1. Monitor memory during analysis (macOS)
   ```bash
   /usr/bin/time -l $CLI_BIN analyze /usr/bin/curl -o mem-test.json 2>&1 | grep "maximum resident set size"
   ```

2. Monitor memory during analysis (Linux)
   ```bash
   /usr/bin/time -v $CLI_BIN analyze /usr/bin/curl -o mem-test.json 2>&1 | grep "Maximum resident set size"
   ```

**Pass Criteria**:
- [ ] Peak memory < 50MB for curl
- [ ] Peak memory < 100MB for kubectl

**Results**:
- curl: _____ MB
- kubectl: _____ MB

---

## ✅ Test Category 7: Error Handling & Edge Cases

### Test 7.1: Invalid Arguments

**Objective**: Verify argument validation

**Steps**:
1. Test invalid category
   ```bash
   $CLI_BIN generate curl-analysis.json -o tests/ -c invalid-category
   ```

2. Test invalid format
   ```bash
   $CLI_BIN run tests/ -f invalid-format -o reports/
   ```

**Pass Criteria**:
- [ ] Clear error messages
- [ ] Non-zero exit codes
- [ ] Suggestions for valid values

**Error Messages**:
1. _____________________________________________________________
2. _____________________________________________________________

---

### Test 7.2: Large Workload

**Objective**: Test with large, complex CLI tool

**Steps**:
1. Analyze kubectl (100+ subcommands)
   ```bash
   $CLI_BIN analyze /usr/local/bin/kubectl -o kubectl-analysis.json
   ```

2. Generate all tests
   ```bash
   $CLI_BIN generate kubectl-analysis.json -o kubectl-tests/ -c all
   ```

**Pass Criteria**:
- [ ] Completes without crashing
- [ ] Memory usage reasonable
- [ ] All subcommands detected

**Results**:
- Subcommands detected: _____
- Total tests generated: _____
- Time: _____ seconds

---

### Test 7.3: Special Characters

**Objective**: Handle binary paths with special characters

**Steps**:
1. Create symlink with space
   ```bash
   ln -s /usr/bin/curl "$TEST_WORKSPACE/my curl"
   $CLI_BIN analyze "$TEST_WORKSPACE/my curl" -o special-char.json
   ```

**Pass Criteria**:
- [ ] Handles paths with spaces
- [ ] Analysis completes successfully

**Notes**:
_______________________________________________________________

---

## ✅ Test Category 8: Integration with Real Projects

### Test 8.1: Real-World CLI Tool Testing

**Objective**: End-to-end test with actual CLI tool

**Tool**: _____________ (e.g., your own CLI tool)

**Steps**:
1. Analyze
   ```bash
   $CLI_BIN analyze /path/to/your/tool -o tool-analysis.json
   ```

2. Generate
   ```bash
   $CLI_BIN generate tool-analysis.json -o tool-tests/ -c all
   ```

3. Run
   ```bash
   $CLI_BIN run tool-tests/ -f all -o tool-reports/
   ```

4. Review reports
   ```bash
   open tool-reports/tool-report.html  # macOS
   xdg-open tool-reports/tool-report.html  # Linux
   ```

**Pass Criteria**:
- [ ] Full workflow completes
- [ ] Tests are relevant and useful
- [ ] Reports provide actionable insights
- [ ] Success rate > 60%

**Results**:
- Options detected: _____
- Subcommands detected: _____
- Tests generated: _____
- Tests passed: _____
- Success rate: _____%

**Insights Gained**:
_______________________________________________________________
_______________________________________________________________

---

## ✅ Test Category 9: Visual/UI Verification

### Test 9.1: HTML Report Quality

**Objective**: Verify HTML report is usable

**Steps**:
1. Open HTML report in browser
   ```bash
   open reports-all/curl-report.html
   ```

2. Visual checks:
   - [ ] Page loads without errors
   - [ ] Bootstrap CSS applied correctly
   - [ ] Tables are formatted
   - [ ] No broken images/links
   - [ ] Responsive design (resize browser)
   - [ ] No external CDN dependencies (check Network tab)

**Screenshot**: (Attach if needed)

**Issues Found**:
_______________________________________________________________

---

### Test 9.2: Markdown Report Readability

**Objective**: Verify Markdown is well-formatted

**Steps**:
1. View in terminal
   ```bash
   cat reports/curl-report.md
   ```

2. View in Markdown viewer
   - GitHub: Upload as gist
   - VS Code: Open with Markdown preview
   - macOS: qlmanage -p curl-report.md

**Pass Criteria**:
- [ ] Headers render correctly
- [ ] Tables aligned
- [ ] Code blocks formatted
- [ ] Emoji render (✓, ✗, etc.)

**Notes**:
_______________________________________________________________

---

## 📊 Overall Test Summary

### Environment Information

- **OS**: _________________ (macOS 14.x, Ubuntu 22.04, etc.)
- **Shell**: _________________ (bash 5.2, zsh 5.9, etc.)
- **BATS Version**: _________________ (bats 1.10.0, etc.)
- **Rust Version**: _________________ (rustc 1.80.0, etc.)

### Test Results by Category

| Category | Total Tests | Passed | Failed | Notes |
|----------|-------------|--------|--------|-------|
| 1. analyze | _____ | _____ | _____ | _____ |
| 2. generate | _____ | _____ | _____ | _____ |
| 3. run | _____ | _____ | _____ | _____ |
| 4. completion | _____ | _____ | _____ | _____ |
| 5. Help/Docs | _____ | _____ | _____ | _____ |
| 6. Performance | _____ | _____ | _____ | _____ |
| 7. Errors | _____ | _____ | _____ | _____ |
| 8. Integration | _____ | _____ | _____ | _____ |
| 9. Visual/UI | _____ | _____ | _____ | _____ |
| **TOTAL** | **_____** | **_____** | **_____** | |

### Critical Issues Found

1. _____________________________________________________________
2. _____________________________________________________________
3. _____________________________________________________________

### Non-Critical Issues

1. _____________________________________________________________
2. _____________________________________________________________

### Suggestions for Improvement

1. _____________________________________________________________
2. _____________________________________________________________
3. _____________________________________________________________

---

## ✅ Sign-Off

**Tester Name**: _________________________________

**Date**: _________________________________

**Overall Assessment**:
- [ ] **PASS** - Ready for release
- [ ] **PASS with Minor Issues** - Release with known issues documented
- [ ] **FAIL** - Critical issues must be fixed before release

**Signature**: _________________________________

---

## 📝 Appendix: Quick Command Reference

```bash
# Analysis
$CLI_BIN analyze /usr/bin/curl -o curl-analysis.json

# Generation (specific categories)
$CLI_BIN generate curl-analysis.json -o tests/ -c basic,security

# Generation (all categories)
$CLI_BIN generate curl-analysis.json -o tests/ -c all

# Generation (including intensive tests)
$CLI_BIN generate curl-analysis.json -o tests/ -c all --include-intensive

# Run tests (single format)
$CLI_BIN run tests/ -f markdown -o reports/

# Run tests (all formats)
$CLI_BIN run tests/ -f all -o reports/

# Run with custom options
$CLI_BIN run tests/ -f json -o reports/ --timeout 120 --skip "destructive-ops,performance"

# Shell completion
$CLI_BIN completion bash > cli-completion.bash
$CLI_BIN completion zsh > cli-completion.zsh
$CLI_BIN completion fish > cli-completion.fish
```

---

**Document Version**: 1.0
**Compatible with**: cli-testing-specialist v1.0.5
