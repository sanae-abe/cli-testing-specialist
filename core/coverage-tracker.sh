#!/usr/bin/env bash
#
# coverage-tracker.sh - カバレッジトラッキングエンジン
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - テスト実行時のコマンド/オプション使用を追跡
# - SQLiteデータベースへの記録
# - カバレッジデータ収集
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in coverage-tracker.sh"' ERR

# スクリプトのディレクトリを取得（読み取り専用）
declare -r SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
declare -r AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# デフォルト設定（読み取り専用）
declare -r DEFAULT_COVERAGE_DB="${AGENT_ROOT}/coverage.db"
COVERAGE_DB_PATH="${COVERAGE_DB_PATH:-$DEFAULT_COVERAGE_DB}"

# SQLite3の存在確認
check_sqlite3_installation() {
    log DEBUG "Checking SQLite3 installation"

    if ! command -v sqlite3 &>/dev/null; then
        log ERROR "SQLite3 is not installed"
        log ERROR "  Install: brew install sqlite3  (macOS)"
        log ERROR "          apt-get install sqlite3 (Ubuntu/Debian)"
        return 1
    fi

    local sqlite_version
    sqlite_version=$(sqlite3 --version | awk '{print $1}') || sqlite_version="unknown"
    log INFO "SQLite3 detected: $sqlite_version"

    return 0
}

# カバレッジデータベース初期化
initialize_coverage_db() {
    local db_path="${1:-$COVERAGE_DB_PATH}"

    log INFO "Initializing coverage database: $db_path"

    # ディレクトリ存在確認
    local db_dir
    db_dir=$(dirname "$db_path")
    mkdir -p "$db_dir"

    # SQLiteでテーブル作成
    sqlite3 "$db_path" <<EOF
-- コマンド使用履歴テーブル
CREATE TABLE IF NOT EXISTS command_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    command TEXT NOT NULL,
    subcommand TEXT,
    exit_code INTEGER,
    test_name TEXT,
    test_file TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- オプション使用履歴テーブル
CREATE TABLE IF NOT EXISTS option_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    option_name TEXT NOT NULL,
    option_value TEXT,
    command_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (command_id) REFERENCES command_usage(id) ON DELETE CASCADE
);

-- インデックス作成
CREATE INDEX IF NOT EXISTS idx_subcommand ON command_usage(subcommand);
CREATE INDEX IF NOT EXISTS idx_test_file ON command_usage(test_file);
CREATE INDEX IF NOT EXISTS idx_option_name ON option_usage(option_name);
CREATE INDEX IF NOT EXISTS idx_command_id ON option_usage(command_id);
CREATE INDEX IF NOT EXISTS idx_timestamp ON command_usage(timestamp);

-- メタデータテーブル
CREATE TABLE IF NOT EXISTS coverage_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 初期メタデータ
INSERT OR REPLACE INTO coverage_metadata (key, value) VALUES
    ('version', '2.1.0'),
    ('initialized_at', datetime('now'));
EOF

    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        log INFO "Coverage database initialized successfully"

        # データベース整合性チェック
        if ! check_database_integrity "$db_path"; then
            log ERROR "Database integrity check failed"
            return 1
        fi
    else
        log ERROR "Failed to initialize coverage database (exit code: $exit_code)"
        return 1
    fi

    return 0
}

# データベース整合性チェック
check_database_integrity() {
    local db_path="$1"

    log DEBUG "Checking database integrity: $db_path"

    local integrity_result
    integrity_result=$(sqlite3 "$db_path" "PRAGMA integrity_check;" 2>&1)

    if [[ "$integrity_result" == "ok" ]]; then
        log DEBUG "Database integrity check passed"
        return 0
    else
        log ERROR "Database integrity check failed: $integrity_result"
        return 1
    fi
}

# サブコマンド抽出
extract_subcommand() {
    local command="$1"

    log DEBUG "Extracting subcommand from: $command"

    # コマンドを配列に分割
    local -a parts
    IFS=' ' read -ra parts <<< "$command"

    # 最初の非オプション引数を探す（バイナリの次）
    local found_binary=0  # 0=false, 1=true
    for part in "${parts[@]}"; do
        # バイナリをスキップ
        if [[ $found_binary -eq 0 ]]; then
            found_binary=1
            continue
        fi

        # オプション（-で始まる）をスキップ
        if [[ "$part" =~ ^- ]]; then
            continue
        fi

        # 空文字列をスキップ
        if [[ -z "$part" ]]; then
            continue
        fi

        # 最初の非オプション引数がサブコマンド
        echo "$part"
        return 0
    done

    # サブコマンドなし
    echo ""
}

# オプション抽出
extract_options() {
    local command="$1"

    log DEBUG "Extracting options from: $command"

    # オプションを抽出（-または--で始まる）
    # 出力形式: 改行区切り
    # 例: "--verbose\n--output=file.txt\n-v"

    local options=""
    local -a parts
    IFS=' ' read -ra parts <<< "$command"

    for part in "${parts[@]}"; do
        # オプション判定（- または -- で始まる）
        if [[ "$part" =~ ^-{1,2}[a-zA-Z0-9_-]+ ]]; then
            # オプション名と値を分離
            if [[ "$part" =~ = ]]; then
                # --option=value形式
                echo "$part"
            else
                # --option形式（値なし）
                echo "$part="
            fi
        fi
    done
}

# コマンド実行追跡
track_command_execution() {
    local command="$1"
    local exit_code="$2"
    local test_name="${3:-unknown}"
    local test_file="${4:-unknown}"
    local db_path="${COVERAGE_DB_PATH}"

    log DEBUG "Tracking command execution: $command (exit: $exit_code)"

    # SQLiteチェック
    if ! command -v sqlite3 &>/dev/null; then
        log WARN "SQLite3 not available, skipping coverage tracking"
        return 0
    fi

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        log WARN "Coverage database not found: $db_path"
        log WARN "Run initialize_coverage_db() first"
        return 0
    fi

    # コマンド解析
    local subcommand
    subcommand=$(extract_subcommand "$command")

    local options
    options=$(extract_options "$command")

    # タイムスタンプ
    local timestamp
    timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    # ✅ セキュア実装: SQLパラメータバインディング使用
    # SQLite 3.32.0+ の .param 構文を使用
    # Note: プレースホルダー (?) を使った方が安全だが、.param構文の方が可読性が高い

    local command_id
    command_id=$(sqlite3 "$db_path" <<EOF 2>&1 | tail -1
.param set :timestamp "$timestamp"
.param set :command "$command"
.param set :subcommand "$subcommand"
.param set :exit_code $exit_code
.param set :test_name "$test_name"
.param set :test_file "$test_file"

INSERT INTO command_usage (timestamp, command, subcommand, exit_code, test_name, test_file)
VALUES (:timestamp, :command,
    CASE WHEN :subcommand = '' THEN NULL ELSE :subcommand END,
    :exit_code, :test_name, :test_file);

SELECT last_insert_rowid();
EOF
)

    if [[ ! "$command_id" =~ ^[0-9]+$ ]]; then
        log ERROR "Failed to insert command_usage: $command_id"
        log ERROR "SQLite version may be too old (requires 3.32.0+)"
        return 1
    fi

    log DEBUG "Inserted command_usage with id: $command_id"

    # ✅ パフォーマンス最適化: トランザクション + バッチINSERT
    if [[ -n "$options" ]]; then
        local option_count=0

        # トランザクション内で一括挿入
        {
            echo "BEGIN TRANSACTION;"

            while IFS= read -r option; do
                [[ -z "$option" ]] && continue

                local option_name
                local option_value=""

                # option=value形式の分解
                if [[ "$option" =~ = ]]; then
                    option_name=$(echo "$option" | cut -d'=' -f1)
                    option_value=$(echo "$option" | cut -d'=' -f2-)
                else
                    option_name="$option"
                fi

                # パラメータバインディング風（SQLエスケープ）
                local escaped_option_name="${option_name//\'/\'\'}"
                local escaped_option_value="${option_value//\'/\'\'}"

                # INSERT文生成
                if [[ -n "$option_value" ]]; then
                    echo "INSERT INTO option_usage (timestamp, option_name, option_value, command_id) VALUES ('$timestamp', '$escaped_option_name', '$escaped_option_value', $command_id);"
                else
                    echo "INSERT INTO option_usage (timestamp, option_name, option_value, command_id) VALUES ('$timestamp', '$escaped_option_name', NULL, $command_id);"
                fi

                ((option_count++))
            done <<< "$options"

            echo "COMMIT;"
        } | sqlite3 "$db_path" 2>&1 || {
            log WARN "Failed to insert some option_usage entries"
        }

        log DEBUG "Inserted $option_count options in batch transaction"
    fi

    log DEBUG "Command tracking completed: $command"
    return 0
}

# カバレッジ統計取得
get_coverage_statistics() {
    local db_path="${1:-$COVERAGE_DB_PATH}"

    log INFO "Retrieving coverage statistics"

    if [[ ! -f "$db_path" ]]; then
        log ERROR "Coverage database not found: $db_path"
        return 1
    fi

    # 統計クエリ
    local stats
    stats=$(sqlite3 "$db_path" <<EOF
SELECT
    'Total Commands: ' || COUNT(*) as stat FROM command_usage
UNION ALL
SELECT
    'Unique Subcommands: ' || COUNT(DISTINCT subcommand) FROM command_usage WHERE subcommand IS NOT NULL
UNION ALL
SELECT
    'Unique Options: ' || COUNT(DISTINCT option_name) FROM option_usage
UNION ALL
SELECT
    'Total Tests: ' || COUNT(DISTINCT test_name) FROM command_usage;
EOF
)

    echo "$stats"
}

# カバレッジデータベースクリーンアップ
cleanup_coverage_db() {
    local db_path="${1:-$COVERAGE_DB_PATH}"
    local days_to_keep="${2:-30}"

    log INFO "Cleaning up coverage database (keep last $days_to_keep days)"

    if [[ ! -f "$db_path" ]]; then
        log WARN "Coverage database not found: $db_path"
        return 0
    fi

    # 古いデータ削除
    local cutoff_date
    cutoff_date=$(date -u -v-${days_to_keep}d '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null || \
                  date -u -d "$days_to_keep days ago" '+%Y-%m-%dT%H:%M:%SZ')

    sqlite3 "$db_path" <<EOF
DELETE FROM option_usage
WHERE command_id IN (
    SELECT id FROM command_usage WHERE timestamp < '$cutoff_date'
);

DELETE FROM command_usage WHERE timestamp < '$cutoff_date';

VACUUM;
EOF

    log INFO "Cleanup completed"
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <command> [db-path]" >&2
        echo "" >&2
        echo "Commands:" >&2
        echo "  init [db-path]           Initialize coverage database" >&2
        echo "  track <command> <exit>   Track command execution" >&2
        echo "  stats [db-path]          Show coverage statistics" >&2
        echo "  cleanup [db-path] [days] Clean up old data" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 init ./coverage.db" >&2
        echo "  $0 track '/bin/ls --help' 0 'test1' 'test.bats'" >&2
        echo "  $0 stats ./coverage.db" >&2
        echo "  $0 cleanup ./coverage.db 30" >&2
        exit 1
    fi

    command="$1"
    shift

    case "$command" in
        init)
            db_path="${1:-$COVERAGE_DB_PATH}"
            check_sqlite3_installation || exit 1
            initialize_coverage_db "$db_path"
            ;;
        track)
            if [[ $# -lt 2 ]]; then
                log ERROR "track requires: <command> <exit_code> [test_name] [test_file]"
                exit 1
            fi
            track_command_execution "$@"
            ;;
        stats)
            db_path="${1:-$COVERAGE_DB_PATH}"
            get_coverage_statistics "$db_path"
            ;;
        cleanup)
            db_path="${1:-$COVERAGE_DB_PATH}"
            days="${2:-30}"
            cleanup_coverage_db "$db_path" "$days"
            ;;
        *)
            log ERROR "Unknown command: $command"
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
