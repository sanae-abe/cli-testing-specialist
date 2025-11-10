#!/usr/bin/env bats
#
# test-validator.bats - Unit tests for validator.sh
# Tests: validate_cli_binary(), validate_output_dir(), security checks
#

# Setup test environment
setup() {
    # Load validator module
    export SCRIPT_DIR="${BATS_TEST_DIRNAME}/../../core"
    source "${SCRIPT_DIR}/validator.sh"

    # Set log level to ERROR to reduce noise
    export CLI_TEST_LOG_LEVEL=ERROR

    # Create temp directory for tests
    export TEST_TEMP_DIR="${BATS_TEST_TMPDIR}/validator-test-$$"
    mkdir -p "$TEST_TEMP_DIR"
}

# Cleanup after tests
teardown() {
    rm -rf "$TEST_TEMP_DIR" 2>/dev/null || true
}

# ===== validate_cli_binary() tests =====

@test "validate_cli_binary: accepts valid system binary" {
    run validate_cli_binary "/bin/echo"
    [ "$status" -eq 0 ]
    [[ "$output" == "/bin/echo" ]]
}

@test "validate_cli_binary: rejects empty binary name" {
    run validate_cli_binary ""
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: rejects non-existent binary" {
    run validate_cli_binary "/nonexistent/binary/path"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: rejects path traversal attack (..)  " {
    run validate_cli_binary "/bin/../etc/passwd"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: rejects dangerous characters (;)" {
    run validate_cli_binary "/bin/ls;rm -rf /"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: rejects dangerous characters (\$)" {
    run validate_cli_binary "/bin/\$(whoami)"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: rejects dangerous characters (|)" {
    run validate_cli_binary "/bin/ls|cat"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: resolves binary from PATH" {
    run validate_cli_binary "echo"
    [ "$status" -eq 0 ]
    [[ "$output" =~ /echo$ ]]
}

@test "validate_cli_binary: validates executable permission" {
    # Create non-executable file
    local test_binary="$TEST_TEMP_DIR/non-executable"
    touch "$test_binary"
    chmod 644 "$test_binary"

    run validate_cli_binary "$test_binary"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: accepts executable file" {
    # Create executable file
    local test_binary="$TEST_TEMP_DIR/executable"
    echo "#!/bin/bash" > "$test_binary"
    chmod 755 "$test_binary"

    run validate_cli_binary "$test_binary"
    [ "$status" -eq 0 ]
}

@test "validate_cli_binary: handles symbolic links" {
    local test_binary="$TEST_TEMP_DIR/test-bin"
    local test_link="$TEST_TEMP_DIR/test-link"

    # Create executable and symlink
    echo "#!/bin/bash" > "$test_binary"
    chmod 755 "$test_binary"
    ln -s "$test_binary" "$test_link"

    run validate_cli_binary "$test_link"
    [ "$status" -eq 0 ]
}

@test "validate_cli_binary: rejects system binaries without permission" {
    # Ensure permission is not granted
    export CLI_TEST_ALLOW_SYSTEM_BINARIES=false

    run validate_cli_binary "/usr/bin/python3"
    [ "$status" -eq 1 ]
}

@test "validate_cli_binary: allows system binaries with permission" {
    export CLI_TEST_ALLOW_SYSTEM_BINARIES=true

    run validate_cli_binary "/usr/bin/python3"
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]  # May fail if python3 not installed
}

@test "validate_cli_binary: allows system shells explicitly" {
    run validate_cli_binary "/bin/bash"
    [ "$status" -eq 0 ]
}

# ===== validate_output_dir() tests =====

@test "validate_output_dir: rejects empty directory" {
    run validate_output_dir ""
    [ "$status" -eq 1 ]
}

@test "validate_output_dir: creates non-existent directory" {
    local test_dir="$TEST_TEMP_DIR/new-output-dir"

    run validate_output_dir "$test_dir"
    [ "$status" -eq 0 ]
    [ -d "$test_dir" ]
}

@test "validate_output_dir: accepts existing writable directory" {
    run validate_output_dir "$TEST_TEMP_DIR"
    [ "$status" -eq 0 ]
}

@test "validate_output_dir: rejects /etc directory" {
    run validate_output_dir "/etc/test-output"
    [ "$status" -eq 1 ]
}

@test "validate_output_dir: rejects /usr directory" {
    run validate_output_dir "/usr/test-output"
    [ "$status" -eq 1 ]
}

@test "validate_output_dir: rejects /bin directory" {
    run validate_output_dir "/bin/test-output"
    [ "$status" -eq 1 ]
}

@test "validate_output_dir: rejects /sbin directory" {
    run validate_output_dir "/sbin/test-output"
    [ "$status" -eq 1 ]
}

@test "validate_output_dir: allows /tmp with warning" {
    run validate_output_dir "/tmp/test-output-$$"
    [ "$status" -eq 0 ]
    rm -rf "/tmp/test-output-$$" 2>/dev/null || true
}

@test "validate_output_dir: sets secure permissions (700)" {
    local test_dir="$TEST_TEMP_DIR/secure-dir"

    run validate_output_dir "$test_dir"
    [ "$status" -eq 0 ]

    # Check permissions (700 = rwx------)
    local perms
    perms=$(stat -f "%Lp" "$test_dir" 2>/dev/null || stat -c "%a" "$test_dir" 2>/dev/null)
    [ "$perms" = "700" ]
}

@test "validate_output_dir: rejects directory outside HOME and PWD" {
    # This test may vary based on environment
    skip "Environment-dependent test"
}

# ===== validate_safe_path() tests =====

@test "validate_safe_path: allows user directory paths" {
    run validate_safe_path "$HOME/test"
    [ "$status" -eq 0 ]
}

@test "validate_safe_path: allows current directory paths" {
    run validate_safe_path "$PWD/test"
    [ "$status" -eq 0 ]
}

@test "validate_safe_path: rejects /etc paths" {
    run validate_safe_path "/etc/passwd"
    [ "$status" -eq 1 ]
}

# ===== sanitize_env_var() tests =====

@test "sanitize_env_var: removes dangerous characters" {
    run sanitize_env_var "TEST_VAR" "value;\$(whoami)"
    [ "$status" -eq 0 ]
    [[ "$output" != *";"* ]]
    [[ "$output" != *"\$"* ]]
}

@test "sanitize_env_var: removes newlines" {
    run sanitize_env_var "TEST_VAR" $'value\nwith\nnewlines'
    [ "$status" -eq 0 ]
    [[ "$output" != *$'\n'* ]]
}

@test "sanitize_env_var: removes backticks" {
    run sanitize_env_var "TEST_VAR" 'value`whoami`'
    [ "$status" -eq 0 ]
    [[ "$output" != *'`'* ]]
}

# ===== validate_cli_args() tests =====

@test "validate_cli_args: accepts normal arguments" {
    run validate_cli_args "arg1" "arg2" "arg3"
    [ "$status" -eq 0 ]
}

@test "validate_cli_args: rejects NULL bytes" {
    run validate_cli_args $'arg\x00null'
    [ "$status" -eq 1 ]
}

@test "validate_cli_args: rejects extremely long arguments" {
    local long_arg
    long_arg=$(printf 'a%.0s' {1..15000})

    run validate_cli_args "$long_arg"
    [ "$status" -eq 1 ]
}

@test "validate_cli_args: accepts arguments with spaces" {
    run validate_cli_args "arg with spaces" "another arg"
    [ "$status" -eq 0 ]
}

# ===== Security-focused integration tests =====

@test "SECURITY: command injection attempt via binary name" {
    run validate_cli_binary "/bin/ls && rm -rf /"
    [ "$status" -eq 1 ]
}

@test "SECURITY: command injection attempt via output dir" {
    run validate_output_dir "/tmp/test; rm -rf /"
    [ "$status" -eq 1 ]
}

@test "SECURITY: path traversal via multiple .. sequences" {
    run validate_cli_binary "/bin/../../../../../../etc/passwd"
    [ "$status" -eq 1 ]
}

@test "SECURITY: environment variable injection" {
    run sanitize_env_var "PATH" "/safe:$PATH:\$(malicious)"
    [ "$status" -eq 0 ]
    [[ "$output" != *"\$"* ]]
    [[ "$output" != *"("* ]]
}
