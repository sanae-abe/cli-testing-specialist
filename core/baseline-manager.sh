#!/usr/bin/env bash
#
# baseline-manager.sh - パフォーマンスベースライン管理
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - ベースライン保存・読み込み
# - パフォーマンス比較
# - 回帰検出
# - 統計計算
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in baseline-manager.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# デフォルト設定
DEFAULT_BASELINE_DB="$AGENT_ROOT/baselines.json"
BASELINE_DB_PATH="${BASELINE_DB_PATH:-$DEFAULT_BASELINE_DB}"

# 回帰検出閾値（パーセント）
REGRESSION_THRESHOLD="${REGRESSION_THRESHOLD:-20}"  # +20%で回帰
IMPROVEMENT_THRESHOLD="${IMPROVEMENT_THRESHOLD:-10}"  # -10%で改善

# ベースラインデータベース初期化
initialize_baseline_db() {
    local db_path="${1:-$BASELINE_DB_PATH}"

    log INFO "Initializing baseline database: $db_path"

    # ディレクトリ作成
    local db_dir
    db_dir=$(dirname "$db_path")
    mkdir -p "$db_dir"

    # 空のJSONファイル作成
    if [[ ! -f "$db_path" ]]; then
        echo '{"baselines": {}, "metadata": {"version": "2.1.0", "created_at": ""}}' | \
            jq --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
               '.metadata.created_at = $timestamp' > "$db_path"
        log INFO "Baseline database initialized: $db_path"
    else
        log INFO "Baseline database already exists: $db_path"
    fi

    return 0
}

# ベースライン保存
save_baseline() {
    local command="$1"
    local profile_json="$2"
    local db_path="${3:-$BASELINE_DB_PATH}"

    log INFO "Saving baseline for command: $command"
    log DEBUG "Profile JSON: $profile_json"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        initialize_baseline_db "$db_path"
    fi

    # プロファイルデータ読み込み
    if [[ -f "$profile_json" ]]; then
        local profile_data
        profile_data=$(<"$profile_json")
    else
        # JSONデータとして直接渡された場合
        local profile_data="$profile_json"
    fi

    # メトリクス抽出
    local metrics
    metrics=$(echo "$profile_data" | jq '.metrics')

    # ベースラインデータ構築
    local baseline_data
    baseline_data=$(jq -n \
        --arg command "$command" \
        --argjson metrics "$metrics" \
        --arg created_at "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --arg updated_at "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        '{
            command: $command,
            baseline_data: $metrics,
            statistics: {
                samples: 1
            },
            created_at: $created_at,
            updated_at: $updated_at
        }')

    # データベースに保存
    local command_key
    command_key=$(echo "$command" | tr ' /' '_-' | tr -d '.')

    local updated_db
    updated_db=$(jq --arg key "$command_key" --argjson baseline "$baseline_data" \
        '.baselines[$key] = $baseline' "$db_path")

    echo "$updated_db" > "$db_path"

    log INFO "Baseline saved: $command_key"
    return 0
}

# ベースライン更新（統計追加）
update_baseline() {
    local command="$1"
    local profile_json="$2"
    local db_path="${3:-$BASELINE_DB_PATH}"

    log INFO "Updating baseline statistics for command: $command"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        log ERROR "Baseline database not found: $db_path"
        return 1
    fi

    local command_key
    command_key=$(echo "$command" | tr ' /' '_-' | tr -d '.')

    # 既存ベースライン取得
    local existing_baseline
    existing_baseline=$(jq -r --arg key "$command_key" '.baselines[$key]' "$db_path")

    if [[ "$existing_baseline" == "null" ]]; then
        log WARN "No existing baseline found, creating new one"
        save_baseline "$command" "$profile_json" "$db_path"
        return 0
    fi

    # 新しいプロファイルデータ読み込み
    local new_metrics
    if [[ -f "$profile_json" ]]; then
        new_metrics=$(jq '.metrics' "$profile_json")
    else
        new_metrics=$(echo "$profile_json" | jq '.metrics')
    fi

    # サンプル数インクリメント
    local samples
    samples=$(echo "$existing_baseline" | jq '.statistics.samples')
    samples=$((samples + 1))

    # 統計更新（簡易版: 移動平均）
    local avg_wall_time
    avg_wall_time=$(echo "$existing_baseline" | jq -r '.baseline_data.wall_time_ms // 0')
    local new_wall_time
    new_wall_time=$(echo "$new_metrics" | jq -r '.wall_time_ms')

    local updated_avg_wall_time
    updated_avg_wall_time=$(awk -v old="$avg_wall_time" -v new="$new_wall_time" -v n="$samples" \
        'BEGIN {printf "%.0f", (old * (n - 1) + new) / n}')

    # ベースラインデータ更新
    local updated_baseline
    updated_baseline=$(echo "$existing_baseline" | jq \
        --arg updated_at "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --argjson samples "$samples" \
        --argjson avg_wall_time "$updated_avg_wall_time" \
        '.statistics.samples = $samples |
         .baseline_data.wall_time_ms = $avg_wall_time |
         .updated_at = $updated_at')

    # データベース更新
    local updated_db
    updated_db=$(jq --arg key "$command_key" --argjson baseline "$updated_baseline" \
        '.baselines[$key] = $baseline' "$db_path")

    echo "$updated_db" > "$db_path"

    log INFO "Baseline updated: $command_key (samples: $samples)"
    return 0
}

# ベースライン読み込み
load_baseline() {
    local command="$1"
    local db_path="${2:-$BASELINE_DB_PATH}"

    log DEBUG "Loading baseline for command: $command"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        log ERROR "Baseline database not found: $db_path"
        return 1
    fi

    local command_key
    command_key=$(echo "$command" | tr ' /' '_-' | tr -d '.')

    # ベースライン取得
    local baseline
    baseline=$(jq -r --arg key "$command_key" '.baselines[$key]' "$db_path")

    if [[ "$baseline" == "null" ]]; then
        log WARN "No baseline found for command: $command"
        echo "{}"
        return 1
    fi

    echo "$baseline"
    return 0
}

# パフォーマンス比較
compare_with_baseline() {
    local command="$1"
    local current_profile_json="$2"
    local db_path="${3:-$BASELINE_DB_PATH}"
    local output_json="${4:-}"

    log INFO "Comparing performance with baseline"
    log DEBUG "Command: $command"

    # ベースライン読み込み
    local baseline
    baseline=$(load_baseline "$command" "$db_path") || {
        log WARN "No baseline available for comparison"
        echo '{"baseline_exists": false}'
        return 0
    }

    # 現在のメトリクス読み込み
    local current_metrics
    if [[ -f "$current_profile_json" ]]; then
        current_metrics=$(jq '.metrics' "$current_profile_json")
    else
        current_metrics=$(echo "$current_profile_json" | jq '.metrics')
    fi

    # ベースラインメトリクス
    local baseline_metrics
    baseline_metrics=$(echo "$baseline" | jq '.baseline_data')

    # メトリクス比較
    local baseline_wall_time
    baseline_wall_time=$(echo "$baseline_metrics" | jq -r '.wall_time_ms // 0')

    local current_wall_time
    current_wall_time=$(echo "$current_metrics" | jq -r '.wall_time_ms')

    # 差分計算（パーセント）
    local wall_time_diff_percent
    if [[ $baseline_wall_time -gt 0 ]]; then
        wall_time_diff_percent=$(awk -v current="$current_wall_time" -v baseline="$baseline_wall_time" \
            'BEGIN {printf "%.2f", ((current - baseline) / baseline) * 100}')
    else
        wall_time_diff_percent="0"
    fi

    # 回帰検出
    local regression_detected="false"
    local improvement_detected="false"
    local performance_status="stable"

    if (( $(echo "$wall_time_diff_percent > $REGRESSION_THRESHOLD" | bc -l) )); then
        regression_detected="true"
        performance_status="regression"
        log WARN "Performance regression detected: +${wall_time_diff_percent}%"
    elif (( $(echo "$wall_time_diff_percent < -$IMPROVEMENT_THRESHOLD" | bc -l) )); then
        improvement_detected="true"
        performance_status="improvement"
        log INFO "Performance improvement detected: ${wall_time_diff_percent}%"
    fi

    # 比較結果JSON構築
    local comparison_result
    comparison_result=$(jq -n \
        --arg command "$command" \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --argjson baseline_metrics "$baseline_metrics" \
        --argjson current_metrics "$current_metrics" \
        --arg wall_time_diff_percent "$wall_time_diff_percent" \
        --arg regression_detected "$regression_detected" \
        --arg improvement_detected "$improvement_detected" \
        --arg performance_status "$performance_status" \
        --argjson regression_threshold "$REGRESSION_THRESHOLD" \
        --argjson improvement_threshold "$IMPROVEMENT_THRESHOLD" \
        '{
            baseline_exists: true,
            command: $command,
            timestamp: $timestamp,
            baseline_metrics: $baseline_metrics,
            current_metrics: $current_metrics,
            comparison: {
                wall_time_diff_percent: ($wall_time_diff_percent | tonumber),
                regression_detected: ($regression_detected == "true"),
                improvement_detected: ($improvement_detected == "true"),
                performance_status: $performance_status
            },
            thresholds: {
                regression_threshold: $regression_threshold,
                improvement_threshold: $improvement_threshold
            }
        }')

    if [[ -n "$output_json" ]]; then
        echo "$comparison_result" > "$output_json"
        log INFO "Comparison result saved: $output_json"
    fi

    echo "$comparison_result"
    return 0
}

# ベースライン一覧
list_baselines() {
    local db_path="${1:-$BASELINE_DB_PATH}"

    log INFO "Listing all baselines"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        log ERROR "Baseline database not found: $db_path"
        return 1
    fi

    # ベースライン一覧取得
    local baselines
    baselines=$(jq -r '.baselines | to_entries[] | "\(.key): \(.value.command) (samples: \(.value.statistics.samples))"' "$db_path")

    echo "$baselines"
    return 0
}

# ベースライン削除
delete_baseline() {
    local command="$1"
    local db_path="${2:-$BASELINE_DB_PATH}"

    log INFO "Deleting baseline for command: $command"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        log ERROR "Baseline database not found: $db_path"
        return 1
    fi

    local command_key
    command_key=$(echo "$command" | tr ' /' '_-' | tr -d '.')

    # ベースライン削除
    local updated_db
    updated_db=$(jq --arg key "$command_key" 'del(.baselines[$key])' "$db_path")

    echo "$updated_db" > "$db_path"

    log INFO "Baseline deleted: $command_key"
    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <subcommand> [args...]" >&2
        echo "" >&2
        echo "Subcommands:" >&2
        echo "  init [db-path]                                    Initialize baseline database" >&2
        echo "  save <command> <profile-json> [db-path]           Save baseline" >&2
        echo "  update <command> <profile-json> [db-path]         Update baseline statistics" >&2
        echo "  load <command> [db-path]                          Load baseline" >&2
        echo "  compare <command> <profile-json> [db] [output]    Compare with baseline" >&2
        echo "  list [db-path]                                    List all baselines" >&2
        echo "  delete <command> [db-path]                        Delete baseline" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 init ./baselines.json" >&2
        echo "  $0 save '/bin/ls -la' ./profile.json" >&2
        echo "  $0 compare '/bin/ls -la' ./profile.json" >&2
        echo "  $0 list" >&2
        exit 1
    fi

    subcommand="$1"
    shift

    case "$subcommand" in
        init)
            db_path="${1:-$BASELINE_DB_PATH}"
            initialize_baseline_db "$db_path"
            ;;
        save)
            if [[ $# -lt 2 ]]; then
                log ERROR "save requires: <command> <profile-json> [db-path]"
                exit 1
            fi
            save_baseline "$@"
            ;;
        update)
            if [[ $# -lt 2 ]]; then
                log ERROR "update requires: <command> <profile-json> [db-path]"
                exit 1
            fi
            update_baseline "$@"
            ;;
        load)
            if [[ $# -lt 1 ]]; then
                log ERROR "load requires: <command> [db-path]"
                exit 1
            fi
            load_baseline "$@"
            ;;
        compare)
            if [[ $# -lt 2 ]]; then
                log ERROR "compare requires: <command> <profile-json> [db-path] [output-json]"
                exit 1
            fi
            compare_with_baseline "$@"
            ;;
        list)
            db_path="${1:-$BASELINE_DB_PATH}"
            list_baselines "$db_path"
            ;;
        delete)
            if [[ $# -lt 1 ]]; then
                log ERROR "delete requires: <command> [db-path]"
                exit 1
            fi
            delete_baseline "$@"
            ;;
        *)
            log ERROR "Unknown subcommand: $subcommand"
            exit 1
            ;;
    esac

    exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        log INFO "Operation completed successfully"
    else
        log ERROR "Operation failed (exit code: $exit_code)"
    fi

    exit $exit_code
fi
