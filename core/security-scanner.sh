#!/usr/bin/env bash
#
# security-scanner.sh - セキュリティスキャンエンジン
# CLI Testing Specialist Agent v2.2.0
#
# 機能:
# - OWASP Top 10チェック
# - コマンドインジェクション検出
# - パストラバーサル検出
# - 権限エスカレーションチェック
# - 入力検証チェック
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in security-scanner.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# デフォルト設定
DEFAULT_SECURITY_DB="$AGENT_ROOT/security-findings.json"
SECURITY_DB_PATH="${SECURITY_DB_PATH:-$DEFAULT_SECURITY_DB}"

# 脆弱性の重要度レベル
SEVERITY_CRITICAL="critical"
SEVERITY_HIGH="high"
SEVERITY_MEDIUM="medium"
SEVERITY_LOW="low"
SEVERITY_INFO="info"

# セキュリティデータベース初期化
initialize_security_db() {
    local db_path="${1:-$SECURITY_DB_PATH}"

    log INFO "Initializing security findings database: $db_path"

    # ディレクトリ作成
    local db_dir
    db_dir=$(dirname "$db_path")
    mkdir -p "$db_dir"

    # 空のJSONファイル作成
    if [[ ! -f "$db_path" ]]; then
        echo '{"findings": [], "metadata": {"version": "2.2.0", "created_at": "", "scan_count": 0}}' | \
            jq --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
               '.metadata.created_at = $timestamp' > "$db_path"
        log INFO "Security database initialized: $db_path"
    else
        log INFO "Security database already exists: $db_path"
    fi

    return 0
}

# コマンドインジェクション検出
detect_command_injection() {
    local command="$1"
    local test_name="${2:-unknown}"
    local output_json="${3:-}"

    log INFO "Checking for command injection vulnerabilities"
    log DEBUG "Command: $command"

    local findings=()
    local severity="$SEVERITY_INFO"

    # 危険なパターン検出
    local dangerous_patterns=(
        ';'          # コマンド連結
        '&&'         # 論理AND
        '||'         # 論理OR
        '|'          # パイプ
        '$('         # コマンド置換
        '`'          # バッククォート
        '>'          # リダイレクト
        '<'          # 入力リダイレクト
        '\${'        # 変数展開
    )

    local found_patterns=()
    for pattern in "${dangerous_patterns[@]}"; do
        if [[ "$command" == *"$pattern"* ]]; then
            found_patterns+=("$pattern")
        fi
    done

    # 脆弱性判定
    if [[ ${#found_patterns[@]} -gt 0 ]]; then
        if [[ "$command" == *';'* ]] || [[ "$command" == *'&&'* ]] || [[ "$command" == *'$('* ]]; then
            severity="$SEVERITY_HIGH"
        elif [[ "$command" == *'|'* ]] || [[ "$command" == *'>'* ]]; then
            severity="$SEVERITY_MEDIUM"
        else
            severity="$SEVERITY_LOW"
        fi

        local patterns_str
        patterns_str=$(printf '%s, ' "${found_patterns[@]}")
        patterns_str="${patterns_str%, }"

        local finding
        finding=$(jq -n \
            --arg type "command_injection" \
            --arg severity "$severity" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg patterns "$patterns_str" \
            --arg description "Potential command injection: found dangerous patterns ($patterns_str)" \
            --arg recommendation "Sanitize user input, use parameterized commands, avoid shell expansion" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                patterns_detected: $patterns,
                description: $description,
                recommendation: $recommendation,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')

        log WARN "Command injection risk detected: $severity - $patterns_str"
    else
        local finding
        finding=$(jq -n \
            --arg type "command_injection" \
            --arg severity "$SEVERITY_INFO" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg description "No command injection patterns detected" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                description: $description,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')
    fi

    if [[ -n "$output_json" ]]; then
        echo "$finding" > "$output_json"
        log INFO "Command injection check saved: $output_json"
    fi

    echo "$finding"
    return 0
}

# パストラバーサル検出
detect_path_traversal() {
    local command="$1"
    local test_name="${2:-unknown}"
    local output_json="${3:-}"

    log INFO "Checking for path traversal vulnerabilities"
    log DEBUG "Command: $command"

    local severity="$SEVERITY_INFO"
    local found_patterns=()

    # 危険なパストラバーサルパターン
    local traversal_patterns=(
        '../'
        '..\'
        '/..'
        '\..'
        '%2e%2e'
        '%252e%252e'
    )

    for pattern in "${traversal_patterns[@]}"; do
        if [[ "$command" == *"$pattern"* ]]; then
            found_patterns+=("$pattern")
        fi
    done

    # 脆弱性判定
    if [[ ${#found_patterns[@]} -gt 0 ]]; then
        # 複数のパストラバーサルが連続している場合は重要度高
        if [[ "$command" == *'../..'* ]] || [[ "$command" == *'../../'* ]]; then
            severity="$SEVERITY_HIGH"
        else
            severity="$SEVERITY_MEDIUM"
        fi

        local patterns_str
        patterns_str=$(printf '%s, ' "${found_patterns[@]}")
        patterns_str="${patterns_str%, }"

        local finding
        finding=$(jq -n \
            --arg type "path_traversal" \
            --arg severity "$severity" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg patterns "$patterns_str" \
            --arg description "Potential path traversal: found directory traversal patterns ($patterns_str)" \
            --arg recommendation "Validate file paths, use absolute paths, restrict access to allowed directories" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                patterns_detected: $patterns,
                description: $description,
                recommendation: $recommendation,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')

        log WARN "Path traversal risk detected: $severity - $patterns_str"
    else
        local finding
        finding=$(jq -n \
            --arg type "path_traversal" \
            --arg severity "$SEVERITY_INFO" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg description "No path traversal patterns detected" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                description: $description,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')
    fi

    if [[ -n "$output_json" ]]; then
        echo "$finding" > "$output_json"
        log INFO "Path traversal check saved: $output_json"
    fi

    echo "$finding"
    return 0
}

# 権限エスカレーションチェック
check_privilege_escalation() {
    local command="$1"
    local test_name="${2:-unknown}"
    local output_json="${3:-}"

    log INFO "Checking for privilege escalation risks"
    log DEBUG "Command: $command"

    local severity="$SEVERITY_INFO"
    local found_risks=()

    # 危険なコマンド/オプション
    local dangerous_commands=(
        'sudo'
        'su'
        'chmod 777'
        'chmod 666'
        'chown'
        'setuid'
        'setgid'
    )

    for cmd in "${dangerous_commands[@]}"; do
        if [[ "$command" == *"$cmd"* ]]; then
            found_risks+=("$cmd")
        fi
    done

    # 脆弱性判定
    if [[ ${#found_risks[@]} -gt 0 ]]; then
        if [[ "$command" == *'sudo'* ]] || [[ "$command" == *'su '* ]]; then
            severity="$SEVERITY_CRITICAL"
        elif [[ "$command" == *'chmod 777'* ]] || [[ "$command" == *'chmod 666'* ]]; then
            severity="$SEVERITY_HIGH"
        else
            severity="$SEVERITY_MEDIUM"
        fi

        local risks_str
        risks_str=$(printf '%s, ' "${found_risks[@]}")
        risks_str="${risks_str%, }"

        local finding
        finding=$(jq -n \
            --arg type "privilege_escalation" \
            --arg severity "$severity" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg risks "$risks_str" \
            --arg description "Privilege escalation risk: found dangerous privilege operations ($risks_str)" \
            --arg recommendation "Avoid privilege escalation, use principle of least privilege, audit permission changes" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                risks_detected: $risks,
                description: $description,
                recommendation: $recommendation,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')

        log WARN "Privilege escalation risk detected: $severity - $risks_str"
    else
        local finding
        finding=$(jq -n \
            --arg type "privilege_escalation" \
            --arg severity "$SEVERITY_INFO" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg description "No privilege escalation risks detected" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                description: $description,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')
    fi

    if [[ -n "$output_json" ]]; then
        echo "$finding" > "$output_json"
        log INFO "Privilege escalation check saved: $output_json"
    fi

    echo "$finding"
    return 0
}

# 危険なオプション検出
detect_dangerous_options() {
    local command="$1"
    local test_name="${2:-unknown}"
    local output_json="${3:-}"

    log INFO "Checking for dangerous command options"
    log DEBUG "Command: $command"

    local severity="$SEVERITY_INFO"
    local found_options=()

    # 危険なオプション
    local dangerous_options=(
        '--exec'
        '--eval'
        '--no-sandbox'
        '--allow-root'
        '--insecure'
        '--no-verify'
        '--skip-verify'
        '-f'  # force
    )

    for opt in "${dangerous_options[@]}"; do
        if [[ "$command" == *"$opt"* ]]; then
            found_options+=("$opt")
        fi
    done

    # 脆弱性判定
    if [[ ${#found_options[@]} -gt 0 ]]; then
        if [[ "$command" == *'--exec'* ]] || [[ "$command" == *'--eval'* ]] || [[ "$command" == *'--no-sandbox'* ]]; then
            severity="$SEVERITY_HIGH"
        elif [[ "$command" == *'--insecure'* ]] || [[ "$command" == *'--no-verify'* ]]; then
            severity="$SEVERITY_MEDIUM"
        else
            severity="$SEVERITY_LOW"
        fi

        local options_str
        options_str=$(printf '%s, ' "${found_options[@]}")
        options_str="${options_str%, }"

        local finding
        finding=$(jq -n \
            --arg type "dangerous_options" \
            --arg severity "$severity" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg options "$options_str" \
            --arg description "Dangerous options detected: ($options_str)" \
            --arg recommendation "Avoid using dangerous options, use safer alternatives, validate necessity" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                options_detected: $options,
                description: $description,
                recommendation: $recommendation,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')

        log WARN "Dangerous options detected: $severity - $options_str"
    else
        local finding
        finding=$(jq -n \
            --arg type "dangerous_options" \
            --arg severity "$SEVERITY_INFO" \
            --arg command "$command" \
            --arg test_name "$test_name" \
            --arg description "No dangerous options detected" \
            '{
                type: $type,
                severity: $severity,
                command: $command,
                test_name: $test_name,
                description: $description,
                timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
            }')
    fi

    if [[ -n "$output_json" ]]; then
        echo "$finding" > "$output_json"
        log INFO "Dangerous options check saved: $output_json"
    fi

    echo "$finding"
    return 0
}

# 包括的セキュリティスキャン
comprehensive_security_scan() {
    local command="$1"
    local test_name="${2:-unknown}"
    local output_json="${3:-}"

    log INFO "Running comprehensive security scan"
    log DEBUG "Command: $command"
    log DEBUG "Test: $test_name"

    # 各チェック実行
    local injection_check
    injection_check=$(detect_command_injection "$command" "$test_name")

    local traversal_check
    traversal_check=$(detect_path_traversal "$command" "$test_name")

    local privilege_check
    privilege_check=$(check_privilege_escalation "$command" "$test_name")

    local options_check
    options_check=$(detect_dangerous_options "$command" "$test_name")

    # 最高重要度を判定
    local max_severity="$SEVERITY_INFO"
    local severities=(
        "$(echo "$injection_check" | jq -r '.severity')"
        "$(echo "$traversal_check" | jq -r '.severity')"
        "$(echo "$privilege_check" | jq -r '.severity')"
        "$(echo "$options_check" | jq -r '.severity')"
    )

    for sev in "${severities[@]}"; do
        case "$sev" in
            "$SEVERITY_CRITICAL") max_severity="$SEVERITY_CRITICAL" ;;
            "$SEVERITY_HIGH") [[ "$max_severity" != "$SEVERITY_CRITICAL" ]] && max_severity="$SEVERITY_HIGH" ;;
            "$SEVERITY_MEDIUM") [[ "$max_severity" != "$SEVERITY_CRITICAL" && "$max_severity" != "$SEVERITY_HIGH" ]] && max_severity="$SEVERITY_MEDIUM" ;;
            "$SEVERITY_LOW") [[ "$max_severity" == "$SEVERITY_INFO" ]] && max_severity="$SEVERITY_LOW" ;;
        esac
    done

    # 統合結果
    local scan_result
    scan_result=$(jq -n \
        --arg command "$command" \
        --arg test_name "$test_name" \
        --arg max_severity "$max_severity" \
        --argjson injection "$injection_check" \
        --argjson traversal "$traversal_check" \
        --argjson privilege "$privilege_check" \
        --argjson options "$options_check" \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        '{
            command: $command,
            test_name: $test_name,
            scan_timestamp: $timestamp,
            overall_severity: $max_severity,
            findings: {
                command_injection: $injection,
                path_traversal: $traversal,
                privilege_escalation: $privilege,
                dangerous_options: $options
            }
        }')

    if [[ -n "$output_json" ]]; then
        echo "$scan_result" > "$output_json"
        log INFO "Comprehensive security scan saved: $output_json"
    fi

    # サマリー表示
    log INFO "Security scan completed: overall severity = $max_severity"

    echo "$scan_result"
    return 0
}

# セキュリティファインディングをデータベースに保存
save_finding_to_db() {
    local finding_json="$1"
    local db_path="${2:-$SECURITY_DB_PATH}"

    log DEBUG "Saving security finding to database"

    # データベース存在確認
    if [[ ! -f "$db_path" ]]; then
        initialize_security_db "$db_path"
    fi

    # ファインディング追加
    local updated_db
    updated_db=$(jq --argjson finding "$finding_json" \
        '.findings += [$finding] | .metadata.scan_count += 1' "$db_path")

    echo "$updated_db" > "$db_path"

    log DEBUG "Security finding saved to database"
    return 0
}

# セキュリティスキャン統計取得
get_security_statistics() {
    local db_path="${1:-$SECURITY_DB_PATH}"

    log INFO "Retrieving security statistics"

    if [[ ! -f "$db_path" ]]; then
        log ERROR "Security database not found: $db_path"
        return 1
    fi

    # 統計計算
    local stats
    stats=$(jq '{
        total_scans: .metadata.scan_count,
        total_findings: (.findings | length),
        by_severity: {
            critical: (.findings | map(select(.overall_severity == "critical")) | length),
            high: (.findings | map(select(.overall_severity == "high")) | length),
            medium: (.findings | map(select(.overall_severity == "medium")) | length),
            low: (.findings | map(select(.overall_severity == "low")) | length),
            info: (.findings | map(select(.overall_severity == "info")) | length)
        }
    }' "$db_path")

    echo "$stats"
    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <subcommand> [args...]" >&2
        echo "" >&2
        echo "Subcommands:" >&2
        echo "  init [db-path]                              Initialize security database" >&2
        echo "  injection <command> [test-name] [output]    Check command injection" >&2
        echo "  traversal <command> [test-name] [output]    Check path traversal" >&2
        echo "  privilege <command> [test-name] [output]    Check privilege escalation" >&2
        echo "  options <command> [test-name] [output]      Check dangerous options" >&2
        echo "  scan <command> [test-name] [output]         Comprehensive security scan" >&2
        echo "  stats [db-path]                             Show security statistics" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 init ./security-findings.json" >&2
        echo "  $0 scan 'rm -rf /tmp/test' 'test1' ./scan-result.json" >&2
        echo "  $0 stats" >&2
        exit 1
    fi

    subcommand="$1"
    shift

    case "$subcommand" in
        init)
            db_path="${1:-$SECURITY_DB_PATH}"
            initialize_security_db "$db_path"
            ;;
        injection)
            if [[ $# -lt 1 ]]; then
                log ERROR "injection requires: <command> [test-name] [output-json]"
                exit 1
            fi
            detect_command_injection "$@"
            ;;
        traversal)
            if [[ $# -lt 1 ]]; then
                log ERROR "traversal requires: <command> [test-name] [output-json]"
                exit 1
            fi
            detect_path_traversal "$@"
            ;;
        privilege)
            if [[ $# -lt 1 ]]; then
                log ERROR "privilege requires: <command> [test-name] [output-json]"
                exit 1
            fi
            check_privilege_escalation "$@"
            ;;
        options)
            if [[ $# -lt 1 ]]; then
                log ERROR "options requires: <command> [test-name] [output-json]"
                exit 1
            fi
            detect_dangerous_options "$@"
            ;;
        scan)
            if [[ $# -lt 1 ]]; then
                log ERROR "scan requires: <command> [test-name] [output-json]"
                exit 1
            fi
            comprehensive_security_scan "$@"
            ;;
        stats)
            db_path="${1:-$SECURITY_DB_PATH}"
            get_security_statistics "$db_path"
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
