# CLI Testing Specialist - Rust v1.0.0 Design Document

**Version**: 1.0.0
**Status**: Design Phase
**Target Release**: 2026 (Quality-first, no hard deadline)
**Author**: Sanae Abe
**Last Updated**: 2025-11-11

---

## Table of Contents

- [Overview](#overview)
- [Design Philosophy](#design-philosophy)
- [Architecture](#architecture)
- [Type System Design](#type-system-design)
- [Module Structure](#module-structure)
- [CLI Interface](#cli-interface)
- [Data Flow](#data-flow)
- [Performance Strategy](#performance-strategy)
- [Security Considerations](#security-considerations)
- [Testing Strategy](#testing-strategy)
- [Deployment & Distribution](#deployment--distribution)
- [Migration from Bash Prototype](#migration-from-bash-prototype)
- [Implementation Roadmap](#implementation-roadmap)

---

## Overview

### Project Goal

Build a production-ready CLI testing framework in Rust that:
- Analyzes CLI tools automatically
- Generates comprehensive BATS test suites
- Produces detailed reports (Markdown/JSON/HTML/JUnit)
- Runs 10-100x faster than the Bash prototype
- Distributes as a single binary with zero dependencies

### Why Rust v1.0 (Skip Bash Release)

**Decision**: Skip Bash v1.0 public release, go directly to Rust v1.0

**Rationale**:
1. **No backward compatibility burden** - Start with optimal design
2. **Bash prototype value** - Already serves as perfect specification
3. **User experience** - First release delivers best-in-class performance
4. **Technical debt** - Zero legacy constraints

---

## Design Philosophy

### Core Principles

1. **Type Safety First**
   - Leverage Rust's type system for compile-time guarantees
   - Make invalid states unrepresentable
   - Use `Result<T, E>` for all fallible operations

2. **Zero-Cost Abstractions**
   - Performance equal to hand-written C
   - Iterators over loops where possible
   - Inline small functions

3. **Explicit Over Implicit**
   - Clear error messages
   - Verbose logging with `tracing` crate
   - No silent failures

4. **Backward Compatibility (Future)**
   - JSON format stability
   - BATS output format consistency
   - CLI interface versioning

5. **User-Centric Design**
   - Single binary, zero dependencies
   - Intuitive CLI interface
   - Helpful error messages

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                  CLI Interface                       │
│            (clap v4.x - derive API)                 │
└─────────────┬───────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────┐
│                 Command Router                       │
│         (analyze | generate | run | validate)       │
└─────────────┬───────────────────────────────────────┘
              │
    ┌─────────┴─────────┬─────────────┬──────────────┐
    ▼                   ▼             ▼              ▼
┌─────────┐      ┌──────────┐  ┌──────────┐  ┌────────────┐
│Analyzer │      │Generator │  │  Runner  │  │  Reporter  │
│  Module │      │  Module  │  │  Module  │  │   Module   │
└─────────┘      └──────────┘  └──────────┘  └────────────┘
    │                   │             │              │
    ▼                   ▼             ▼              ▼
┌─────────────────────────────────────────────────────┐
│              Shared Type System                      │
│  (CliAnalysis, TestCase, Report, Error types)       │
└─────────────────────────────────────────────────────┘
```

### Module Dependency Graph

```
cli::commands
    ├─> analyzer::cli_parser
    ├─> analyzer::option_inferrer
    ├─> analyzer::subcommand_detector
    ├─> generator::test_generator
    ├─> generator::templates
    ├─> generator::bats_writer
    ├─> runner::bats_executor
    ├─> reporter::markdown
    ├─> reporter::json
    ├─> reporter::html
    └─> reporter::junit

All modules depend on:
    ├─> types::*
    └─> error::CliTestError
```

---

## Type System Design

### Core Types

```rust
// src/types/analysis.rs

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// CLI analysis result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliAnalysis {
    /// Path to the analyzed binary
    pub binary_path: PathBuf,

    /// Binary name (extracted from path)
    pub binary_name: String,

    /// Version string (if detected)
    pub version: Option<String>,

    /// Raw help output
    pub help_output: String,

    /// Detected subcommands (recursive)
    pub subcommands: Vec<Subcommand>,

    /// Global options
    pub global_options: Vec<CliOption>,

    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Subcommand definition (recursive)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subcommand {
    pub name: String,
    pub description: Option<String>,
    pub options: Vec<CliOption>,
    pub subcommands: Vec<Subcommand>,  // Recursive
    pub depth: u8,  // Recursion depth (max 3)
}

/// CLI option definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CliOption {
    pub short: Option<String>,      // -h
    pub long: Option<String>,        // --help
    pub description: Option<String>,
    pub option_type: OptionType,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Option type inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    /// Boolean flag (--verbose)
    Flag,

    /// String value (--name <value>)
    String,

    /// Numeric value (--timeout 30)
    Numeric { min: Option<i64>, max: Option<i64> },

    /// File/directory path (--config /path)
    Path,

    /// Enum value (--format json|yaml|xml)
    Enum { values: Vec<String> },
}

/// Analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisMetadata {
    pub analyzed_at: String,      // ISO 8601
    pub analyzer_version: String, // "1.0.0"
    pub total_subcommands: usize,
    pub total_options: usize,
    pub analysis_duration_ms: u64,
}
```

### Test Case Types

```rust
// src/types/test_case.rs

/// Test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,              // "basic-001"
    pub name: String,            // "Help display test"
    pub category: TestCategory,
    pub command: String,         // "cli-test --help"
    pub expected_exit: i32,      // 0
    pub assertions: Vec<Assertion>,
    pub tags: Vec<String>,       // ["basic", "help"]
}

/// Test category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestCategory {
    Basic,
    Help,
    Security,
    Path,
    MultiShell,
    InputValidation,
    DestructiveOps,
    DirectoryTraversal,
    Performance,
}

/// Assertion types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assertion {
    ExitCode(i32),
    OutputContains(String),
    OutputMatches(String),      // Regex
    OutputNotContains(String),
    FileExists(PathBuf),
    FileNotExists(PathBuf),
}
```

### Error Types

```rust
// src/error.rs

use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum CliTestError {
    #[error("Binary not found: {0}")]
    BinaryNotFound(PathBuf),

    #[error("Binary not executable: {0}")]
    BinaryNotExecutable(PathBuf),

    #[error("Failed to execute binary: {0}")]
    ExecutionFailed(String),

    #[error("Invalid help output")]
    InvalidHelpOutput,

    #[error("Failed to parse option: {0}")]
    OptionParseError(String),

    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    #[error("BATS execution failed: {0}")]
    BatsExecutionFailed(String),

    #[error("Report generation failed: {0}")]
    ReportError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, CliTestError>;
```

---

## Module Structure

### Project Structure

```
cli-testing-specialist/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs         # CLI command definitions
│   ├── analyzer/
│   │   ├── mod.rs
│   │   ├── cli_parser.rs       # Help output parsing
│   │   ├── option_inferrer.rs  # Option type inference
│   │   └── subcommand_detector.rs  # Recursive subcommand detection
│   ├── generator/
│   │   ├── mod.rs
│   │   ├── test_generator.rs   # Test case generation
│   │   ├── templates.rs        # Test templates
│   │   └── bats_writer.rs      # BATS file writing
│   ├── runner/
│   │   ├── mod.rs
│   │   └── bats_executor.rs    # BATS test execution
│   ├── reporter/
│   │   ├── mod.rs
│   │   ├── markdown.rs         # Markdown report
│   │   ├── json.rs             # JSON report
│   │   ├── html.rs             # HTML report
│   │   └── junit.rs            # JUnit XML report
│   ├── types/
│   │   ├── mod.rs
│   │   ├── analysis.rs         # CliAnalysis types
│   │   └── test_case.rs        # TestCase types
│   ├── error.rs                # Error types
│   └── utils/
│       ├── mod.rs
│       ├── shell_detector.rs   # Shell detection
│       └── validator.rs        # Input validation
├── tests/
│   ├── integration_test.rs     # Integration tests
│   └── fixtures/               # Test fixtures
├── benches/
│   └── benchmark.rs            # Performance benchmarks
├── config/                     # YAML configs (from Bash prototype)
│   ├── option-patterns.yaml
│   ├── numeric-constraints.yaml
│   └── enum-definitions.yaml
└── templates/                  # BATS templates (from Bash prototype)
    ├── basic-validation.fragment
    ├── security-scanner.fragment
    ├── input-validation.fragment
    ├── destructive-ops.fragment
    └── directory-traversal.fragment
```

---

## CLI Interface

### Command Structure

```rust
// src/cli/commands.rs

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cli-test")]
#[command(version, about = "Comprehensive CLI testing framework", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a CLI tool
    Analyze {
        /// Path to the CLI binary
        binary: PathBuf,

        /// Output JSON file
        #[arg(short, long, default_value = "cli-analysis.json")]
        output: PathBuf,

        /// Maximum recursion depth for subcommands
        #[arg(short, long, default_value = "3")]
        depth: u8,

        /// Enable parallel processing
        #[arg(long)]
        parallel: bool,
    },

    /// Generate test cases from analysis
    Generate {
        /// Analysis JSON file
        analysis: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "test-output")]
        output: PathBuf,

        /// Test categories (comma-separated or "all")
        #[arg(short, long, default_value = "all")]
        categories: String,
    },

    /// Run BATS tests and generate reports
    Run {
        /// Test directory
        test_dir: PathBuf,

        /// Report format
        #[arg(short, long, default_value = "markdown")]
        format: ReportFormat,

        /// Output directory
        #[arg(short, long, default_value = "reports")]
        output: PathBuf,
    },

    /// Validate analysis JSON file
    Validate {
        /// Analysis JSON file
        file: PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum ReportFormat {
    Markdown,
    Json,
    Html,
    Junit,
    All,
}
```

### Usage Examples

```bash
# Analyze CLI tool
cli-test analyze /usr/bin/curl -o curl-analysis.json

# Analyze with parallel processing
cli-test analyze /usr/local/bin/kubectl --parallel

# Generate all test categories
cli-test generate curl-analysis.json -o tests

# Generate specific categories
cli-test generate curl-analysis.json -o tests -c basic,security,path

# Run tests and generate HTML report
cli-test run tests -f html -o reports

# Validate analysis file
cli-test validate cli-analysis.json
```

---

## Data Flow

### 1. Analyze Phase

```
User Input: /usr/bin/curl
    ↓
CLI Parser (clap)
    ↓
Analyzer::analyze()
    ├─> Execute binary with --help
    ├─> Parse help output (regex patterns)
    ├─> Infer option types (numeric/path/enum)
    ├─> Detect subcommands recursively (max depth 3)
    └─> Build CliAnalysis struct
    ↓
Serialize to JSON (serde_json)
    ↓
Write to file: cli-analysis.json
```

### 2. Generate Phase

```
User Input: cli-analysis.json, categories="all"
    ↓
CLI Parser (clap)
    ↓
Read & Deserialize JSON
    ↓
TestGenerator::generate()
    ├─> Load YAML configs (option-patterns, constraints)
    ├─> Load BATS templates
    ├─> For each category:
    │   ├─> Basic: help, version, exit codes
    │   ├─> Security: injection, null bytes, TOCTOU
    │   ├─> Path: special chars, deep hierarchies
    │   ├─> InputValidation: numeric/path/enum tests
    │   ├─> DestructiveOps: confirmation prompts
    │   └─> DirectoryTraversal: limits, symlinks
    └─> Generate TestCase structs
    ↓
BatsWriter::write()
    ├─> Render BATS templates
    └─> Write *.bats files
```

### 3. Run Phase

```
User Input: test-output/, format="html"
    ↓
CLI Parser (clap)
    ↓
BatsExecutor::run()
    ├─> Execute bats *.bats
    ├─> Capture stdout/stderr
    └─> Parse test results
    ↓
Reporter::generate()
    ├─> Markdown: summary + details
    ├─> JSON: structured results
    ├─> HTML: interactive report
    └─> JUnit: CI/CD integration
    ↓
Write report files
```

---

## Performance Strategy

### Optimization Targets

Based on Bash prototype benchmarks:

| Operation | Bash Prototype | Rust Target | Speedup |
|-----------|---------------|-------------|---------|
| Small CLI analysis (curl) | 2-5s | 0.1-0.3s | 10-50x |
| Large CLI analysis (kubectl) | 20-30s | 2-4s | 8-15x |
| Test generation | 5-10s | 0.5-1s | 10-20x |
| JSON processing (10MB) | 200-300ms | 60-100ms | 2-3x |

### Performance Techniques

1. **Intelligent Parallel Processing Strategy**

The system automatically selects the optimal parallel processing strategy based on workload characteristics:

```rust
/// Parallel processing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    /// Single-threaded execution for small workloads
    Sequential,

    /// Parallel execution per test category (medium workloads)
    CategoryLevel,

    /// Maximum parallelism (large workloads, 4+ CPU cores)
    TestLevel,
}

/// Automatic strategy selection
pub fn choose_strategy(workload: &Workload) -> ParallelStrategy {
    let total_tests = workload.total_estimated_tests();

    if total_tests < 20 || workload.num_categories <= 1 {
        ParallelStrategy::Sequential  // Avoid thread overhead
    } else if total_tests < 100 || workload.num_cpus < 4 {
        ParallelStrategy::CategoryLevel  // Balanced parallelism
    } else {
        ParallelStrategy::TestLevel  // Maximum performance
    }
}
```

**Strategy Selection Algorithm:**

| Workload Size | Categories | CPU Cores | Strategy | Rationale |
|--------------|-----------|-----------|----------|-----------|
| <20 tests | 1 | Any | Sequential | Thread overhead > benefit |
| 20-100 tests | 2-5 | Any | CategoryLevel | Balanced performance |
| 100+ tests | 6+ | 4+ | TestLevel | Maximum parallelism |
| 100+ tests | 6+ | <4 | CategoryLevel | CPU-bound limitation |

**Performance Characteristics:**

- **Sequential**: No thread overhead, predictable execution order
- **CategoryLevel**: 2-4x speedup on 4-core systems, efficient memory usage
- **TestLevel**: 4-8x speedup on 8+ core systems, maximum throughput

**Benchmark Results** (strategy_selection):
- Strategy selection overhead: ~390ns (0.39μs)
- Negligible impact on total execution time (<0.01%)

**Test-Level Parallelism Implementation:**

The system implements adaptive test-level parallelism within individual categories:

```rust
// src/generator/test_level_parallel.rs
pub fn parallel_generate<F>(test_builders: Vec<F>) -> Result<Vec<TestCase>>
where
    F: Fn() -> Result<TestCase> + Send + Sync,
{
    let test_count = test_builders.len();

    // Strategy: Use sequential for small workloads to avoid thread overhead
    if test_count < 10 {
        test_builders.into_iter().map(|f| f()).collect()
    } else {
        // Parallel execution for medium/large workloads
        test_builders.par_iter().map(|f| f()).collect()
    }
}
```

**Applied in `generate_help_tests()`:**

```rust
fn generate_help_tests(&self) -> Result<Vec<TestCase>> {
    // Sequential for <10 subcommands
    if self.analysis.subcommands.len() < 10 {
        // ... sequential implementation
        return Ok(tests);
    }

    // Parallel for 10+ subcommands
    let tests: Vec<TestCase> = self
        .analysis
        .subcommands
        .par_iter()  // Test-level parallelism
        .enumerate()
        .filter_map(|(idx, subcommand)| {
            // Skip 'help' meta-command
            if subcommand.name.to_lowercase() == "help" {
                return None;
            }
            Some(TestCase::new(...))
        })
        .collect();

    Ok(tests)
}
```

**Performance Impact:**
- Small CLIs (curl, 13 options): ~6% improvement (13.8ms → 12.9ms)
- Medium CLIs (npm, 30+ subcommands): Expected 10-20% improvement
- Large CLIs (kubectl, 100+ subcommands): Expected 15-30% improvement

**Threshold Selection:**
- `< 10 tests`: Sequential (avoid thread overhead)
- `>= 10 tests`: Parallel (optimal performance)

**Usage:**
```rust
let generator = TestGenerator::new(analysis, categories);

// Automatic strategy selection (recommended)
let tests = generator.generate_with_strategy()?;

// Manual control (advanced users)
let tests = generator.generate()?;           // Sequential
let tests = generator.generate_parallel()?;  // CategoryLevel + TestLevel (auto)
```

2. **Zero-Copy String Processing**
```rust
use std::borrow::Cow;

fn process_line(line: &str) -> Cow<str> {
    if needs_modification(line) {
        Cow::Owned(modify(line))
    } else {
        Cow::Borrowed(line)  // Zero copy
    }
}
```

3. **Memory Pre-allocation**
```rust
// Pre-allocate based on estimated size
let mut options = Vec::with_capacity(estimated_count);
let mut buffer = String::with_capacity(4096);
```

4. **Compile-Time Regex**
```rust
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref OPTION_PATTERN: Regex = Regex::new(r"--(\w+)").unwrap();
}
```

---

## Security Considerations

### Input Validation

```rust
use std::path::{Path, PathBuf};

/// Validate binary path
pub fn validate_binary_path(path: &Path) -> Result<PathBuf> {
    // Check existence
    if !path.exists() {
        return Err(CliTestError::BinaryNotFound(path.to_path_buf()));
    }

    // Check executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = path.metadata()?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(CliTestError::BinaryNotExecutable(path.to_path_buf()));
        }
    }

    // Resolve to canonical path (prevent path traversal)
    let canonical = path.canonicalize()?;

    Ok(canonical)
}
```

### Command Execution Safety

```rust
use std::process::Command;
use std::time::Duration;

/// Execute binary with timeout
pub fn execute_with_timeout(
    binary: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<String> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Wait with timeout
    let result = wait_timeout::ChildExt::wait_timeout(&mut child, timeout)?;

    match result {
        Some(status) => {
            let output = child.wait_with_output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        None => {
            // Timeout - kill process
            child.kill()?;
            Err(CliTestError::ExecutionFailed("Timeout".to_string()))
        }
    }
}
```

### Memory Safety

Rust's ownership system provides:
- No buffer overflows (compile-time bounds checking)
- No use-after-free (ownership system)
- No data races (Send/Sync traits)
- No null pointer dereference (Option type)

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_type_inference_numeric() {
        let option = "--timeout 30";
        let result = infer_option_type(option);
        assert_eq!(result, OptionType::Numeric { min: None, max: None });
    }

    #[test]
    fn test_option_type_inference_path() {
        let option = "--config /path/to/file";
        let result = infer_option_type(option);
        assert_eq!(result, OptionType::Path);
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_analyze_command() {
    let mut cmd = Command::cargo_bin("cli-test").unwrap();

    cmd.arg("analyze")
       .arg("/usr/bin/curl")
       .arg("-o")
       .arg("test-analysis.json");

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("Analysis complete"));

    // Verify JSON file exists
    assert!(std::path::Path::new("test-analysis.json").exists());
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_option_parsing_doesnt_panic(s in ".*") {
        let _ = parse_option(&s);  // Should not panic
    }
}
```

---

## Deployment & Distribution

### 1. Crates.io

```toml
# Cargo.toml
[package]
name = "cli-testing-specialist"
version = "1.0.0"
authors = ["Sanae Abe <email@example.com>"]
edition = "2021"
license = "MIT"
description = "Comprehensive testing framework for CLI tools"
repository = "https://github.com/sanae-abe/cli-testing-specialist"
keywords = ["cli", "testing", "security", "automation", "bats"]
categories = ["command-line-utilities", "development-tools::testing"]
```

Install:
```bash
cargo install cli-testing-specialist
```

### 2. Homebrew Formula

```ruby
# Formula/cli-testing-specialist.rb
class CliTestingSpecialist < Formula
  desc "Comprehensive testing framework for CLI tools"
  homepage "https://github.com/sanae-abe/cli-testing-specialist"
  url "https://github.com/sanae-abe/cli-testing-specialist/archive/v1.0.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/cli-test", "--version"
  end
end
```

Install:
```bash
brew install cli-testing-specialist
```

### 3. GitHub Releases (Pre-built Binaries)

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload
        uses: actions/upload-artifact@v4
        with:
          name: cli-test-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/cli-test*
```

### 4. Docker Image

```dockerfile
# Dockerfile
FROM rust:1.75-alpine AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/cli-test /usr/local/bin/
ENTRYPOINT ["cli-test"]
```

---

## Migration from Bash Prototype

### Asset Reuse

| Asset | Bash Location | Rust Usage |
|-------|--------------|-----------|
| YAML Configs | config/*.yaml | Load with serde_yaml |
| BATS Templates | templates/*.fragment | Embed with include_str! |
| Test Patterns | core/test-generator.sh | Translate to Rust |
| Algorithm Logic | All *.sh files | Reference as specification |

### Translation Strategy

```rust
// Example: Bash → Rust translation

// Bash (core/test-generator.sh:492-495)
// if [ "$status" -ne 139 ]; then
//     echo "PASS: Null byte handled safely"
// fi

// Rust (src/generator/security_tests.rs)
fn generate_null_byte_test(binary: &Path) -> TestCase {
    TestCase {
        id: "security-null-byte-001".to_string(),
        name: "Null byte injection handling".to_string(),
        category: TestCategory::Security,
        command: format!("{} $'\\x00'", binary.display()),
        expected_exit: 0,  // Not 139 (SIGSEGV)
        assertions: vec![
            Assertion::ExitCode(0),
            // Exit code != 139 means handled safely
        ],
        tags: vec!["security".to_string(), "injection".to_string()],
    }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Goals**: Project setup, core types, basic CLI

**Tasks**:
- [x] Initialize Cargo project
- [ ] Define core types (CliAnalysis, TestCase, Error)
- [ ] Implement basic CLI structure (clap)
- [ ] Setup CI/CD (GitHub Actions)
- [ ] Write initial tests

**Deliverables**:
- Compiling project with all types defined
- Basic `cli-test --help` working
- CI running on push

### Phase 2: Analyzer Module (Week 3-4)

**Goals**: CLI analysis engine

**Tasks**:
- [ ] Implement help output parser
- [ ] Implement option type inference
- [ ] Implement subcommand detector (recursive)
- [ ] Add YAML config loading
- [ ] Write comprehensive unit tests

**Deliverables**:
- `cli-test analyze /usr/bin/curl` working
- JSON output matches Bash prototype format
- 90%+ test coverage

### Phase 3: Generator Module (Week 5-6)

**Goals**: Test case generation

**Tasks**:
- [ ] Implement test generator core
- [ ] Load and process BATS templates
- [ ] Implement all test categories:
  - [ ] Basic validation
  - [ ] Security scanner
  - [ ] Path handling
  - [ ] Input validation
  - [ ] Destructive operations
  - [ ] Directory traversal
- [ ] Write BATS files
- [ ] Add parallel generation support

**Deliverables**:
- `cli-test generate` working
- All test categories implemented
- BATS files identical to Bash prototype

### Phase 4: Runner & Reporter Modules (Week 7-8)

**Goals**: Test execution and reporting

**Tasks**:
- [ ] Implement BATS executor
- [ ] Implement reporters:
  - [ ] Markdown
  - [ ] JSON
  - [ ] HTML (with embedded CSS/JS)
  - [ ] JUnit XML
- [ ] Add report aggregation
- [ ] Performance optimization

**Deliverables**:
- `cli-test run` working
- All report formats generated
- HTML report matches Bash prototype

### Phase 5: Polish & Optimization (Week 9-10)

**Goals**: Performance tuning, documentation

**Tasks**:
- [ ] Performance benchmarking
- [ ] Parallel processing optimization
- [ ] Memory usage optimization
- [ ] Comprehensive documentation
- [ ] Example projects
- [ ] Error message improvements

**Deliverables**:
- 10x faster than Bash prototype
- Complete documentation
- Ready for beta release

### Phase 6: Testing & Release (Week 11-12)

**Goals**: Quality assurance, public release

**Tasks**:
- [ ] Integration testing with real CLI tools
- [ ] User acceptance testing
- [ ] Security audit
- [ ] Package for distributions
- [ ] Write release notes
- [ ] Publish v1.0.0

**Deliverables**:
- Rust v1.0.0 released
- Available on crates.io
- Homebrew formula published
- Documentation site live

---

## Success Criteria

### Functional Requirements

- [ ] Analyze any CLI tool and generate comprehensive tests
- [ ] Support all test categories from Bash prototype
- [ ] Generate BATS files compatible with Bash prototype
- [ ] Produce reports in 4 formats (MD/JSON/HTML/JUnit)
- [ ] Single binary, zero runtime dependencies

### Performance Requirements

- [ ] 10x faster than Bash prototype (minimum)
- [ ] Analyze kubectl (100+ subcommands) in < 5 seconds
- [ ] Memory usage < 50MB for typical workloads

### Quality Requirements

- [ ] 90%+ test coverage
- [ ] Zero compiler warnings with `clippy`
- [ ] All tests pass on Linux/macOS/Windows
- [ ] Comprehensive error handling
- [ ] User-friendly error messages

### Distribution Requirements

- [ ] Published on crates.io
- [ ] Homebrew formula available
- [ ] Pre-built binaries for major platforms
- [ ] Docker image available

---

## Versioning & Maintenance Policy

### Semantic Versioning (SemVer)

このプロジェクトは[Semantic Versioning 2.0.0](https://semver.org/)に従います。

**バージョン番号の形式**: `MAJOR.MINOR.PATCH`

#### MAJOR バージョン（破壊的変更）

以下の場合にMAJORバージョンを上げます：

1. **JSON出力フォーマットの破壊的変更**
   - `CliAnalysis`構造体のフィールド削除・リネーム
   - `TestReport`構造体の互換性のない変更
   - 例: v1.x → v2.0（`metadata`フィールドの構造変更）

2. **CLI引数の破壊的変更**
   - 既存コマンドの削除
   - 必須引数の追加
   - フラグ動作の根本的変更
   - 例: `--categories`のデフォルト値変更

3. **BATS出力フォーマットの破壊的変更**
   - 生成されるテストファイル構造の非互換変更
   - テスト命名規則の大幅変更

4. **最小サポートRustバージョンのメジャー更新**
   - 例: Rust 1.70 → 1.80（破壊的機能使用時）

#### MINOR バージョン（後方互換な機能追加）

以下の場合にMINORバージョンを上げます：

1. **新機能追加**
   - 新しいテストカテゴリの追加
   - 新しいレポート形式の追加
   - 新しいCLIコマンドの追加
   - 例: v1.0 → v1.1（`--format csv`追加）

2. **JSON出力への後方互換なフィールド追加**
   - 新しいメタデータフィールド
   - オプショナルフィールドの追加
   - 例: `CliAnalysis`に`framework_type: Option<String>`追加

3. **非推奨化（Deprecation）**
   - 機能の非推奨化マーク（削除は次のMAJOR）
   - 警告メッセージの表示

#### PATCH バージョン（バグ修正）

以下の場合にPATCHバージョンを上げます：

1. **バグ修正**
   - 誤った動作の修正
   - クラッシュ・パニックの修正
   - メモリリーク修正

2. **パフォーマンス改善**
   - 最適化（動作変更なし）
   - 内部実装の改善

3. **ドキュメント修正**
   - README、ドキュメントの誤字修正
   - コードコメントの改善

4. **依存関係の更新**
   - セキュリティパッチ
   - マイナー依存更新

### 後方互換性保証

#### 保証される互換性

1. **JSON形式の読み込み互換性**
   - 古いバージョンで生成された`cli-analysis.json`は新バージョンで読み込み可能
   - フィールド追加は許可、削除・リネームは禁止（MAJOR変更時を除く）

2. **CLI引数の互換性**
   - 既存のコマンド・フラグは維持
   - デフォルト値の変更は非推奨期間を設ける

3. **BATS出力の互換性**
   - 生成されるテストファイルは既存のBATSランナーで実行可能
   - テスト構造の大幅変更はMAJOR変更

#### 非保証（内部実装）

1. **内部モジュール構造**
   - `src/`以下の内部APIは変更可能
   - 公開API（`lib.rs`）のみ互換性保証

2. **YAML設定ファイル形式**
   - `config/*.yaml`は内部実装として変更可能
   - マイグレーション機能で対応

3. **ログ出力フォーマット**
   - デバッグログは変更可能
   - 機械読み取り用出力（JSON等）は保証

### 非推奨化ポリシー

#### 非推奨化プロセス

1. **警告期間**: 最低2つのMINORバージョン
   - v1.5で非推奨化 → v1.6, v1.7で警告表示 → v2.0で削除

2. **警告方法**:
   ```rust
   #[deprecated(since = "1.5.0", note = "Use new_function() instead")]
   pub fn old_function() { ... }
   ```

3. **ユーザー通知**:
   - CHANGELOGに明記
   - `--help`出力に警告表示
   - 実行時警告メッセージ

#### 非推奨化例

```rust
// v1.5.0: 非推奨化
#[deprecated(since = "1.5.0", note = "Use TestCategory::Security instead")]
pub const SECURITY_TESTS: &str = "security";

// v1.6.0, v1.7.0: 警告期間（機能は維持）
// 実行時: "Warning: SECURITY_TESTS is deprecated, use TestCategory::Security"

// v2.0.0: 削除
// BREAKING CHANGE: Removed deprecated SECURITY_TESTS constant
```

### 設定ファイルマイグレーション

#### ユーザー設定ファイル

ユーザー固有の設定ファイル（`.cli-test-config.yml`）はバージョン管理対象：

```yaml
# .cli-test-config.yml
version: "1.0"  # 設定ファイルのバージョン
tool:
  name: "my-cli"
  # ... 設定内容
```

#### マイグレーション機能

```rust
/// User configuration with version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Configuration file version (SemVer)
    pub version: String,

    /// Tool-specific settings
    pub tool: ToolConfig,

    /// Test generation settings (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_settings: Option<TestSettings>,
}

/// Migrate configuration file to current version
pub fn migrate_config(config_path: &Path) -> Result<UserConfig> {
    let content = fs::read_to_string(config_path)?;
    let mut config: UserConfig = serde_yaml::from_str(&content)?;

    // Version detection
    let config_version = Version::parse(&config.version)?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    if config_version.major < current_version.major {
        // Major version migration
        config = migrate_major_version(config, config_version.major)?;
        log::warn!("Migrated config from v{} to v{}", config_version, current_version);
    }

    if config_version.minor < current_version.minor {
        // Minor version migration (add new fields with defaults)
        config = migrate_minor_version(config)?;
    }

    // Update version
    config.version = current_version.to_string();

    Ok(config)
}

/// Migrate from v1.x to v2.x
fn migrate_major_version(mut config: UserConfig, from_major: u64) -> Result<UserConfig> {
    match from_major {
        1 => {
            // Example: Rename field
            // config.new_field = config.old_field.clone();
            // config.old_field is removed in v2.0
            Ok(config)
        }
        _ => Err(CliTestError::UnsupportedConfigVersion(from_major)),
    }
}

/// Migrate minor version (add new fields with defaults)
fn migrate_minor_version(mut config: UserConfig) -> Result<UserConfig> {
    // Add new optional fields if missing
    if config.test_settings.is_none() {
        config.test_settings = Some(TestSettings::default());
    }
    Ok(config)
}
```

#### マイグレーション戦略

1. **自動検出**: 設定ファイル読み込み時にバージョンチェック
2. **自動マイグレーション**: 可能な場合は自動変換
3. **ユーザー通知**: マイグレーション実施時は警告表示
4. **バックアップ**: マイグレーション前に`.bak`ファイル作成

### リリースプロセス

#### リリース前チェックリスト

1. **コード品質**
   - [ ] `cargo test` 全テスト合格
   - [ ] `cargo clippy -- -D warnings` 警告ゼロ
   - [ ] `cargo fmt --check` フォーマット確認

2. **ドキュメント更新**
   - [ ] CHANGELOG.md に変更内容記載
   - [ ] README.md のバージョン番号更新
   - [ ] API documentation (`cargo doc`)

3. **後方互換性確認**
   - [ ] 旧バージョンの`cli-analysis.json`で動作確認
   - [ ] マイグレーションテスト実施

4. **セキュリティ監査**
   - [ ] `cargo audit` セキュリティスキャン
   - [ ] 依存関係の脆弱性確認

#### リリース手順

1. **バージョンタグ作成**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

2. **crates.io 公開**
   ```bash
   cargo publish --dry-run
   cargo publish
   ```

3. **GitHub Release 作成**
   - バイナリアップロード
   - CHANGELOGからリリースノート生成

4. **Homebrew Formula 更新**
   - SHA256ハッシュ更新
   - バージョン番号更新

### サポートポリシー

#### Long-Term Support (LTS)

- **MAJORバージョン**: 次のMAJORリリース後6ヶ月間セキュリティパッチ提供
- **MINORバージョン**: 最新のみサポート
- **PATCHバージョン**: 最新のみサポート

#### セキュリティパッチ

- セキュリティ脆弱性発見時は緊急PATCHリリース
- サポート期間内の全MAJORバージョンにバックポート

---

## Open Questions

1. **Parallel processing granularity**
   - Parallelize at binary level or subcommand level?
   - Trade-off: Speed vs. memory usage

2. **YAML config caching**
   - Cache parsed YAML in memory or re-read?
   - Impact on memory usage for large configs

3. **Template engine choice**
   - Use handlebars crate or simple string replacement?
   - Trade-off: Flexibility vs. performance

4. **Logging strategy**
   - Use `tracing` or `env_logger`?
   - Structured logging vs. simple text

5. **Plugin system**
   - Support custom test generators via plugins?
   - Future consideration for v2.0

---

## Conclusion

This design provides a solid foundation for Rust v1.0 development. The architecture leverages Rust's strengths (type safety, performance, zero-cost abstractions) while maintaining compatibility with the Bash prototype's proven algorithms and data formats.

Next steps:
1. Review this design document with iterative-review
2. Refine based on feedback
3. Begin implementation with Phase 1
