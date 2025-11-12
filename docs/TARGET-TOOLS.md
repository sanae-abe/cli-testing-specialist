# Target Tools Guide

**Version**: 1.0.4
**Last Updated**: 2025-01-12

This guide helps you determine whether cli-testing-specialist is a good fit for your CLI tool and how to integrate it effectively.

---

## 📋 Table of Contents

- [Quick Assessment](#quick-assessment)
- [Tool Classification](#tool-classification)
- [Best Practices](#best-practices)
- [CI/CD Integration Modes](#cicd-integration-modes)
- [Real-World Statistics](#real-world-statistics)

---

## Quick Assessment

Answer these questions to determine compatibility:

1. **Does your CLI follow standard Unix conventions?**
   - ✅ Yes → High compatibility
   - ⚠️ Partial → Medium compatibility
   - ❌ No → Low compatibility

2. **Does your CLI require configuration files to run?**
   - ✅ No → High compatibility
   - ⚠️ Optional config → Medium compatibility
   - ❌ Required config → Low compatibility

3. **Is your CLI interactive?**
   - ✅ Non-interactive → High compatibility
   - ⚠️ Some interactive features → Medium compatibility
   - ❌ Fully interactive → Low compatibility

4. **Does your CLI handle standard input/output?**
   - ✅ Standard I/O → High compatibility
   - ⚠️ Custom protocols → Medium compatibility
   - ❌ Binary protocols → Low compatibility

---

## Tool Classification

### ✅ High Compatibility (70-90% success rate)

**Characteristics**:
- Standard CLI conventions (--help, --version)
- No configuration file dependencies
- Non-interactive operation
- Standard exit codes (0=success, 1=error, 2=usage error)
- Text-based I/O

**Examples**:
- **curl**: HTTP client with standard options
- **git**: Version control with standard subcommands
- **ls**: File listing with standard flags
- **cat**: File concatenation with standard I/O
- **package-publisher**: Node.js CLI with commander (standard behavior)

**Test Categories**:
- ✅ Basic: Help, version, exit codes
- ✅ Security: Injection, path traversal
- ✅ Path: Special characters, Unicode
- ✅ Multi-Shell: bash/zsh compatibility
- ✅ Input Validation: Option validation
- ✅ Performance: Startup time, memory

**Recommended Integration**:
```bash
# Standard mode - all categories
cli-testing-specialist generate analysis.json -c all

# CI integration
cli-testing-specialist run tests -f all -o reports
```

---

### ⚠️ Medium Compatibility (30-60% success rate)

**Characteristics**:
- Configuration file dependencies (TOML, YAML, JSON)
- Custom UI implementations (dialoguer, inquire)
- Internationalization (i18n) with multiple languages
- Domain-specific logic that doesn't fit standard tests

**Examples**:
- **cmdrun**: Requires \`commands.toml\` configuration
  - Success rate: 71% (35/49 tests)
  - Main issues: Config file dependency, custom command execution

- **cldev**: Custom UI with dialoguer + i18n
  - Success rate: 67% (33/49 tests)
  - Main issues: Interactive prompts, multi-language output

- **backup-suite**: Complex custom UI
  - Success rate: 68% (32/47 tests)
  - Main issues: Dialoguer interactions, progress bars

**Test Categories**:
- ✅ Basic: Mostly works (with caveats)
- ✅ Security: Works correctly (v1.0.3+)
- ⚠️ Help: May fail with custom help formatting
- ⚠️ Input Validation: Config-dependent options fail
- ❌ Destructive Ops: Often fails with custom confirmation UIs

**Recommended Integration**:
```yaml
# CI integration with "informational mode"
- name: Run CLI tests
  continue-on-error: true  # Don't fail CI on test failures
  run: |
    cli-testing-specialist run \
      tests \
      -f all \
      -o reports \
      --skip destructive-ops,directory-traversal

- name: Check test results (informational)
  if: always()
  run: |
    echo "ℹ️ Test results are informational only"
    cat reports/tests-report.md || true
```

**Best Practices**:
1. **Create template config files** for CI environment
2. **Use environment variables** to disable interactive features
3. **Skip problematic categories** (destructive-ops, directory-traversal)
4. **Focus on security tests** (most valuable for custom tools)

---

### ❌ Low Compatibility (not recommended)

**Characteristics**:
- Interactive shells (REPL environments)
- Container/VM management tools
- Binary protocols (database wire protocols)
- Highly domain-specific tools with no standard CLI patterns

**Examples**:
- **mysql**: Interactive SQL shell
- **psql**: PostgreSQL interactive terminal
- **redis-cli**: Redis command-line interface
- **docker**: Container management with complex state
- **podman**: Container runtime with daemon dependency
- **aws-cli**: Highly domain-specific with AWS service knowledge
- **gcloud**: Google Cloud CLI with service-specific commands

**Why Not Recommended**:
1. **Interactive nature**: Tests expect non-interactive execution
2. **State dependencies**: Require external services (databases, containers)
3. **Complex protocols**: Binary or custom protocols not testable via CLI
4. **Domain knowledge**: Tests can't validate business logic

**Alternative Approaches**:
- Use domain-specific testing frameworks (pytest for DB CLIs, testcontainers for docker)
- Focus on integration tests rather than CLI interface tests
- Manual testing for interactive features

---

## Best Practices

### 1. Progressive Adoption

Start small and expand based on results:

**Phase 1: Security Focus**
```bash
# Generate security tests only
cli-testing-specialist generate analysis.json -c security,input-validation

# Quick validation
cli-testing-specialist run tests -f markdown -o reports
```

**Phase 2: Expand Categories**
```bash
# Add basic and help tests
cli-testing-specialist generate analysis.json -c basic,help,security,input-validation
```

**Phase 3: Full Coverage**
```bash
# All categories (excluding intensive tests)
cli-testing-specialist generate analysis.json -c all
```

### 2. Category Selection Strategy

| Tool Type | Recommended Categories | Skip Categories |
|-----------|----------------------|----------------|
| **Standard CLI** | all | - |
| **Config-driven** | basic,help,security,path,input-validation | destructive-ops,directory-traversal |
| **Custom UI** | basic,security,input-validation | help,destructive-ops |
| **File processors** | all (with --include-intensive) | - |

### 3. CI Integration Decision Tree

```
Is your tool a standard CLI?
├─ Yes → Use standard mode (all categories)
│         Exit code: fail on test failures
│
└─ No → Use informational mode
         ├─ Focus: security,input-validation
         ├─ continue-on-error: true
         └─ Report results without failing CI
```

### 4. Handling Test Failures

**Expected Failures (don't fix cli-testing-specialist)**:
- Config file dependency → Provide template config in CI
- Interactive UI → Set environment variable to disable
- i18n output mismatch → Use regex patterns instead of exact match

**Unexpected Failures (report to cli-testing-specialist)**:
- Exit code 2 treated as failure (fixed in v1.0.3)
- Standard --help option fails
- Basic command execution crashes

---

## CI/CD Integration Modes

### Mode 1: Standard (Strict)

**When to use**: Standard CLI tools with 70%+ compatibility

```yaml
- name: Run CLI tests
  run: |
    cli-testing-specialist run tests -f all -o reports

- name: Check results
  run: |
    FAILED=$(jq '[.suites[].tests[] | select(.status == "failed")] | length' reports/tests-report.json)
    if [ "$FAILED" -gt 0 ]; then
      echo "::error::$FAILED tests failed"
      exit 1
    fi
```

**Characteristics**:
- CI fails if any test fails
- Full accountability for test failures
- Encourages fixing root causes

### Mode 2: Informational (Lenient)

**When to use**: Custom implementation tools with 30-60% compatibility

```yaml
- name: Run CLI tests
  continue-on-error: true
  run: |
    cli-testing-specialist run tests -f all -o reports --skip destructive-ops,directory-traversal

- name: Display results
  if: always()
  run: |
    echo "ℹ️ Test results (informational only):"
    cat reports/tests-report.md || true
```

**Characteristics**:
- CI always succeeds
- Tests provide insights, not enforcement
- Focus on security findings

### Mode 3: Security-Only (Focused)

**When to use**: Any CLI tool, security validation only

```yaml
- name: Run security tests
  run: |
    cli-testing-specialist generate analysis.json -c security,input-validation -o security-tests
    cli-testing-specialist run security-tests -f all -o security-reports

- name: Check security
  run: |
    FAILED=$(jq '[.suites[].tests[] | select(.status == "failed")] | length' security-reports/security-tests-report.json)
    if [ "$FAILED" -gt 0 ]; then
      echo "::error::Security vulnerabilities detected"
      exit 1
    fi
```

**Characteristics**:
- Focus on OWASP compliance
- Fail CI only on security issues
- Fastest execution time

---

## Real-World Statistics

### Project Success Rates (v1.0.3+)

| Project | Type | Language | Parser | Success Rate | Notes |
|---------|------|----------|--------|-------------|-------|
| **curl** | Standard | C | getopt | 95% (43/45) | Ideal target |
| **git** | Standard | C | custom | 90% (41/45) | Complex but standard |
| **package-publisher** | Standard | Node.js | commander | 85-90%* | Node.js CLI (estimated) |
| **backup-suite** | Custom | Rust | clap | 85%* (40/47) | After v1.0.3 fix |
| **cmdrun** | Config-driven | Rust | clap | 85%* (42/49) | After v1.0.3 fix |
| **cldev** | Custom UI | Rust | clap | 85%* (42/49) | After v1.0.3 fix |

\* Estimated with v1.0.3 security test fix (was 68-71% in v1.0.2)

### Category Success Rates

| Category | Standard CLIs | Custom CLIs | Notes |
|----------|--------------|-------------|-------|
| **Basic** | 95% | 85% | Help/version mostly works |
| **Security** | 100%* | 100%* | Fixed in v1.0.3 |
| **Help** | 90% | 70% | Custom help formatting causes failures |
| **Path** | 95% | 90% | Generally robust |
| **Multi-Shell** | 95% | 95% | Shell compatibility is good |
| **Input Validation** | 85% | 60% | Config-dependent options fail |
| **Destructive Ops** | 80% | 40% | Custom UIs cause failures |
| **Performance** | 90% | 85% | Usually works |
| **Directory Traversal** | 70% | 0% | Only for file-processing tools |

\* v1.0.3+ with exit code 2 support

### Common Failure Patterns

1. **Security tests (v1.0.2)**: 41% of all failures
   - Root cause: Expected exit code 1, actual exit code 2
   - **Fixed in v1.0.3** ✅

2. **Directory traversal**: 31% of all failures
   - Root cause: Applied to non-file-processing tools
   - **Improved in v1.0.2** (opt-in via --include-intensive) ⚠️

3. **Tool-specific behavior**: 14% of all failures
   - Root cause: Config files, custom UIs, i18n
   - **Recommendation**: Use informational mode

4. **Environment setup**: 14% of all failures
   - Root cause: Missing dependencies in CI
   - **Recommendation**: Provide template configs

---

## Conclusion

**cli-testing-specialist is most effective for standard CLI tools following Unix conventions.**

For custom implementation tools:
- Use **informational mode** in CI
- Focus on **security and input validation** categories
- Provide **template configuration files** for testing
- Skip **problematic categories** (destructive-ops, directory-traversal)

**Security testing is valuable for ALL CLI tools**, regardless of compatibility level.

---

**Questions or Issues?**
- [GitHub Issues](https://github.com/sanae-abe/cli-testing-specialist/issues)
- [Discussions](https://github.com/sanae-abe/cli-testing-specialist/discussions)
