# CLI Testing Specialist - Usage Guide

**Version**: 1.0.9
**Last Updated**: 2025-01-16

This guide provides comprehensive usage examples and best practices for CLI Testing Specialist.

---

## Table of Contents

- [Basic Workflow](#basic-workflow)
- [Command Reference](#command-reference)
- [Common Use Cases](#common-use-cases)
- [Test Categories](#test-categories)
- [Report Formats](#report-formats)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

---

## Basic Workflow

### 1. Analyze → Generate → Run

The standard three-step workflow for testing any CLI tool:

```bash
# Step 1: Analyze the CLI tool
cli-testing-specialist analyze /usr/bin/curl -o curl-analysis.json

# Step 2: Generate test cases
cli-testing-specialist generate curl-analysis.json -o curl-tests -c all

# Step 3: Run tests and generate reports
cli-testing-specialist run curl-tests -f all -o curl-reports
```

### 2. Quick Test (One-Liner)

For rapid testing during development:

```bash
# Analyze, generate, and run in one command (requires shell scripting)
cli-testing-specialist analyze /usr/bin/curl -o /tmp/analysis.json && \
cli-testing-specialist generate /tmp/analysis.json -o /tmp/tests -c basic,security && \
cli-testing-specialist run /tmp/tests -f markdown -o /tmp/reports && \
cat /tmp/reports/*-report.md
```

---

## Command Reference

### `analyze` - Analyze CLI Tool

Extract CLI structure and options from a binary.

#### Basic Usage

```bash
# Analyze a system binary
cli-testing-specialist analyze /usr/bin/curl -o curl-analysis.json

# Analyze a local binary (development)
cli-testing-specialist analyze ./target/release/my-cli -o analysis.json

# Analyze with verbose output
cli-testing-specialist analyze /usr/bin/git -o git-analysis.json --verbose
```

#### Output Format

The analysis file is a JSON document containing:
- Binary metadata (name, version, path)
- Global options (flags, arguments)
- Subcommands (recursive structure)
- Inferred option types (numeric, path, enum, etc.)

**Example output structure**:
```json
{
  "binary_name": "curl",
  "version": "8.4.0",
  "global_options": [
    {
      "short": "-v",
      "long": "--verbose",
      "option_type": "Flag",
      "description": "Make the operation more talkative"
    }
  ],
  "subcommands": [],
  "metadata": {
    "total_options": 237,
    "analysis_duration_ms": 109
  }
}
```

---

### `generate` - Generate Test Cases

Create BATS test files from analysis results.

#### Basic Usage

```bash
# Generate all test categories
cli-testing-specialist generate analysis.json -o tests -c all

# Generate specific categories
cli-testing-specialist generate analysis.json -o tests -c basic,security,help

# Include resource-intensive tests
cli-testing-specialist generate analysis.json -o tests -c all --include-intensive
```

#### Category Options

| Category | Description | Default | Intensive |
|----------|-------------|---------|-----------|
| `basic` | Help, version, exit codes | ✅ | No |
| `help` | Subcommand help validation | ✅ | No |
| `security` | Injection, null bytes, path traversal | ✅ | No |
| `path` | Special characters, Unicode | ✅ | No |
| `input-validation` | Numeric/enum validation | ✅ | No |
| `destructive-ops` | Confirmation prompts | ✅ | No |
| `performance` | Startup time, memory | ✅ | No |
| `multi-shell` | bash/zsh compatibility | ✅ | No |
| `directory-traversal` | Large directories, symlinks | ⚠️ | **Yes** |

**Note**: Use `--include-intensive` to enable `directory-traversal` tests (may consume significant disk space/time).

#### Output Structure

```
tests/
├── basic.bats           # Basic validation tests
├── security.bats        # Security vulnerability tests
├── help.bats            # Help text validation
├── path.bats            # Path handling tests
├── input-validation.bats
├── destructive-ops.bats
├── performance.bats
└── multi-shell.bats
```

---

### `run` - Execute Tests

Run BATS test suites and generate reports.

#### Basic Usage

```bash
# Run all tests with Markdown report
cli-testing-specialist run tests -f markdown -o reports

# Run with all report formats
cli-testing-specialist run tests -f all -o reports

# Run with custom timeout (default: 300s)
cli-testing-specialist run tests -f json -o reports --timeout 600
```

#### Report Format Options

| Format | File Extension | Use Case |
|--------|---------------|----------|
| `markdown` | `.md` | GitHub/GitLab display |
| `json` | `.json` | CI/CD integration, programmatic processing |
| `html` | `.html` | Interactive browser viewing |
| `junit` | `.xml` | CI/CD (GitHub Actions, GitLab CI, Jenkins) |
| `all` | All formats | Comprehensive reporting |

#### Skip Categories

```bash
# Skip performance tests (for fast feedback)
cli-testing-specialist run tests -f markdown -o reports --skip performance

# Skip multiple categories
cli-testing-specialist run tests -f json -o reports --skip performance,multi-shell
```

---

## Common Use Cases

### Use Case 1: Testing a Rust CLI (clap)

```bash
# Build your CLI
cargo build --release

# Analyze
cli-testing-specialist analyze ./target/release/my-cli -o analysis.json

# Generate tests (all categories)
cli-testing-specialist generate analysis.json -o tests -c all

# Run tests
cli-testing-specialist run tests -f all -o reports

# View HTML report
open reports/my-cli-tests-report.html  # macOS
# xdg-open reports/my-cli-tests-report.html  # Linux
```

### Use Case 2: Testing a Node.js CLI (commander.js)

```bash
# Build your CLI
npm run build

# Make binary executable
chmod +x ./bin/my-cli.js

# Analyze
cli-testing-specialist analyze ./bin/my-cli.js -o analysis.json

# Generate and run
cli-testing-specialist generate analysis.json -o tests -c all
cli-testing-specialist run tests -f markdown -o reports
```

**Note**: Node.js CLIs may use different exit codes than Rust CLIs:
- commander.js: Exit code 1 for errors
- clap: Exit code 2 for usage errors

### Use Case 3: Security Testing Only

```bash
# Generate security-focused tests
cli-testing-specialist generate analysis.json -o tests -c security,input-validation,destructive-ops

# Run with detailed logging
cli-testing-specialist run tests -f json -o reports --verbose

# Check for failures
jq '.total_failed' reports/tests-report.json
```

### Use Case 4: CI/CD Integration

```bash
# In your CI script (e.g., .github/workflows/test.yml)

# 1. Build binary
cargo build --release

# 2. Analyze
cli-testing-specialist analyze ./target/release/my-cli -o analysis.json

# 3. Generate tests
cli-testing-specialist generate analysis.json -o tests -c all

# 4. Run tests with JUnit XML for CI
cli-testing-specialist run tests -f junit -o reports

# 5. Upload reports as artifacts
# (GitHub Actions: upload-artifact@v4)
```

### Use Case 5: Development Workflow

```bash
# Quick test during development (basic tests only)
cli-testing-specialist analyze ./target/debug/my-cli -o /tmp/analysis.json && \
cli-testing-specialist generate /tmp/analysis.json -o /tmp/tests -c basic,help && \
cli-testing-specialist run /tmp/tests -f markdown

# Full test before commit
cli-testing-specialist analyze ./target/release/my-cli -o analysis.json && \
cli-testing-specialist generate analysis.json -o tests -c all && \
cli-testing-specialist run tests -f all -o reports
```

---

## Test Categories

### Basic Validation

Tests fundamental CLI functionality.

**What's tested**:
- `--help` displays help text
- `--version` displays version
- Exit codes (0 for success, non-zero for errors)
- No-args behavior (help/error/execute)

**Example test**:
```bash
@test "curl --help should display help and exit 0" {
  run curl --help
  [ "$status" -eq 0 ]
  [[ "$output" =~ "Usage:" ]]
}
```

### Security

Tests for common security vulnerabilities.

**What's tested**:
- Command injection (`;`, `|`, `&&`, etc.)
- Null byte injection (`\0`)
- Path traversal (`../`, `../../etc/passwd`)
- Special characters (`'`, `"`, `$`, `` ` ``)

**Example test**:
```bash
@test "curl should reject command injection in URL" {
  run curl "http://example.com; cat /etc/passwd"
  [ "$status" -ne 0 ]
}
```

### Input Validation

Tests option validation and error handling.

**What's tested**:
- Invalid numeric values (negative, zero, non-numeric)
- Invalid enum values
- Invalid path formats
- Boundary conditions

**Example test**:
```bash
@test "curl --max-time should reject negative values" {
  run curl --max-time -1 http://example.com
  [ "$status" -ne 0 ]
  [[ "$output" =~ "invalid" ]]
}
```

### Destructive Operations

Tests confirmation prompts and safety mechanisms.

**What's tested**:
- `--yes` / `--force` flag presence
- Confirmation prompt behavior
- Non-interactive mode support

**Example test**:
```bash
@test "rm should require --force for destructive operation" {
  run rm /important/file
  [ "$status" -ne 0 ]
  [[ "$output" =~ "confirm" || "$output" =~ "force" ]]
}
```

### Help

Tests help text quality and consistency.

**What's tested**:
- Help text presence for all subcommands
- Usage examples
- Option descriptions
- Consistent formatting

**Example test**:
```bash
@test "git commit --help should show usage" {
  run git commit --help
  [ "$status" -eq 0 ]
  [[ "$output" =~ "usage:" || "$output" =~ "Usage:" ]]
}
```

### Path Handling

Tests file and directory path processing.

**What's tested**:
- Special characters in paths (`spaces`, `!@#$%`)
- Unicode filenames
- Deep directory hierarchies
- Non-existent paths

**Example test**:
```bash
@test "curl should handle paths with spaces" {
  run curl --output "file with spaces.txt" http://example.com
  [ "$status" -eq 0 ]
}
```

### Performance

Tests execution speed and resource usage.

**What's tested**:
- Startup time (< 1 second)
- Memory usage (reasonable limits)
- Large input handling

**Example test**:
```bash
@test "curl should start in under 1 second" {
  start=$(date +%s)
  run curl --version
  end=$(date +%s)
  duration=$((end - start))
  [ "$duration" -lt 1 ]
}
```

### Multi-Shell

Tests compatibility across different shells.

**What's tested**:
- bash compatibility
- zsh compatibility
- Exit code consistency
- Output consistency

**Example test**:
```bash
@test "curl --help should work in zsh" {
  run zsh -c "curl --help"
  [ "$status" -eq 0 ]
}
```

### Directory Traversal (Intensive)

Tests handling of large/complex directory structures.

**What's tested**:
- Large file counts (1000+ files)
- Deep nesting (10+ levels)
- Symlink loops
- Filesystem edge cases

**⚠️ Warning**: These tests are resource-intensive. Use `--include-intensive` flag.

---

## Report Formats

### Markdown Reports

Best for: GitHub/GitLab display, quick review

```bash
cli-testing-specialist run tests -f markdown -o reports
cat reports/tests-report.md
```

**Features**:
- Summary statistics table
- Test suite breakdown
- Detailed failure messages
- Shell compatibility matrix

### JSON Reports

Best for: CI/CD integration, programmatic analysis

```bash
cli-testing-specialist run tests -f json -o reports

# Extract statistics with jq
jq '.total_tests' reports/tests-report.json
jq '.total_passed' reports/tests-report.json
jq '[.suites[].tests[] | select(.status == "failed")] | length' reports/tests-report.json
```

**Structure**:
```json
{
  "binary_name": "curl",
  "total_tests": 42,
  "total_passed": 40,
  "total_failed": 2,
  "total_skipped": 0,
  "duration_secs": 5.2,
  "test_suites": [
    {
      "name": "basic",
      "tests": [...]
    }
  ]
}
```

### HTML Reports

Best for: Interactive viewing, presentations

```bash
cli-testing-specialist run tests -f html -o reports
open reports/tests-report.html
```

**Features**:
- 📊 Visual statistics cards
- 📈 Progress bar with success rate
- 🔍 Collapsible test details
- 🎨 Professional design (Bootstrap 5)
- ⚡ Zero CDN dependencies (self-contained)
- 📱 Fully responsive

### JUnit XML Reports

Best for: CI/CD systems (GitHub Actions, GitLab CI, Jenkins)

```bash
cli-testing-specialist run tests -f junit -o reports
```

**CI Integration Example**:
```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cli-testing-specialist run tests -f junit -o reports

- name: Publish test results
  uses: EnricoMi/publish-unit-test-result-action@v2
  with:
    files: reports/*.xml
```

---

## Advanced Usage

### Custom Configuration

Create a configuration file to customize test generation:

```yaml
# cli-test-config.yml
version: "1.0"

# Skip specific test categories
skip_categories:
  - performance
  - directory-traversal

# Custom test priorities
test_priorities:
  security: critical
  input-validation: high
  basic: medium

# Resource limits
resource_limits:
  timeout_seconds: 600
  max_memory_mb: 1024
```

Use with:
```bash
cli-testing-specialist generate analysis.json -o tests -c all --config cli-test-config.yml
```

### Parallel Test Execution

BATS automatically runs test files in parallel. Adjust parallelism:

```bash
# Run with 4 parallel jobs
cli-testing-specialist run tests -f markdown -o reports --jobs 4

# Run serially (for debugging)
cli-testing-specialist run tests -f markdown -o reports --jobs 1
```

### Filtering Tests

```bash
# Run only security tests
cli-testing-specialist run tests -f markdown -o reports --category security

# Skip performance tests
cli-testing-specialist run tests -f markdown -o reports --skip performance
```

### Debugging Test Failures

```bash
# Enable verbose output
cli-testing-specialist run tests -f markdown -o reports --verbose

# Run a single test file manually
bats tests/security.bats

# Run with BATS verbose mode
bats -p tests/security.bats
```

---

## Troubleshooting

### Issue: "BATS not found"

**Solution**: Install BATS (Bash Automated Testing System)

```bash
# macOS
brew install bats-core

# Ubuntu/Debian
sudo apt-get install bats

# Manual installation
git clone https://github.com/bats-core/bats-core.git
cd bats-core
sudo ./install.sh /usr/local
```

### Issue: "Binary not found"

**Solution**: Ensure the binary path is correct and executable

```bash
# Check binary exists
ls -la /usr/bin/curl

# Make binary executable
chmod +x ./target/release/my-cli

# Use absolute path
cli-testing-specialist analyze $(pwd)/target/release/my-cli -o analysis.json
```

### Issue: "No help output detected"

**Solution**: Verify your CLI supports `--help` or `-h`

```bash
# Test manually
/usr/bin/curl --help
/usr/bin/curl -h
/usr/bin/curl help

# If none work, your CLI may not be compatible
```

### Issue: "Tests failing in CI but passing locally"

**Common causes**:
1. **Different binary**: CI may build a different binary version
   ```bash
   # Verify binary in CI
   cli-testing-specialist analyze ./my-cli --verbose
   ```

2. **Resource limits**: CI environments may have strict limits
   ```bash
   # Skip intensive tests in CI
   cli-testing-specialist generate analysis.json -o tests -c all  # Don't use --include-intensive
   ```

3. **Shell differences**: CI may use different shell
   ```bash
   # Test in specific shell
   bash -c "cli-testing-specialist run tests -f markdown"
   ```

### Issue: "Analysis timeout"

**Solution**: Increase timeout for slow binaries

```bash
# Default timeout: 30 seconds
cli-testing-specialist analyze ./slow-cli -o analysis.json --timeout 120
```

---

## Best Practices

### 1. Version Your Analysis Files

Keep analysis files in version control to track CLI evolution:

```bash
# Generate versioned analysis
cli-testing-specialist analyze ./my-cli -o analysis-v1.0.0.json

# Commit to git
git add analysis-v1.0.0.json
git commit -m "Add CLI analysis for v1.0.0"
```

### 2. Use Specific Categories in Development

Run fast tests during development, comprehensive tests before release:

```bash
# Development (fast feedback)
cli-testing-specialist generate analysis.json -o tests -c basic,help

# Pre-release (comprehensive)
cli-testing-specialist generate analysis.json -o tests -c all
```

### 3. Integrate with CI/CD

Add CLI testing to your CI pipeline:

```yaml
# .github/workflows/test.yml
name: CLI Testing

on: [push, pull_request]

jobs:
  cli-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build CLI
        run: cargo build --release

      - name: Install BATS
        run: sudo apt-get install -y bats

      - name: Install cli-testing-specialist
        run: cargo install cli-testing-specialist

      - name: Analyze CLI
        run: cli-testing-specialist analyze ./target/release/my-cli -o analysis.json

      - name: Generate tests
        run: cli-testing-specialist generate analysis.json -o tests -c all

      - name: Run tests
        run: cli-testing-specialist run tests -f junit -o reports

      - name: Upload reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-reports
          path: reports/
```

### 4. Review Generated Tests

Always review generated tests before integrating:

```bash
# Generate tests
cli-testing-specialist generate analysis.json -o tests -c all

# Review test files
cat tests/security.bats
cat tests/basic.bats

# Run manually if needed
bats tests/security.bats
```

### 5. Keep Tests Updated

Regenerate tests when your CLI changes:

```bash
# After adding new options/subcommands
cli-testing-specialist analyze ./my-cli -o analysis.json
cli-testing-specialist generate analysis.json -o tests -c all

# Compare with previous tests
diff -r tests/ tests-old/
```

### 6. Use HTML Reports for Stakeholders

Generate HTML reports for non-technical stakeholders:

```bash
cli-testing-specialist run tests -f html -o reports
open reports/tests-report.html
```

### 7. Monitor Test Trends

Track test results over time in CI:

```bash
# Extract metrics
jq '.total_passed, .total_failed, .duration_secs' reports/tests-report.json

# Store in metrics system (e.g., Prometheus, CloudWatch)
```

---

## Next Steps

- **[Configuration Guide](./CONFIGURATION.md)**: Advanced configuration options
- **[Report Formats](./REPORT-FORMATS.md)**: Detailed report format specifications
- **[Target Tools](./TARGET-TOOLS.md)**: CLI tool compatibility guidelines
- **[Contributing Guide](../CONTRIBUTING.md)**: How to contribute to the project

---

**Questions or Issues?**

- 📖 [Documentation](../README.md)
- 🐛 [Report an Issue](https://github.com/sanae-abe/cli-testing-specialist/issues)
- 💬 [Discussions](https://github.com/sanae-abe/cli-testing-specialist/discussions)
