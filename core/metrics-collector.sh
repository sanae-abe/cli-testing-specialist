#!/usr/bin/env bash
#
# metrics-collector.sh - 詳細メトリクス収集エンジン
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - システムコール追跡（strace/dtruss統合）
# - ファイルディスクリプタリーク検出
# - ネットワークI/O測定
# - 詳細パフォーマンスメトリクス収集
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in metrics-collector.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# トレーシングツール検出
detect_tracing_tool() {
    log DEBUG "Detecting available tracing tools"

    # Linux: strace
    if command -v strace &>/dev/null; then
        echo "strace"
        log INFO "Using strace for system call tracing"
        return 0
    fi

    # macOS: dtruss (DTrace)
    if command -v dtruss &>/dev/null; then
        # 権限チェック
        if dtruss -h &>/dev/null 2>&1; then
            echo "dtruss"
            log INFO "Using dtruss for system call tracing"
            return 0
        else
            log WARN "dtruss found but requires elevated privileges"
        fi
    fi

    # Fallback: なし
    echo "none"
    log WARN "No tracing tool available (strace/dtruss)"
    return 0
}

# システムコール統計収集
collect_syscall_stats() {
    local command="$1"
    local output_json="${2:-}"

    log INFO "Collecting system call statistics"
    log DEBUG "Command: $command"

    local tracing_tool
    tracing_tool=$(detect_tracing_tool)

    local temp_dir
    temp_dir=$(mktemp -d)
    local trace_output="$temp_dir/trace.txt"
    local command_output="$temp_dir/command_output.txt"

    local exit_code=0
    local syscall_count=0
    local total_time_us=0
    local syscall_errors=0

    case "$tracing_tool" in
        strace)
            # strace -c: システムコール統計サマリー
            # -o: 出力ファイル
            # -q: 静粛モード
            strace -c -o "$trace_output" -q \
                bash -c "$command" > "$command_output" 2>&1 || exit_code=$?

            # strace出力パース
            if [[ -f "$trace_output" ]]; then
                # Example output:
                # % time     seconds  usecs/call     calls    errors syscall
                # ------ ----------- ----------- --------- --------- ----------------
                #  45.16    0.000028          28         1           execve
                #  ...
                # ------ ----------- ----------- --------- --------- ----------------
                # 100.00    0.000062                     7           total

                # 総コール数
                if grep -q "total" "$trace_output"; then
                    syscall_count=$(grep "total" "$trace_output" | awk '{print $4}')
                fi

                # 総時間（マイクロ秒）
                if grep -q "seconds" "$trace_output"; then
                    local total_seconds
                    total_seconds=$(grep "total" "$trace_output" | awk '{print $2}')
                    total_time_us=$(awk -v sec="$total_seconds" 'BEGIN {printf "%.0f", sec * 1000000}')
                fi

                # エラー数
                syscall_errors=$(grep -v "total\|%\|--" "$trace_output" | awk '{sum+=$5} END {print sum+0}')
            fi
            ;;

        dtruss)
            # dtruss: -c オプションでカウント
            # 注意: macOSではSIPの制限があるためsudoが必要な場合がある
            log WARN "dtruss requires elevated privileges, skipping syscall tracing"
            ;;

        *)
            log WARN "Syscall tracing not available"
            ;;
    esac

    # メトリクスJSON構築
    local metrics
    metrics=$(jq -n \
        --arg tool "$tracing_tool" \
        --argjson syscall_count "${syscall_count:-0}" \
        --argjson total_time_us "${total_time_us:-0}" \
        --argjson syscall_errors "${syscall_errors:-0}" \
        --argjson exit_code "$exit_code" \
        '{
            tracing_tool: $tool,
            syscall_count: $syscall_count,
            syscall_time_us: $total_time_us,
            syscall_errors: $syscall_errors,
            command_exit_code: $exit_code
        }')

    if [[ -n "$output_json" ]]; then
        echo "$metrics" > "$output_json"
        log INFO "Syscall statistics saved: $output_json"
    fi

    # 一時ファイル削除
    rm -rf "$temp_dir"

    echo "$metrics"
    return 0
}

# ファイルディスクリプタリーク検出
detect_fd_leaks() {
    local command="$1"
    local output_json="${2:-}"

    log INFO "Detecting file descriptor leaks"
    log DEBUG "Command: $command"

    local temp_dir
    temp_dir=$(mktemp -d)
    local fd_before="$temp_dir/fd_before.txt"
    local fd_after="$temp_dir/fd_after.txt"

    # コマンド実行前のFDカウント
    local fd_count_before
    fd_count_before=$(lsof -p $$ 2>/dev/null | wc -l | tr -d ' ')

    # コマンド実行
    local exit_code=0
    bash -c "$command" > /dev/null 2>&1 || exit_code=$?

    # コマンド実行後のFDカウント
    local fd_count_after
    fd_count_after=$(lsof -p $$ 2>/dev/null | wc -l | tr -d ' ')

    local fd_leaked=$((fd_count_after - fd_count_before))

    # リーク検出判定
    local leak_detected="false"
    if [[ $fd_leaked -gt 5 ]]; then
        leak_detected="true"
        log WARN "Potential file descriptor leak detected: +$fd_leaked FDs"
    fi

    # メトリクスJSON構築
    local metrics
    metrics=$(jq -n \
        --argjson fd_before "$fd_count_before" \
        --argjson fd_after "$fd_count_after" \
        --argjson fd_leaked "$fd_leaked" \
        --argjson leak_detected "$([ "$leak_detected" = "true" ] && echo true || echo false)" \
        --argjson exit_code "$exit_code" \
        '{
            fd_count_before: $fd_before,
            fd_count_after: $fd_after,
            fd_leaked: $fd_leaked,
            leak_detected: $leak_detected,
            command_exit_code: $exit_code
        }')

    if [[ -n "$output_json" ]]; then
        echo "$metrics" > "$output_json"
        log INFO "FD leak detection saved: $output_json"
    fi

    # 一時ファイル削除
    rm -rf "$temp_dir"

    echo "$metrics"
    return 0
}

# I/Oメトリクス収集
collect_io_metrics() {
    local pid="$1"
    local duration_seconds="${2:-5}"
    local output_json="${3:-}"

    log INFO "Collecting I/O metrics for PID: $pid"
    log DEBUG "Duration: ${duration_seconds}s"

    local bytes_read=0
    local bytes_written=0
    local read_ops=0
    local write_ops=0

    # macOS: lsof でファイルアクセス監視（簡易版）
    if command -v lsof &>/dev/null; then
        local lsof_output
        lsof_output=$(lsof -p "$pid" 2>/dev/null || echo "")

        # ファイル数をカウント（読み書き操作の概算）
        read_ops=$(echo "$lsof_output" | grep -c "REG" || echo "0")
        write_ops=$read_ops  # 簡易的に同じ値を使用
    fi

    # Linux: /proc/{pid}/io から取得
    if [[ -f "/proc/$pid/io" ]]; then
        bytes_read=$(grep "read_bytes" "/proc/$pid/io" | awk '{print $2}')
        bytes_written=$(grep "write_bytes" "/proc/$pid/io" | awk '{print $2}')
    fi

    # メトリクスJSON構築
    local metrics
    metrics=$(jq -n \
        --argjson bytes_read "${bytes_read:-0}" \
        --argjson bytes_written "${bytes_written:-0}" \
        --argjson read_ops "${read_ops:-0}" \
        --argjson write_ops "${write_ops:-0}" \
        '{
            bytes_read: $bytes_read,
            bytes_written: $bytes_written,
            read_ops: $read_ops,
            write_ops: $write_ops,
            total_io_bytes: ($bytes_read + $bytes_written)
        }')

    if [[ -n "$output_json" ]]; then
        echo "$metrics" > "$output_json"
        log INFO "I/O metrics saved: $output_json"
    fi

    echo "$metrics"
    return 0
}

# ネットワークI/O測定
collect_network_metrics() {
    local pid="$1"
    local output_json="${2:-}"

    log INFO "Collecting network metrics for PID: $pid"

    local connections=0
    local listening_ports=0
    local established_connections=0

    # lsof でネットワーク接続監視
    if command -v lsof &>/dev/null; then
        local lsof_output
        lsof_output=$(lsof -p "$pid" -i 2>/dev/null || echo "")

        connections=$(echo "$lsof_output" | grep -c "ESTABLISHED\|LISTEN" || echo "0")
        listening_ports=$(echo "$lsof_output" | grep -c "LISTEN" || echo "0")
        established_connections=$(echo "$lsof_output" | grep -c "ESTABLISHED" || echo "0")
    fi

    # メトリクスJSON構築
    local metrics
    metrics=$(jq -n \
        --argjson connections "${connections:-0}" \
        --argjson listening_ports "${listening_ports:-0}" \
        --argjson established_connections "${established_connections:-0}" \
        '{
            total_connections: $connections,
            listening_ports: $listening_ports,
            established_connections: $established_connections
        }')

    if [[ -n "$output_json" ]]; then
        echo "$metrics" > "$output_json"
        log INFO "Network metrics saved: $output_json"
    fi

    echo "$metrics"
    return 0
}

# 統合メトリクス収集
collect_all_metrics() {
    local command="$1"
    local output_json="${2:-}"

    log INFO "Collecting comprehensive metrics"

    local syscall_metrics
    syscall_metrics=$(collect_syscall_stats "$command")

    local fd_leak_metrics
    fd_leak_metrics=$(detect_fd_leaks "$command")

    # 統合JSON
    local all_metrics
    all_metrics=$(jq -n \
        --argjson syscall "$syscall_metrics" \
        --argjson fd_leak "$fd_leak_metrics" \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        '{
            timestamp: $timestamp,
            syscall_stats: $syscall,
            fd_leak_detection: $fd_leak
        }')

    if [[ -n "$output_json" ]]; then
        echo "$all_metrics" > "$output_json"
        log INFO "Comprehensive metrics saved: $output_json"
    fi

    echo "$all_metrics"
    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <subcommand> [args...]" >&2
        echo "" >&2
        echo "Subcommands:" >&2
        echo "  syscall <command> [output-json]     Collect syscall statistics" >&2
        echo "  fdleak <command> [output-json]      Detect file descriptor leaks" >&2
        echo "  io <pid> [duration] [output-json]   Collect I/O metrics" >&2
        echo "  network <pid> [output-json]         Collect network metrics" >&2
        echo "  all <command> [output-json]         Collect all metrics" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 syscall '/bin/ls -la' ./syscall-stats.json" >&2
        echo "  $0 fdleak '/bin/echo hello' ./fd-leak.json" >&2
        echo "  $0 all '/bin/ls' ./all-metrics.json" >&2
        exit 1
    fi

    subcommand="$1"
    shift

    case "$subcommand" in
        syscall)
            if [[ $# -lt 1 ]]; then
                log ERROR "syscall requires: <command> [output-json]"
                exit 1
            fi
            collect_syscall_stats "$@"
            ;;
        fdleak)
            if [[ $# -lt 1 ]]; then
                log ERROR "fdleak requires: <command> [output-json]"
                exit 1
            fi
            detect_fd_leaks "$@"
            ;;
        io)
            if [[ $# -lt 1 ]]; then
                log ERROR "io requires: <pid> [duration] [output-json]"
                exit 1
            fi
            collect_io_metrics "$@"
            ;;
        network)
            if [[ $# -lt 1 ]]; then
                log ERROR "network requires: <pid> [output-json]"
                exit 1
            fi
            collect_network_metrics "$@"
            ;;
        all)
            if [[ $# -lt 1 ]]; then
                log ERROR "all requires: <command> [output-json]"
                exit 1
            fi
            collect_all_metrics "$@"
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
