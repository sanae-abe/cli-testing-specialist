# CLI Testing Specialist Agent

**Languages**: [English](README.md) | [日本語](README.ja.md)

**Last Updated**: 2025-11-10
**Release Target**: v1.0.0 (2026-02-07)
**Claude Code Exclusive**: Secure and comprehensive CLI tool testing framework

---

## 📑 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Features](#features)
- [Report Formats](#report-formats)
- [CI/CD Integration](#cicd-integration)
- [Security Features](#security-features)
- [Configuration](#configuration)
- [File Structure](#file-structure)
- [Sample Reports](#sample-reports)
- [License](#license)
- [Contributing](#contributing)
- [Support](#support)

---

## Overview

CLI Testing Specialist Agent is a comprehensive testing framework that automatically validates the quality and security of CLI tools.

### Key Features

- 🔒 **Security Testing**: OWASP-compliant automated scanning
- ✅ **Comprehensive Validation**: 11 categories, 140-160 test cases
- 🎯 **Input Validation Testing** (Phase 2.5): Automatic validation of numeric/path/enum options
- 🛡️ **Destructive Operation Testing** (Phase 2.5): Confirmation prompt and safety validation
- 🐚 **Multi-Shell Support**: bash/zsh (future support planned: fish)
- 📊 **Detailed Reports**: Markdown/JSON/HTML/JUnit XML
- 🔄 **CI/CD Integration**: GitHub Actions & GitLab CI support
- 🐳 **Docker Integration**: Test execution in isolated environments (optional)
- ⚡ **Performance Boost** (Phase 2.5): 5-10x faster test generation

---

## Quick Start

```bash
# 1. Analyze CLI tool
bash core/cli-analyzer.sh /usr/local/bin/your-cli

# 2. Generate tests (all categories)
bash core/test-generator.sh cli-analysis.json test-output all

# 3. Run tests
bats test-output/*.bats

# 4. Generate HTML report
bash core/run-tests.sh test-output html ./reports

# 5. Open in browser
open reports/test-report.html  # macOS
# xdg-open reports/test-report.html  # Linux
```

---

## Installation

```bash
# Automatic installation via Claude Code (recommended)
# Agent will automatically execute setup

# Or manual installation
git clone <repository-url>
cd cli-testing-specialist
./bin/cli-test --version
```

### Dependencies

CLI Testing Specialist Agent depends on the following tools:

#### Required (Core Features)
- **Bash 4.0+**: Test engine execution environment
- **jq**: JSON processing (CLI metadata analysis, report generation)
- **BATS**: Test execution framework
  ```bash
  # macOS
  brew install bats-core

  # Ubuntu/Debian
  apt-get install bats

  # Manual installation
  git clone https://github.com/bats-core/bats-core.git
  cd bats-core
  sudo ./install.sh /usr/local
  ```

#### Required for Phase 2.5+ (Input Validation Features)
- **yq v4.x**: YAML processing (option type inference, constraint definitions)
  ```bash
  # macOS
  brew install yq

  # Ubuntu/Debian (snap)
  snap install yq

  # Linux (binary)
  wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
  chmod +x yq_linux_amd64
  sudo mv yq_linux_amd64 /usr/local/bin/yq

  # Verify installation
  yq --version  # Should show: yq (https://github.com/mikefarah/yq/) version 4.x
  ```

#### Optional (Extended Features)
- **SQLite3**: Coverage tracking (Phase 2 feature)
  ```bash
  # macOS
  brew install sqlite3

  # Ubuntu/Debian
  apt-get install sqlite3
  ```
- **Docker**: Test execution in isolated environments
- **envsubst** (gettext): Template variable substitution (Bash fallback available)

#### Dependency Check

```bash
# Verify required tools
command -v bash && echo "✓ Bash"
command -v jq && echo "✓ jq"
command -v bats && echo "✓ BATS"
command -v yq && echo "✓ yq (Phase 2.5+)"
command -v sqlite3 && echo "✓ SQLite3 (Phase 2)"

# yq version check (must be v4.x for Phase 2.5)
yq --version 2>&1 | grep -q "version 4" && echo "✓ yq v4.x" || echo "⚠ yq v4.x required"
```

---

## Features

| Category | Description | Test Count |
|---------|------|---------|
| Basic Validation | Help, version, exit codes | 10 |
| Subcommand Help | Comprehensive validation of all subcommands | Dynamic |
| Security | Injection, secret leaks, TOCTOU | 25 |
| Path Handling | Special characters, deep hierarchies, Unicode | 20 |
| Multi-Shell | bash/zsh compatibility | 12 |
| Input Validation (Basic) | Invalid values, edge cases | 12 |
| **Input Validation (Extended)** 🆕 | **Numeric/path/enum option validation** | **25** |
| **Destructive Operations** 🆕 | **Confirmation prompts, --yes/--force flags** | **16** |
| **Directory Traversal Limits** 🆕 | **Large file count, deep nesting, symlink loops** | **12** |
| Output Validation | Format, color output | 8 |
| Environment Dependencies | OS, environment variables | 10 |
| Performance | Startup time, memory usage | 6 |
| Documentation Consistency | README vs help | 5 |
| **Reports** | **4 formats (Markdown/JSON/HTML/JUnit)** | - |

**Total**: Approximately 152-172 test cases (41 in Phase 2.5, 12 in Phase 2.6)

---

## Report Formats

### 1. Markdown Format (`.md`)
Human-readable format that can be directly displayed on GitHub/GitLab

```bash
bash core/run-tests.sh ./generated-tests markdown ./reports
```

### 2. JSON Format (`.json`)
Optimal for CI/CD integration and programmatic processing

```bash
bash core/run-tests.sh ./generated-tests json ./reports

# Get success rate with jq
jq -r '.summary.success_rate' reports/test-report.json
```

### 3. HTML Format (`.html`) - **New Feature**
Interactive browser display, GitHub Pages publication support

```bash
bash core/run-tests.sh ./generated-tests html ./reports
open reports/test-report.html  # Open in browser
```

**HTML Features**:
- Modern design with Bootstrap 5
- Real-time search and filtering
- Animated success rate graphs
- Shell compatibility matrix display
- Responsive design support

### 4. All Formats at Once (`all`)

```bash
bash core/run-tests.sh ./generated-tests all ./reports
```

For details, see [`docs/REPORT-FORMATS.md`](docs/REPORT-FORMATS.md).

---

## CI/CD Integration

### GitHub Actions

Automatic testing and report publishing with `.github/workflows/cli-test.yml`

**Features**:
- Ubuntu/macOS × Bash/Zsh matrix testing
- Automatic HTML report deployment to GitHub Pages
- Test results saved as Artifacts
- Automatic ShellCheck linting

**Usage**:
1. Enable GitHub Pages in repository settings
2. Auto-execution on push to main branch
3. View reports at `https://[username].github.io/[repo]/`

### GitLab CI/CD

Multi-shell environment testing and GitLab Pages publishing with `.gitlab-ci.yml`

**Features**:
- Bash/Zsh/Dash compatibility testing
- Report aggregation stage
- Automatic deployment to GitLab Pages
- Regression testing via scheduled execution

**Pipeline Stages**:
1. `validate` - Structure validation & ShellCheck
2. `test` - Test execution in multiple shell environments
3. `report` - Report aggregation
4. `deploy` - GitLab Pages deployment

---

## Security Features

### Input Validation
- CLI binary path verification
- Path traversal attack defense
- Command injection protection

### Secure Execution Environment
- Temporary file umask 077
- TOCTOU attack protection (using mktemp)
- Docker non-root user execution

### Security Scanning
- OWASP Top 10 compliance
- Secret leak detection
- Dependency vulnerability scanning

---

## Configuration

### Default Configuration File

```yaml
# ~/.config/cli-test/config.yaml
cli-testing-specialist:
  version: "1.1.0"

  test_categories:
    enabled:
      - basic-validation
      - help-checker
      - security-scanner
      - path-handler
      - shell-compatibility

  docker:
    enabled: true
    fallback_to_native: true

  logging:
    level: "INFO"
    file: "/tmp/cli-test.log"
```

For details, refer to `config/schema.yaml`.

---

## Sample Reports

You can generate and review sample tests and reports:

```bash
# Run sample tests and generate all format reports
bash core/run-tests.sh sample-tests all sample-reports

# Generated files
sample-reports/
├── test-report.html  # HTML report (22KB)
├── test-report.json  # JSON report (255B)
└── test-report.md    # Markdown report (968B)

# Open HTML report in browser
open sample-reports/test-report.html
```

**Sample Report**: [`sample-reports/test-report.html`](sample-reports/test-report.html)

---

## License

MIT License

---

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

---

## Support

- **Documentation**: `docs/` directory
  - [`REPORT-FORMATS.md`](docs/REPORT-FORMATS.md) - Detailed report format guide
  - [`INPUT-VALIDATION-GUIDE.md`](docs/INPUT-VALIDATION-GUIDE.md) - Input validation guide
  - [`INPUT-VALIDATION-PLAN-v2.md`](docs/INPUT-VALIDATION-PLAN-v2.md) - Phase 2.5 implementation plan
  - [`PHASE2-PLAN.md`](docs/PHASE2-PLAN.md) - Phase 2 implementation plan
  - [`PHASE25-FINAL-REPORT.md`](docs/PHASE25-FINAL-REPORT.md) - Phase 2.5 final report
- **Issues**: GitHub Issues

---

## File Structure

```
cli-testing-specialist/
├── core/
│   ├── cli-analyzer.sh            # CLI analysis engine
│   ├── test-generator.sh          # BATS generation engine (Phase 2.5 extended)
│   ├── option-analyzer.sh         # Option type inference engine (Phase 2.5 new)
│   ├── coverage-tracker.sh        # Coverage tracking (Phase 2)
│   ├── run-tests.sh               # Test execution & report generation
│   ├── report-generator-html.sh   # HTML report generation
│   ├── shell-detector.sh          # Shell detection engine
│   └── validator.sh               # Input validation engine
├── config/                        # Phase 2.5 new
│   ├── option-patterns.yaml       # Option type pattern definitions
│   ├── numeric-constraints.yaml   # Numeric constraint definitions
│   └── enum-definitions.yaml      # Enum definitions
├── templates/                     # Phase 2.5 new
│   ├── bats-test.template         # BATS template
│   ├── input-validation.fragment  # Input validation test fragment
│   └── destructive-ops.fragment   # Destructive operation test fragment
├── docs/
│   ├── REPORT-FORMATS.md          # Report format guide
│   ├── INPUT-VALIDATION-GUIDE.md  # Input validation guide (Phase 2.5 new)
│   ├── PHASE2-PLAN.md             # Phase 2 implementation plan
│   └── INPUT-VALIDATION-PLAN-v2.md # Phase 2.5 implementation plan
├── .github/workflows/cli-test.yml # GitHub Actions configuration
├── .gitlab-ci.yml                 # GitLab CI configuration
├── sample-tests/demo.bats         # Sample tests
├── sample-reports/                # Sample report output
└── README.md                      # This file
```

---
