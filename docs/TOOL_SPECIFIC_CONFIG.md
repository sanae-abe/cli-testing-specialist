# Tool-Specific Test Configuration

**Version**: 1.0.0
**Last Updated**: 2025-11-14
**Status**: Design Proposal

## Overview

This document defines the `.cli-test-config.yml` format for tool-specific test adjustments, addressing the mismatch between generic test generation and tool-specific implementation details.

## Problem Statement

### Real-World Issue: backup-suite Test Failures

When testing backup-suite with cli-testing-specialist, 8 tests failed due to implementation mismatch:

1. **Security Tests (3/3 failed)**
   - Generic expectation: Reject dangerous inputs with exit ≠ 0
   - backup-suite reality: `--lang` is enum with safe fallback, returns exit 0
   - Issue: No security risk, but test assumes input validation failure

2. **Directory Traversal (3/3 failed)**
   - Generic expectation: Test directories exist
   - backup-suite reality: Paths don't exist in CI environment
   - Issue: Test setup missing, not a tool bug

3. **Destructive Operations (2/2 failed)**
   - Generic expectation: Confirmation prompt in non-TTY works
   - backup-suite reality: dialoguer requires TTY
   - Issue: CI/CD incompatibility, needs env var workaround

**Conclusion**: These are **test design mismatches**, not tool bugs.

## Solution: `.cli-test-config.yml`

A YAML configuration file that allows tool authors to adjust test behavior without modifying the tool or test generator.

### File Location

Place in the tool's project root or specify with `--config`:

```bash
# Auto-detected from current directory
cli-testing-specialist generate analysis.json

# Explicit path
cli-testing-specialist generate analysis.json --config /path/to/.cli-test-config.yml
```

### Schema

```yaml
# .cli-test-config.yml
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"

# Test category adjustments
test_adjustments:
  # Security test customization
  security:
    # Skip security tests for specific options
    skip_options:
      - name: "lang"
        reason: "Enum option with safe fallback, no injection risk"
      - name: "format"
        reason: "Validated against enum, command injection not possible"

    # Custom security tests
    custom_tests:
      - name: "test_path_injection_in_backup_target"
        command: "backup create --target='$(whoami)'"
        expected_exit_code: 1
        description: "Verify path validation rejects command substitution"

  # Directory traversal test setup
  directory_traversal:
    # RECOMMENDED: Declarative approach (no command execution)
    test_directories:
      - path: "/tmp/test-large-dir"
        create: true
        file_count: 100
        cleanup: true
      - path: "/tmp/test-deep-dir"
        create: true
        depth: 5
        cleanup: true

    # ALTERNATIVE: Imperative commands (requires --allow-setup flag)
    setup_commands:
      - "mkdir -p /tmp/test-large-dir"
      - "mkdir -p /tmp/test-deep-dir/level1/level2/level3"

    teardown_commands:
      - "rm -rf /tmp/test-large-dir"
      - "rm -rf /tmp/test-deep-dir"

    # Skip all directory traversal tests
    skip: false

    # Skip specific tests
    skip_tests:
      - "test_symlink_loop"  # Not supported by this tool

  # Destructive operation test customization
  destructive_ops:
    # Environment variables for CI/CD
    env_vars:
      BACKUP_SUITE_YES: "true"  # Auto-confirm destructive operations
      CI: "true"                 # Disable interactive prompts

    # Expected exit codes for confirmation rejection
    cancel_exit_code: 2  # backup-suite uses exit 2, not default 1

    # Commands that require special handling
    special_commands:
      - command: "remove"
        requires_tty: false  # Can run in CI with env vars
        confirm_flag: "--yes"
      - command: "cleanup"
        requires_tty: true   # Skip in CI (dialoguer limitation)

  # Path handling test customization
  path:
    # Skip Unicode path tests (filesystem limitation)
    skip_unicode: false

    # Custom path separators (Windows compatibility)
    path_separator: "/"  # Unix-style, even on Windows

  # Multi-shell test customization
  multi_shell:
    # Shells to test
    shells: ["bash", "zsh"]  # Skip fish/dash

    # Shell-specific env vars
    bash_env:
      BACKUP_SUITE_SHELL: "bash"
    zsh_env:
      BACKUP_SUITE_SHELL: "zsh"

  # Performance test customization
  performance:
    # Startup time threshold (ms)
    max_startup_time: 500

    # Memory usage threshold (MB)
    max_memory_mb: 100

    # Skip performance tests in CI
    skip_in_ci: true

# Global test settings
global:
  # Timeout for all tests (seconds)
  timeout: 30

  # Retry failed tests
  retry_count: 0

  # Verbose output
  verbose: false

  # Environment variables for all tests
  env_vars:
    LANG: "en_US.UTF-8"
    TZ: "UTC"

# CI/CD specific settings
ci:
  # Detect CI environment
  auto_detect: true

  # CI-specific adjustments
  skip_tty_tests: true
  skip_intensive_tests: true

  # CI environment variables (auto-detected)
  # - GITHUB_ACTIONS
  # - GITLAB_CI
  # - CIRCLECI
  # - TRAVIS
```

## Implementation

### Phase 1: Schema Definition (v1.1.0)

```rust
// src/types/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct CliTestConfig {
    pub version: String,
    pub tool_name: String,
    pub tool_version: Option<String>,
    pub test_adjustments: TestAdjustments,
    pub global: Option<GlobalSettings>,
    pub ci: Option<CiSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestAdjustments {
    pub security: Option<SecurityAdjustments>,
    pub directory_traversal: Option<DirectoryTraversalAdjustments>,
    pub destructive_ops: Option<DestructiveOpsAdjustments>,
    pub path: Option<PathAdjustments>,
    pub multi_shell: Option<MultiShellAdjustments>,
    pub performance: Option<PerformanceAdjustments>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityAdjustments {
    pub skip_options: Vec<SkipOption>,
    pub custom_tests: Vec<CustomSecurityTest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SkipOption {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DirectoryTraversalAdjustments {
    // RECOMMENDED: Declarative approach
    pub test_directories: Vec<TestDirectory>,

    // ALTERNATIVE: Imperative commands (requires --allow-setup)
    pub setup_commands: Vec<String>,
    pub teardown_commands: Vec<String>,

    pub skip: bool,
    pub skip_tests: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestDirectory {
    pub path: String,
    pub create: bool,
    pub file_count: Option<usize>,
    pub depth: Option<usize>,
    pub cleanup: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DestructiveOpsAdjustments {
    pub env_vars: HashMap<String, String>,
    pub cancel_exit_code: i32,
    pub special_commands: Vec<SpecialCommand>,
}
```

### Phase 2: Config Loading (v1.1.0)

```rust
// src/config/loader.rs
use crate::types::config::CliTestConfig;
use std::path::Path;

pub fn load_config(path: Option<&Path>) -> Result<Option<CliTestConfig>> {
    // 1. Check explicit path
    if let Some(p) = path {
        return Ok(Some(load_from_file(p)?));
    }

    // 2. Check current directory
    let default_path = Path::new(".cli-test-config.yml");
    if default_path.exists() {
        return Ok(Some(load_from_file(default_path)?));
    }

    // 3. No config found (use defaults)
    Ok(None)
}

fn load_from_file(path: &Path) -> Result<CliTestConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = serde_yaml::from_str(&content)?;

    // Validate schema version
    validate_version(&config)?;

    Ok(config)
}
```

### Phase 3: Test Generator Integration (v1.1.0)

```rust
// src/generator/test_generator.rs
impl TestGenerator {
    pub fn generate_with_config(
        &self,
        config: Option<&CliTestConfig>,
    ) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Apply security test adjustments
        if let Some(cfg) = config {
            tests.extend(self.generate_security_tests_adjusted(cfg)?);
        } else {
            tests.extend(self.generate_security_tests()?);
        }

        // ... other categories

        Ok(tests)
    }

    fn generate_security_tests_adjusted(
        &self,
        config: &CliTestConfig,
    ) -> Result<Vec<TestCase>> {
        let skip_opts = config
            .test_adjustments
            .security
            .as_ref()
            .map(|s| &s.skip_options)
            .unwrap_or(&vec![]);

        // Filter out skipped options
        let tests: Vec<_> = self
            .analysis
            .options
            .iter()
            .filter(|opt| !skip_opts.iter().any(|s| s.name == opt.name))
            .flat_map(|opt| self.generate_security_test_for_option(opt))
            .collect();

        Ok(tests)
    }
}
```

### Phase 4: BATS Test Setup/Teardown (v1.1.0)

```bash
# Generated BATS test with setup/teardown
setup() {
  # From .cli-test-config.yml: test_adjustments.directory_traversal.setup_commands
  mkdir -p /tmp/test-large-dir
  mkdir -p /tmp/test-deep-dir/level1/level2/level3
}

teardown() {
  # From .cli-test-config.yml: test_adjustments.directory_traversal.teardown_commands
  rm -rf /tmp/test-large-dir
  rm -rf /tmp/test-deep-dir
}

@test "directory traversal: large file count" {
  # Test uses /tmp/test-large-dir created in setup()
  run backup-suite backup --source /tmp/test-large-dir
  [ "$status" -eq 0 ]
}
```

## Usage Examples

### Example 1: backup-suite Configuration

```yaml
version: "1.0"
tool_name: "backup-suite"
tool_version: "1.0.0"

test_adjustments:
  security:
    skip_options:
      - name: "lang"
        reason: "Enum option with safe fallback"

  directory_traversal:
    setup_commands:
      - "mkdir -p /tmp/backup-test-dir"
    teardown_commands:
      - "rm -rf /tmp/backup-test-dir"

  destructive_ops:
    env_vars:
      BACKUP_SUITE_YES: "true"
    cancel_exit_code: 2
```

### Example 2: kubectl-like Tool

```yaml
version: "1.0"
tool_name: "kubectl"

test_adjustments:
  security:
    skip_options:
      - name: "context"
        reason: "Validated against kubeconfig, safe"
      - name: "namespace"
        reason: "Alphanumeric validation, no injection"

  destructive_ops:
    env_vars:
      KUBECTL_FORCE: "true"
    special_commands:
      - command: "delete"
        requires_tty: false
        confirm_flag: "--force"

global:
  timeout: 60  # kubectl can be slow
```

## Benefits

### For Tool Authors

1. **No code changes required**: Adjust tests without modifying tool
2. **CI/CD compatibility**: Define env vars and setup/teardown
3. **Security clarity**: Document why certain tests are skipped
4. **Faster adoption**: Remove false positives immediately

### For Test Generator

1. **Higher accuracy**: Tests match tool reality
2. **Reduced false positives**: Skip irrelevant tests
3. **Better documentation**: Config file serves as test specification
4. **Backward compatibility**: Config is optional, defaults work

## Migration Path

### v1.0.x → v1.1.0 (Config Support)

1. Existing projects without config: No change, use defaults
2. Projects with config: Tests automatically adjusted
3. No breaking changes to CLI or API

### Future Extensions (v1.2.0+)

- Custom test templates
- Plugin system for test categories
- Integration with CI/CD platforms
- Shared config repository (e.g., github.com/cli-test-configs)

## Security Considerations

### Risk: Config-based Test Skipping

**Concern**: Tool authors could skip all security tests

**Mitigation**:
1. Require `reason` field for all skips (documentation)
2. Generate warning in report for skipped security tests
3. CI/CD integration validates config against policy

```yaml
# .cli-test-policy.yml (CI/CD enforcement)
security:
  max_skipped_options: 3
  require_reason: true
  warn_on_skip: true
```

### Risk: Setup Commands Execution

**Concern**: Arbitrary command execution in setup/teardown

**Mitigation** (Multi-layered defense):

#### Layer 1: Explicit Consent (REQUIRED)
```bash
# setup_commands are DISABLED by default
# Requires explicit --allow-setup flag
cli-testing-specialist run tests --allow-setup
```

#### Layer 2: Command Validation
```rust
// Forbidden patterns (always rejected)
const FORBIDDEN: &[&str] = &[
    "|", ";", "&&", "||",     // Command chaining
    "`", "$(", "$(",           // Command substitution
    "sudo", "su",              // Privilege escalation
    "curl", "wget", "nc",      // Network access
    "rm -rf /", "rm -rf /*",   // Dangerous deletions
];

// Allowed commands (configurable allowlist)
const ALLOWED: &[&str] = &[
    "mkdir", "touch", "rm", "cp", "mv",
    "echo", "cat", "ls",
];
```

#### Layer 3: Resource Limits
```rust
// Apply same ResourceLimits as test execution
let limits = ResourceLimits {
    timeout: Duration::from_secs(30),
    memory_mb: 100,
    max_processes: 10,
};
```

#### Layer 4: Safe Alternative (RECOMMENDED)
Use declarative `test_directories` instead of imperative commands:

```yaml
# SAFE: No command execution
test_directories:
  - path: "/tmp/test-dir"
    create: true
    file_count: 100
    cleanup: true

# RISKY: Requires --allow-setup + validation
setup_commands:
  - "mkdir -p /tmp/test-dir"
```

## Documentation Updates

### README.md Addition

```markdown
## Tool-Specific Configuration

Customize test behavior for your CLI tool:

1. Create `.cli-test-config.yml` in your project root
2. Define test adjustments (skip options, setup commands, env vars)
3. Run tests normally - config is auto-detected

See [docs/TOOL_SPECIFIC_CONFIG.md](./docs/TOOL_SPECIFIC_CONFIG.md) for details.
```

### CLI Help Update

```
USAGE:
    cli-testing-specialist generate <analysis.json> [OPTIONS]

OPTIONS:
    --config <PATH>     Path to .cli-test-config.yml (default: auto-detect)
    --allow-setup       Allow setup/teardown commands execution
```

## Implementation Timeline

| Phase | Version | Tasks | Effort |
|-------|---------|-------|--------|
| Schema Definition | v1.1.0 | Define config types, validation | 1h |
| Config Loading | v1.1.0 | File parsing, auto-detection | 1h |
| Test Adjustment | v1.1.0 | Apply config to test generation | 1.5h |
| Documentation | v1.1.0 | README, examples, tests | 0.5h |

**Total**: 4h (task-12)

## Testing Strategy

1. **Unit Tests**: Config parsing and validation
2. **Integration Tests**: backup-suite test suite with config
3. **Regression Tests**: Verify no impact when config absent
4. **Security Tests**: Validate setup command sandboxing

## Success Criteria

✅ backup-suite test success rate: 100% (from 52.9%)
✅ Config file auto-detection works
✅ Security test skipping documented and warned
✅ Setup/teardown commands execute safely
✅ Backward compatibility maintained

---

## References

- [backup-suite Test Failures Analysis](https://github.com/sanae-abe/backup-suite/issues/1)
- [OWASP Input Validation](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html)
- [YAML Specification](https://yaml.org/spec/1.2.2/)
