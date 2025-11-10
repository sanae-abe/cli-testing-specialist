#!/usr/bin/env bash
#
# cli-analyzer.sh - CLIツール解析エンジン（修正版）
# CLI Testing Specialist Agent v1.1.0
#
# 主な修正点:
# - build_command_tree のサブシェル問題修正（プロセス置換使用）
# - 入力バリデーション追加
# - タイムアウト処理追加
# - 再帰的サブコマンド解析（深さ制限付き）

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in cli-analyzer.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# jqの存在確認（フォールバック対応）
if ! command -v jq &>/dev/null; then
    log ERROR "jq is required but not installed"
    log ERROR "  Install: brew install jq  (macOS)"
    log ERROR "          apt-get install jq  (Ubuntu/Debian)"
    exit 1
fi

# メイン解析関数
analyze_cli_tool() {
    local cli_binary="$1"
    local output_file="${2:-cli-analysis.json}"

    log INFO "Starting CLI tool analysis"
    log DEBUG "Input binary: $cli_binary"
    log DEBUG "Output file: $output_file"

    # 入力バリデーション（セキュリティ強化）
    local validated_binary
    validated_binary=$(validate_cli_binary "$cli_binary") || {
        log ERROR "Binary validation failed"
        return 1
    }

    log INFO "Analyzing CLI tool: $validated_binary"

    # メインヘルプ解析（タイムアウト付き）
    local main_help
    log DEBUG "Fetching main help..."
    main_help=$(timeout 10 "$validated_binary" --help 2>&1) || {
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log ERROR "Timeout while getting help from $validated_binary"
        else
            log WARN "Failed to get help (exit code: $exit_code)"
            log WARN "  Some CLIs may not support --help"
        fi
        main_help=""
    }

    if [[ -z "$main_help" ]]; then
        log WARN "No help output received, trying --version"
        main_help=$(timeout 5 "$validated_binary" --version 2>&1) || main_help="No help available"
    fi

    # サブコマンド抽出
    log DEBUG "Extracting subcommands..."
    local subcommands
    subcommands=$(extract_subcommands "$main_help")
    local subcommand_count
    subcommand_count=$(echo "$subcommands" | jq 'length')
    log INFO "Detected $subcommand_count subcommands"
    log DEBUG "Subcommands: $(echo "$subcommands" | jq -c '.')"

    # オプション抽出
    log DEBUG "Extracting options..."
    local options
    options=$(extract_options "$main_help")
    local option_count
    option_count=$(echo "$options" | jq 'length')
    log INFO "Detected $option_count options"
    log DEBUG "Options: $(echo "$options" | jq -c '.')"

    # 再帰的にサブコマンドを解析（修正版：プロセス置換使用）
    log INFO "Building command tree (max depth: 3)..."
    local command_tree
    command_tree=$(build_command_tree "$validated_binary" "$subcommands" 3)

    # JSON出力
    log DEBUG "Generating JSON analysis..."
    local agent_version
    agent_version=$(cat "$SCRIPT_DIR/../VERSION" 2>/dev/null || echo "1.1.0-dev")

    jq -n \
        --arg binary "$validated_binary" \
        --arg binary_basename "$(basename "$validated_binary")" \
        --argjson subcommands "$subcommands" \
        --argjson options "$options" \
        --argjson tree "$command_tree" \
        --arg version "$agent_version" \
        '{
            binary: $binary,
            binary_basename: $binary_basename,
            subcommands: $subcommands,
            subcommand_count: ($subcommands | length),
            options: $options,
            option_count: ($options | length),
            command_tree: $tree,
            analyzed_at: (now | strftime("%Y-%m-%dT%H:%M:%SZ")),
            agent_version: $version
        }' > "$output_file"

    log INFO "CLI analysis completed"
    log INFO "  Subcommands: $subcommand_count"
    log INFO "  Options: $option_count"
    log INFO "  Output: $output_file"

    echo "$output_file"
}

# サブコマンド抽出
extract_subcommands() {
    local help_text="$1"

    log DEBUG "Extracting subcommands from help text (${#help_text} chars)"

    # "Available Commands:", "Commands:", "Subcommands:" セクションを検出
    local subcommands
    subcommands=$(echo "$help_text" | awk '
        BEGIN { in_commands = 0 }

        # コマンドセクションの開始を検出
        /^(Available )?Commands?:?$/ || /^Subcommands:?$/ {
            in_commands = 1
            next
        }

        # 空行でセクション終了
        in_commands && /^$/ {
            in_commands = 0
            next
        }

        # コマンド行を抽出（先頭が空白で始まる）
        in_commands && /^[[:space:]]+[a-zA-Z]/ {
            # 最初の単語（コマンド名）を抽出（macOS awk互換）
            gsub(/^[[:space:]]+/, "")  # 先頭の空白を削除
            cmd_name = $1
            if (cmd_name && cmd_name ~ /^[a-zA-Z]/) print cmd_name
        }
    ' | sort -u | jq -R -s 'split("\n") | map(select(length > 0))')

    # 結果が空の場合は空の配列を返す
    if [[ -z "$subcommands" ]] || [[ "$subcommands" == "null" ]]; then
        subcommands="[]"
    fi

    log DEBUG "Extracted $(echo "$subcommands" | jq 'length') subcommands"
    echo "$subcommands"
}

# オプション抽出
extract_options() {
    local help_text="$1"

    log DEBUG "Extracting options from help text"

    # "--" または "-" で始まる行からオプションを抽出
    local options
    options=$(echo "$help_text" | grep -E '^\s+(-{1,2}[a-zA-Z0-9]|--[a-zA-Z0-9-]+)' | awk '{
        # --option, -o 形式を抽出
        match($0, /(--?[a-zA-Z0-9][a-zA-Z0-9-]*)/)
        if (RSTART) {
            opt = substr($0, RSTART, RLENGTH)
            # --help-all のような長いオプションも抽出
            print opt
        }
    }' | sort -u | jq -R -s 'split("\n") | map(select(length > 0))')

    # 結果が空の場合は空の配列を返す
    if [[ -z "$options" ]] || [[ "$options" == "null" ]]; then
        options="[]"
    fi

    log DEBUG "Extracted $(echo "$options" | jq 'length') options"
    echo "$options"
}

# サブコマンドツリー構築（修正版：プロセス置換使用）
build_command_tree() {
    local cli_binary="$1"
    local subcommands="$2"
    local max_depth="${3:-3}"  # デフォルト3階層まで

    log DEBUG "Building command tree (cli=$cli_binary, depth=$max_depth)"

    # 深さ制限チェック
    if [[ $max_depth -le 0 ]]; then
        log DEBUG "Max depth reached, returning empty tree"
        echo "{}"
        return 0
    fi

    local tree="{}"
    local subcommand_count
    subcommand_count=$(echo "$subcommands" | jq 'length')

    if [[ $subcommand_count -eq 0 ]]; then
        log DEBUG "No subcommands, returning empty tree"
        echo "{}"
        return 0
    fi

    log DEBUG "Processing $subcommand_count subcommands at depth $max_depth"

    # プロセス置換を使用してサブシェル問題を回避（重要な修正）
    while IFS= read -r cmd; do
        [[ -z "$cmd" ]] && continue

        log DEBUG "Analyzing subcommand: $cmd"

        # サブコマンドのヘルプを取得（タイムアウト付き）
        local cmd_help
        cmd_help=$(timeout 10 $cli_binary $cmd --help 2>&1) || {
            log WARN "Failed to get help for subcommand: $cmd"
            cmd_help=""
        }

        # さらにサブコマンドがあるか確認
        local nested_subcommands
        nested_subcommands=$(extract_subcommands "$cmd_help")

        # 再帰的に深い階層も解析（深さ制限付き）
        local nested_tree="{}"
        local nested_count
        nested_count=$(echo "$nested_subcommands" | jq 'length')

        if [[ $max_depth -gt 1 ]] && [[ $nested_count -gt 0 ]]; then
            log DEBUG "Recursively analyzing '$cmd' (depth=$((max_depth - 1)), nested=$nested_count)"
            nested_tree=$(build_command_tree "$cli_binary $cmd" "$nested_subcommands" $((max_depth - 1)))
        fi

        # ツリーに追加
        tree=$(echo "$tree" | jq \
            --arg cmd "$cmd" \
            --arg help "$cmd_help" \
            --argjson nested "$nested_subcommands" \
            --argjson nested_tree "$nested_tree" \
            '.[$cmd] = {
                help: $help,
                subcommands: $nested,
                tree: $nested_tree,
                has_nested: ($nested | length > 0)
            }')

    done < <(echo "$subcommands" | jq -r '.[]')  # プロセス置換（修正ポイント）

    log DEBUG "Command tree built successfully"
    echo "$tree"
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <cli-binary> [output-file]" >&2
        echo "" >&2
        echo "Example:" >&2
        echo "  $0 /usr/bin/git" >&2
        echo "  $0 ./my-cli analysis.json" >&2
        exit 1
    fi

    # 解析実行
    analyze_cli_tool "$@"
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        log INFO "Analysis completed successfully"
    else
        log ERROR "Analysis failed with exit code: $exit_code"
    fi

    exit $exit_code
fi
