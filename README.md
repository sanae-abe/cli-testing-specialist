# CLI Testing Specialist

**Languages**: [English](README.md) | [日本語](README.ja.md)

[![Crates.io](https://img.shields.io/crates/v/cli-testing-specialist)](https://crates.io/crates/cli-testing-specialist)
[![Downloads](https://img.shields.io/crates/d/cli-testing-specialist)](https://crates.io/crates/cli-testing-specialist)
[![License: MIT](https://img.shields.io/crates/l/cli-testing-specialist)](LICENSE)
[![Docs.rs](https://docs.rs/cli-testing-specialist/badge.svg)](https://docs.rs/cli-testing-specialist)

**Version**: 1.0.10
**Last Updated**: 2025-11-16
**Status**: Production Ready
**License**: MIT

A comprehensive testing framework that automatically validates the quality and security of CLI tools. Built with Rust for maximum performance and reliability.

---

## 📑 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Target Tools](#target-tools)
- [Features](#features)
- [Report Formats](#report-formats)
- [CI/CD Integration](#cicd-integration)
- [Security Features](#security-features)
- [Configuration](#configuration)
- [File Structure](#file-structure)
- [License](#license)
- [Contributing](#contributing)
- [Support](#support)

---

## Overview

CLI Testing Specialist is a production-ready testing framework that automatically generates and executes comprehensive test suites for CLI tools.

### Key Features

- 🎯 **100% Inference Accuracy**: Execution-based no-args behavior detection (v1.0.9)
- 🔒 **Security Testing**: OWASP-compliant automated security scanning
- ✅ **Comprehensive Validation**: 9 test categories, 45-55 test cases (configurable)
- 🎯 **Input Validation**: Automatic validation of numeric/path/enum options
- 🛡️ **Destructive Operation Testing**: Confirmation prompt and safety validation
- 🐚 **Multi-Shell Support**: bash/zsh compatibility testing
- 📊 **Detailed Reports**: Markdown/JSON/HTML/JUnit XML formats (Interactive HTML with filtering)
- 🔄 **CI/CD Ready**: GitHub Actions & GitLab CI integration examples
- ⚡ **High Performance**: Written in Rust for blazing-fast execution
- 📦 **Single Binary**: Zero runtime dependencies

---

## Quick Start

```bash
# 1. Install cli-testing-specialist
cargo install --git https://github.com/sanae-abe/cli-testing-specialist

# 2. Analyze CLI tool
cli-testing-specialist analyze /usr/bin/curl -o curl-analysis.json

# 3. Generate tests (all categories)
cli-testing-specialist generate curl-analysis.json -o curl-tests -c all

# 4. Run tests and generate reports
cli-testing-specialist run curl-tests -f all -o curl-reports

# 5. View HTML report
open curl-reports/curl-tests-report.html  # macOS
# xdg-open curl-reports/curl-tests-report.html  # Linux
```

---

## Installation

### From Crates.io (Recommended)

```bash
# Install from crates.io
cargo install cli-testing-specialist

# Verify installation
cli-testing-specialist --version
```

### From Source

```bash
# Install from GitHub
cargo install --git https://github.com/sanae-abe/cli-testing-specialist

# Verify installation
cli-testing-specialist --version
```

### For Development

```bash
# Clone repository
git clone https://github.com/sanae-abe/cli-testing-specialist
cd cli-testing-specialist

# Install Git hooks (auto-format on commit)
./scripts/install-hooks.sh

# Build and test
cargo build
cargo test
```

### Dependencies

#### Required for Test Execution
- **BATS (Bash Automated Testing System)**: Test execution framework
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

#### CLI Testing Specialist Binary
- **Zero runtime dependencies**: Single self-contained binary

---

## Target Tools

cli-testing-specialist is optimized for **standard CLI tools**. See [docs/TARGET-TOOLS.md](./docs/TARGET-TOOLS.md) for detailed guidance.

### ✅ High Compatibility (70-90% success rate)
- Standard CLI tools (curl, git, ls, cat)
- Tools with standard options (--help, --version)
- Non-interactive tools
- **Example**: package-publisher (Node.js CLI)

### ⚠️ Medium Compatibility (30-60% success rate)
- Configuration-driven tools (cmdrun with commands.toml)
- Custom UI implementations (cldev with dialoguer, i18n)
- **Recommendation**: Use "informational mode" in CI

### ❌ Low Compatibility (not recommended)
- Interactive shells (mysql, psql, redis-cli)
- Container management (docker, podman)
- Domain-specific tools with custom protocols

### 🌐 Language Support Status

**✅ Tested & Supported**:
- C/C++ (getopt, custom parsers) - curl, git
- Rust (clap) - backup-suite, cmdrun, cldev
- Node.js (commander) - package-publisher
- **Python (argparse)** - test_argparse.py (16/16 tests, 100%)

**⚠️ Untested (Estimated 70-80% compatible)**:
- **Go** (cobra, urfave/cli) - gh, kubectl, docker
- **Python** (click, typer) - Likely compatible, untested
- **Ruby** (thor, gli)

**📋 Planned Testing**: v1.1.0+

**See [docs/TARGET-TOOLS.md](./docs/TARGET-TOOLS.md) for complete guidelines, language-specific details, and best practices.**

### 🎯 Verified Success Cases

Real-world CLI tools tested with 100% success rate:

| Tool | Language | Framework | Tests | Success Rate | Notes |
|------|----------|-----------|-------|--------------|-------|
| **package-publisher** | Node.js | commander.js | 17/17 | 100% | NPM package publisher with multi-command support |
| **backup-suite** | Rust | clap | 15/15 | 100% | Backup automation tool with encryption |
| **cmdrun** | Rust | clap | 14/14 | 100% | Command runner with TOML configuration |
| **cldev** | Rust | clap | 15/15 | 100% | Interactive development CLI with i18n |

**Framework Compatibility Verified**:
- ✅ **commander.js** (Node.js): Exit code 1 for errors (differs from clap's exit 2)
- ✅ **clap** (Rust): Standard Unix exit codes (0=success, 1=error, 2=usage)
- ✅ **Custom parsers**: getopt-based tools (curl, git)

---

## Features

| Category | Description | Default |
|---------|------|---------|
| **Basic Validation** | Help, version, exit codes | ✅ Enabled |
| **Help** | Comprehensive subcommand help validation | ✅ Enabled |
| **Security** | Command injection, null bytes, path traversal | ✅ Enabled |
| **Path Handling** | Special characters, deep hierarchies, Unicode | ✅ Enabled |
| **Multi-Shell** | bash/zsh compatibility | ✅ Enabled |
| **Input Validation** | Numeric/path/enum option validation | ✅ Enabled |
| **Destructive Operations** | Confirmation prompts, --yes/--force flags | ✅ Enabled |
| **Performance** | Startup time, memory usage | ✅ Enabled |
| **Directory Traversal** | Large file count, deep nesting, symlink loops | ⚠️ Opt-in* |

\* Directory Traversal tests are **opt-in** via `--include-intensive` flag to prevent CI environment issues (disk space, resource limits).

### Test Generation Options

```bash
# Default: All categories except Directory Traversal
cli-testing-specialist generate analysis.json -c all

# Include resource-intensive tests
cli-testing-specialist generate analysis.json -c all --include-intensive

# Specific categories only
cli-testing-specialist generate analysis.json -c basic,security,path
```

---

## Performance Benchmarks

Built with Rust for maximum performance - **10x+ faster** than shell-based alternatives.

### Benchmark Results

Measured with Criterion.rs on production hardware:

| CLI Tool | Complexity | Analysis Time | vs Bash Prototype |
|----------|-----------|---------------|-------------------|
| curl | Small (~50-100 options) | **109 ms** | 11x faster |
| docker | Medium (~100+ options) | **224 ms** | 11x faster |
| kubectl | Large (100+ subcommands) | **230 ms** | 17x faster |
| npm | Medium (many subcommands) | **329 ms** | 7x faster |

### Key Performance Metrics

- **Small CLIs**: Sub-second analysis (~110ms for curl)
- **Medium CLIs**: ~200-350ms range (docker, kubectl, npm)
- **Large CLIs**: < 500ms even with 100+ subcommands
- **Memory Usage**: < 50MB for typical workloads
- **Speedup vs Bash**: 11-17x faster (exceeds 10x target)

### Optimization Techniques

- **LTO**: Link-Time Optimization (`lto = "thin"`)
- **Parallel Processing**: rayon for concurrent test generation
- **Efficient I/O**: BufReader/BufWriter with 64KB buffers
- **Binary Stripping**: Minimal binary size

**See [docs/PERFORMANCE.md](./docs/PERFORMANCE.md) for detailed benchmarks and methodology.**

---

## Report Formats

### 1. Markdown Format (`.md`)
Human-readable format for GitHub/GitLab display

```bash
cli-testing-specialist run ./tests -f markdown -o ./reports
```

### 2. JSON Format (`.json`)
Optimal for CI/CD integration and programmatic processing

```bash
cli-testing-specialist run ./tests -f json -o ./reports

# Parse with jq
jq '[.suites[].tests[] | select(.status == "passed")] | length' reports/tests-report.json
```

### 3. HTML Format (`.html`)
Interactive browser display with search and filtering

```bash
cli-testing-specialist run ./tests -f html -o ./reports
open reports/tests-report.html
```

**HTML Features**:
- 📊 Visual statistics cards (Passed/Failed/Skipped/Duration)
- 📈 Progress bar showing success rate
- 📋 Test suite breakdown with detailed results
- 🎨 Clean, professional design (Bootstrap 5)
- 🚀 Zero CDN dependencies (embedded CSS)
- 📱 Fully responsive layout
- ⚡ Fast loading with self-contained HTML

### 4. JUnit XML Format (`.xml`)
CI/CD integration (GitHub Actions, GitLab CI, Jenkins)

```bash
cli-testing-specialist run ./tests -f junit -o ./reports
```

### 5. All Formats at Once

```bash
cli-testing-specialist run ./tests -f all -o ./reports
```

For details, see [`docs/REPORT-FORMATS.md`](docs/REPORT-FORMATS.md).

---

## CI/CD Integration

### GitHub Actions (Recommended - 3 lines!)

**New in v1.1.0**: Use the official GitHub Action for the easiest integration:

```yaml
name: CLI Testing

on: [push, pull_request]

jobs:
  cli-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build your CLI
        run: cargo build --release

      - name: Test CLI
        uses: ./.github/actions/cli-testing-specialist
        with:
          binary: ./target/release/your-cli
          categories: all
          format: all

      - name: Upload test reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: cli-test-reports
          path: cli-test-reports/
```

**Advanced Configuration**:

```yaml
      - name: Test CLI with custom settings
        uses: ./.github/actions/cli-testing-specialist
        with:
          binary: ./target/release/your-cli
          categories: 'basic,security,path'  # Specific categories
          format: 'junit'                    # CI-friendly format
          output: 'test-results'             # Custom output directory
          include-intensive: 'false'         # Skip resource-intensive tests
          version: '1.1.0'                   # Specific version
```

### GitHub Actions (Manual Setup)

If you prefer manual setup:

```yaml
name: CLI Testing

on: [push, pull_request]

jobs:
  cli-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cli-testing-specialist
        run: |
          cargo install --git https://github.com/sanae-abe/cli-testing-specialist

      - name: Build your CLI
        run: cargo build --release

      - name: Analyze CLI
        run: |
          cli-testing-specialist analyze \
            ./target/release/your-cli \
            -o analysis.json

      - name: Generate tests
        run: |
          cli-testing-specialist generate \
            analysis.json \
            -o tests \
            -c all

      - name: Install BATS
        run: sudo apt-get install -y bats

      - name: Run tests
        run: |
          cli-testing-specialist run \
            tests \
            -f all \
            -o reports

      - name: Upload reports
        uses: actions/upload-artifact@v4
        with:
          name: test-reports
          path: reports/
```

### GitLab CI

```yaml
cli-test:
  image: rust:latest
  script:
    - cargo install --git https://github.com/sanae-abe/cli-testing-specialist
    - cargo build --release
    - cli-testing-specialist analyze ./target/release/your-cli -o analysis.json
    - cli-testing-specialist generate analysis.json -o tests -c all
    - apt-get update && apt-get install -y bats
    - cli-testing-specialist run tests -f all -o reports
  artifacts:
    paths:
      - reports/
```

---

## Security Features

### Security Test Philosophy

**IMPORTANT**: Security tests expect tools to **reject** malicious inputs with non-zero exit codes.

```rust
// Command injection test
cli-test --name 'test; rm -rf /'
// Expected: exit code 1 (rejection) ✅
// NOT exit code 0 (success) ❌
```

### Security Test Categories

1. **Injection Attacks**: Command injection, null byte injection
   - Expected behavior: Tool **rejects** with exit code 1
   - Tags: `injection`, `critical`

2. **Path Traversal**: Directory traversal attempts
   - Expected behavior: Tool **rejects** with exit code 1
   - Tags: `path-traversal`, `critical`

3. **Buffer Overflow**: Extremely long inputs
   - Expected behavior: Graceful handling (informational)
   - Tags: `buffer-overflow`, `informational`

### Input Validation

- CLI binary path verification
- Path canonicalization (prevent path traversal)
- Timeout enforcement (prevent hang)
- Safe command execution

---

## Configuration

### Command-Line Options

```bash
# Analyze with custom output
cli-testing-specialist analyze /usr/bin/curl -o custom-analysis.json

# Generate specific categories
cli-testing-specialist generate analysis.json -c basic,security,path

# Include intensive tests
cli-testing-specialist generate analysis.json -c all --include-intensive

# Run with timeout
cli-testing-specialist run tests --timeout 120 -f all -o reports

# Skip specific categories
cli-testing-specialist run tests --skip destructive-ops,directory-traversal
```

### Environment Variables

```bash
# Skip directory traversal tests
export SKIP_DIRECTORY_TRAVERSAL_TESTS=1
cli-testing-specialist run tests -f all -o reports
```

### Tool-Specific Configuration

Create a `.cli-test-config.yml` file in your project root to customize test generation for your CLI tool.

**Auto-Detection**: The tool automatically looks for `.cli-test-config.yml` in the current directory.

**Basic Example**:
```yaml
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"

test_adjustments:
  security:
    skip_options:
      - name: "lang"
        reason: "Language selection is an enum, not a security risk"
    custom_tests:
      - name: "custom-security-test"
        command: "backup-suite --config /etc/passwd"
        expected_exit_code: 1
        description: "Reject system config file access"

  directory_traversal:
    test_directories:
      - path: "/tmp/test-100-files"
        file_count: 100
        create: true
        cleanup: true
      - path: "/tmp/test-deep-5"
        depth: 5
        create: true
        cleanup: true

  destructive_ops:
    env_vars:
      BACKUP_SUITE_YES: "true"
      CI: "true"
    cancel_exit_code: 2

global:
  timeout: 60
```

**Configuration Reference**:

- **security.skip_options**: Skip security tests for safe enum options
- **security.custom_tests**: Add tool-specific security tests
- **directory_traversal.test_directories**: Declarative test directory configuration
- **destructive_ops.env_vars**: Environment variables for auto-confirmation
- **destructive_ops.cancel_exit_code**: Expected exit code when operation is cancelled
- **global.timeout**: Default timeout for all tests (seconds)

**Security Considerations** (4-Layer Defense):

1. **Layer 1**: Explicit user consent via `.cli-test-config.yml` creation
2. **Layer 2**: Command validation (forbidden patterns: pipes, command substitution, sudo, curl, etc.)
3. **Layer 3**: Resource limits (max 200 characters per command)
4. **Layer 4**: Declarative alternatives (prefer `test_directories` over `setup_commands`)

**Examples**:
- Reference implementation: [`examples/backup-suite.cli-test-config.yml`](./examples/backup-suite.cli-test-config.yml)
- Implementation guide: [`examples/backup-suite-implementation.md`](./examples/backup-suite-implementation.md)

**See [docs/TOOL_SPECIFIC_CONFIG.md](./docs/TOOL_SPECIFIC_CONFIG.md) for complete documentation.**

---

## File Structure

```
cli-testing-specialist/
├── Cargo.toml              # Rust project configuration
├── Cargo.lock
├── README.md               # This file
├── LICENSE                 # MIT License
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library exports
│   ├── cli/                # CLI interface (clap)
│   ├── analyzer/           # CLI analysis engine
│   ├── generator/          # Test case generation
│   ├── runner/             # BATS test execution
│   ├── reporter/           # Report generation (MD/JSON/HTML/JUnit)
│   ├── types/              # Type definitions
│   ├── error.rs            # Error types
│   └── utils/              # Utilities
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── docs/
    ├── RUST_V1_DESIGN.md   # Design document
    ├── TARGET-TOOLS.md     # Target tool guidelines
    └── REPORT-FORMATS.md   # Report format guide
```

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

For major changes, please open an issue first to discuss the proposed changes.

---

## Support

- **Documentation**: [`docs/`](docs/) directory
  - [Design Document](docs/RUST_V1_DESIGN.md) - Architecture and implementation
  - [Target Tools Guide](docs/TARGET-TOOLS.md) - Compatibility guidelines
  - [Report Formats](docs/REPORT-FORMATS.md) - Report format reference
- **Issues**: [GitHub Issues](https://github.com/sanae-abe/cli-testing-specialist/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sanae-abe/cli-testing-specialist/discussions)

---

## Changelog

### v1.0.10 (2025-11-16) - CI/CD Infrastructure Fixes 🔧

**CI/CD Improvements**:
- **setrlimit Test Isolation**: Ignored 5 setrlimit-related tests in Linux CI environments
  - Tests were setting process-wide memory limits (100MB) affecting parallel tests
  - Prevented "failed to allocate an alternative stack" errors in Code Coverage
  - Tests still run in local development and macOS/Windows environments
- **Multi-Shell Test Support**: Added zsh installation to Ubuntu CI environments
  - Fixed 2/3 test pass rate to 3/3 by installing missing zsh package
  - All multi-shell tests (bash/zsh/sh) now pass successfully

**Windows Platform**:
- Added `Win32_System_Threading` feature for Job Object support

**Documentation**:
- Fixed rustdoc examples and HTML tag escaping
- Updated author email to real address

**All CI/CD pipelines now passing successfully** ✅

### v1.0.9 (2025-11-12) - Execution-based Inference 🎯

**Revolutionary No-Args Behavior Detection**:
- **100% Inference Accuracy**: Directly executes binaries to measure actual exit codes
- **Solves cldev-type CLI Problem**: Identical Usage patterns (`[OPTIONS] <COMMAND>`), different behaviors
- **Safety Measures**: 1-second timeout, output discarding, non-TTY mode, interactive tools detection
- **Test Success Rate**: 93.3% → **100%** (15/15 tests passed across cldev/cmdrun/backup-suite)

**HTML Report Improvements**:
- Fixed filter bug: Skipped filter now correctly hides error detail rows
- Interactive filtering works perfectly for All/Passed/Failed/Skipped states

**Technical**:
- New method: `BehaviorInferrer::execute_and_measure()`
- Dependency: `wait-timeout = "0.2"` for process timeout handling
- 109 unit tests passing (100% pass rate)

### v1.0.8 (2025-11-12)

**No-Args Test Assertion Relaxation**:
- Removed strict output assertions (exit codes only)
- Test Success Rate: 86.7% → 93.3%
- Reason: CLIs show different error formats (short message vs full help)

### v1.0.7 (2025-11-12)

**Clippy Warning Fix**:
- Renamed `TestCategory::default()` to `standard_categories()`
- Added Git Hooks configuration in `.claude/CLAUDE.md`

### v1.0.6 (2025-11-12)

**Required Arguments Detection**:
- Automatic extraction from Usage lines (`<ID>`, `<FILE>`)
- Test template improvements (dummy arguments, dynamic option selection)
- Test Success Rate: 85.0% → 92.9% for cmdrun

### v1.0.5 (2025-11-12)

**Dependency Updates**:
- All 7 Dependabot PRs merged (GitHub Actions, indicatif 0.18, thiserror 2.0, colored 3.0, criterion 0.7)
- MSRV bumped to Rust 1.80
- 0 vulnerabilities with `cargo audit`

### v1.0.4 (2025-01-12)

**Documentation Improvements** 📚:
- Added comprehensive `docs/TARGET-TOOLS.md` guide for tool compatibility assessment
- Tool classification system: High/Medium/Low compatibility with success rate estimates

### v1.0.3 (2025-01-12)

**Critical Security Test Fix** 🔒:
- Fixed security test design to accept **any non-zero exit code** (not just exit code 1)
- Now correctly handles Unix convention: exit code 2 for usage errors

### v1.0.2 (2025-01-12)

**Security Fix** 🔒:
- Security tests now correctly expect tools to **reject** attacks
- Directory Traversal tests now **opt-in** via `--include-intensive` flag

### v1.0.1 (2025-01-11)

- Initial production release
- 9 test categories with 45-55 test cases
- 4 report formats (Markdown/JSON/HTML/JUnit)

---

**Built with ❤️ using Rust**
