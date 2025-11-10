#!/usr/bin/env bash
#
# profiler.sh - パフォーマンスプロファイリングエンジン
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - 実行時間測定（ミリ秒精度）
# - メモリ使用量追跡
# - CPU使用率モニタリング
# - プロファイルデータのJSON保存
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in profiler.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# デフォルト設定
DEFAULT_PROFILE_DB="$AGENT_ROOT/profiles.json"
PROFILE_DB_PATH="${PROFILE_DB_PATH:-$DEFAULT_PROFILE_DB}"

# GNU time検出（オプショナル）
detect_time_command() {
    log DEBUG "Detecting time command variant"

    # GNU time (gtime) 優先
    if command -v gtime &>/dev/null; then
        echo "gtime"
        log DEBUG "Using GNU time (gtime)"
        return 0
    fi

    # BSD time (/usr/bin/time)
    if [[ -x /usr/bin/time ]]; then
        echo "/usr/bin/time"
        log DEBUG "Using BSD time (/usr/bin/time)"
        return 0
    fi

    # Fallback: builtin time
    echo "time"
    log WARN "Using shell builtin time (limited metrics)"
    return 0
}

# コマンド実行プロファイリング
profile_execution() {
    local command="$1"
    local output_json="${2:-}"
    local test_name="${3:-unknown}"
    local test_file="${4:-unknown}"

    log INFO "Profiling command execution"
    log DEBUG "Command: $command"
    log DEBUG "Test: $test_name"

    # タイムスタンプ（開始）
    local start_time
    start_time=$(date +%s)
    local start_time_ms
    if command -v gdate &>/dev/null; then
        # GNU date (macOSの場合: brew install coreutils)
        start_time_ms=$(gdate +%s%3N)
    else
        # BSD date (秒単位)
        start_time_ms=$((start_time * 1000))
    fi

    # 一時ファイル
    local temp_dir
    temp_dir=$(mktemp -d)
    local time_output="$temp_dir/time_output.txt"
    local command_output="$temp_dir/command_output.txt"
    local command_stderr="$temp_dir/command_stderr.txt"

    # time コマンドの種類を検出
    local time_cmd
    time_cmd=$(detect_time_command)

    # プロファイリング実行
    local exit_code=0

    case "$time_cmd" in
        gtime)
            # GNU time: 詳細メトリクス取得
            gtime -v -o "$time_output" \
                bash -c "$command" \
                > "$command_output" 2> "$command_stderr" || exit_code=$?
            ;;
        /usr/bin/time)
            # BSD time: -l オプションで詳細情報
            /usr/bin/time -l -o "$time_output" \
                bash -c "$command" \
                > "$command_output" 2> "$command_stderr" || exit_code=$?
            ;;
        *)
            # Fallback: 簡易測定
            log WARN "Using fallback profiling (limited metrics)"
            bash -c "$command" > "$command_output" 2> "$command_stderr" || exit_code=$?
            echo "elapsed_time: 0" > "$time_output"
            ;;
    esac

    # タイムスタンプ（終了）
    local end_time
    end_time=$(date +%s)
    local end_time_ms
    if command -v gdate &>/dev/null; then
        end_time_ms=$(gdate +%s%3N)
    else
        end_time_ms=$((end_time * 1000))
    fi

    local elapsed_ms=$((end_time_ms - start_time_ms))

    log DEBUG "Command execution completed (exit code: $exit_code)"
    log DEBUG "Elapsed time: ${elapsed_ms}ms"

    # time 出力のパース
    local metrics
    metrics=$(parse_time_output "$time_output" "$time_cmd" "$elapsed_ms")

    # プロファイルデータ構築
    local profile_data
    profile_data=$(jq -n \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --arg command "$command" \
        --arg test_name "$test_name" \
        --arg test_file "$test_file" \
        --argjson exit_code "$exit_code" \
        --argjson metrics "$metrics" \
        '{
            timestamp: $timestamp,
            command: $command,
            test_name: $test_name,
            test_file: $test_file,
            exit_code: $exit_code,
            metrics: $metrics
        }')

    # JSON出力
    if [[ -n "$output_json" ]]; then
        echo "$profile_data" > "$output_json"
        log INFO "Profile data saved: $output_json"
    fi

    # 一時ファイル削除
    rm -rf "$temp_dir"

    # プロファイルデータを返す
    echo "$profile_data"
    return $exit_code
}

# time出力のパース
parse_time_output() {
    local time_output="$1"
    local time_cmd="$2"
    local elapsed_ms="$3"

    log DEBUG "Parsing time output (command: $time_cmd)"

    local cpu_percent=0
    local max_memory_kb=0
    local user_time_ms=0
    local system_time_ms=0
    local wall_time_ms="$elapsed_ms"

    if [[ ! -f "$time_output" ]]; then
        log WARN "Time output file not found: $time_output"
    else
        case "$time_cmd" in
            gtime)
                # GNU time フォーマット
                # Example:
                #   User time (seconds): 0.12
                #   System time (seconds): 0.05
                #   Percent of CPU this job got: 85%
                #   Maximum resident set size (kbytes): 4096

                if grep -q "User time" "$time_output"; then
                    local user_sec
                    user_sec=$(grep "User time" "$time_output" | awk '{print $4}')
                    user_time_ms=$(awk -v sec="$user_sec" 'BEGIN {printf "%.0f", sec * 1000}')
                fi

                if grep -q "System time" "$time_output"; then
                    local system_sec
                    system_sec=$(grep "System time" "$time_output" | awk '{print $4}')
                    system_time_ms=$(awk -v sec="$system_sec" 'BEGIN {printf "%.0f", sec * 1000}')
                fi

                if grep -q "Percent of CPU" "$time_output"; then
                    cpu_percent=$(grep "Percent of CPU" "$time_output" | awk '{print $7}' | tr -d '%')
                fi

                if grep -q "Maximum resident set size" "$time_output"; then
                    max_memory_kb=$(grep "Maximum resident set size" "$time_output" | awk '{print $6}')
                fi
                ;;

            /usr/bin/time)
                # BSD time フォーマット (-l オプション)
                # Example:
                #         0.12 real         0.05 user         0.03 sys
                #      4096  maximum resident set size

                if grep -q "real" "$time_output"; then
                    local real_sec
                    real_sec=$(grep "real" "$time_output" | awk '{print $1}')
                    wall_time_ms=$(awk -v sec="$real_sec" 'BEGIN {printf "%.0f", sec * 1000}')
                fi

                if grep -q "user" "$time_output"; then
                    local user_sec
                    user_sec=$(grep "user" "$time_output" | awk '{print $3}')
                    user_time_ms=$(awk -v sec="$user_sec" 'BEGIN {printf "%.0f", sec * 1000}')
                fi

                if grep -q "sys" "$time_output"; then
                    local system_sec
                    system_sec=$(grep "sys" "$time_output" | awk '{print $5}')
                    system_time_ms=$(awk -v sec="$system_sec" 'BEGIN {printf "%.0f", sec * 1000}')
                fi

                if grep -q "maximum resident set size" "$time_output"; then
                    max_memory_kb=$(grep "maximum resident set size" "$time_output" | awk '{print $1}')
                fi

                # CPU% 計算
                if [[ $wall_time_ms -gt 0 ]]; then
                    local total_cpu_ms=$((user_time_ms + system_time_ms))
                    cpu_percent=$(awk -v cpu="$total_cpu_ms" -v wall="$wall_time_ms" \
                        'BEGIN {printf "%.0f", (cpu / wall) * 100}')
                fi
                ;;

            *)
                # Fallback
                log DEBUG "Using fallback metrics"
                ;;
        esac
    fi

    # JSON構築
    jq -n \
        --argjson wall_time_ms "$wall_time_ms" \
        --argjson user_time_ms "$user_time_ms" \
        --argjson system_time_ms "$system_time_ms" \
        --argjson cpu_percent "$cpu_percent" \
        --argjson max_memory_kb "$max_memory_kb" \
        '{
            wall_time_ms: $wall_time_ms,
            user_time_ms: $user_time_ms,
            system_time_ms: $system_time_ms,
            cpu_percent: $cpu_percent,
            max_memory_kb: $max_memory_kb,
            max_memory_mb: ($max_memory_kb / 1024 | floor)
        }'
}

# メトリクス収集（詳細）
collect_metrics() {
    local pid="$1"
    local output_json="${2:-}"

    log DEBUG "Collecting detailed metrics for PID: $pid"

    # プロセス情報収集
    local ps_output
    ps_output=$(ps -p "$pid" -o rss,vsz,pcpu,pmem 2>/dev/null || echo "0 0 0.0 0.0")

    local rss vsz pcpu pmem
    read -r rss vsz pcpu pmem <<< "$ps_output"

    # メトリクスJSON構築
    local metrics
    metrics=$(jq -n \
        --argjson rss "${rss:-0}" \
        --argjson vsz "${vsz:-0}" \
        --arg pcpu "${pcpu:-0.0}" \
        --arg pmem "${pmem:-0.0}" \
        '{
            rss_kb: $rss,
            vsz_kb: $vsz,
            cpu_percent: ($pcpu | tonumber),
            memory_percent: ($pmem | tonumber)
        }')

    if [[ -n "$output_json" ]]; then
        echo "$metrics" > "$output_json"
    fi

    echo "$metrics"
}

# プロファイルデータベース初期化
initialize_profile_db() {
    local db_path="${1:-$PROFILE_DB_PATH}"

    log INFO "Initializing profile database: $db_path"

    # ディレクトリ作成
    local db_dir
    db_dir=$(dirname "$db_path")
    mkdir -p "$db_dir"

    # 空のJSONファイル作成
    if [[ ! -f "$db_path" ]]; then
        echo '{"profiles": [], "metadata": {"version": "2.1.0", "created_at": ""}}' | \
            jq --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
               '.metadata.created_at = $timestamp' > "$db_path"
        log INFO "Profile database initialized: $db_path"
    else
        log INFO "Profile database already exists: $db_path"
    fi

    return 0
}

# プロファイルデータ保存
save_profile_to_db() {
    local profile_json="$1"
    local db_path="${2:-$PROFILE_DB_PATH}"

    log DEBUG "Saving profile to database: $db_path"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        initialize_profile_db "$db_path"
    fi

    # プロファイルデータを追加
    local updated_db
    updated_db=$(jq --argjson profile "$profile_json" \
        '.profiles += [$profile]' "$db_path")

    echo "$updated_db" > "$db_path"

    log DEBUG "Profile saved to database"
    return 0
}

# プロファイル統計取得
get_profile_statistics() {
    local db_path="${1:-$PROFILE_DB_PATH}"
    local command_filter="${2:-}"

    log INFO "Retrieving profile statistics"

    if [[ ! -f "$db_path" ]]; then
        log ERROR "Profile database not found: $db_path"
        return 1
    fi

    local stats
    if [[ -n "$command_filter" ]]; then
        # 特定コマンドの統計
        stats=$(jq --arg cmd "$command_filter" '
            .profiles
            | map(select(.command == $cmd))
            | {
                count: length,
                avg_wall_time_ms: (map(.metrics.wall_time_ms) | add / length | floor),
                avg_memory_kb: (map(.metrics.max_memory_kb) | add / length | floor),
                avg_cpu_percent: (map(.metrics.cpu_percent) | add / length | floor)
            }
        ' "$db_path")
    else
        # 全体統計
        stats=$(jq '
            .profiles
            | {
                count: length,
                avg_wall_time_ms: (map(.metrics.wall_time_ms) | add / length | floor),
                avg_memory_kb: (map(.metrics.max_memory_kb) | add / length | floor),
                avg_cpu_percent: (map(.metrics.cpu_percent) | add / length | floor)
            }
        ' "$db_path")
    fi

    echo "$stats"
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <subcommand> [args...]" >&2
        echo "" >&2
        echo "Subcommands:" >&2
        echo "  init [db-path]                     Initialize profile database" >&2
        echo "  profile <command> [output-json]    Profile command execution" >&2
        echo "  stats [db-path] [command-filter]   Show profile statistics" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 init ./profiles.json" >&2
        echo "  $0 profile '/bin/ls -la' ./profile-result.json" >&2
        echo "  $0 stats ./profiles.json '/bin/ls -la'" >&2
        exit 1
    fi

    subcommand="$1"
    shift

    case "$subcommand" in
        init)
            db_path="${1:-$PROFILE_DB_PATH}"
            initialize_profile_db "$db_path"
            ;;
        profile)
            if [[ $# -lt 1 ]]; then
                log ERROR "profile requires: <command> [output-json] [test-name] [test-file]"
                exit 1
            fi
            profile_execution "$@"
            ;;
        stats)
            db_path="${1:-$PROFILE_DB_PATH}"
            command_filter="${2:-}"
            get_profile_statistics "$db_path" "$command_filter"
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
