#!/usr/bin/env bash
#
# docker-test-runner.sh - Docker環境内でのCLIテスト実行エンジン
# CLI Testing Specialist Agent v1.1.0
#
# 複数のDocker環境（alpine, ubuntu, debian）でテストを実行し、
# 環境別のレポートを生成します。

set -euo pipefail
IFS=$'\n\t'

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_TEST_ROOT="$(dirname "$SCRIPT_DIR")"

# ロガーの読み込み
source "$CLI_TEST_ROOT/utils/logger.sh"

# デフォルト設定
DEFAULT_ENVIRONMENTS="alpine,ubuntu,debian"
DEFAULT_TIMEOUT=300  # 5分

# 使用方法
show_usage() {
    cat <<EOF
Docker Test Runner - CLI Testing Specialist

Usage: $0 [OPTIONS] <cli-binary> <test-directory> <output-directory>

Arguments:
  cli-binary          CLI binary to test
  test-directory      Directory containing .bats test files
  output-directory    Directory for test results

Options:
  -e, --environments  Docker environments to test (default: $DEFAULT_ENVIRONMENTS)
                      Available: alpine,ubuntu,debian,all
  -t, --timeout       Test timeout in seconds (default: $DEFAULT_TIMEOUT)
  -v, --verbose       Enable verbose logging
  -h, --help          Show this help message

Examples:
  # Test on all environments
  $0 /usr/bin/git ./tests ./output

  # Test on specific environments
  $0 -e alpine,ubuntu /usr/bin/docker ./tests ./output

  # Custom timeout
  $0 -t 600 /usr/bin/ls ./tests ./output

EOF
}

# 引数パース
parse_arguments() {
    local environments="$DEFAULT_ENVIRONMENTS"
    local timeout="$DEFAULT_TIMEOUT"
    local cli_binary=""
    local test_dir=""
    local output_dir=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -e|--environments)
                environments="$2"
                shift 2
                ;;
            -t|--timeout)
                timeout="$2"
                shift 2
                ;;
            -v|--verbose)
                export CLI_TEST_LOG_LEVEL=DEBUG
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            -*)
                log ERROR "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                if [[ -z "$cli_binary" ]]; then
                    cli_binary="$1"
                elif [[ -z "$test_dir" ]]; then
                    test_dir="$1"
                elif [[ -z "$output_dir" ]]; then
                    output_dir="$1"
                else
                    log ERROR "Too many arguments"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done

    # 必須引数チェック
    if [[ -z "$cli_binary" ]] || [[ -z "$test_dir" ]] || [[ -z "$output_dir" ]]; then
        log ERROR "Missing required arguments"
        show_usage
        exit 1
    fi

    # "all" を全環境に展開
    if [[ "$environments" == "all" ]]; then
        environments="alpine,ubuntu,debian"
    fi

    export CLI_BINARY="$cli_binary"
    export TEST_DIR="$test_dir"
    export OUTPUT_DIR="$output_dir"
    export ENVIRONMENTS="$environments"
    export TIMEOUT="$timeout"
}

# Docker環境チェック
check_docker() {
    log INFO "Checking Docker availability"

    if ! command -v docker &> /dev/null; then
        log ERROR "Docker is not installed or not in PATH"
        return 1
    fi

    if ! docker info &> /dev/null; then
        log ERROR "Docker daemon is not running"
        return 1
    fi

    log INFO "Docker is available and running"
    return 0
}

# Dockerイメージのビルド
build_docker_image() {
    local environment="$1"
    local dockerfile="$CLI_TEST_ROOT/docker/Dockerfile.$environment"
    local image_name="cli-testing-specialist:$environment"

    log INFO "Building Docker image for $environment"

    if [[ ! -f "$dockerfile" ]]; then
        log ERROR "Dockerfile not found: $dockerfile"
        return 1
    fi

    # イメージが既に存在するか確認
    if docker image inspect "$image_name" &> /dev/null; then
        log DEBUG "Image already exists: $image_name (skipping build)"
        echo "$image_name"
        return 0
    fi

    # ビルド実行
    local build_output
    build_output=$(docker build -f "$dockerfile" -t "$image_name" "$CLI_TEST_ROOT/docker" 2>&1) || {
        log ERROR "Failed to build Docker image for $environment"
        log ERROR "$build_output"
        return 1
    }

    log INFO "Successfully built: $image_name"
    echo "$image_name"
}

# Docker環境内でテスト実行
run_tests_in_docker() {
    local environment="$1"
    local image_name="$2"
    local cli_binary="$3"
    local test_dir="$4"
    local output_dir="$5"
    local timeout="$6"

    log INFO "Running tests in Docker environment: $environment"

    # 環境別出力ディレクトリ作成
    local env_output_dir="$output_dir/$environment"
    mkdir -p "$env_output_dir"

    # CLIバイナリ名を取得
    local cli_name
    cli_name=$(basename "$cli_binary")

    # Docker実行
    local container_name="cli-test-$environment-$$"
    local exit_code=0

    # テスト実行（タイムアウト付き）
    docker run --rm \
        --name "$container_name" \
        --mount type=bind,source="$(realpath "$cli_binary")",target="/test-cli/$cli_name",readonly \
        --mount type=bind,source="$(realpath "$test_dir")",target=/test-suite,readonly \
        --mount type=bind,source="$(realpath "$env_output_dir")",target=/test-output \
        --env CLI_BINARY="/test-cli/$cli_name" \
        --env TEST_DIR=/test-suite \
        --env OUTPUT_DIR=/test-output \
        --env ENVIRONMENT="$environment" \
        "$image_name" \
        timeout "$timeout" bats --formatter tap /test-suite/*.bats > "$env_output_dir/test.tap" 2>&1 || exit_code=$?

    # タイムアウトチェック
    if [[ $exit_code -eq 124 ]]; then
        log WARN "Tests timed out in $environment environment (${timeout}s)"
        echo "timeout" > "$env_output_dir/status"
        return 2
    fi

    # 成功/失敗ステータス記録
    if [[ $exit_code -eq 0 ]]; then
        log INFO "All tests passed in $environment environment"
        echo "success" > "$env_output_dir/status"
    else
        log WARN "Some tests failed in $environment environment (exit code: $exit_code)"
        echo "failed" > "$env_output_dir/status"
    fi

    # 環境情報の記録
    docker run --rm "$image_name" sh -c 'uname -a' > "$env_output_dir/system-info.txt" 2>&1

    return $exit_code
}

# 統合レポート生成
generate_summary_report() {
    local output_dir="$1"
    local environments="$2"

    log INFO "Generating summary report"

    local report_file="$output_dir/docker-test-summary.md"
    local json_file="$output_dir/docker-test-summary.json"

    # JSONレポート初期化
    echo '{
  "test_run": {
    "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
    "cli_binary": "'"$CLI_BINARY"'"
  },
  "environments": {}
}' > "$json_file"

    # Markdownレポート開始
    cat > "$report_file" <<EOF
# Docker Environment Test Summary

**Test Run**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**CLI Binary**: $CLI_BINARY

---

## Test Results by Environment

EOF

    # 環境ごとの結果を集計
    local env_array
    IFS=',' read -ra env_array <<< "$environments"

    local total_passed=0
    local total_failed=0
    local total_timeout=0

    for env in "${env_array[@]}"; do
        local env_dir="$output_dir/$env"

        if [[ ! -d "$env_dir" ]]; then
            log WARN "No results found for environment: $env"
            continue
        fi

        local status
        status=$(cat "$env_dir/status" 2>/dev/null || echo "unknown")

        local system_info
        system_info=$(cat "$env_dir/system-info.txt" 2>/dev/null || echo "N/A")

        # TAP結果解析
        local test_count=0
        local passed=0
        local failed=0

        if [[ -f "$env_dir/test.tap" ]]; then
            test_count=$(grep -c "^1\.\." "$env_dir/test.tap" 2>/dev/null || echo 0)
            passed=$(grep -c "^ok " "$env_dir/test.tap" 2>/dev/null || echo 0)
            failed=$(grep -c "^not ok " "$env_dir/test.tap" 2>/dev/null || echo 0)
        fi

        # ステータスカウント
        case "$status" in
            success) ((total_passed++)) ;;
            failed) ((total_failed++)) ;;
            timeout) ((total_timeout++)) ;;
        esac

        # Markdownレポートに追加
        local status_emoji
        case "$status" in
            success) status_emoji="✅" ;;
            failed) status_emoji="❌" ;;
            timeout) status_emoji="⏱️" ;;
            *) status_emoji="❓" ;;
        esac

        cat >> "$report_file" <<EOF
### $env $status_emoji

- **Status**: $status
- **Tests**: $passed passed, $failed failed (total: $test_count)
- **System**: $system_info

EOF

        # JSONに追加
        local env_json
        env_json=$(jq -n \
            --arg status "$status" \
            --argjson passed "$passed" \
            --argjson failed "$failed" \
            --argjson total "$test_count" \
            --arg system "$system_info" \
            '{status: $status, tests: {passed: $passed, failed: $failed, total: $total}, system: $system}')

        jq --arg env "$env" --argjson data "$env_json" \
            '.environments[$env] = $data' "$json_file" > "$json_file.tmp" && mv "$json_file.tmp" "$json_file"
    done

    # サマリー追加
    cat >> "$report_file" <<EOF

---

## Summary

- **Environments Passed**: $total_passed
- **Environments Failed**: $total_failed
- **Environments Timeout**: $total_timeout

EOF

    # JSONサマリー追加
    jq --argjson passed "$total_passed" \
       --argjson failed "$total_failed" \
       --argjson timeout "$total_timeout" \
       '.summary = {passed: $passed, failed: $failed, timeout: $timeout}' \
       "$json_file" > "$json_file.tmp" && mv "$json_file.tmp" "$json_file"

    log INFO "Summary report generated:"
    log INFO "  - Markdown: $report_file"
    log INFO "  - JSON: $json_file"
}

# メイン処理
main() {
    log INFO "Docker Test Runner - Starting"

    # 引数パース
    parse_arguments "$@"

    # Docker確認
    check_docker || exit 1

    # 出力ディレクトリ作成
    mkdir -p "$OUTPUT_DIR"

    # 環境リストをパース
    local env_array
    IFS=',' read -ra env_array <<< "$ENVIRONMENTS"

    log INFO "Target environments: ${env_array[*]}"

    # 各環境でテスト実行
    local overall_exit_code=0

    for env in "${env_array[@]}"; do
        log INFO "=== Processing environment: $env ==="

        # イメージビルド
        local image_name
        image_name=$(build_docker_image "$env") || {
            log ERROR "Failed to build image for $env, skipping..."
            overall_exit_code=1
            continue
        }

        # テスト実行
        run_tests_in_docker "$env" "$image_name" "$CLI_BINARY" "$TEST_DIR" "$OUTPUT_DIR" "$TIMEOUT" || {
            local exit_code=$?
            if [[ $exit_code -ne 2 ]]; then  # 2 = timeout
                overall_exit_code=1
            fi
        }

        log INFO "Completed: $env"
        echo ""
    done

    # 統合レポート生成
    generate_summary_report "$OUTPUT_DIR" "$ENVIRONMENTS"

    log INFO "=== Docker Test Runner - Completed ==="

    exit $overall_exit_code
}

# メイン実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
