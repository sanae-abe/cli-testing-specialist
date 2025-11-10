#!/usr/bin/env bash
#
# coverage-analyzer.sh - カバレッジ分析エンジン
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - analysis.json（全機能定義）とcoverage.db（使用履歴）を突合
# - カバレッジ率計算
# - 未カバー領域特定
# - 改善提案生成
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in coverage-analyzer.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# デフォルト設定
DEFAULT_COVERAGE_DB="${AGENT_ROOT}/coverage.db"
COVERAGE_DB_PATH="${COVERAGE_DB_PATH:-$DEFAULT_COVERAGE_DB}"

# カバレッジ分析メイン関数
analyze_coverage() {
    local analysis_json="$1"
    local coverage_db="$2"
    local output_json="$3"

    log INFO "Starting coverage analysis"
    log DEBUG "Analysis JSON: $analysis_json"
    log DEBUG "Coverage DB: $coverage_db"
    log DEBUG "Output JSON: $output_json"

    # 入力検証
    if [[ ! -f "$analysis_json" ]]; then
        log ERROR "Analysis JSON not found: $analysis_json"
        return 1
    fi

    if [[ ! -f "$coverage_db" ]]; then
        log ERROR "Coverage database not found: $coverage_db"
        return 1
    fi

    # JSONバリデーション
    if ! jq empty "$analysis_json" 2>/dev/null; then
        log ERROR "Invalid JSON format: $analysis_json"
        return 1
    fi

    # analysis.jsonから全機能を読み込み
    local cli_binary
    cli_binary=$(jq -r '.binary' "$analysis_json")

    local all_subcommands
    all_subcommands=$(jq -r '.subcommands[]? // empty' "$analysis_json" 2>/dev/null || echo "")

    local all_options
    all_options=$(jq -r '.options[]? // empty' "$analysis_json" 2>/dev/null || echo "")

    log INFO "CLI Binary: $cli_binary"
    log DEBUG "All subcommands: $(echo "$all_subcommands" | wc -l | tr -d ' ') items"
    log DEBUG "All options: $(echo "$all_options" | wc -l | tr -d ' ') items"

    # coverage.dbから使用履歴を読み込み
    local used_subcommands
    used_subcommands=$(sqlite3 "$coverage_db" \
        "SELECT DISTINCT subcommand FROM command_usage WHERE subcommand IS NOT NULL AND subcommand != '';" 2>/dev/null || echo "")

    local used_options
    used_options=$(sqlite3 "$coverage_db" \
        "SELECT DISTINCT option_name FROM option_usage WHERE option_name != '';" 2>/dev/null || echo "")

    log DEBUG "Used subcommands: $(echo "$used_subcommands" | wc -l | tr -d ' ') items"
    log DEBUG "Used options: $(echo "$used_options" | wc -l | tr -d ' ') items"

    # カバレッジ計算
    local coverage_result
    coverage_result=$(calculate_coverage \
        "$all_subcommands" "$used_subcommands" \
        "$all_options" "$used_options" \
        "$cli_binary")

    # 未カバー領域特定
    local uncovered
    uncovered=$(identify_uncovered \
        "$all_subcommands" "$used_subcommands" \
        "$all_options" "$used_options")

    # カバレッジマトリクス生成
    local coverage_matrix
    coverage_matrix=$(generate_coverage_matrix \
        "$coverage_db" "$all_subcommands" "$all_options")

    # 改善提案生成
    local recommendations
    recommendations=$(generate_recommendations "$uncovered")

    # 結果を統合してJSON出力
    local final_result
    final_result=$(jq -n \
        --argjson summary "$coverage_result" \
        --argjson uncovered "$uncovered" \
        --argjson matrix "$coverage_matrix" \
        --argjson recommendations "$recommendations" \
        '{
            generated_at: $summary.generated_at,
            cli_binary: $summary.cli_binary,
            total_features: $summary.total_features,
            covered_features: $summary.covered_features,
            coverage_rate: $summary.coverage_rate,
            summary: $summary.summary,
            uncovered: $uncovered,
            coverage_matrix: $matrix,
            recommendations: $recommendations
        }')

    echo "$final_result" > "$output_json"

    log INFO "Coverage analysis completed: $output_json"
    log INFO "Overall coverage rate: $(echo "$final_result" | jq -r '.coverage_rate')%"

    return 0
}

# カバレッジ率計算
calculate_coverage() {
    local all_subcommands="$1"
    local used_subcommands="$2"
    local all_options="$3"
    local used_options="$4"
    local cli_binary="$5"

    log DEBUG "Calculating coverage rates"

    # 総数（空行除外）
    local total_subcommands
    total_subcommands=$(echo "$all_subcommands" | grep -v '^$' | wc -l | tr -d ' ')

    local total_options
    total_options=$(echo "$all_options" | grep -v '^$' | wc -l | tr -d ' ')

    # カバー数
    local covered_subcommands=0
    if [[ -n "$all_subcommands" ]]; then
        while IFS= read -r subcmd; do
            [[ -z "$subcmd" ]] && continue
            if echo "$used_subcommands" | grep -qxF -- "$subcmd"; then
                ((covered_subcommands++))
            fi
        done <<< "$all_subcommands"
    fi

    local covered_options=0
    if [[ -n "$all_options" ]]; then
        while IFS= read -r opt; do
            [[ -z "$opt" ]] && continue
            if echo "$used_options" | grep -qxF -- "$opt"; then
                ((covered_options++))
            fi
        done <<< "$all_options"
    fi

    # カバレッジ率計算（小数点2桁）
    local subcommand_rate=0
    if [[ $total_subcommands -gt 0 ]]; then
        subcommand_rate=$(awk -v c="$covered_subcommands" -v t="$total_subcommands" \
            'BEGIN {printf "%.2f", (c / t) * 100}')
    fi

    local option_rate=0
    if [[ $total_options -gt 0 ]]; then
        option_rate=$(awk -v c="$covered_options" -v t="$total_options" \
            'BEGIN {printf "%.2f", (c / t) * 100}')
    fi

    # 全体カバレッジ率（サブコマンド+オプションの合計）
    local total_features=$((total_subcommands + total_options))
    local covered_features=$((covered_subcommands + covered_options))
    local overall_rate=0

    if [[ $total_features -gt 0 ]]; then
        overall_rate=$(awk -v c="$covered_features" -v t="$total_features" \
            'BEGIN {printf "%.2f", (c / t) * 100}')
    fi

    # JSON構築
    jq -n \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --arg cli_binary "$cli_binary" \
        --argjson total_features "$total_features" \
        --argjson covered_features "$covered_features" \
        --arg overall_rate "$overall_rate" \
        --argjson total_sub "$total_subcommands" \
        --argjson covered_sub "$covered_subcommands" \
        --arg sub_rate "$subcommand_rate" \
        --argjson total_opt "$total_options" \
        --argjson covered_opt "$covered_options" \
        --arg opt_rate "$option_rate" \
        '{
            generated_at: $timestamp,
            cli_binary: $cli_binary,
            total_features: $total_features,
            covered_features: $covered_features,
            coverage_rate: ($overall_rate | tonumber),
            summary: {
                subcommands: {
                    total: $total_sub,
                    covered: $covered_sub,
                    coverage_rate: ($sub_rate | tonumber)
                },
                options: {
                    total: $total_opt,
                    covered: $covered_opt,
                    coverage_rate: ($opt_rate | tonumber)
                }
            }
        }'
}

# 未カバー領域特定
identify_uncovered() {
    local all_subcommands="$1"
    local used_subcommands="$2"
    local all_options="$3"
    local used_options="$4"

    log DEBUG "Identifying uncovered areas"

    # 未カバーサブコマンド
    local uncovered_subcommands="[]"
    if [[ -n "$all_subcommands" ]]; then
        local uncovered_sub_list=""
        while IFS= read -r subcmd; do
            [[ -z "$subcmd" ]] && continue
            if ! echo "$used_subcommands" | grep -qxF -- "$subcmd"; then
                uncovered_sub_list+="$subcmd"$'\n'
            fi
        done <<< "$all_subcommands"

        # JSON配列化
        if [[ -n "$uncovered_sub_list" ]]; then
            uncovered_subcommands=$(echo "$uncovered_sub_list" | grep -v '^$' | jq -R -s -c 'split("\n") | map(select(length > 0))')
        fi
    fi

    # 未カバーオプション
    local uncovered_options="[]"
    if [[ -n "$all_options" ]]; then
        local uncovered_opt_list=""
        while IFS= read -r opt; do
            [[ -z "$opt" ]] && continue
            if ! echo "$used_options" | grep -qxF -- "$opt"; then
                uncovered_opt_list+="$opt"$'\n'
            fi
        done <<< "$all_options"

        # JSON配列化
        if [[ -n "$uncovered_opt_list" ]]; then
            uncovered_options=$(echo "$uncovered_opt_list" | grep -v '^$' | jq -R -s -c 'split("\n") | map(select(length > 0))')
        fi
    fi

    # JSON構築
    jq -n \
        --argjson subcommands "$uncovered_subcommands" \
        --argjson options "$uncovered_options" \
        '{
            subcommands: $subcommands,
            options: $options
        }'
}

# カバレッジマトリクス生成
generate_coverage_matrix() {
    local coverage_db="$1"
    local all_subcommands="$2"
    local all_options="$3"

    log DEBUG "Generating coverage matrix"

    local matrix="{}"

    # サブコマンドごとの詳細を構築
    if [[ -n "$all_subcommands" ]]; then
        while IFS= read -r subcmd; do
            [[ -z "$subcmd" ]] && continue

            # 使用回数
            local usage_count
            usage_count=$(sqlite3 "$coverage_db" \
                "SELECT COUNT(*) FROM command_usage WHERE subcommand = '$subcmd';" 2>/dev/null || echo "0")

            # カバー済み判定
            local covered="false"
            [[ $usage_count -gt 0 ]] && covered="true"

            # このサブコマンドで使用されたオプション
            local options_covered="[]"
            if [[ $usage_count -gt 0 ]]; then
                local used_opts
                used_opts=$(sqlite3 "$coverage_db" \
                    "SELECT DISTINCT o.option_name FROM option_usage o
                     JOIN command_usage c ON o.command_id = c.id
                     WHERE c.subcommand = '$subcmd';" 2>/dev/null || echo "")

                if [[ -n "$used_opts" ]]; then
                    options_covered=$(echo "$used_opts" | jq -R -s -c 'split("\n") | map(select(length > 0))')
                fi
            fi

            # 未使用オプション（全オプションから使用済みを除外）
            local options_uncovered="[]"
            if [[ -n "$all_options" ]]; then
                local uncovered_opts=""
                while IFS= read -r opt; do
                    [[ -z "$opt" ]] && continue
                    if ! echo "$options_covered" | jq -e --arg opt "$opt" 'index($opt)' >/dev/null 2>&1; then
                        uncovered_opts+="$opt"$'\n'
                    fi
                done <<< "$all_options"

                if [[ -n "$uncovered_opts" ]]; then
                    options_uncovered=$(echo "$uncovered_opts" | grep -v '^$' | jq -R -s -c 'split("\n") | map(select(length > 0))')
                fi
            fi

            # マトリクスに追加
            matrix=$(echo "$matrix" | jq \
                --arg subcmd "$subcmd" \
                --argjson covered "$covered" \
                --argjson usage "$usage_count" \
                --argjson opts_covered "$options_covered" \
                --argjson opts_uncovered "$options_uncovered" \
                '. + {($subcmd): {
                    covered: $covered,
                    usage_count: $usage,
                    options_covered: $opts_covered,
                    options_uncovered: $opts_uncovered
                }}')
        done <<< "$all_subcommands"
    fi

    echo "$matrix"
}

# 改善提案生成
generate_recommendations() {
    local uncovered="$1"

    log DEBUG "Generating recommendations"

    local recommendations="[]"

    # 未カバーサブコマンドの提案
    local uncovered_subcommands
    uncovered_subcommands=$(echo "$uncovered" | jq -r '.subcommands[]?' 2>/dev/null || echo "")

    if [[ -n "$uncovered_subcommands" ]]; then
        while IFS= read -r subcmd; do
            [[ -z "$subcmd" ]] && continue

            local recommendation
            recommendation=$(jq -n \
                --arg type "uncovered_subcommand" \
                --arg target "$subcmd" \
                --arg priority "high" \
                --arg suggestion "Add test case for '$subcmd' to improve coverage" \
                '{
                    type: $type,
                    target: $target,
                    priority: $priority,
                    suggestion: $suggestion
                }')

            recommendations=$(echo "$recommendations" | jq --argjson rec "$recommendation" '. + [$rec]')
        done <<< "$uncovered_subcommands"
    fi

    # 未カバーオプションの提案
    local uncovered_options
    uncovered_options=$(echo "$uncovered" | jq -r '.options[]?' 2>/dev/null || echo "")

    if [[ -n "$uncovered_options" ]]; then
        local option_count=0
        while IFS= read -r opt; do
            [[ -z "$opt" ]] && continue
            ((option_count++))

            # 最初の5個のみ高優先度、それ以降は中優先度
            local priority="medium"
            [[ $option_count -le 5 ]] && priority="high"

            local recommendation
            recommendation=$(jq -n \
                --arg type "uncovered_option" \
                --arg target "$opt" \
                --arg priority "$priority" \
                --arg suggestion "Test '$opt' option with various edge cases" \
                '{
                    type: $type,
                    target: $target,
                    priority: $priority,
                    suggestion: $suggestion
                }')

            recommendations=$(echo "$recommendations" | jq --argjson rec "$recommendation" '. + [$rec]')
        done <<< "$uncovered_options"
    fi

    echo "$recommendations"
}

# カバレッジサマリー表示
show_coverage_summary() {
    local coverage_json="$1"

    log INFO "Coverage Summary:"
    log INFO "  Overall: $(jq -r '.coverage_rate' "$coverage_json")%"
    log INFO "  Subcommands: $(jq -r '.summary.subcommands.coverage_rate' "$coverage_json")% ($(jq -r '.summary.subcommands.covered' "$coverage_json")/$(jq -r '.summary.subcommands.total' "$coverage_json"))"
    log INFO "  Options: $(jq -r '.summary.options.coverage_rate' "$coverage_json")% ($(jq -r '.summary.options.covered' "$coverage_json")/$(jq -r '.summary.options.total' "$coverage_json"))"

    local uncovered_sub_count
    uncovered_sub_count=$(jq -r '.uncovered.subcommands | length' "$coverage_json")

    local uncovered_opt_count
    uncovered_opt_count=$(jq -r '.uncovered.options | length' "$coverage_json")

    if [[ $uncovered_sub_count -gt 0 ]] || [[ $uncovered_opt_count -gt 0 ]]; then
        log INFO "  Uncovered: $uncovered_sub_count subcommands, $uncovered_opt_count options"
    fi

    local rec_count
    rec_count=$(jq -r '.recommendations | length' "$coverage_json")

    if [[ $rec_count -gt 0 ]]; then
        log INFO "  Recommendations: $rec_count items"
    fi
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 3 ]]; then
        echo "Usage: $0 <analysis-json> <coverage-db> <output-json>" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <analysis-json>  CLI analysis JSON file (from cli-analyzer.sh)" >&2
        echo "  <coverage-db>    Coverage database (from coverage-tracker.sh)" >&2
        echo "  <output-json>    Output coverage result JSON file" >&2
        echo "" >&2
        echo "Example:" >&2
        echo "  $0 ./test-output/analysis.json ./coverage.db ./coverage-result.json" >&2
        exit 1
    fi

    # 分析実行
    analyze_coverage "$@"
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        # サマリー表示
        show_coverage_summary "$3"
        log INFO "Coverage analysis completed successfully"
    else
        log ERROR "Coverage analysis failed (exit code: $exit_code)"
    fi

    exit $exit_code
fi
