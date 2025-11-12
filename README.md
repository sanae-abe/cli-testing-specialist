# CLI Testing Specialist

**Languages**: [English](README.md) | [日本語](README.ja.md)

**Version**: 1.0.4
**Last Updated**: 2025-01-12
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

- 🔒 **Security Testing**: OWASP-compliant automated security scanning
- ✅ **Comprehensive Validation**: 9 test categories, 45-55 test cases (configurable)
- 🎯 **Input Validation**: Automatic validation of numeric/path/enum options
- 🛡️ **Destructive Operation Testing**: Confirmation prompt and safety validation
- 🐚 **Multi-Shell Support**: bash/zsh compatibility testing
- 📊 **Detailed Reports**: Markdown/JSON/HTML/JUnit XML formats
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

### From Source (Recommended)

```bash
# Install from GitHub
cargo install --git https://github.com/sanae-abe/cli-testing-specialist

# Verify installation
cli-testing-specialist --version
```

### From Crates.io (Coming Soon)

```bash
cargo install cli-testing-specialist
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

**See [docs/TARGET-TOOLS.md](./docs/TARGET-TOOLS.md) for complete guidelines and best practices.**

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

### GitHub Actions

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

### v1.0.4 (2025-01-12)

**Documentation Improvements** 📚:
- Added comprehensive `docs/TARGET-TOOLS.md` guide for tool compatibility assessment
- Tool classification system: High/Medium/Low compatibility with success rate estimates
- CI/CD integration modes: Standard (Strict), Informational (Lenient), Security-Only (Focused)
- Real-world statistics from 4 projects: backup-suite, cmdrun, cldev, package-publisher
- Best practices for progressive adoption and category selection
- Added `todo.md` for project roadmap and improvement tracking

**Impact**:
- Users can now determine if cli-testing-specialist is suitable for their CLI tool
- Clear guidance on informational mode for custom implementation tools (30-60% success rate)
- Documented that security testing is valuable for ALL CLI tools regardless of compatibility

### v1.0.3 (2025-01-12)

**Critical Security Test Fix** 🔒:
- Fixed security test design to accept **any non-zero exit code** (not just exit code 1)
- Changed `expected_exit: i32` → `Option<i32>` to support flexible exit code validation
- Added `.expect_nonzero_exit()` method for security tests
- Now correctly handles Unix convention: exit code 2 for command-line usage errors (clap/commander/argparse standard)
- Affects: command injection, null byte injection, path traversal tests
- **Breaking Change**: But necessary fix for incorrect test design

**Impact**:
- cldev, cmdrun, package-publisher security tests now pass correctly (they return exit code 2)
- BATS generation: `[ "$status" -ne 0 ]` instead of `[ "$status" -eq 1 ]`

### v1.0.2 (2025-01-12)

**Security Fix** 🔒:
- Fixed critical security test design flaw where tests expected malicious inputs to succeed (exit code 0)
- Security tests now correctly expect tools to **reject** attacks (exit code 1)
- Affects: command injection, null byte injection, path traversal tests

**Features**:
- Directory Traversal tests now **opt-in** via `--include-intensive` flag
- Improved `.gitignore` to exclude analysis results and test outputs

### v1.0.1 (2025-01-11)

- Initial production release
- 9 test categories with 45-55 test cases
- 4 report formats (Markdown/JSON/HTML/JUnit)
- Multi-project CI integration examples

---

**Built with ❤️ using Rust**
