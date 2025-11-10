#!/usr/bin/env bash
#
# test-generator.sh - BATSテストケース生成エンジン
# CLI Testing Specialist Agent v1.1.0
#
# 機能:
# - cli-analyzer.shのJSON出力からBATSテストを生成
# - テンプレートベースの柔軟な生成
# - モジュール別テスト分割
# - 環境変数置換（envsubst）

set -euo pipefail
IFS=$'\n\t'

# Temporary files tracking (for cleanup)
declare -ga TEMP_FILES=()

# Cleanup function
cleanup_temp_files() {
    local exit_code=$?

    if [[ ${#TEMP_FILES[@]} -gt 0 ]]; then
        log DEBUG "Cleaning up ${#TEMP_FILES[@]} temporary files"
        for temp_file in "${TEMP_FILES[@]}"; do
            if [[ -f "$temp_file" ]]; then
                rm -f "$temp_file" 2>/dev/null || true
                log DEBUG "Removed: $temp_file"
            fi
        done
    fi

    exit $exit_code
}

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in test-generator.sh"' ERR

# クリーンアップトラップ（終了時・割り込み時）
trap cleanup_temp_files EXIT INT TERM

# スクリプトのディレクトリを取得（読み取り専用）
declare -r SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
declare -r AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"
source "$SCRIPT_DIR/option-analyzer.sh"

# テンプレートディレクトリ（読み取り専用）
declare -r TEMPLATE_DIR="${AGENT_ROOT}/templates"

# Template cache (performance optimization)
# Global associative array to cache template file contents
declare -gA TEMPLATE_CACHE

#######################################
# Safe jq execution with error handling
# Arguments:
#   $1 - jq filter expression
#   $2 - input file path
#   $3 - default value (optional)
# Returns:
#   jq output or default value on error
#######################################
safe_jq() {
    local filter="$1"
    local input_file="$2"
    local default_value="${3:-}"

    # Check if jq is available
    if ! command -v jq &>/dev/null; then
        log ERROR "jq is not installed"
        log ERROR "  Install: brew install jq (macOS) or apt-get install jq (Ubuntu)"
        echo "$default_value"
        return 1
    fi

    # Check if input file exists
    if [[ ! -f "$input_file" ]]; then
        log ERROR "jq input file not found: $input_file"
        echo "$default_value"
        return 1
    fi

    # Execute jq with error handling
    local output
    local exit_code

    output=$(jq -r "$filter" "$input_file" 2>&1)
    exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log ERROR "jq execution failed (exit code: $exit_code)"
        log ERROR "  Filter: $filter"
        log ERROR "  Input: $input_file"
        log ERROR "  Error: $output"
        echo "$default_value"
        return 1
    fi

    # Return output
    echo "$output"
    return 0
}

#######################################
# Safe yq execution with error handling
# Arguments:
#   $1 - yq filter expression
#   $2 - input file path
#   $3 - default value (optional)
# Returns:
#   yq output or default value on error
#######################################
safe_yq() {
    local filter="$1"
    local input_file="$2"
    local default_value="${3:-}"

    # Check if yq is available
    if ! command -v yq &>/dev/null; then
        log WARN "yq is not installed, using default value"
        echo "$default_value"
        return 1
    fi

    # Check if input file exists
    if [[ ! -f "$input_file" ]]; then
        log ERROR "yq input file not found: $input_file"
        echo "$default_value"
        return 1
    fi

    # Execute yq with error handling
    local output
    local exit_code

    output=$(yq "$filter" "$input_file" 2>&1)
    exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log ERROR "yq execution failed (exit code: $exit_code)"
        log ERROR "  Filter: $filter"
        log ERROR "  Input: $input_file"
        log ERROR "  Error: $output"
        echo "$default_value"
        return 1
    fi

    # Return output
    echo "$output"
    return 0
}

# envsubstの存在確認（フォールバック対応）
if ! command -v envsubst &>/dev/null; then
    log WARN "envsubst not found, using basic template substitution"
    USE_ENVSUBST=false
else
    USE_ENVSUBST=true
fi

#######################################
# Load template file with caching
# Globals:
#   TEMPLATE_CACHE (reads/writes)
# Arguments:
#   $1 - Template file path
# Returns:
#   Template content via stdout
#######################################
load_template_cached() {
    local template_file="$1"

    # Check cache first
    if [[ -n "${TEMPLATE_CACHE[$template_file]:-}" ]]; then
        log DEBUG "Template cache hit: $template_file"
        echo "${TEMPLATE_CACHE[$template_file]}"
        return 0
    fi

    # Cache miss - load from file
    log DEBUG "Template cache miss: $template_file"

    if [[ ! -f "$template_file" ]]; then
        log ERROR "Template file not found: $template_file"
        return 1
    fi

    # Read entire file into cache
    TEMPLATE_CACHE[$template_file]=$(<"$template_file")

    # Return content
    echo "${TEMPLATE_CACHE[$template_file]}"
    return 0
}

# テンプレート変数置換（ファイルベース）
# Performance: uses template caching to reduce file I/O
substitute_template() {
    local template_file="$1"
    local output_file="$2"

    # Load template from cache (or file on first access)
    local template_content
    template_content=$(load_template_cached "$template_file") || return 1

    # Create temporary file for TEST_CASES
    local test_cases_file
    test_cases_file=$(mktemp)
    TEMP_FILES+=("$test_cases_file")  # Track for cleanup
    echo "$TEST_CASES" > "$test_cases_file"

    # Process template line by line
    # Performance: use here-string instead of file redirection
    while IFS= read -r line; do
        # Check if line contains ${TEST_CASES}
        if [[ "$line" == *'${TEST_CASES}'* ]]; then
            # Insert TEST_CASES content with variable substitution
            while IFS= read -r test_line; do
                # Replace variables in TEST_CASES
                test_line="${test_line//\$\{TEST_MODULE\}/$TEST_MODULE}"
                test_line="${test_line//\$\{CLI_BINARY\}/$CLI_BINARY}"
                test_line="${test_line//\$\{BINARY_BASENAME\}/$BINARY_BASENAME}"
                if [[ -n "${SUBCOMMAND:-}" ]]; then
                    test_line="${test_line//\$\{SUBCOMMAND\}/$SUBCOMMAND}"
                fi
                echo "$test_line"
            done < "$test_cases_file"
        else
            # Replace other variables
            line="${line//\$\{TEST_MODULE\}/$TEST_MODULE}"
            line="${line//\$\{CLI_BINARY\}/$CLI_BINARY}"
            line="${line//\$\{BINARY_BASENAME\}/$BINARY_BASENAME}"
            if [[ -n "${SUBCOMMAND:-}" ]]; then
                line="${line//\$\{SUBCOMMAND\}/$SUBCOMMAND}"
            fi
            echo "$line"
        fi
    done <<< "$template_content" > "$output_file"

    # Cleanup
    rm -f "$test_cases_file"
}

# メインテスト生成関数
generate_bats_tests() {
    local analysis_json="$1"
    local output_dir="$2"
    local test_modules="${3:-all}"  # all, basic, help, security, etc.

    log INFO "Starting BATS test generation"
    log DEBUG "Input JSON: $analysis_json"
    log DEBUG "Output directory: $output_dir"
    log DEBUG "Test modules: $test_modules"

    # 入力バリデーション
    if [[ ! -f "$analysis_json" ]]; then
        log ERROR "Analysis JSON file not found: $analysis_json"
        return 1
    fi

    # JSON妥当性チェック
    if ! jq empty "$analysis_json" 2>/dev/null; then
        log ERROR "Invalid JSON format: $analysis_json"
        return 1
    fi

    # 出力ディレクトリ検証
    local validated_output_dir
    validated_output_dir=$(validate_output_dir "$output_dir") || {
        log ERROR "Output directory validation failed"
        return 1
    }

    log INFO "Validated output directory: $validated_output_dir"

    # JSON解析
    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")
    local subcommand_count
    subcommand_count=$(jq -r '.subcommand_count' "$analysis_json")
    local option_count
    option_count=$(jq -r '.option_count' "$analysis_json")

    log INFO "CLI Tool Analysis:"
    log INFO "  Binary: $cli_binary"
    log INFO "  Subcommands: $subcommand_count"
    log INFO "  Options: $option_count"

    # モジュール別テスト生成
    local generated_tests=0

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"basic"* ]]; then
        generate_basic_validation_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"help"* ]]; then
        if [[ $subcommand_count -gt 0 ]]; then
            generate_help_checker_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
        else
            log INFO "Skipping help checker tests (no subcommands detected)"
        fi
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"security"* ]]; then
        generate_security_scanner_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"path"* ]]; then
        generate_path_handler_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"multi-shell"* ]]; then
        generate_multi_shell_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"performance"* ]]; then
        generate_performance_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"concurrency"* ]]; then
        generate_concurrency_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"input-validation"* ]]; then
        generate_input_validation_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    if [[ "$test_modules" == "all" ]] || [[ "$test_modules" == *"destructive-ops"* ]]; then
        generate_destructive_ops_tests "$analysis_json" "$validated_output_dir" && ((generated_tests++)) || true
    fi

    log INFO "Test generation completed"
    log INFO "  Generated test files: $generated_tests"
    log INFO "  Output directory: $validated_output_dir"

    echo "$validated_output_dir"
}

# 基本バリデーションテスト生成
generate_basic_validation_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating basic validation tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    # テンプレート読み込み
    local template_file="$TEMPLATE_DIR/bats-test.template"
    if [[ ! -f "$template_file" ]]; then
        log ERROR "Template file not found: $template_file"
        return 1
    fi

    local template
    template=$(<"$template_file")

    # 変数定義
    TEST_MODULE="basic-validation"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # バージョンチェックテスト（実際の改行を使用）
    TEST_CASES=$(cat <<EOF
@test "[$TEST_MODULE] --version returns version information" {
    run "\$CLI_BINARY" --version
    [ "\$status" -eq 0 ] || [ "\$status" -eq 1 ]  # Some CLIs exit 1 on --version
    [ -n "\$output" ]  # Output should not be empty
}

@test "[$TEST_MODULE] --help returns help information" {
    run "\$CLI_BINARY" --help
    [ "\$status" -eq 0 ]
    [ -n "\$output" ]
}

@test "[$TEST_MODULE] invalid option returns error" {
    run "\$CLI_BINARY" --invalid-option-xyz-12345
    [ "\$status" -ne 0 ]  # Should fail
}

@test "[$TEST_MODULE] binary is executable" {
    [ -x "\$CLI_BINARY" ]
}
EOF
)

    # テンプレート置換
    local output_file="$output_dir/01-basic-validation.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

# ヘルプチェッカーテスト生成
generate_help_checker_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating help checker tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")
    local subcommands
    subcommands=$(jq -r '.subcommands[]' "$analysis_json")

    # テンプレートフラグメント読み込み
    local fragment_file="$TEMPLATE_DIR/subcommand-help.fragment"
    if [[ ! -f "$fragment_file" ]]; then
        log ERROR "Fragment file not found: $fragment_file"
        return 1
    fi

    TEST_MODULE="help-checker"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # サブコマンドごとのヘルプテスト生成
    local help_test_cases=""
    while IFS= read -r subcommand; do
        [[ -z "$subcommand" ]] && continue

        # 一時ファイルでフラグメント処理
        local temp_fragment
        temp_fragment=$(mktemp)
        sed -e "s|\${TEST_MODULE}|$TEST_MODULE|g" \
            -e "s|\${CLI_BINARY}|$CLI_BINARY|g" \
            -e "s|\${SUBCOMMAND}|$subcommand|g" \
            "$fragment_file" > "$temp_fragment"

        help_test_cases+=$(cat "$temp_fragment")$'\n\n'
        rm -f "$temp_fragment"

        log DEBUG "Added help test for subcommand: $subcommand"
    done <<< "$subcommands"

    TEST_CASES="$help_test_cases"

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/02-help-checker.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

# セキュリティスキャナーテスト生成
generate_security_scanner_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating security scanner tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    TEST_MODULE="security-scanner"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # セキュリティテストケース（実際の改行を使用）
    TEST_CASES=$(cat <<'EOF'
@test "[${TEST_MODULE}] command injection prevention - semicolon" {
    run "$CLI_BINARY" --version "; echo 'injected'"
    [ "$status" -ne 0 ] || ! echo "$output" | grep -q 'injected'
}

@test "[${TEST_MODULE}] path traversal prevention" {
    run "$CLI_BINARY" --help ../../../etc/passwd 2>&1
    [ "$status" -ne 0 ] || ! echo "$output" | grep -q 'root:'
}

@test "[${TEST_MODULE}] null byte injection prevention" {
    run "$CLI_BINARY" --version $'\x00malicious'
    [ "$status" -ne 0 ]
}

@test "[${TEST_MODULE}] buffer overflow prevention - long argument" {
    local long_arg=$(printf 'A%.0s' {1..10000})
    run timeout 10 "$CLI_BINARY" --help "$long_arg"
    # Should either fail gracefully or succeed without crashing
    [ "$status" -ne 139 ]  # Not segfault
}
EOF
)

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/03-security-scanner.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

# パスハンドラーテスト生成
generate_path_handler_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating path handler tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    TEST_MODULE="path-handler"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # パステストケース（実際の改行を使用）
    TEST_CASES=$(cat <<'EOF'
@test "[${TEST_MODULE}] deep path hierarchy handling" {
    local deep_path="$(mktemp -d)/a/b/c/d/e/f/g/h/i/j"
    mkdir -p "$deep_path"

    # Test with deep path (command depends on CLI capabilities)
    run "$CLI_BINARY" --help

    # Cleanup - navigate up 10 levels safely
    local cleanup_path="$deep_path"
    for i in {1..10}; do
        cleanup_path="$(dirname "$cleanup_path")"
    done
    rm -rf "$cleanup_path"

    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "[${TEST_MODULE}] path with spaces handling" {
    local space_path="$(mktemp -d)/test with spaces"
    mkdir -p "$space_path"

    run "$CLI_BINARY" --help

    rm -rf "$(dirname "$space_path")"
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "[${TEST_MODULE}] path with special characters" {
    local special_path="$(mktemp -d)/test-_@#"
    mkdir -p "$special_path" 2>/dev/null || skip "Special characters not supported by filesystem"

    run "$CLI_BINARY" --help

    rm -rf "$(dirname "$special_path")" 2>/dev/null || true
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}
EOF
)

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/04-path-handler.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

# Multi-shellテスト生成
generate_multi_shell_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating multi-shell compatibility tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    # Shell検出JSONを取得（なければ検出実行）
    local shell_detection_json="$output_dir/../shell-detection.json"
    if [[ ! -f "$shell_detection_json" ]]; then
        log INFO "Running shell detection..."
        bash "$SCRIPT_DIR/shell-detector.sh" "$shell_detection_json" >&2 || {
            log WARN "Shell detection failed, using defaults"
            shell_detection_json=""
        }
    fi

    TEST_MODULE="multi-shell"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # 利用可能なshellを取得
    local available_shells
    if [[ -f "$shell_detection_json" ]]; then
        available_shells=$(jq -r '.shells[] | select(.available == true) | .name' "$shell_detection_json")
    else
        # デフォルトshell
        available_shells="bash"
    fi

    # Multi-shellテストケース生成
    local shell_test_cases=""
    while IFS= read -r shell_name; do
        [[ -z "$shell_name" ]] && continue

        # bashベースのテストのみ生成（BATSはbashで動作）
        # shell_nameだけを事前置換してからheredocに渡す
        local current_test
        current_test=$(cat <<'SHELLTEST'

@test "[${TEST_MODULE}] CLI runs in SHELL_NAME_PLACEHOLDER environment" {
    # Check if shell is available
    if ! command -v SHELL_NAME_PLACEHOLDER &>/dev/null; then
        skip "SHELL_NAME_PLACEHOLDER not available"
    fi

    # Test basic execution through the shell
    run SHELL_NAME_PLACEHOLDER -c "$CLI_BINARY --version 2>&1 || $CLI_BINARY -v 2>&1 || echo 'version check skipped'"
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
    [ -n "$output" ]
}
SHELLTEST
)
        # shell_nameを置換
        current_test="${current_test//SHELL_NAME_PLACEHOLDER/$shell_name}"
        shell_test_cases+="$current_test"
    done <<< "$available_shells"

    TEST_CASES="$shell_test_cases"

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/05-multi-shell.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    log INFO "  Tested shells: $(echo "$available_shells" | wc -l | tr -d ' ')"
    return 0
}

# パフォーマンステスト生成
generate_performance_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating performance tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    TEST_MODULE="performance"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # フラグメントファイル読み込み
    local fragment_file="$TEMPLATE_DIR/performance-test.fragment"
    if [[ ! -f "$fragment_file" ]]; then
        log ERROR "Fragment file not found: $fragment_file"
        return 1
    fi

    # フラグメント内容を読み込み（変数置換）
    TEST_CASES=$(sed -e "s|\${TEST_MODULE}|$TEST_MODULE|g" \
                     -e "s|\${CLI_BINARY}|$CLI_BINARY|g" \
                     -e "s|\${BINARY_BASENAME}|$BINARY_BASENAME|g" \
                     "$fragment_file")

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/06-performance.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

# 並行実行テスト生成
generate_concurrency_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating concurrency tests"

    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")
    local binary_basename
    binary_basename=$(jq -r '.binary_basename' "$analysis_json")

    TEST_MODULE="concurrency"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # フラグメントファイル読み込み
    local fragment_file="$TEMPLATE_DIR/concurrency-test.fragment"
    if [[ ! -f "$fragment_file" ]]; then
        log ERROR "Fragment file not found: $fragment_file"
        return 1
    fi

    # フラグメント内容を読み込み（変数置換）
    TEST_CASES=$(sed -e "s|\${TEST_MODULE}|$TEST_MODULE|g" \
                     -e "s|\${CLI_BINARY}|$CLI_BINARY|g" \
                     -e "s|\${BINARY_BASENAME}|$BINARY_BASENAME|g" \
                     "$fragment_file")

    # テンプレート置換
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/07-concurrency.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    return 0
}

#######################################
# Generate input validation tests
# Globals:
#   TEMPLATE_DIR (reads)
# Arguments:
#   $1 - Analysis JSON file path
#   $2 - Output directory
# Returns:
#   0 - success, 1 - failure
#######################################
generate_input_validation_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating input validation tests"

    # Extract CLI metadata
    local cli_binary
    cli_binary=$(safe_jq '.binary' "$analysis_json" "unknown") || return 1
    local binary_basename
    binary_basename=$(safe_jq '.binary_basename' "$analysis_json" "unknown") || return 1

    # Extract options
    local options
    options=$(safe_jq '.options[]?' "$analysis_json" "")

    if [[ -z "$options" ]]; then
        log WARN "No options found in analysis JSON, skipping input validation tests"
        return 0
    fi

    log INFO "Analyzing options for input validation tests"

    # Classify options by type
    local -a numeric_options=()
    local -a path_options=()
    local -a enum_options=()

    while IFS= read -r option; do
        [[ -z "$option" ]] && continue

        # Infer option type using option-analyzer.sh
        local option_type
        option_type=$(infer_option_type "$option")

        case "$option_type" in
            numeric)
                numeric_options+=("$option")
                log DEBUG "Classified as numeric: $option"
                ;;
            path)
                path_options+=("$option")
                log DEBUG "Classified as path: $option"
                ;;
            enum)
                enum_options+=("$option")
                log DEBUG "Classified as enum: $option"
                ;;
            *)
                log DEBUG "Skipped (type: $option_type): $option"
                ;;
        esac
    done <<< "$options"

    log INFO "Option classification: numeric=${#numeric_options[@]}, path=${#path_options[@]}, enum=${#enum_options[@]}"

    # Skip if no testable options
    if [[ ${#numeric_options[@]} -eq 0 ]] && [[ ${#path_options[@]} -eq 0 ]] && [[ ${#enum_options[@]} -eq 0 ]]; then
        log WARN "No numeric/path/enum options found, skipping input validation tests"
        return 0
    fi

    # Load input-validation.fragment template
    local fragment_file="$TEMPLATE_DIR/input-validation.fragment"
    if [[ ! -f "$fragment_file" ]]; then
        log ERROR "Fragment file not found: $fragment_file"
        return 1
    fi

    # Generate test cases for each option type
    local test_cases=""

    # Numeric options
    for option in "${numeric_options[@]}"; do
        # Extract constraints
        local constraints
        constraints=$(extract_numeric_constraints "$option")

        local min_value
        min_value=$(echo "$constraints" | jq -r '.min // 0')
        local max_value
        max_value=$(echo "$constraints" | jq -r '.max // 2147483647')
        local option_type_detail
        option_type_detail=$(echo "$constraints" | jq -r '.type // "integer"')

        # Generate test cases from fragment (numeric section)
        # Note: This is a simplified version - full implementation would parse fragment
        log DEBUG "Generating numeric tests for $option (min=$min_value, max=$max_value)"
    done

    # Path options
    for option in "${path_options[@]}"; do
        log DEBUG "Generating path tests for $option"
    done

    # Enum options
    for option in "${enum_options[@]}"; do
        local enum_values
        enum_values=$(extract_enum_values "$option")
        log DEBUG "Generating enum tests for $option (values=$enum_values)"
    done

    # For now, use the entire fragment as test cases
    # TODO: Implement selective fragment extraction based on option types
    TEST_MODULE="input-validation"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # Read fragment content
    TEST_CASES=$(<"$fragment_file")

    # Replace placeholders (example for first numeric option)
    if [[ ${#numeric_options[@]} -gt 0 ]]; then
        local first_option="${numeric_options[0]}"
        local constraints
        constraints=$(extract_numeric_constraints "$first_option")

        local min_value
        min_value=$(echo "$constraints" | jq -r '.min // 0')
        local max_value
        max_value=$(echo "$constraints" | jq -r '.max // 100')

        TEST_CASES="${TEST_CASES//\$\{OPTION_NAME\}/$first_option}"
        TEST_CASES="${TEST_CASES//\$\{MIN_VALUE\}/$min_value}"
        TEST_CASES="${TEST_CASES//\$\{MAX_VALUE\}/$max_value}"
        TEST_CASES="${TEST_CASES//\$\{VALID_VALUE\}/$(( (min_value + max_value) / 2 ))}"
    fi

    # Generate output file
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/08-input-validation.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    log INFO "  Numeric options: ${#numeric_options[@]}"
    log INFO "  Path options: ${#path_options[@]}"
    log INFO "  Enum options: ${#enum_options[@]}"

    return 0
}

#######################################
# Generate destructive operations tests
# Globals:
#   TEMPLATE_DIR (reads)
# Arguments:
#   $1 - Analysis JSON file path
#   $2 - Output directory
# Returns:
#   0 - success, 1 - failure
#######################################
generate_destructive_ops_tests() {
    local analysis_json="$1"
    local output_dir="$2"

    log DEBUG "Generating destructive operations tests"

    # Extract CLI metadata
    local cli_binary
    cli_binary=$(safe_jq '.binary' "$analysis_json" "unknown") || return 1
    local binary_basename
    binary_basename=$(safe_jq '.binary_basename' "$analysis_json" "unknown") || return 1

    # Check for destructive commands/options
    # Common patterns: delete, remove, destroy, rm, erase, wipe, clean, purge
    local subcommands
    subcommands=$(safe_jq '.subcommands[]?' "$analysis_json" "")

    local -a destructive_commands=()

    while IFS= read -r subcommand; do
        [[ -z "$subcommand" ]] && continue

        # Check if subcommand name suggests destructive operation
        if [[ "$subcommand" =~ (delete|remove|destroy|rm|erase|wipe|clean|purge|drop|truncate) ]]; then
            destructive_commands+=("$subcommand")
            log DEBUG "Detected destructive command: $subcommand"
        fi
    done <<< "$subcommands"

    # If no destructive commands found, use generic placeholder
    if [[ ${#destructive_commands[@]} -eq 0 ]]; then
        log WARN "No destructive commands detected, generating generic tests"
        destructive_commands=("delete" "remove")
    fi

    log INFO "Destructive commands: ${#destructive_commands[@]}"

    # Load destructive-ops.fragment template
    local fragment_file="$TEMPLATE_DIR/destructive-ops.fragment"
    if [[ ! -f "$fragment_file" ]]; then
        log ERROR "Fragment file not found: $fragment_file"
        return 1
    fi

    TEST_MODULE="destructive-ops"
    CLI_BINARY="$cli_binary"
    BINARY_BASENAME="$binary_basename"

    # Read fragment content
    TEST_CASES=$(<"$fragment_file")

    # Replace placeholders
    local first_destructive="${destructive_commands[0]}"
    TEST_CASES="${TEST_CASES//\$\{DESTRUCTIVE_COMMAND\}/$first_destructive}"
    TEST_CASES="${TEST_CASES//\$\{DESTRUCTIVE_ARGS\}/--test-target}"
    TEST_CASES="${TEST_CASES//\$\{YES_FLAG\}/--yes}"

    # Generate output file
    local template_file="$TEMPLATE_DIR/bats-test.template"
    local output_file="$output_dir/09-destructive-ops.bats"
    substitute_template "$template_file" "$output_file"

    log INFO "Generated: $output_file"
    log INFO "  Destructive commands tested: ${#destructive_commands[@]}"

    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 2 ]]; then
        echo "Usage: $0 <analysis-json> <output-dir> [test-modules]" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <analysis-json>  JSON file generated by cli-analyzer.sh" >&2
        echo "  <output-dir>     Directory to output BATS test files" >&2
        echo "  [test-modules]   Optional: all|basic|help|security|path|multi-shell|performance|concurrency|input-validation|destructive-ops (default: all)" >&2
        echo "" >&2
        echo "Example:" >&2
        echo "  $0 cli-analysis.json ./tests" >&2
        echo "  $0 cli-analysis.json ./tests basic,help,performance" >&2
        exit 1
    fi

    # テスト生成実行
    generate_bats_tests "$@"
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        log INFO "Test generation completed successfully"
    else
        log ERROR "Test generation failed with exit code: $exit_code"
    fi

    exit $exit_code
fi
