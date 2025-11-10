#!/usr/bin/env bash
#
# run-tests.sh - BATSテスト実行エンジン
# CLI Testing Specialist Agent v1.1.0
#
# 機能:
# - 生成されたBATSテストの実行
# - テスト結果の収集と集計
# - レポート生成（Markdown/JSON/HTML/JUnit XML）
# - 環境変数管理
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in run-tests.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# BATSの存在確認
check_bats_installation() {
    log DEBUG "Checking BATS installation"

    if ! command -v bats &>/dev/null; then
        log ERROR "BATS is not installed"
        log ERROR "  Install: brew install bats-core  (macOS)"
        log ERROR "          apt-get install bats     (Ubuntu/Debian)"
        log ERROR "          npm install -g bats      (npm)"
        return 1
    fi

    local bats_version
    bats_version=$(bats --version | head -1) || bats_version="unknown"
    log INFO "BATS detected: $bats_version"

    return 0
}

# テストディレクトリの検証
validate_test_directory() {
    local test_dir="$1"

    log DEBUG "Validating test directory: $test_dir"

    if [[ ! -d "$test_dir" ]]; then
        log ERROR "Test directory not found: $test_dir"
        return 1
    fi

    # .batsファイルの存在確認
    local bats_files
    bats_files=$(find "$test_dir" -name "*.bats" -type f 2>/dev/null | wc -l)

    if [[ $bats_files -eq 0 ]]; then
        log ERROR "No .bats files found in: $test_dir"
        return 1
    fi

    log INFO "Found $bats_files BATS test files"
    return 0
}

# BATSテスト実行
run_bats_tests() {
    local test_dir="$1"
    local output_format="${2:-tap}"  # tap, pretty, json, junit
    local output_file="${3:-}"

    log INFO "Running BATS tests"
    log DEBUG "Test directory: $test_dir"
    log DEBUG "Output format: $output_format"

    # BATSコマンド構築
    local bats_cmd="bats"
    local bats_args=()

    case "$output_format" in
        tap)
            # TAP形式（デフォルト）
            ;;
        pretty)
            bats_args+=("--pretty")
            ;;
        json)
            bats_args+=("--formatter" "json")
            ;;
        junit)
            bats_args+=("--formatter" "junit")
            ;;
        *)
            log WARN "Unknown format: $output_format, using TAP"
            ;;
    esac

    # タイムアウト設定
    bats_args+=("--timing")

    # 再帰的にすべての.batsファイルを実行
    local test_files
    mapfile -t test_files < <(find "$test_dir" -name "*.bats" -type f | sort)

    log INFO "Executing ${#test_files[@]} test files"

    local exit_code=0
    local test_results

    if [[ -n "$output_file" ]]; then
        # ファイル出力
        test_results=$("$bats_cmd" "${bats_args[@]}" "${test_files[@]}" 2>&1 | tee "$output_file") || exit_code=$?
    else
        # 標準出力
        test_results=$("$bats_cmd" "${bats_args[@]}" "${test_files[@]}" 2>&1) || exit_code=$?
    fi

    log DEBUG "BATS exit code: $exit_code"

    # 結果をエコー（パイプチェーンでキャプチャするため）
    echo "$test_results"

    return $exit_code
}

# テスト結果の集計
aggregate_test_results() {
    local tap_output="$1"

    log DEBUG "Aggregating test results from TAP output"

    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    local skipped_tests=0

    # TAP形式をパース
    while IFS= read -r line; do
        if [[ "$line" =~ ^1\.\.([0-9]+) ]]; then
            # テスト総数
            total_tests="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^ok\ [0-9]+ ]]; then
            # パス
            if [[ "$line" =~ \#\ skip ]]; then
                ((skipped_tests++))
            else
                ((passed_tests++))
            fi
        elif [[ "$line" =~ ^not\ ok\ [0-9]+ ]]; then
            # 失敗
            ((failed_tests++))
        fi
    done <<< "$tap_output"

    # 成功率計算（整数演算回避）
    local success_rate="0.00"
    if [[ $total_tests -gt 0 ]]; then
        success_rate=$(awk -v p="$passed_tests" -v t="$total_tests" 'BEGIN {printf "%.2f", (p / t) * 100}')
    fi

    log DEBUG "Aggregation results: total=$total_tests, passed=$passed_tests, failed=$failed_tests, skipped=$skipped_tests"

    # JSON形式で集計結果を返す
    jq -n \
        --arg total "$total_tests" \
        --arg passed "$passed_tests" \
        --arg failed "$failed_tests" \
        --arg skipped "$skipped_tests" \
        --arg success_rate "$success_rate" \
        '{
            total: ($total | tonumber),
            passed: ($passed | tonumber),
            failed: ($failed | tonumber),
            skipped: ($skipped | tonumber),
            success_rate: ($success_rate | tonumber)
        }'
}

# Markdownレポート生成
generate_markdown_report() {
    local test_results="$1"
    local summary_json="$2"
    local output_file="$3"

    log INFO "Generating Markdown report: $output_file"

    local total passed failed skipped success_rate
    total=$(echo "$summary_json" | jq -r '.total')
    passed=$(echo "$summary_json" | jq -r '.passed')
    failed=$(echo "$summary_json" | jq -r '.failed')
    skipped=$(echo "$summary_json" | jq -r '.skipped')
    success_rate=$(echo "$summary_json" | jq -r '.success_rate')

    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    cat > "$output_file" <<EOF
# CLI Testing Report

**Generated:** $timestamp
**Agent Version:** $(cat "$AGENT_ROOT/VERSION" 2>/dev/null || echo "1.1.0-dev")

## Test Summary

| Metric | Value |
|--------|-------|
| **Total Tests** | $total |
| **Passed** | ✅ $passed |
| **Failed** | ❌ $failed |
| **Skipped** | ⏭️  $skipped |
| **Success Rate** | ${success_rate}% |

## Test Results

\`\`\`tap
$test_results
\`\`\`

## Status

EOF

    if [[ $failed -eq 0 ]]; then
        echo "✅ **All tests passed successfully!**" >> "$output_file"
    else
        echo "❌ **$failed test(s) failed. Please review the results above.**" >> "$output_file"
    fi

    log INFO "Markdown report generated: $output_file"
}

# JSONレポート生成
generate_json_report() {
    local test_results="$1"
    local summary_json="$2"
    local output_file="$3"

    log INFO "Generating JSON report: $output_file"

    local timestamp
    timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    local agent_version
    agent_version=$(cat "$AGENT_ROOT/VERSION" 2>/dev/null || echo "1.1.0-dev")

    # 失敗したテストの詳細を抽出
    local failed_tests_json="[]"
    local current_test=""
    local test_index=0

    while IFS= read -r line; do
        if [[ "$line" =~ ^not\ ok\ ([0-9]+)\ (.+) ]]; then
            local test_num="${BASH_REMATCH[1]}"
            local test_name="${BASH_REMATCH[2]}"

            # スキップでない失敗のみ
            if [[ ! "$line" =~ \#\ skip ]]; then
                failed_tests_json=$(echo "$failed_tests_json" | jq \
                    --arg num "$test_num" \
                    --arg name "$test_name" \
                    '. += [{test_number: ($num | tonumber), test_name: $name}]')
            fi
        fi
    done <<< "$test_results"

    jq -n \
        --argjson summary "$summary_json" \
        --argjson failed_tests "$failed_tests_json" \
        --arg timestamp "$timestamp" \
        --arg version "$agent_version" \
        '{
            report_type: "cli_testing_results",
            generated_at: $timestamp,
            agent_version: $version,
            summary: $summary,
            failed_tests: $failed_tests
        }' > "$output_file"

    log INFO "JSON report generated: $output_file"
}

# HTMLレポート生成
generate_html_report() {
    local test_results="$1"
    local summary_json="$2"
    local output_file="$3"

    log INFO "Generating HTML report: $output_file"

    # HTML生成スクリプトを呼び出し
    if [[ -x "$SCRIPT_DIR/report-generator-html.sh" ]]; then
        # 一時ファイルに結果を保存
        local tmp_tap_file="/tmp/tap-results-$$.tap"
        local tmp_json_file="/tmp/summary-$$.json"

        echo "$test_results" > "$tmp_tap_file"
        echo "$summary_json" > "$tmp_json_file"

        # HTML生成実行
        "$SCRIPT_DIR/report-generator-html.sh" "$tmp_tap_file" "$tmp_json_file" "$output_file"

        # 一時ファイル削除
        rm -f "$tmp_tap_file" "$tmp_json_file"

        log INFO "HTML report generated: $output_file"
    else
        log ERROR "HTML report generator not found: $SCRIPT_DIR/report-generator-html.sh"
        return 1
    fi
}

# JUnit XMLレポート生成
generate_junit_report() {
    local test_results="$1"
    local summary_json="$2"
    local output_file="$3"

    log INFO "Generating JUnit XML report: $output_file"

    local total passed failed skipped
    total=$(echo "$summary_json" | jq -r '.total')
    passed=$(echo "$summary_json" | jq -r '.passed')
    failed=$(echo "$summary_json" | jq -r '.failed')
    skipped=$(echo "$summary_json" | jq -r '.skipped')

    local timestamp
    timestamp=$(date -u '+%Y-%m-%dT%H:%M:%S')

    # 開始時刻を記録（概算）
    local start_time
    start_time=$(date -u '+%s')

    # XML生成開始
    {
        echo '<?xml version="1.0" encoding="UTF-8"?>'
        echo '<testsuites>'

        # テストスイート開始（全体を1つのスイートとして扱う）
        local total_time="0"
        echo "  <testsuite name=\"cli-tests\" tests=\"$total\" failures=\"$failed\" skipped=\"$skipped\" time=\"$total_time\" timestamp=\"$timestamp\">"

        # TAP出力をパースしてテストケースを生成
        local current_file=""
        local test_number=0
        local in_failure=false
        local failure_message=""
        local test_name=""
        local test_time="0"

        while IFS= read -r line; do
            # ファイル名の抽出（BATSのTAP出力から）
            if [[ "$line" =~ ^#\ (.+\.bats) ]]; then
                current_file="${BASH_REMATCH[1]}"
                current_file=$(basename "$current_file" .bats)
            fi

            # テスト成功
            if [[ "$line" =~ ^ok\ ([0-9]+)\ (.+) ]]; then
                test_number="${BASH_REMATCH[1]}"
                test_name="${BASH_REMATCH[2]}"

                # タイミング情報の抽出（あれば）
                if [[ "$line" =~ in\ ([0-9.]+)s ]]; then
                    test_time="${BASH_REMATCH[1]}"
                else
                    test_time="0"
                fi

                # スキップチェック
                if [[ "$line" =~ \#\ skip ]]; then
                    echo "    <testcase classname=\"${current_file:-unknown}\" name=\"$test_name\" time=\"$test_time\">"
                    echo "      <skipped/>"
                    echo "    </testcase>"
                else
                    echo "    <testcase classname=\"${current_file:-unknown}\" name=\"$test_name\" time=\"$test_time\"/>"
                fi
            fi

            # テスト失敗
            if [[ "$line" =~ ^not\ ok\ ([0-9]+)\ (.+) ]]; then
                test_number="${BASH_REMATCH[1]}"
                test_name="${BASH_REMATCH[2]}"
                in_failure=true
                failure_message=""

                # タイミング情報の抽出（あれば）
                if [[ "$line" =~ in\ ([0-9.]+)s ]]; then
                    test_time="${BASH_REMATCH[1]}"
                else
                    test_time="0"
                fi
            fi

            # 失敗メッセージの収集
            if [[ $in_failure == true ]]; then
                if [[ "$line" =~ ^#\ (.+) ]]; then
                    local msg="${BASH_REMATCH[1]}"
                    # ファイル名以外のコメントを失敗メッセージとして収集
                    if [[ ! "$msg" =~ \.bats$ ]]; then
                        failure_message+="$msg"$'\n'
                    fi
                elif [[ "$line" =~ ^(ok|not\ ok)\ [0-9]+ ]] || [[ "$line" =~ ^1\.\.[0-9]+ ]]; then
                    # 次のテストケースまたは終了時に失敗情報を出力
                    if [[ -n "$test_name" ]]; then
                        echo "    <testcase classname=\"${current_file:-unknown}\" name=\"$test_name\" time=\"$test_time\">"
                        echo "      <failure message=\"Test failed\">"
                        # XMLエスケープ
                        echo "$failure_message" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g'
                        echo "      </failure>"
                        echo "    </testcase>"
                    fi
                    in_failure=false
                    test_name=""
                    failure_message=""
                fi
            fi
        done <<< "$test_results"

        # 最後の失敗が未出力の場合
        if [[ $in_failure == true ]] && [[ -n "$test_name" ]]; then
            echo "    <testcase classname=\"${current_file:-unknown}\" name=\"$test_name\" time=\"$test_time\">"
            echo "      <failure message=\"Test failed\">"
            echo "$failure_message" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g'
            echo "      </failure>"
            echo "    </testcase>"
        fi

        # テストスイート終了
        echo "  </testsuite>"
        echo '</testsuites>'
    } > "$output_file"

    log INFO "JUnit XML report generated: $output_file"
}

# メイン実行関数
run_tests_main() {
    local test_dir="$1"
    local report_format="${2:-markdown}"  # markdown, json, html, junit, all
    local output_dir="${3:-.}"

    log INFO "Starting test execution"
    log DEBUG "Test directory: $test_dir"
    log DEBUG "Report format: $report_format"
    log DEBUG "Output directory: $output_dir"

    # BATS確認
    check_bats_installation || return 1

    # テストディレクトリ検証
    validate_test_directory "$test_dir" || return 1

    # 出力ディレクトリ検証
    local validated_output_dir
    validated_output_dir=$(validate_output_dir "$output_dir") || return 1

    # BATSテスト実行
    local test_results
    local exit_code=0
    test_results=$(run_bats_tests "$test_dir" "tap") || exit_code=$?

    # 結果集計
    local summary_json
    summary_json=$(aggregate_test_results "$test_results")

    log INFO "Test execution completed (exit code: $exit_code)"
    log INFO "Summary: $(echo "$summary_json" | jq -c '.')"

    # レポート生成
    case "$report_format" in
        markdown)
            generate_markdown_report "$test_results" "$summary_json" "$validated_output_dir/test-report.md"
            ;;
        json)
            generate_json_report "$test_results" "$summary_json" "$validated_output_dir/test-report.json"
            ;;
        html)
            generate_html_report "$test_results" "$summary_json" "$validated_output_dir/test-report.html"
            ;;
        junit)
            generate_junit_report "$test_results" "$summary_json" "$validated_output_dir/test-report.xml"
            ;;
        all)
            generate_markdown_report "$test_results" "$summary_json" "$validated_output_dir/test-report.md"
            generate_json_report "$test_results" "$summary_json" "$validated_output_dir/test-report.json"
            generate_html_report "$test_results" "$summary_json" "$validated_output_dir/test-report.html"
            generate_junit_report "$test_results" "$summary_json" "$validated_output_dir/test-report.xml"
            ;;
        *)
            log ERROR "Unknown report format: $report_format"
            log ERROR "Valid formats: markdown, json, html, junit, all"
            return 1
            ;;
    esac

    log INFO "Test execution and reporting completed"
    log INFO "  Exit code: $exit_code"
    log INFO "  Reports: $validated_output_dir"

    return $exit_code
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <test-dir> [report-format] [output-dir]" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <test-dir>        Directory containing .bats test files" >&2
        echo "  [report-format]   markdown|json|html|junit|all (default: markdown)" >&2
        echo "  [output-dir]      Output directory for reports (default: .)" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 ./generated-tests" >&2
        echo "  $0 ./generated-tests html ./reports" >&2
        echo "  $0 ./generated-tests junit ./reports" >&2
        echo "  $0 ./generated-tests all ./reports" >&2
        exit 1
    fi

    # テスト実行
    run_tests_main "$@"
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        log INFO "All tests completed successfully"
    else
        log WARN "Some tests failed (exit code: $exit_code)"
    fi

    exit $exit_code
fi
