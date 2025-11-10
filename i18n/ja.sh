#!/usr/bin/env bash
#
# ja.sh - Japanese (日本語) Messages for CLI Testing Specialist
#
# Format: Key-Value associative array (individual assignments for Bash compatibility)
# Keys: snake_case (e.g., cli_analysis_started)
# Values: Japanese messages
#
# Note: MESSAGES array is declared in utils/i18n-loader.sh

# CLI Analyzer Messages
MESSAGES[cli_analysis_started]="CLI解析を開始します"
MESSAGES[cli_analysis_completed]="CLI解析が完了しました"
MESSAGES[analyzing_cli_tool]="CLIツールを解析中: %s"
MESSAGES[input_binary]="入力バイナリ: %s"
MESSAGES[output_file]="出力ファイル: %s"
MESSAGES[binary_validation_failed]="バイナリの検証に失敗しました"

# Help Fetching Messages
MESSAGES[fetching_main_help]="メインヘルプを取得中..."
MESSAGES[failed_to_get_help]="ヘルプの取得に失敗しました (終了コード: %s)"
MESSAGES[timeout_getting_help]="ヘルプ取得がタイムアウトしました: %s"
MESSAGES[no_help_output]="ヘルプ出力が空です"
MESSAGES[failed_to_get_help_for_subcommand]="サブコマンドのヘルプ取得に失敗: %s"

# Subcommand & Option Messages
MESSAGES[extracting_subcommands]="サブコマンドを抽出中..."
MESSAGES[detected_subcommands]="サブコマンド検出: %s個"
MESSAGES[extracting_options]="オプションを抽出中..."
MESSAGES[detected_options]="オプション検出: %s個"
MESSAGES[processing_subcommands]="サブコマンド処理中: %s/%s"
MESSAGES[analyzing_subcommand]="サブコマンド解析中: %s (深さ: %s)"
MESSAGES[no_subcommands_empty_tree]="サブコマンドなし、空のツリーを返します"

# Command Tree Messages
MESSAGES[building_command_tree]="コマンドツリーを構築中（最大深さ: %s）..."
MESSAGES[command_tree_built]="コマンドツリー構築完了"
MESSAGES[max_depth_reached]="最大深さに到達しました、再帰を停止します"
MESSAGES[recursively_analyzing]="再帰的解析中: %s (深さ: %s/%s)"

# JSON Generation Messages
MESSAGES[generating_json_analysis]="JSON解析結果を生成中..."
MESSAGES[analysis_completed_successfully]="解析が正常に完了しました"
MESSAGES[analysis_failed]="解析に失敗しました: %s"

# Logger Messages
MESSAGES[logger_initialized]="ロガー初期化完了: ログファイル=%s"
MESSAGES[log_level_changed]="ログレベル変更: %s → %s"
MESSAGES[invalid_log_level]="無効なログレベル: %s（有効: DEBUG, INFO, WARN, ERROR）"
MESSAGES[rotating_log_file]="ログファイルローテーション中..."

# Dependency Messages
MESSAGES[jq_not_installed]="jqがインストールされていません"
MESSAGES[jq_install_instruction]="インストール: brew install jq (macOS) または apt-get install jq (Linux)"
