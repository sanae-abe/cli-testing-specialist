#!/usr/bin/env bash
#
# shell-detector.sh - Shell検出・互換性チェックエンジン
# CLI Testing Specialist Agent v1.2.0
#
# 機能:
# - bash/zsh/fish/sh などのshell検出
# - バージョン情報取得
# - 各shell の利用可能性チェック
# - JSON形式での出力

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in shell-detector.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"

# サポートするshellのリスト
SUPPORTED_SHELLS=(bash zsh fish sh ksh dash)

# Shell情報を取得
detect_shell() {
    local shell_name="$1"
    local shell_info="{}"

    log DEBUG "Detecting shell: $shell_name"

    # shellの存在確認
    if ! command -v "$shell_name" &>/dev/null; then
        shell_info=$(jq -n \
            --arg name "$shell_name" \
            --argjson available false \
            '{
                name: $name,
                available: $available,
                path: null,
                version: null,
                error: "Shell not found in PATH"
            }')
        log DEBUG "$shell_name: not found"
        echo "$shell_info"
        return 0
    fi

    local shell_path
    shell_path=$(command -v "$shell_name")

    # バージョン情報取得
    local version=""
    local version_error=""

    case "$shell_name" in
        bash)
            version=$("$shell_path" --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown")
            ;;
        zsh)
            version=$("$shell_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1 || echo "unknown")
            ;;
        fish)
            version=$("$shell_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown")
            ;;
        sh|dash|ksh)
            # sh/dash/ksh はバージョン取得が難しい場合がある
            version=$("$shell_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1 || echo "unknown")
            if [[ "$version" == "unknown" ]]; then
                # フォールバック: --help や -c 'echo $version' などを試す
                version="present"
            fi
            ;;
        *)
            version="unknown"
            ;;
    esac

    shell_info=$(jq -n \
        --arg name "$shell_name" \
        --argjson available true \
        --arg path "$shell_path" \
        --arg version "$version" \
        '{
            name: $name,
            available: $available,
            path: $path,
            version: $version
        }')

    log INFO "$shell_name: $version at $shell_path"
    echo "$shell_info"
}

# 全shellを検出
detect_all_shells() {
    local output_json="$1"

    log INFO "Starting shell detection"
    log DEBUG "Supported shells: ${SUPPORTED_SHELLS[*]}"

    local shells_array="[]"
    local available_count=0

    for shell_name in "${SUPPORTED_SHELLS[@]}"; do
        local shell_info
        shell_info=$(detect_shell "$shell_name")

        # 配列に追加
        shells_array=$(echo "$shells_array" | jq --argjson shell "$shell_info" '. + [$shell]')

        # 利用可能数をカウント
        if echo "$shell_info" | jq -e '.available' >/dev/null 2>&1; then
            ((available_count++)) || true
        fi
    done

    # 最終JSON生成
    local final_json
    final_json=$(jq -n \
        --argjson shells "$shells_array" \
        --argjson total "${#SUPPORTED_SHELLS[@]}" \
        --argjson available "$available_count" \
        --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
        '{
            detection_type: "multi_shell_analysis",
            generated_at: $timestamp,
            agent_version: "1.2.0",
            summary: {
                total_shells: $total,
                available_shells: $available,
                unavailable_shells: ($total - $available)
            },
            shells: $shells
        }')

    # ファイルに保存
    echo "$final_json" | jq '.' > "$output_json"

    log INFO "Shell detection completed"
    log INFO "  Total shells checked: ${#SUPPORTED_SHELLS[@]}"
    log INFO "  Available shells: $available_count"
    log INFO "  Output: $output_json"

    echo "$output_json"
}

# Shell互換性テスト
test_shell_compatibility() {
    local shell_path="$1"
    local test_script="$2"

    log DEBUG "Testing compatibility: $shell_path with script: $test_script"

    # 基本的な互換性テスト
    local test_result
    if timeout 5 "$shell_path" -c "$test_script" &>/dev/null; then
        test_result="compatible"
        log DEBUG "Compatibility test passed for $shell_path"
    else
        test_result="incompatible"
        log DEBUG "Compatibility test failed for $shell_path"
    fi

    echo "$test_result"
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <output-json>" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <output-json>  JSON file to output shell detection results" >&2
        echo "" >&2
        echo "Example:" >&2
        echo "  $0 shell-detection.json" >&2
        exit 1
    fi

    # Shell検出実行
    detect_all_shells "$@"
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        log INFO "Shell detection completed successfully"
    else
        log ERROR "Shell detection failed with exit code: $exit_code"
    fi

    exit $exit_code
fi
