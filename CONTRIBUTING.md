# Contributing to CLI Testing Specialist

Thank you for your interest in contributing to CLI Testing Specialist! This document provides guidelines and instructions for contributing to the project.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Documentation](#documentation)
- [Release Process](#release-process)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, gender, gender identity and expression, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, or nationality.

### Expected Behavior

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

---

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.75.0 or later)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **BATS** (Bash Automated Testing System)
  ```bash
  # macOS
  brew install bats-core

  # Ubuntu/Debian
  sudo apt-get install bats
  ```

- **Git**
  ```bash
  # macOS
  brew install git

  # Ubuntu/Debian
  sudo apt-get install git
  ```

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/cli-testing-specialist.git
   cd cli-testing-specialist
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/sanae-abe/cli-testing-specialist.git
   ```

---

## Development Setup

### 1. Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Verify installation
cargo test
```

### 2. Install Git Hooks

We use Git hooks to ensure code quality:

```bash
# Install pre-commit and pre-push hooks
./scripts/install-hooks.sh
```

**Hooks installed**:
- **pre-commit**: Runs `cargo fmt` (auto-format code)
- **pre-push**: Runs `cargo clippy` and `cargo test --lib --bins`

### 3. Configure Your Editor

#### VS Code

Recommended extensions:
- `rust-analyzer`: Rust language support
- `Even Better TOML`: TOML syntax highlighting
- `crates`: Cargo.toml dependency management

**settings.json**:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Enable "Rustfmt" in Settings → Languages & Frameworks → Rust → Rustfmt
3. Enable "Run clippy" in Settings → Languages & Frameworks → Rust → External Linters

---

## Development Workflow

### 1. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create a feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description
```

### 2. Make Changes

```bash
# Make your changes
vim src/analyzer/cli_parser.rs

# Format code
cargo fmt

# Check for issues
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test
```

### 3. Commit Changes

Follow our [commit guidelines](#commit-guidelines):

```bash
git add src/analyzer/cli_parser.rs
git commit -m "feat(analyzer): Add support for parsing nested subcommands"
```

### 4. Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create a Pull Request on GitHub
```

---

## Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://rust-lang.github.io/api-guidelines/).

#### Key Principles

1. **Use `rustfmt`** for consistent formatting
   ```bash
   cargo fmt
   ```

2. **Pass `clippy` with zero warnings**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Use meaningful names**
   ```rust
   // Good
   fn parse_cli_options(help_output: &str) -> Vec<CliOption>

   // Bad
   fn parse(s: &str) -> Vec<CO>
   ```

4. **Add documentation comments**
   ```rust
   /// Parse CLI options from help output
   ///
   /// # Arguments
   ///
   /// * `help_output` - Raw help text from --help command
   ///
   /// # Returns
   ///
   /// Vector of parsed CLI options
   ///
   /// # Examples
   ///
   /// ```
   /// let options = parse_cli_options("--help  Display help");
   /// assert_eq!(options.len(), 1);
   /// ```
   pub fn parse_cli_options(help_output: &str) -> Vec<CliOption> {
       // Implementation
   }
   ```

5. **Handle errors properly**
   ```rust
   // Good: Use Result type
   fn analyze_binary(path: &Path) -> Result<CliAnalysis> {
       let output = execute_binary(path)?;
       Ok(parse_output(&output))
   }

   // Bad: Use unwrap()
   fn analyze_binary(path: &Path) -> CliAnalysis {
       let output = execute_binary(path).unwrap(); // ❌ Never use unwrap in production code
       parse_output(&output)
   }
   ```

### Error Handling

- **Never use `unwrap()` or `expect()` in production code**
- Use `?` operator for propagating errors
- Use `Result<T>` for fallible operations
- Provide helpful error messages with context

```rust
// Good
fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .map_err(|e| CliTestError::Config(format!("Failed to read config: {}", e)))?;

    serde_yaml::from_str(&content)
        .map_err(|e| CliTestError::Config(format!("Invalid YAML: {}", e)))
}

// Bad
fn load_config(path: &Path) -> Config {
    let content = fs::read_to_string(path).unwrap(); // ❌
    serde_yaml::from_str(&content).unwrap() // ❌
}
```

### Security Best Practices

1. **Validate all external input**
   ```rust
   fn validate_binary_path(path: &Path) -> Result<PathBuf> {
       // Check path traversal
       let canonical = path.canonicalize()
           .map_err(|_| CliTestError::BinaryNotFound(path.to_path_buf()))?;

       // Ensure binary is executable
       if !canonical.is_file() {
           return Err(CliTestError::BinaryNotFound(canonical));
       }

       Ok(canonical)
   }
   ```

2. **Use safe defaults**
   ```rust
   impl Default for ResourceLimits {
       fn default() -> Self {
           Self {
               timeout_secs: 30,      // Prevent infinite execution
               memory_mb: 500,        // Limit memory usage
               max_processes: 100,    // Prevent fork bombs
           }
       }
   }
   ```

3. **Sanitize paths and commands**
   ```rust
   // Never execute unsanitized user input
   fn execute_command(binary: &Path, args: &[&str]) -> Result<String> {
       let output = Command::new(binary)
           .args(args)
           .output()?;

       Ok(String::from_utf8_lossy(&output.stdout).to_string())
   }
   ```

---

## Testing Guidelines

### Test Structure

We use Rust's built-in testing framework:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_short_option() {
        let input = "-h, --help  Display help";
        let options = parse_cli_options(input);

        assert_eq!(options.len(), 1);
        assert_eq!(options[0].short, Some("-h".to_string()));
        assert_eq!(options[0].long, Some("--help".to_string()));
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "not a valid option";
        let options = parse_cli_options(input);

        assert!(options.is_empty());
    }
}
```

### Test Categories

1. **Unit Tests** (`#[test]`)
   - Test individual functions
   - Mock external dependencies
   - Fast execution (< 1ms per test)

2. **Integration Tests** (`tests/`)
   - Test module interactions
   - Use real files/binaries when safe
   - Moderate execution time (< 100ms per test)

3. **End-to-End Tests** (manual)
   - Test entire workflow
   - Use real CLI tools
   - Run manually before release

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_short_option

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests

# Run tests in release mode (faster)
cargo test --release
```

### Test Coverage

We aim for **80%+ code coverage**:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

### Mutation Testing

We use `cargo-mutants` for mutation testing (target: 70%+ mutation score):

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing
cargo mutants

# Run on specific file
cargo mutants --file src/analyzer/cli_parser.rs
```

---

## Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **refactor**: Code refactoring without feature changes
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks (dependencies, build, etc.)

### Scopes

- `analyzer`: CLI analysis module
- `generator`: Test generation module
- `runner`: Test execution module
- `reporter`: Report generation module
- `cli`: CLI interface
- `config`: Configuration handling
- `security`: Security features
- `ci`: CI/CD configuration

### Examples

```bash
# Feature addition
git commit -m "feat(analyzer): Add support for Python argparse CLIs"

# Bug fix
git commit -m "fix(generator): Fix security test generation for paths with spaces"

# Documentation
git commit -m "docs: Add examples to CONTRIBUTING.md"

# Performance improvement
git commit -m "perf(analyzer): Use parallel processing for large CLIs"

# Breaking change
git commit -m "feat(cli)!: Change --output to --output-dir

BREAKING CHANGE: --output flag renamed to --output-dir for clarity"
```

---

## Pull Request Process

### Before Submitting

1. **Update your branch**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

3. **Update documentation**
   - Update README.md if needed
   - Update CHANGELOG.md
   - Add/update rustdoc comments

### PR Title

Follow the same format as commit messages:

```
feat(analyzer): Add support for Python argparse CLIs
```

### PR Description Template

```markdown
## Description

Brief description of the changes.

## Motivation

Why is this change needed? What problem does it solve?

## Changes

- Added X feature
- Fixed Y bug
- Updated Z documentation

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
- [ ] No new warnings from clippy
- [ ] CHANGELOG.md updated

## Related Issues

Closes #123
```

### Review Process

1. **Automated Checks**: CI runs tests, clippy, and formatting checks
2. **Code Review**: Maintainers review your code
3. **Feedback**: Address review comments
4. **Approval**: PR approved by maintainer
5. **Merge**: PR merged into main branch

### Addressing Feedback

```bash
# Make requested changes
vim src/analyzer/cli_parser.rs

# Commit changes
git add src/analyzer/cli_parser.rs
git commit -m "refactor(analyzer): Address review feedback"

# Push to your branch
git push origin feature/your-feature-name
```

---

## Documentation

### Types of Documentation

1. **Code Documentation** (rustdoc)
   - Public APIs must have documentation comments
   - Include examples for complex functions
   - Document panics, errors, and safety

2. **User Documentation** (README, guides)
   - Keep README.md up to date
   - Add examples to docs/USAGE.md
   - Update docs/TARGET-TOOLS.md for compatibility

3. **API Documentation** (docs.rs)
   - Auto-generated from rustdoc comments
   - Test examples with `cargo test --doc`

### Writing Documentation

```rust
/// Parse CLI options from help output
///
/// This function extracts option definitions from the output of `--help` command.
/// It recognizes short options (e.g., `-h`) and long options (e.g., `--help`).
///
/// # Arguments
///
/// * `help_output` - Raw help text from `--help` command
///
/// # Returns
///
/// Vector of parsed CLI options with inferred types
///
/// # Examples
///
/// ```
/// use cli_testing_specialist::analyzer::parse_cli_options;
///
/// let help = "-h, --help  Display help\n-v, --version  Show version";
/// let options = parse_cli_options(help);
///
/// assert_eq!(options.len(), 2);
/// assert_eq!(options[0].short, Some("-h".to_string()));
/// ```
///
/// # Errors
///
/// Returns `CliTestError::InvalidHelpOutput` if help text is empty
pub fn parse_cli_options(help_output: &str) -> Result<Vec<CliOption>> {
    // Implementation
}
```

### Building Documentation

```bash
# Build documentation
cargo doc --no-deps

# Build and open in browser
cargo doc --no-deps --open

# Check for broken links
cargo rustdoc -- -D warnings
```

---

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. **Update Version**
   ```bash
   # Update Cargo.toml
   version = "1.1.0"

   # Update README.md
   **Version**: 1.1.0
   ```

2. **Update CHANGELOG.md**
   ```markdown
   ## [1.1.0] - 2025-01-20

   ### Added
   - Support for Python argparse CLIs
   - New `--include-intensive` flag

   ### Fixed
   - Security test generation for paths with spaces

   ### Changed
   - Improved error messages
   ```

3. **Run Full Test Suite**
   ```bash
   cargo test --release
   cargo clippy --all-targets --all-features -- -D warnings
   cargo tarpaulin
   ```

4. **Create Git Tag**
   ```bash
   git tag -a v1.1.0 -m "Release v1.1.0"
   git push upstream v1.1.0
   ```

5. **Publish to Crates.io**
   ```bash
   cargo publish --dry-run
   cargo publish
   ```

6. **Create GitHub Release**
   - Go to GitHub Releases
   - Create new release from tag
   - Copy CHANGELOG.md content
   - Upload binaries (if available)

---

## Getting Help

### Resources

- **Documentation**: [README.md](README.md), [docs/USAGE.md](docs/USAGE.md)
- **API Docs**: [docs.rs/cli-testing-specialist](https://docs.rs/cli-testing-specialist)
- **Issues**: [GitHub Issues](https://github.com/sanae-abe/cli-testing-specialist/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sanae-abe/cli-testing-specialist/discussions)

### Questions?

- 💬 **Discussions**: Ask questions in GitHub Discussions
- 🐛 **Bugs**: Report bugs in GitHub Issues
- 💡 **Feature Requests**: Suggest features in GitHub Issues with `enhancement` label

---

## License

By contributing to CLI Testing Specialist, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to CLI Testing Specialist!** 🎉
