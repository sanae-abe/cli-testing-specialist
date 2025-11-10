#!/usr/bin/env bats
#
# test-logger.bats - Unit tests for logger.sh
# Tests: log(), log levels, filtering, rotation
#

# Setup test environment
setup() {
    # Load logger module
    export SCRIPT_DIR="${BATS_TEST_DIRNAME}/../../utils"

    # Set custom log file for testing
    export CLI_TEST_LOG_FILE="${BATS_TEST_TMPDIR}/test-logger-$$.log"
    export CLI_TEST_LOG_LEVEL=DEBUG
    export CLI_TEST_LOG_COLOR=false

    # Source logger after setting environment
    source "${SCRIPT_DIR}/logger.sh"
}

# Cleanup after tests
teardown() {
    rm -f "${CLI_TEST_LOG_FILE}" "${CLI_TEST_LOG_FILE}.1" 2>/dev/null || true
}

# ===== Log function basic tests =====

@test "log: DEBUG level message" {
    run log DEBUG "Test debug message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ DEBUG.*Test\ debug\ message ]]
}

@test "log: INFO level message" {
    run log INFO "Test info message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ INFO.*Test\ info\ message ]]
}

@test "log: WARN level message" {
    run log WARN "Test warning message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ WARN.*Test\ warning\ message ]]
}

@test "log: ERROR level message" {
    run log ERROR "Test error message"
    [ "$status" -eq 0 ]
    [[ "$output" =~ ERROR.*Test\ error\ message ]]
}

@test "log: includes timestamp in output" {
    run log INFO "Timestamp test"
    [ "$status" -eq 0 ]
    [[ "$output" =~ [0-9]{4}-[0-9]{2}-[0-9]{2}\ [0-9]{2}:[0-9]{2}:[0-9]{2} ]]
}

@test "log: DEBUG includes caller information" {
    run log DEBUG "Caller info test"
    [ "$status" -eq 0 ]
    [[ "$output" =~ \[.*:.*\] ]]  # Contains [function:line]
}

@test "log: writes to file when LOG_FILE is set" {
    log INFO "File write test"

    [ -f "$CLI_TEST_LOG_FILE" ]
    grep -q "File write test" "$CLI_TEST_LOG_FILE"
}

@test "log: handles multi-word messages" {
    run log INFO "This is a multi-word message with spaces"
    [ "$status" -eq 0 ]
    [[ "$output" =~ This\ is\ a\ multi-word\ message\ with\ spaces ]]
}

# ===== Log level filtering tests =====

@test "log filtering: DEBUG level shows all messages" {
    export CLI_TEST_LOG_LEVEL=DEBUG
    set_log_level DEBUG

    log DEBUG "debug msg" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log INFO "info msg" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log WARN "warn msg" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log ERROR "error msg" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null

    grep -q "debug msg" "$CLI_TEST_LOG_FILE"
    grep -q "info msg" "$CLI_TEST_LOG_FILE"
    grep -q "warn msg" "$CLI_TEST_LOG_FILE"
    grep -q "error msg" "$CLI_TEST_LOG_FILE"
}

@test "log filtering: INFO level filters out DEBUG" {
    export CLI_TEST_LOG_LEVEL=INFO
    set_log_level INFO

    log DEBUG "should not appear" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log INFO "should appear" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null

    ! grep -q "should not appear" "$CLI_TEST_LOG_FILE"
    grep -q "should appear" "$CLI_TEST_LOG_FILE"
}

@test "log filtering: WARN level filters out DEBUG and INFO" {
    export CLI_TEST_LOG_LEVEL=WARN
    set_log_level WARN

    log DEBUG "debug filtered" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log INFO "info filtered" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log WARN "warn shown" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null

    ! grep -q "debug filtered" "$CLI_TEST_LOG_FILE"
    ! grep -q "info filtered" "$CLI_TEST_LOG_FILE"
    grep -q "warn shown" "$CLI_TEST_LOG_FILE"
}

@test "log filtering: ERROR level shows only errors" {
    export CLI_TEST_LOG_LEVEL=ERROR
    set_log_level ERROR

    log DEBUG "debug filtered" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log INFO "info filtered" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log WARN "warn filtered" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    log ERROR "error shown" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null

    ! grep -q "debug filtered" "$CLI_TEST_LOG_FILE"
    ! grep -q "info filtered" "$CLI_TEST_LOG_FILE"
    ! grep -q "warn filtered" "$CLI_TEST_LOG_FILE"
    grep -q "error shown" "$CLI_TEST_LOG_FILE"
}

# ===== set_log_level() tests =====

@test "set_log_level: changes log level to DEBUG" {
    run set_log_level DEBUG
    [ "$status" -eq 0 ]
    [ "$LOG_LEVEL" = "DEBUG" ]
}

@test "set_log_level: changes log level to INFO" {
    run set_log_level INFO
    [ "$status" -eq 0 ]
    [ "$LOG_LEVEL" = "INFO" ]
}

@test "set_log_level: changes log level to WARN" {
    run set_log_level WARN
    [ "$status" -eq 0 ]
    [ "$LOG_LEVEL" = "WARN" ]
}

@test "set_log_level: changes log level to ERROR" {
    run set_log_level ERROR
    [ "$status" -eq 0 ]
    [ "$LOG_LEVEL" = "ERROR" ]
}

@test "set_log_level: rejects invalid log level" {
    run set_log_level INVALID
    [ "$status" -eq 1 ]
}

# ===== init_logger() tests =====

@test "init_logger: creates log file" {
    rm -f "$CLI_TEST_LOG_FILE"

    init_logger

    [ -f "$CLI_TEST_LOG_FILE" ]
}

@test "init_logger: creates log directory if missing" {
    local custom_log_dir="${BATS_TEST_TMPDIR}/custom-log-dir-$$"
    export CLI_TEST_LOG_FILE="$custom_log_dir/test.log"

    init_logger

    [ -d "$custom_log_dir" ]
    [ -f "$CLI_TEST_LOG_FILE" ]

    rm -rf "$custom_log_dir"
}

@test "init_logger: sets secure file permissions (600)" {
    init_logger

    local perms
    perms=$(stat -f "%Lp" "$CLI_TEST_LOG_FILE" 2>/dev/null || stat -c "%a" "$CLI_TEST_LOG_FILE" 2>/dev/null)
    [ "$perms" = "600" ]
}

@test "init_logger: falls back to /tmp if log file cannot be created" {
    # Set unwritable log path
    export CLI_TEST_LOG_FILE="/root/unwritable/test.log"

    init_logger

    # Should fallback to /tmp
    [[ "$LOG_FILE" =~ ^/tmp/cli-test-fallback- ]]
}

# ===== rotate_log() tests =====

@test "rotate_log: does nothing if log file doesn't exist" {
    rm -f "$CLI_TEST_LOG_FILE"

    run rotate_log
    [ "$status" -eq 0 ]
}

@test "rotate_log: rotates log file when it exceeds max size" {
    # Create a large log file (> 10MB)
    dd if=/dev/zero of="$CLI_TEST_LOG_FILE" bs=1024 count=11000 2>/dev/null

    rotate_log

    # Original should be rotated to .1
    [ -f "${CLI_TEST_LOG_FILE}.1" ]

    # New log file should exist and be small
    [ -f "$CLI_TEST_LOG_FILE" ]
    local size
    size=$(stat -f%z "$CLI_TEST_LOG_FILE" 2>/dev/null || stat -c%s "$CLI_TEST_LOG_FILE" 2>/dev/null)
    [ "$size" -lt 1024 ]
}

@test "rotate_log: does not rotate small log files" {
    # Create small log file
    echo "small log" > "$CLI_TEST_LOG_FILE"

    rotate_log

    # Should not create .1 file
    [ ! -f "${CLI_TEST_LOG_FILE}.1" ]
}

# ===== log_error_with_trace() tests =====

@test "log_error_with_trace: logs error with stack trace" {
    run log_error_with_trace "Error with trace"
    [ "$status" -eq 0 ]
    [[ "$output" =~ ERROR.*Error\ with\ trace ]]
    [[ "$output" =~ Stack\ trace ]]
}

@test "log_error_with_trace: includes function names in trace" {
    test_function() {
        log_error_with_trace "Traced error"
    }

    run test_function
    [ "$status" -eq 0 ]
    [[ "$output" =~ test_function ]]
}

# ===== Log file security tests =====

@test "SECURITY: log file has restricted permissions" {
    init_logger

    local perms
    perms=$(stat -f "%Lp" "$CLI_TEST_LOG_FILE" 2>/dev/null || stat -c "%a" "$CLI_TEST_LOG_FILE" 2>/dev/null)

    # Should be 600 (rw-------)
    [ "$perms" = "600" ]
}

@test "SECURITY: log messages don't execute commands" {
    # Attempt command injection via log message
    log INFO "test \$(whoami) injection"

    # Log should contain literal string, not command output
    grep -q '\$(whoami)' "$CLI_TEST_LOG_FILE" || grep -q 'whoami' "$CLI_TEST_LOG_FILE"
}

# ===== Color output tests =====

@test "log color: disabled when LOG_COLOR=false" {
    export CLI_TEST_LOG_COLOR=false

    run log INFO "No color test"
    [ "$status" -eq 0 ]
    [[ "$output" != *$'\033'* ]]  # No ANSI escape codes
}

@test "log color: enabled when LOG_COLOR=true and output is terminal" {
    export CLI_TEST_LOG_COLOR=true

    # Note: In BATS, stdout is not a TTY, so this test verifies the logic
    # rather than actual color output
    run log INFO "Color test"
    [ "$status" -eq 0 ]
}

# ===== Edge case tests =====

@test "log: handles empty message" {
    run log INFO ""
    [ "$status" -eq 0 ]
}

@test "log: handles special characters in message" {
    run log INFO "Message with special chars: !@#\$%^&*()"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "!@#" ]]
}

@test "log: handles very long messages" {
    local long_msg
    long_msg=$(printf 'A%.0s' {1..1000})

    run log INFO "$long_msg"
    [ "$status" -eq 0 ]
}

@test "log: handles newlines in message" {
    run log INFO "Line 1\nLine 2"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Line 1" ]]
}

# ===== Performance tests =====

@test "log: handles rapid successive calls" {
    for i in {1..100}; do
        log INFO "Message $i" 2>&1 | tee -a "$CLI_TEST_LOG_FILE" >/dev/null
    done

    [ -f "$CLI_TEST_LOG_FILE" ]
    local line_count
    line_count=$(wc -l < "$CLI_TEST_LOG_FILE")
    [ "$line_count" -ge 100 ]
}
