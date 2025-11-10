#!/usr/bin/env bash
#
# logger.sh - ログシステム
# CLI Testing Specialist Agent v1.1.0
#
# 機能:
# - ログレベル管理（DEBUG, INFO, WARN, ERROR）
# - カラー出力対応
# - ファイル出力
# - スタックトレース

set -euo pipefail

# i18n サポート
# shellcheck source=utils/i18n-loader.sh
source "$(dirname "${BASH_SOURCE[0]}")/i18n-loader.sh"
load_i18n_once

# ログレベル定義
declare -gA LOG_LEVELS=([DEBUG]=0 [INFO]=1 [WARN]=2 [ERROR]=3)
LOG_LEVEL="${CLI_TEST_LOG_LEVEL:-INFO}"
LOG_FILE="${CLI_TEST_LOG_FILE:-/tmp/cli-test-$$.log}"
LOG_COLOR="${CLI_TEST_LOG_COLOR:-true}"

# ログファイル初期化
init_logger() {
    # ログディレクトリ作成
    local log_dir
    log_dir=$(dirname "$LOG_FILE")
    mkdir -p "$log_dir" 2>/dev/null || true

    # ログファイル作成（安全な権限で）
    touch "$LOG_FILE" 2>/dev/null || {
        LOG_FILE="/tmp/cli-test-fallback-$$.log"
        touch "$LOG_FILE" 2>/dev/null || true
    }
    chmod 600 "$LOG_FILE" 2>/dev/null || true

    # shellcheck disable=SC2059
    log DEBUG "$(printf "$(msg logger_initialized)" "$LOG_FILE")"
}

# ログ出力関数
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local caller_info=""

    # ログレベルフィルタ
    if [[ ${LOG_LEVELS[$level]:-0} -lt ${LOG_LEVELS[$LOG_LEVEL]:-1} ]]; then
        return 0
    fi

    # 呼び出し元情報（DEBUGレベルのみ）
    if [[ "$level" == "DEBUG" ]]; then
        local caller_line
        caller_line=$(caller 0 | awk '{print $1}')
        local caller_func
        caller_func=$(caller 0 | awk '{print $2}')
        caller_info=" [$caller_func:$caller_line]"
    fi

    # コンソール出力（色付き）
    local output_message="[$timestamp] [$level]$caller_info $message"

    if [[ "$LOG_COLOR" == "true" ]] && [[ -t 1 ]]; then
        case "$level" in
            DEBUG) echo -e "\033[90m$output_message\033[0m" ;;
            INFO)  echo -e "\033[34m$output_message\033[0m" ;;
            WARN)  echo -e "\033[33m$output_message\033[0m" >&2 ;;
            ERROR) echo -e "\033[31m$output_message\033[0m" >&2 ;;
        esac
    else
        # Always output to stderr to avoid interfering with command substitution
        echo "$output_message" >&2
    fi

    # ファイル出力（常時）
    if [[ -n "${LOG_FILE:-}" ]] && [[ -w "$LOG_FILE" ]]; then
        echo "$output_message" >> "$LOG_FILE" 2>/dev/null || true
    fi
}

# スタックトレース付きエラーログ
log_error_with_trace() {
    local message="$1"
    log ERROR "$message"
    log ERROR "Stack trace:"

    local frame=0
    while caller $frame 2>/dev/null; do
        ((frame++))
    done | while read -r line func file; do
        log ERROR "  at $func ($file:$line)"
    done
}

# ログレベル変更
set_log_level() {
    local new_level="$1"

    if [[ -z "${LOG_LEVELS[$new_level]:-}" ]]; then
        # shellcheck disable=SC2059
        log WARN "$(printf "$(msg invalid_log_level)" "$new_level")"
        return 1
    fi

    LOG_LEVEL="$new_level"
    # shellcheck disable=SC2059
    log INFO "$(printf "$(msg log_level_changed)" "$LOG_LEVEL")"
}

# ログファイルのローテーション
rotate_log() {
    if [[ ! -f "$LOG_FILE" ]]; then
        return 0
    fi

    local log_size
    log_size=$(stat -f%z "$LOG_FILE" 2>/dev/null || stat -c%s "$LOG_FILE" 2>/dev/null || echo 0)
    local max_size=$((10 * 1024 * 1024))  # 10MB

    if [[ $log_size -gt $max_size ]]; then
        # shellcheck disable=SC2059
        log INFO "$(printf "$(msg rotating_log_file)" "$log_size")"
        mv "$LOG_FILE" "${LOG_FILE}.1" 2>/dev/null || true
        touch "$LOG_FILE"
        chmod 600 "$LOG_FILE" 2>/dev/null || true
    fi
}

# 初期化（sourceされた時点で実行）
if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    init_logger
fi
