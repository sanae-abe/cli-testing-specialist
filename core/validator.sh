#!/usr/bin/env bash
#
# validator.sh - 入力バリデーション
# CLI Testing Specialist Agent v1.1.0
#
# セキュリティ機能:
# - CLIバイナリパスの検証
# - パストラバーサル攻撃防御
# - コマンドインジェクション対策
# - 出力ディレクトリの安全性確認

set -euo pipefail

# ロガーの読み込み
# SCRIPT_DIRが未定義の場合のみ設定（sourceされた場合に対応）
if [[ -z "${SCRIPT_DIR:-}" ]]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
source "$SCRIPT_DIR/../utils/logger.sh"

# CLIバイナリ名のバリデーション
validate_cli_binary() {
    local binary="$1"

    log DEBUG "Validating CLI binary: $binary"

    # 1. 空文字チェック
    if [[ -z "$binary" ]]; then
        log ERROR "Binary name is empty"
        return 1
    fi

    # 2. 文字種チェック（英数字、/、_、.、-のみ）
    if [[ ! "$binary" =~ ^[a-zA-Z0-9/_.-]+$ ]]; then
        log ERROR "Invalid binary name contains dangerous characters: $binary"
        log ERROR "  Allowed characters: a-z A-Z 0-9 / _ . -"
        return 1
    fi

    # 3. パストラバーサル防止
    if [[ "$binary" == *".."* ]]; then
        log ERROR "Path traversal detected in binary name: $binary"
        return 1
    fi

    # 4. 絶対パスまたは相対パスに解決
    local abs_binary
    if [[ "$binary" == /* ]]; then
        # 絶対パス
        abs_binary="$binary"
    elif [[ "$binary" == ./* ]] || [[ "$binary" == ../* ]]; then
        # 相対パス
        abs_binary="$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")"
    else
        # コマンド名のみ（PATHから検索）
        abs_binary=$(command -v "$binary" 2>/dev/null || echo "")
        if [[ -z "$abs_binary" ]]; then
            # PATHにない場合は現在ディレクトリを確認
            if [[ -f "./$binary" ]]; then
                abs_binary="$(pwd)/$binary"
            else
                log ERROR "Binary not found: $binary"
                log ERROR "  Not in PATH and not in current directory"
                return 1
            fi
        fi
    fi

    log DEBUG "Resolved binary path: $abs_binary"

    # 5. ファイル存在確認
    if [[ ! -f "$abs_binary" ]]; then
        log ERROR "Binary not found: $abs_binary"
        return 1
    fi

    # 6. 実行可能権限確認
    if [[ ! -x "$abs_binary" ]]; then
        log ERROR "Binary is not executable: $abs_binary"
        log ERROR "  Try: chmod +x $abs_binary"
        return 1
    fi

    # 7. シンボリックリンクの場合、リンク先を検証
    if [[ -L "$abs_binary" ]]; then
        local real_binary
        real_binary=$(realpath "$abs_binary" 2>/dev/null || readlink -f "$abs_binary" 2>/dev/null || echo "")

        if [[ -z "$real_binary" ]]; then
            log ERROR "Cannot resolve symbolic link: $abs_binary"
            return 1
        fi

        log DEBUG "Symbolic link detected: $abs_binary -> $real_binary"

        # リンク先が安全なパスか確認
        validate_safe_path "$real_binary" || return 1
        abs_binary="$real_binary"
    fi

    # 8. 安全なパスか確認
    validate_safe_path "$abs_binary" || return 1

    log INFO "Binary validation passed: $abs_binary"
    echo "$abs_binary"
}

# 安全なパスかチェック
validate_safe_path() {
    local path="$1"

    log DEBUG "Checking if path is safe: $path"

    # システムディレクトリは警告
    case "$path" in
        /bin/*|/sbin/*|/usr/bin/*|/usr/sbin/*)
            # 例外: 明示的に許可されたシステムコマンド
            local basename_path
            basename_path=$(basename "$path")
            case "$basename_path" in
                sh|bash|zsh|fish|dash|ash)
                    log DEBUG "System shell detected (allowed): $path"
                    return 0
                    ;;
            esac

            log WARN "Testing system binary: $path"
            log WARN "  This may have security implications."

            # 環境変数でシステムバイナリを許可している場合
            if [[ "${CLI_TEST_ALLOW_SYSTEM_BINARIES:-false}" == "true" ]]; then
                log WARN "  Proceeding (CLI_TEST_ALLOW_SYSTEM_BINARIES=true)"
                return 0
            fi

            log WARN "  To allow, set: export CLI_TEST_ALLOW_SYSTEM_BINARIES=true"
            log ERROR "System binaries are not allowed by default"
            return 1
            ;;

        /etc/*)
            log ERROR "Cannot test binaries in /etc/: $path"
            return 1
            ;;
    esac

    return 0
}

# 出力ディレクトリのバリデーション
validate_output_dir() {
    local dir="$1"

    log DEBUG "Validating output directory: $dir"

    # 1. 空文字チェック
    if [[ -z "$dir" ]]; then
        log ERROR "Output directory is empty"
        return 1
    fi

    # 2. 絶対パスに解決
    local abs_dir
    if [[ -d "$dir" ]]; then
        abs_dir="$(cd "$dir" && pwd)"
    else
        # ディレクトリが存在しない場合、親ディレクトリから解決
        local parent_dir
        parent_dir=$(dirname "$dir")
        local dir_name
        dir_name=$(basename "$dir")

        if [[ -d "$parent_dir" ]]; then
            abs_dir="$(cd "$parent_dir" && pwd)/$dir_name"
        else
            # realpathでの解決を試みる
            abs_dir=$(realpath -m "$dir" 2>/dev/null || echo "")
            if [[ -z "$abs_dir" ]]; then
                log ERROR "Cannot resolve path: $dir"
                return 1
            fi
        fi
    fi

    log DEBUG "Resolved output directory: $abs_dir"

    # 3. カレントディレクトリまたはホームディレクトリ配下のみ許可
    if [[ "$abs_dir" != "$PWD"* ]] && [[ "$abs_dir" != "$HOME"* ]]; then
        log ERROR "Output directory must be under current or home directory"
        log ERROR "  Attempted: $abs_dir"
        log ERROR "  Current:   $PWD"
        log ERROR "  Home:      $HOME"
        return 1
    fi

    # 4. システムディレクトリへの書き込み防止
    case "$abs_dir" in
        /etc/*|/usr/*|/bin/*|/sbin/*|/var/*|/boot/*|/sys/*|/proc/*)
            log ERROR "Cannot write to system directory: $abs_dir"
            return 1
            ;;
        /tmp/*)
            log WARN "Writing to /tmp is allowed but not recommended"
            log WARN "  Consider using a dedicated directory"
            ;;
    esac

    # 5. ディレクトリ作成（安全な権限で）
    if [[ ! -d "$abs_dir" ]]; then
        log INFO "Creating output directory: $abs_dir"

        mkdir -p "$abs_dir" || {
            log ERROR "Failed to create output directory: $abs_dir"
            return 1
        }

        chmod 700 "$abs_dir" || {
            log WARN "Failed to set directory permissions to 700"
        }

        log DEBUG "Created directory with mode 700: $abs_dir"
    fi

    # 6. 書き込み権限確認
    if [[ ! -w "$abs_dir" ]]; then
        log ERROR "Output directory is not writable: $abs_dir"
        return 1
    fi

    log INFO "Output directory validation passed: $abs_dir"
    echo "$abs_dir"
}

# 環境変数のサニタイズ
sanitize_env_var() {
    local var_name="$1"
    local var_value="$2"

    log DEBUG "Sanitizing environment variable: $var_name"

    # 危険な文字を除去
    local sanitized
    sanitized=$(echo "$var_value" | tr -d '\n\r\t$`"'\''\\' | sed 's/[;&|<>()]//g')

    if [[ "$var_value" != "$sanitized" ]]; then
        log WARN "Environment variable $var_name contains dangerous characters"
        log WARN "  Original:  $var_value"
        log WARN "  Sanitized: $sanitized"
    fi

    echo "$sanitized"
}

# コマンドライン引数のバリデーション
validate_cli_args() {
    local args=("$@")

    log DEBUG "Validating CLI arguments (${#args[@]} args)"

    for arg in "${args[@]}"; do
        # NULL bytes チェック
        if [[ "$arg" == *$'\x00'* ]]; then
            log ERROR "NULL byte detected in argument: $arg"
            return 1
        fi

        # 極端に長い引数
        if [[ ${#arg} -gt 10000 ]]; then
            log ERROR "Argument too long (${#arg} characters): ${arg:0:100}..."
            return 1
        fi
    done

    log DEBUG "CLI arguments validation passed"
    return 0
}

# 使用例（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "=== Validator Test ==="

    # テスト1: CLIバイナリバリデーション
    echo "Test 1: Valid binary"
    if validate_cli_binary "/bin/ls"; then
        echo "  ✓ Passed"
    else
        echo "  ✗ Failed"
    fi

    # テスト2: 無効なバイナリ
    echo "Test 2: Invalid binary"
    if validate_cli_binary "/nonexistent/binary"; then
        echo "  ✗ Should have failed"
    else
        echo "  ✓ Correctly rejected"
    fi

    # テスト3: 出力ディレクトリバリデーション
    echo "Test 3: Valid output directory"
    if validate_output_dir "/tmp/test-output-$$"; then
        echo "  ✓ Passed"
        rmdir "/tmp/test-output-$$" 2>/dev/null || true
    else
        echo "  ✗ Failed"
    fi

    # テスト4: システムディレクトリ
    echo "Test 4: System directory (should fail)"
    if validate_output_dir "/etc/test"; then
        echo "  ✗ Should have failed"
    else
        echo "  ✓ Correctly rejected"
    fi

    echo "=== Tests completed ==="
fi
