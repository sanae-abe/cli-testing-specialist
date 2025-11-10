#!/usr/bin/env bash
#
# security-reporter.sh - セキュリティスキャンレポート生成エンジン
# CLI Testing Specialist Agent v2.2.0
#
# 機能:
# - HTML形式のセキュリティレポート生成（Bootstrap 5）
# - Markdown形式のレポート生成（絵文字+ASCII進捗バー）
# - JSON形式のレポート生成
# - 重要度別脆弱性分類・推奨事項表示
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in security-reporter.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# HTMLレポート生成
generate_html_report() {
    local scan_result_json="$1"
    local output_html="$2"

    log INFO "Generating HTML security report"
    log DEBUG "Input: $scan_result_json"
    log DEBUG "Output: $output_html"

    # JSON読み込み
    if [[ ! -f "$scan_result_json" ]]; then
        log ERROR "Scan result JSON not found: $scan_result_json"
        return 1
    fi

    local scan_data
    scan_data=$(<"$scan_result_json")

    # メトリクス抽出
    local command
    command=$(echo "$scan_data" | jq -r '.command // "unknown"')

    local test_name
    test_name=$(echo "$scan_data" | jq -r '.test_name // "unknown"')

    local scan_timestamp
    scan_timestamp=$(echo "$scan_data" | jq -r '.scan_timestamp // "unknown"')

    local overall_severity
    overall_severity=$(echo "$scan_data" | jq -r '.overall_severity // "info"')

    # 各ファインディング抽出
    local injection_severity
    injection_severity=$(echo "$scan_data" | jq -r '.findings.command_injection.severity // "info"')

    local injection_desc
    injection_desc=$(echo "$scan_data" | jq -r '.findings.command_injection.description // "No data"')

    local traversal_severity
    traversal_severity=$(echo "$scan_data" | jq -r '.findings.path_traversal.severity // "info"')

    local traversal_desc
    traversal_desc=$(echo "$scan_data" | jq -r '.findings.path_traversal.description // "No data"')

    local privilege_severity
    privilege_severity=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.severity // "info"')

    local privilege_desc
    privilege_desc=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.description // "No data"')

    local options_severity
    options_severity=$(echo "$scan_data" | jq -r '.findings.dangerous_options.severity // "info"')

    local options_desc
    options_desc=$(echo "$scan_data" | jq -r '.findings.dangerous_options.description // "No data"')

    # 重要度カウント
    local critical_count=0
    local high_count=0
    local medium_count=0
    local low_count=0
    local info_count=0

    for severity in "$injection_severity" "$traversal_severity" "$privilege_severity" "$options_severity"; do
        case "$severity" in
            critical) ((critical_count++)) || true ;;
            high) ((high_count++)) || true ;;
            medium) ((medium_count++)) || true ;;
            low) ((low_count++)) || true ;;
            info) ((info_count++)) || true ;;
        esac
    done

    local total_findings=$((critical_count + high_count + medium_count + low_count))

    # Overall severity badge
    local overall_badge_class
    case "$overall_severity" in
        critical) overall_badge_class="bg-danger" ;;
        high) overall_badge_class="bg-warning" ;;
        medium) overall_badge_class="bg-info" ;;
        low) overall_badge_class="bg-secondary" ;;
        *) overall_badge_class="bg-success" ;;
    esac

    # HTMLエスケープ関数
    html_escape() {
        echo "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g; s/'"'"'/\&#39;/g'
    }

    local command_escaped
    command_escaped=$(html_escape "$command")

    local test_name_escaped
    test_name_escaped=$(html_escape "$test_name")

    # HTML生成
    cat > "$output_html" <<'EOF'
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Security Scan Report</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
            padding: 2rem 0;
        }
        .container {
            max-width: 1200px;
        }
        .severity-badge {
            font-size: 0.9rem;
            padding: 0.3rem 0.6rem;
            border-radius: 0.25rem;
        }
        .finding-card {
            margin-bottom: 1rem;
            border-left: 4px solid;
        }
        .finding-card.critical {
            border-left-color: #dc3545;
        }
        .finding-card.high {
            border-left-color: #fd7e14;
        }
        .finding-card.medium {
            border-left-color: #0dcaf0;
        }
        .finding-card.low {
            border-left-color: #6c757d;
        }
        .finding-card.info {
            border-left-color: #198754;
        }
        .stat-card {
            text-align: center;
            padding: 1.5rem;
            border-radius: 0.5rem;
            margin-bottom: 1rem;
        }
        .stat-value {
            font-size: 2.5rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
        }
        .stat-label {
            font-size: 0.9rem;
            color: #6c757d;
            text-transform: uppercase;
        }
        .recommendation {
            background-color: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 1rem;
            margin-top: 0.5rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="row mb-4">
            <div class="col-12">
                <h1 class="display-4">🔒 Security Scan Report</h1>
                <p class="lead text-muted">CLI Testing Specialist Agent v2.2.0</p>
            </div>
        </div>

        <!-- Command Information -->
        <div class="card mb-4">
            <div class="card-header bg-primary text-white">
                <h5 class="mb-0">📋 Scan Information</h5>
            </div>
            <div class="card-body">
                <table class="table table-borderless mb-0">
                    <tbody>
                        <tr>
                            <th style="width: 200px;">Command:</th>
                            <td><code>COMMAND_PLACEHOLDER</code></td>
                        </tr>
                        <tr>
                            <th>Test Name:</th>
                            <td>TEST_NAME_PLACEHOLDER</td>
                        </tr>
                        <tr>
                            <th>Scan Timestamp:</th>
                            <td>SCAN_TIMESTAMP_PLACEHOLDER</td>
                        </tr>
                        <tr>
                            <th>Overall Severity:</th>
                            <td><span class="badge OVERALL_BADGE_CLASS_PLACEHOLDER severity-badge">OVERALL_SEVERITY_PLACEHOLDER</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Statistics -->
        <div class="row mb-4">
            <div class="col-md-3">
                <div class="stat-card bg-white border">
                    <div class="stat-value text-danger">CRITICAL_COUNT_PLACEHOLDER</div>
                    <div class="stat-label">Critical</div>
                </div>
            </div>
            <div class="col-md-3">
                <div class="stat-card bg-white border">
                    <div class="stat-value text-warning">HIGH_COUNT_PLACEHOLDER</div>
                    <div class="stat-label">High</div>
                </div>
            </div>
            <div class="col-md-3">
                <div class="stat-card bg-white border">
                    <div class="stat-value text-info">MEDIUM_COUNT_PLACEHOLDER</div>
                    <div class="stat-label">Medium</div>
                </div>
            </div>
            <div class="col-md-3">
                <div class="stat-card bg-white border">
                    <div class="stat-value text-secondary">LOW_COUNT_PLACEHOLDER</div>
                    <div class="stat-label">Low</div>
                </div>
            </div>
        </div>

        <!-- Findings -->
        <div class="card mb-4">
            <div class="card-header bg-secondary text-white">
                <h5 class="mb-0">🔍 Security Findings</h5>
            </div>
            <div class="card-body">

                <!-- Command Injection -->
                <div class="finding-card card INJECTION_SEVERITY_PLACEHOLDER">
                    <div class="card-body">
                        <h6 class="card-title">
                            <span class="badge INJECTION_BADGE_CLASS_PLACEHOLDER severity-badge">INJECTION_SEVERITY_UPPER_PLACEHOLDER</span>
                            Command Injection Detection
                        </h6>
                        <p class="card-text mb-2">INJECTION_DESC_PLACEHOLDER</p>
                        INJECTION_RECOMMENDATION_PLACEHOLDER
                    </div>
                </div>

                <!-- Path Traversal -->
                <div class="finding-card card TRAVERSAL_SEVERITY_PLACEHOLDER">
                    <div class="card-body">
                        <h6 class="card-title">
                            <span class="badge TRAVERSAL_BADGE_CLASS_PLACEHOLDER severity-badge">TRAVERSAL_SEVERITY_UPPER_PLACEHOLDER</span>
                            Path Traversal Detection
                        </h6>
                        <p class="card-text mb-2">TRAVERSAL_DESC_PLACEHOLDER</p>
                        TRAVERSAL_RECOMMENDATION_PLACEHOLDER
                    </div>
                </div>

                <!-- Privilege Escalation -->
                <div class="finding-card card PRIVILEGE_SEVERITY_PLACEHOLDER">
                    <div class="card-body">
                        <h6 class="card-title">
                            <span class="badge PRIVILEGE_BADGE_CLASS_PLACEHOLDER severity-badge">PRIVILEGE_SEVERITY_UPPER_PLACEHOLDER</span>
                            Privilege Escalation Check
                        </h6>
                        <p class="card-text mb-2">PRIVILEGE_DESC_PLACEHOLDER</p>
                        PRIVILEGE_RECOMMENDATION_PLACEHOLDER
                    </div>
                </div>

                <!-- Dangerous Options -->
                <div class="finding-card card OPTIONS_SEVERITY_PLACEHOLDER">
                    <div class="card-body">
                        <h6 class="card-title">
                            <span class="badge OPTIONS_BADGE_CLASS_PLACEHOLDER severity-badge">OPTIONS_SEVERITY_UPPER_PLACEHOLDER</span>
                            Dangerous Options Detection
                        </h6>
                        <p class="card-text mb-2">OPTIONS_DESC_PLACEHOLDER</p>
                        OPTIONS_RECOMMENDATION_PLACEHOLDER
                    </div>
                </div>

            </div>
        </div>

        <!-- Footer -->
        <div class="text-center text-muted mt-4">
            <small>Generated by CLI Testing Specialist Agent v2.2.0 - Phase 2 Week 5-8</small>
        </div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
</body>
</html>
EOF

    # プレースホルダー置換（sedを使用）
    # Command & Test Name
    sed -i.bak "s|COMMAND_PLACEHOLDER|$command_escaped|g" "$output_html"
    sed -i.bak "s|TEST_NAME_PLACEHOLDER|$test_name_escaped|g" "$output_html"
    sed -i.bak "s|SCAN_TIMESTAMP_PLACEHOLDER|$scan_timestamp|g" "$output_html"

    # Overall Severity
    sed -i.bak "s|OVERALL_BADGE_CLASS_PLACEHOLDER|$overall_badge_class|g" "$output_html"
    sed -i.bak "s|OVERALL_SEVERITY_PLACEHOLDER|$overall_severity|g" "$output_html"

    # Counts
    sed -i.bak "s|CRITICAL_COUNT_PLACEHOLDER|$critical_count|g" "$output_html"
    sed -i.bak "s|HIGH_COUNT_PLACEHOLDER|$high_count|g" "$output_html"
    sed -i.bak "s|MEDIUM_COUNT_PLACEHOLDER|$medium_count|g" "$output_html"
    sed -i.bak "s|LOW_COUNT_PLACEHOLDER|$low_count|g" "$output_html"

    # Command Injection
    local injection_badge_class
    case "$injection_severity" in
        critical) injection_badge_class="bg-danger" ;;
        high) injection_badge_class="bg-warning" ;;
        medium) injection_badge_class="bg-info" ;;
        low) injection_badge_class="bg-secondary" ;;
        *) injection_badge_class="bg-success" ;;
    esac

    local injection_severity_upper
    injection_severity_upper=$(echo "$injection_severity" | tr '[:lower:]' '[:upper:]')

    local injection_desc_escaped
    injection_desc_escaped=$(html_escape "$injection_desc")

    local injection_recommendation=""
    if echo "$scan_data" | jq -e '.findings.command_injection.recommendation' > /dev/null 2>&1; then
        local injection_rec
        injection_rec=$(echo "$scan_data" | jq -r '.findings.command_injection.recommendation // ""')
        if [[ -n "$injection_rec" ]]; then
            local injection_rec_escaped
            injection_rec_escaped=$(html_escape "$injection_rec")
            injection_recommendation="<div class=\"recommendation\"><strong>💡 Recommendation:</strong> $injection_rec_escaped</div>"
        fi
    fi

    sed -i.bak "s|INJECTION_SEVERITY_PLACEHOLDER|$injection_severity|g" "$output_html"
    sed -i.bak "s|INJECTION_BADGE_CLASS_PLACEHOLDER|$injection_badge_class|g" "$output_html"
    sed -i.bak "s|INJECTION_SEVERITY_UPPER_PLACEHOLDER|$injection_severity_upper|g" "$output_html"
    sed -i.bak "s|INJECTION_DESC_PLACEHOLDER|$injection_desc_escaped|g" "$output_html"
    sed -i.bak "s|INJECTION_RECOMMENDATION_PLACEHOLDER|$injection_recommendation|g" "$output_html"

    # Path Traversal
    local traversal_badge_class
    case "$traversal_severity" in
        critical) traversal_badge_class="bg-danger" ;;
        high) traversal_badge_class="bg-warning" ;;
        medium) traversal_badge_class="bg-info" ;;
        low) traversal_badge_class="bg-secondary" ;;
        *) traversal_badge_class="bg-success" ;;
    esac

    local traversal_severity_upper
    traversal_severity_upper=$(echo "$traversal_severity" | tr '[:lower:]' '[:upper:]')

    local traversal_desc_escaped
    traversal_desc_escaped=$(html_escape "$traversal_desc")

    local traversal_recommendation=""
    if echo "$scan_data" | jq -e '.findings.path_traversal.recommendation' > /dev/null 2>&1; then
        local traversal_rec
        traversal_rec=$(echo "$scan_data" | jq -r '.findings.path_traversal.recommendation // ""')
        if [[ -n "$traversal_rec" ]]; then
            local traversal_rec_escaped
            traversal_rec_escaped=$(html_escape "$traversal_rec")
            traversal_recommendation="<div class=\"recommendation\"><strong>💡 Recommendation:</strong> $traversal_rec_escaped</div>"
        fi
    fi

    sed -i.bak "s|TRAVERSAL_SEVERITY_PLACEHOLDER|$traversal_severity|g" "$output_html"
    sed -i.bak "s|TRAVERSAL_BADGE_CLASS_PLACEHOLDER|$traversal_badge_class|g" "$output_html"
    sed -i.bak "s|TRAVERSAL_SEVERITY_UPPER_PLACEHOLDER|$traversal_severity_upper|g" "$output_html"
    sed -i.bak "s|TRAVERSAL_DESC_PLACEHOLDER|$traversal_desc_escaped|g" "$output_html"
    sed -i.bak "s|TRAVERSAL_RECOMMENDATION_PLACEHOLDER|$traversal_recommendation|g" "$output_html"

    # Privilege Escalation
    local privilege_badge_class
    case "$privilege_severity" in
        critical) privilege_badge_class="bg-danger" ;;
        high) privilege_badge_class="bg-warning" ;;
        medium) privilege_badge_class="bg-info" ;;
        low) privilege_badge_class="bg-secondary" ;;
        *) privilege_badge_class="bg-success" ;;
    esac

    local privilege_severity_upper
    privilege_severity_upper=$(echo "$privilege_severity" | tr '[:lower:]' '[:upper:]')

    local privilege_desc_escaped
    privilege_desc_escaped=$(html_escape "$privilege_desc")

    local privilege_recommendation=""
    if echo "$scan_data" | jq -e '.findings.privilege_escalation.recommendation' > /dev/null 2>&1; then
        local privilege_rec
        privilege_rec=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.recommendation // ""')
        if [[ -n "$privilege_rec" ]]; then
            local privilege_rec_escaped
            privilege_rec_escaped=$(html_escape "$privilege_rec")
            privilege_recommendation="<div class=\"recommendation\"><strong>💡 Recommendation:</strong> $privilege_rec_escaped</div>"
        fi
    fi

    sed -i.bak "s|PRIVILEGE_SEVERITY_PLACEHOLDER|$privilege_severity|g" "$output_html"
    sed -i.bak "s|PRIVILEGE_BADGE_CLASS_PLACEHOLDER|$privilege_badge_class|g" "$output_html"
    sed -i.bak "s|PRIVILEGE_SEVERITY_UPPER_PLACEHOLDER|$privilege_severity_upper|g" "$output_html"
    sed -i.bak "s|PRIVILEGE_DESC_PLACEHOLDER|$privilege_desc_escaped|g" "$output_html"
    sed -i.bak "s|PRIVILEGE_RECOMMENDATION_PLACEHOLDER|$privilege_recommendation|g" "$output_html"

    # Dangerous Options
    local options_badge_class
    case "$options_severity" in
        critical) options_badge_class="bg-danger" ;;
        high) options_badge_class="bg-warning" ;;
        medium) options_badge_class="bg-info" ;;
        low) options_badge_class="bg-secondary" ;;
        *) options_badge_class="bg-success" ;;
    esac

    local options_severity_upper
    options_severity_upper=$(echo "$options_severity" | tr '[:lower:]' '[:upper:]')

    local options_desc_escaped
    options_desc_escaped=$(html_escape "$options_desc")

    local options_recommendation=""
    if echo "$scan_data" | jq -e '.findings.dangerous_options.recommendation' > /dev/null 2>&1; then
        local options_rec
        options_rec=$(echo "$scan_data" | jq -r '.findings.dangerous_options.recommendation // ""')
        if [[ -n "$options_rec" ]]; then
            local options_rec_escaped
            options_rec_escaped=$(html_escape "$options_rec")
            options_recommendation="<div class=\"recommendation\"><strong>💡 Recommendation:</strong> $options_rec_escaped</div>"
        fi
    fi

    sed -i.bak "s|OPTIONS_SEVERITY_PLACEHOLDER|$options_severity|g" "$output_html"
    sed -i.bak "s|OPTIONS_BADGE_CLASS_PLACEHOLDER|$options_badge_class|g" "$output_html"
    sed -i.bak "s|OPTIONS_SEVERITY_UPPER_PLACEHOLDER|$options_severity_upper|g" "$output_html"
    sed -i.bak "s|OPTIONS_DESC_PLACEHOLDER|$options_desc_escaped|g" "$output_html"
    sed -i.bak "s|OPTIONS_RECOMMENDATION_PLACEHOLDER|$options_recommendation|g" "$output_html"

    # バックアップファイル削除
    rm -f "$output_html.bak"

    log INFO "HTML report generated: $output_html"
    return 0
}

# Markdownレポート生成
generate_markdown_report() {
    local scan_result_json="$1"
    local output_md="$2"

    log INFO "Generating Markdown security report"
    log DEBUG "Input: $scan_result_json"
    log DEBUG "Output: $output_md"

    # JSON読み込み
    if [[ ! -f "$scan_result_json" ]]; then
        log ERROR "Scan result JSON not found: $scan_result_json"
        return 1
    fi

    local scan_data
    scan_data=$(<"$scan_result_json")

    # メトリクス抽出
    local command
    command=$(echo "$scan_data" | jq -r '.command // "unknown"')

    local test_name
    test_name=$(echo "$scan_data" | jq -r '.test_name // "unknown"')

    local scan_timestamp
    scan_timestamp=$(echo "$scan_data" | jq -r '.scan_timestamp // "unknown"')

    local overall_severity
    overall_severity=$(echo "$scan_data" | jq -r '.overall_severity // "info"')

    # 重要度別絵文字
    local severity_emoji=""
    case "$overall_severity" in
        critical) severity_emoji="🔴" ;;
        high) severity_emoji="🟠" ;;
        medium) severity_emoji="🟡" ;;
        low) severity_emoji="🔵" ;;
        *) severity_emoji="🟢" ;;
    esac

    # 各ファインディング抽出
    local injection_severity
    injection_severity=$(echo "$scan_data" | jq -r '.findings.command_injection.severity // "info"')
    local injection_desc
    injection_desc=$(echo "$scan_data" | jq -r '.findings.command_injection.description // "No data"')
    local injection_rec
    injection_rec=$(echo "$scan_data" | jq -r '.findings.command_injection.recommendation // ""')

    local traversal_severity
    traversal_severity=$(echo "$scan_data" | jq -r '.findings.path_traversal.severity // "info"')
    local traversal_desc
    traversal_desc=$(echo "$scan_data" | jq -r '.findings.path_traversal.description // "No data"')
    local traversal_rec
    traversal_rec=$(echo "$scan_data" | jq -r '.findings.path_traversal.recommendation // ""')

    local privilege_severity
    privilege_severity=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.severity // "info"')
    local privilege_desc
    privilege_desc=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.description // "No data"')
    local privilege_rec
    privilege_rec=$(echo "$scan_data" | jq -r '.findings.privilege_escalation.recommendation // ""')

    local options_severity
    options_severity=$(echo "$scan_data" | jq -r '.findings.dangerous_options.severity // "info"')
    local options_desc
    options_desc=$(echo "$scan_data" | jq -r '.findings.dangerous_options.description // "No data"')
    local options_rec
    options_rec=$(echo "$scan_data" | jq -r '.findings.dangerous_options.recommendation // ""')

    # 重要度カウント
    local critical_count=0
    local high_count=0
    local medium_count=0
    local low_count=0
    local info_count=0

    for severity in "$injection_severity" "$traversal_severity" "$privilege_severity" "$options_severity"; do
        case "$severity" in
            critical) ((critical_count++)) || true ;;
            high) ((high_count++)) || true ;;
            medium) ((medium_count++)) || true ;;
            low) ((low_count++)) || true ;;
            info) ((info_count++)) || true ;;
        esac
    done

    # 絵文字取得関数
    get_severity_emoji() {
        case "$1" in
            critical) echo "🔴" ;;
            high) echo "🟠" ;;
            medium) echo "🟡" ;;
            low) echo "🔵" ;;
            *) echo "🟢" ;;
        esac
    }

    # Markdown生成
    cat > "$output_md" <<EOF
# 🔒 Security Scan Report

**CLI Testing Specialist Agent v2.2.0 - Phase 2 Week 5-8**

---

## 📋 Scan Information

| Item | Value |
|------|-------|
| **Command** | \`$command\` |
| **Test Name** | $test_name |
| **Scan Timestamp** | $scan_timestamp |
| **Overall Severity** | $severity_emoji **$overall_severity** |

---

## 📊 Security Findings Summary

| Severity | Count |
|----------|-------|
| 🔴 **Critical** | $critical_count |
| 🟠 **High** | $high_count |
| 🟡 **Medium** | $medium_count |
| 🔵 **Low** | $low_count |
| 🟢 **Info** | $info_count |

---

## 🔍 Detailed Findings

### $(get_severity_emoji "$injection_severity") Command Injection Detection

**Severity:** $injection_severity

**Description:** $injection_desc

EOF

    # 推奨事項追加（Command Injection）
    if [[ -n "$injection_rec" ]]; then
        cat >> "$output_md" <<EOF
**💡 Recommendation:** $injection_rec

EOF
    fi

    cat >> "$output_md" <<EOF
---

### $(get_severity_emoji "$traversal_severity") Path Traversal Detection

**Severity:** $traversal_severity

**Description:** $traversal_desc

EOF

    # 推奨事項追加（Path Traversal）
    if [[ -n "$traversal_rec" ]]; then
        cat >> "$output_md" <<EOF
**💡 Recommendation:** $traversal_rec

EOF
    fi

    cat >> "$output_md" <<EOF
---

### $(get_severity_emoji "$privilege_severity") Privilege Escalation Check

**Severity:** $privilege_severity

**Description:** $privilege_desc

EOF

    # 推奨事項追加（Privilege Escalation）
    if [[ -n "$privilege_rec" ]]; then
        cat >> "$output_md" <<EOF
**💡 Recommendation:** $privilege_rec

EOF
    fi

    cat >> "$output_md" <<EOF
---

### $(get_severity_emoji "$options_severity") Dangerous Options Detection

**Severity:** $options_severity

**Description:** $options_desc

EOF

    # 推奨事項追加（Dangerous Options）
    if [[ -n "$options_rec" ]]; then
        cat >> "$output_md" <<EOF
**💡 Recommendation:** $options_rec

EOF
    fi

    cat >> "$output_md" <<EOF
---

## 🛡️ Overall Assessment

EOF

    # 全体評価
    if [[ "$overall_severity" == "critical" ]] || [[ "$overall_severity" == "high" ]]; then
        cat >> "$output_md" <<EOF
⚠️ **This command contains HIGH-RISK security issues that require immediate attention.**

Immediate remediation is strongly recommended before production use.

EOF
    elif [[ "$overall_severity" == "medium" ]]; then
        cat >> "$output_md" <<EOF
⚠️ **This command contains MEDIUM-RISK security issues.**

Review and address the identified issues before production deployment.

EOF
    elif [[ "$overall_severity" == "low" ]]; then
        cat >> "$output_md" <<EOF
✅ **This command has LOW-RISK findings.**

Consider addressing the minor issues for improved security posture.

EOF
    else
        cat >> "$output_md" <<EOF
✅ **No significant security issues detected.**

The command appears to follow security best practices.

EOF
    fi

    cat >> "$output_md" <<EOF

---

*Generated by CLI Testing Specialist Agent v2.2.0 - Phase 2 Week 5-8*
EOF

    log INFO "Markdown report generated: $output_md"
    return 0
}

# JSONレポート生成
generate_json_report() {
    local scan_result_json="$1"
    local output_json="$2"

    log INFO "Generating JSON security report"
    log DEBUG "Input: $scan_result_json"
    log DEBUG "Output: $output_json"

    # JSON読み込み
    if [[ ! -f "$scan_result_json" ]]; then
        log ERROR "Scan result JSON not found: $scan_result_json"
        return 1
    fi

    local scan_data
    scan_data=$(<"$scan_result_json")

    # 統計追加
    local critical_count=0
    local high_count=0
    local medium_count=0
    local low_count=0
    local info_count=0

    local severities=(
        "$(echo "$scan_data" | jq -r '.findings.command_injection.severity // "info"')"
        "$(echo "$scan_data" | jq -r '.findings.path_traversal.severity // "info"')"
        "$(echo "$scan_data" | jq -r '.findings.privilege_escalation.severity // "info"')"
        "$(echo "$scan_data" | jq -r '.findings.dangerous_options.severity // "info"')"
    )

    for severity in "${severities[@]}"; do
        case "$severity" in
            critical) ((critical_count++)) || true ;;
            high) ((high_count++)) || true ;;
            medium) ((medium_count++)) || true ;;
            low) ((low_count++)) || true ;;
            info) ((info_count++)) || true ;;
        esac
    done

    # 統計付きJSON生成
    local enhanced_json
    enhanced_json=$(echo "$scan_data" | jq \
        --argjson critical "$critical_count" \
        --argjson high "$high_count" \
        --argjson medium "$medium_count" \
        --argjson low "$low_count" \
        --argjson info "$info_count" \
        '. + {
            statistics: {
                by_severity: {
                    critical: $critical,
                    high: $high,
                    medium: $medium,
                    low: $low,
                    info: $info
                },
                total_findings: ($critical + $high + $medium + $low)
            }
        }')

    # 整形して出力
    echo "$enhanced_json" | jq '.' > "$output_json"

    log INFO "JSON report generated: $output_json"
    return 0
}

# メインレポート生成関数
generate_security_report() {
    local scan_result_json="$1"
    local output_file="$2"
    local format="${3:-html}"

    log INFO "Generating security report (format: $format)"

    case "$format" in
        html)
            generate_html_report "$scan_result_json" "$output_file"
            ;;
        markdown|md)
            generate_markdown_report "$scan_result_json" "$output_file"
            ;;
        json)
            generate_json_report "$scan_result_json" "$output_file"
            ;;
        all)
            # 拡張子を除いたベース名取得
            local base_name="${output_file%.*}"

            generate_html_report "$scan_result_json" "${base_name}.html"
            generate_markdown_report "$scan_result_json" "${base_name}.md"
            generate_json_report "$scan_result_json" "${base_name}.json"

            log INFO "All format reports generated: ${base_name}.*"
            ;;
        *)
            log ERROR "Unknown format: $format"
            log ERROR "Supported formats: html, markdown, json, all"
            return 1
            ;;
    esac

    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 2 ]]; then
        echo "Usage: $0 <scan-result-json> <output-file> [format]" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <scan-result-json>  Path to security scan result JSON file" >&2
        echo "  <output-file>       Output report file path" >&2
        echo "  [format]            Report format: html|markdown|json|all (default: html)" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 ./scan-result.json ./security-report.html html" >&2
        echo "  $0 ./scan-result.json ./security-report.md markdown" >&2
        echo "  $0 ./scan-result.json ./security-report.json json" >&2
        echo "  $0 ./scan-result.json ./security-report all" >&2
        exit 1
    fi

    scan_result_json="$1"
    output_file="$2"
    format="${3:-html}"

    generate_security_report "$scan_result_json" "$output_file" "$format"

    exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        log INFO "Security report generation completed successfully"
    else
        log ERROR "Security report generation failed (exit code: $exit_code)"
    fi

    exit $exit_code
fi
