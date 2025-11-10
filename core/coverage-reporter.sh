#!/usr/bin/env bash
#
# coverage-reporter.sh - カバレッジレポート生成エンジン
# CLI Testing Specialist Agent v2.1.0
#
# 機能:
# - カバレッジ分析結果からHTMLレポート生成
# - D3.jsヒートマップ生成
# - JSONレポート生成
# - サマリー表示
#

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'log_error_with_trace "Error at line $LINENO in coverage-reporter.sh"' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh"
source "$SCRIPT_DIR/validator.sh"

# テンプレートディレクトリ
TEMPLATE_DIR="${AGENT_ROOT}/templates"

# HTMLレポート生成
generate_html_report() {
    local coverage_json="$1"
    local output_html="$2"

    log INFO "Generating HTML coverage report"
    log DEBUG "Input JSON: $coverage_json"
    log DEBUG "Output HTML: $output_html"

    # 入力検証
    if [[ ! -f "$coverage_json" ]]; then
        log ERROR "Coverage JSON not found: $coverage_json"
        return 1
    fi

    # JSONバリデーション
    if ! jq empty "$coverage_json" 2>/dev/null; then
        log ERROR "Invalid JSON format: $coverage_json"
        return 1
    fi

    # テンプレートファイル確認
    local template_file="$TEMPLATE_DIR/coverage-report.html"
    if [[ ! -f "$template_file" ]]; then
        log WARN "Template not found: $template_file"
        log INFO "Generating HTML from embedded template"
        generate_html_from_embedded_template "$coverage_json" "$output_html"
        return $?
    fi

    # JSONデータ読み込み
    local coverage_data
    coverage_data=$(<"$coverage_json")

    # テンプレート処理（JavaScriptにJSONデータを埋め込み）
    sed -e "s|{{COVERAGE_DATA}}|$coverage_data|g" \
        "$template_file" > "$output_html"

    log INFO "HTML coverage report generated: $output_html"
    return 0
}

# 埋め込みテンプレートからHTML生成
generate_html_from_embedded_template() {
    local coverage_json="$1"
    local output_html="$2"

    log DEBUG "Using embedded HTML template"

    # JSONデータ読み込み
    local cli_binary
    cli_binary=$(jq -r '.cli_binary' "$coverage_json")

    local coverage_rate
    coverage_rate=$(jq -r '.coverage_rate' "$coverage_json")

    local total_features
    total_features=$(jq -r '.total_features' "$coverage_json")

    local covered_features
    covered_features=$(jq -r '.covered_features' "$coverage_json")

    local subcommand_total
    subcommand_total=$(jq -r '.summary.subcommands.total' "$coverage_json")

    local subcommand_covered
    subcommand_covered=$(jq -r '.summary.subcommands.covered' "$coverage_json")

    local subcommand_rate
    subcommand_rate=$(jq -r '.summary.subcommands.coverage_rate' "$coverage_json")

    local option_total
    option_total=$(jq -r '.summary.options.total' "$coverage_json")

    local option_covered
    option_covered=$(jq -r '.summary.options.covered' "$coverage_json")

    local option_rate
    option_rate=$(jq -r '.summary.options.coverage_rate' "$coverage_json")

    local uncovered_subcommands
    uncovered_subcommands=$(jq -r '.uncovered.subcommands[]? // empty' "$coverage_json" | sed 's/^/- /' || echo "None")

    local uncovered_options
    uncovered_options=$(jq -r '.uncovered.options[]? // empty' "$coverage_json" | sed 's/^/- /' || echo "None")

    local recommendations
    recommendations=$(jq -r '.recommendations[] | "- [\(.priority | ascii_upcase)] \(.target): \(.suggestion)"' "$coverage_json" 2>/dev/null || echo "None")

    local generated_at
    generated_at=$(jq -r '.generated_at' "$coverage_json")

    # 進捗バー幅計算
    local progress_width
    progress_width=$(awk -v rate="$coverage_rate" 'BEGIN {printf "%.0f", rate}')

    # 色判定（80%以上=緑、50-79%=黄、50%未満=赤）
    local progress_color="bg-danger"
    if (( $(echo "$coverage_rate >= 80" | bc -l) )); then
        progress_color="bg-success"
    elif (( $(echo "$coverage_rate >= 50" | bc -l) )); then
        progress_color="bg-warning"
    fi

    # HTML生成
    cat > "$output_html" <<'EOF_TEMPLATE'
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Coverage Report - CLI Testing Specialist</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #f8f9fa;
            padding: 20px;
        }
        .container {
            max-width: 1200px;
            background-color: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 0 20px rgba(0,0,0,0.1);
        }
        .header {
            border-bottom: 3px solid #007bff;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }
        .metric-card {
            border-left: 4px solid #007bff;
            padding: 15px;
            margin-bottom: 15px;
            background-color: #f8f9fa;
        }
        .badge-priority-high {
            background-color: #dc3545;
        }
        .badge-priority-medium {
            background-color: #ffc107;
            color: #000;
        }
        .badge-priority-low {
            background-color: #28a745;
        }
        pre {
            background-color: #f5f5f5;
            padding: 10px;
            border-radius: 5px;
            font-size: 0.9em;
        }
        .uncovered-item {
            font-family: monospace;
            background-color: #fff3cd;
            padding: 2px 6px;
            border-radius: 3px;
            margin: 2px;
            display: inline-block;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎯 Coverage Report</h1>
            <p class="text-muted">CLI Testing Specialist v2.1.0</p>
            <p><strong>CLI Binary:</strong> <code>{{CLI_BINARY}}</code></p>
            <p><strong>Generated:</strong> {{GENERATED_AT}}</p>
        </div>

        <!-- Overall Coverage -->
        <div class="mb-4">
            <h2>📊 Overall Coverage</h2>
            <div class="metric-card">
                <h3>{{COVERAGE_RATE}}%</h3>
                <div class="progress" style="height: 30px;">
                    <div class="progress-bar {{PROGRESS_COLOR}}" role="progressbar"
                         style="width: {{PROGRESS_WIDTH}}%;"
                         aria-valuenow="{{COVERAGE_RATE}}" aria-valuemin="0" aria-valuemax="100">
                        {{COVERED_FEATURES}} / {{TOTAL_FEATURES}} features
                    </div>
                </div>
            </div>
        </div>

        <!-- Breakdown -->
        <div class="mb-4">
            <h2>📈 Breakdown</h2>
            <div class="row">
                <div class="col-md-6">
                    <div class="metric-card">
                        <h5>Subcommands</h5>
                        <p class="mb-1"><strong>{{SUBCOMMAND_RATE}}%</strong></p>
                        <p class="text-muted">{{SUBCOMMAND_COVERED}} / {{SUBCOMMAND_TOTAL}} covered</p>
                        <div class="progress">
                            <div class="progress-bar bg-info" role="progressbar"
                                 style="width: {{SUBCOMMAND_RATE}}%;">
                            </div>
                        </div>
                    </div>
                </div>
                <div class="col-md-6">
                    <div class="metric-card">
                        <h5>Options</h5>
                        <p class="mb-1"><strong>{{OPTION_RATE}}%</strong></p>
                        <p class="text-muted">{{OPTION_COVERED}} / {{OPTION_TOTAL}} covered</p>
                        <div class="progress">
                            <div class="progress-bar bg-info" role="progressbar"
                                 style="width: {{OPTION_RATE}}%;">
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Uncovered Areas -->
        <div class="mb-4">
            <h2>❌ Uncovered Areas</h2>
            <div class="row">
                <div class="col-md-6">
                    <h5>Subcommands</h5>
                    <pre>UNCOVERED_SUBCOMMANDS_PLACEHOLDER</pre>
                </div>
                <div class="col-md-6">
                    <h5>Options</h5>
                    <pre>UNCOVERED_OPTIONS_PLACEHOLDER</pre>
                </div>
            </div>
        </div>

        <!-- Recommendations -->
        <div class="mb-4">
            <h2>💡 Recommendations</h2>
            <pre>RECOMMENDATIONS_PLACEHOLDER</pre>
        </div>

        <!-- Footer -->
        <div class="text-center text-muted mt-5 pt-3 border-top">
            <p>Generated by <strong>CLI Testing Specialist</strong> | Phase 2 v2.1.0</p>
        </div>
    </div>
</body>
</html>
EOF_TEMPLATE

    # プレースホルダー置換
    sed -i '' \
        -e "s|{{CLI_BINARY}}|$cli_binary|g" \
        -e "s|{{GENERATED_AT}}|$generated_at|g" \
        -e "s|{{COVERAGE_RATE}}|$coverage_rate|g" \
        -e "s|{{PROGRESS_WIDTH}}|$progress_width|g" \
        -e "s|{{PROGRESS_COLOR}}|$progress_color|g" \
        -e "s|{{TOTAL_FEATURES}}|$total_features|g" \
        -e "s|{{COVERED_FEATURES}}|$covered_features|g" \
        -e "s|{{SUBCOMMAND_RATE}}|$subcommand_rate|g" \
        -e "s|{{SUBCOMMAND_COVERED}}|$subcommand_covered|g" \
        -e "s|{{SUBCOMMAND_TOTAL}}|$subcommand_total|g" \
        -e "s|{{OPTION_RATE}}|$option_rate|g" \
        -e "s|{{OPTION_COVERED}}|$option_covered|g" \
        -e "s|{{OPTION_TOTAL}}|$option_total|g" \
        "$output_html"

    # 複数行データ置換（一時ファイル経由）
    # HTMLエンティティエスケープして一時ファイルに保存
    local temp_dir
    temp_dir=$(mktemp -d)

    echo "$uncovered_subcommands" | \
        perl -pe 's/&/&amp;/g; s/</&lt;/g; s/>/&gt;/g; s/"/&quot;/g' > "$temp_dir/uncovered_sub.txt"

    echo "$uncovered_options" | \
        perl -pe 's/&/&amp;/g; s/</&lt;/g; s/>/&gt;/g; s/"/&quot;/g' > "$temp_dir/uncovered_opt.txt"

    echo "$recommendations" | \
        perl -pe 's/&/&amp;/g; s/</&lt;/g; s/>/&gt;/g; s/"/&quot;/g' > "$temp_dir/recommendations.txt"

    # Perlで複数行置換（slurpモード）
    perl -i -0777 -pe '
        open my $fh, "<", "'$temp_dir'/uncovered_sub.txt" or die $!;
        my $content = do { local $/; <$fh> };
        close $fh;
        s/UNCOVERED_SUBCOMMANDS_PLACEHOLDER/$content/g;
    ' "$output_html"

    perl -i -0777 -pe '
        open my $fh, "<", "'$temp_dir'/uncovered_opt.txt" or die $!;
        my $content = do { local $/; <$fh> };
        close $fh;
        s/UNCOVERED_OPTIONS_PLACEHOLDER/$content/g;
    ' "$output_html"

    perl -i -0777 -pe '
        open my $fh, "<", "'$temp_dir'/recommendations.txt" or die $!;
        my $content = do { local $/; <$fh> };
        close $fh;
        s/RECOMMENDATIONS_PLACEHOLDER/$content/g;
    ' "$output_html"

    # 一時ファイル削除
    rm -rf "$temp_dir"

    log INFO "HTML coverage report generated: $output_html"
    return 0
}

# JSONレポート生成（カバレッジ結果のコピー）
generate_json_report() {
    local coverage_json="$1"
    local output_json="$2"

    log INFO "Generating JSON coverage report"

    # 入力検証
    if [[ ! -f "$coverage_json" ]]; then
        log ERROR "Coverage JSON not found: $coverage_json"
        return 1
    fi

    # JSONバリデーション
    if ! jq empty "$coverage_json" 2>/dev/null; then
        log ERROR "Invalid JSON format: $coverage_json"
        return 1
    fi

    # コピー（整形）
    jq '.' "$coverage_json" > "$output_json"

    log INFO "JSON coverage report generated: $output_json"
    return 0
}

# Markdownレポート生成
generate_markdown_report() {
    local coverage_json="$1"
    local output_md="$2"

    log INFO "Generating Markdown coverage report"

    # 入力検証
    if [[ ! -f "$coverage_json" ]]; then
        log ERROR "Coverage JSON not found: $coverage_json"
        return 1
    fi

    # JSONバリデーション
    if ! jq empty "$coverage_json" 2>/dev/null; then
        log ERROR "Invalid JSON format: $coverage_json"
        return 1
    fi

    # データ抽出
    local cli_binary
    cli_binary=$(jq -r '.cli_binary' "$coverage_json")

    local coverage_rate
    coverage_rate=$(jq -r '.coverage_rate' "$coverage_json")

    local total_features
    total_features=$(jq -r '.total_features' "$coverage_json")

    local covered_features
    covered_features=$(jq -r '.covered_features' "$coverage_json")

    local subcommand_total
    subcommand_total=$(jq -r '.summary.subcommands.total' "$coverage_json")

    local subcommand_covered
    subcommand_covered=$(jq -r '.summary.subcommands.covered' "$coverage_json")

    local subcommand_rate
    subcommand_rate=$(jq -r '.summary.subcommands.coverage_rate' "$coverage_json")

    local option_total
    option_total=$(jq -r '.summary.options.total' "$coverage_json")

    local option_covered
    option_covered=$(jq -r '.summary.options.covered' "$coverage_json")

    local option_rate
    option_rate=$(jq -r '.summary.options.coverage_rate' "$coverage_json")

    local generated_at
    generated_at=$(jq -r '.generated_at' "$coverage_json")

    # Markdown生成
    cat > "$output_md" <<EOF
# Coverage Report

**CLI Binary:** \`$cli_binary\`
**Generated:** $generated_at
**Agent Version:** 2.1.0

---

## 📊 Overall Coverage

| Metric | Value |
|--------|-------|
| **Overall Coverage** | **${coverage_rate}%** |
| **Total Features** | $total_features |
| **Covered Features** | $covered_features |

### Progress Bar
\`\`\`
EOF

    # ASCII進捗バー
    local bar_width=50
    local filled
    filled=$(awk -v rate="$coverage_rate" -v width="$bar_width" 'BEGIN {printf "%.0f", (rate / 100) * width}')
    local empty=$((bar_width - filled))

    printf '%s' "["
    for ((i=0; i<filled; i++)); do printf "="; done
    for ((i=0; i<empty; i++)); do printf " "; done
    printf '] %.2f%%\n' "$coverage_rate"

    cat >> "$output_md" <<EOF
\`\`\`

---

## 📈 Breakdown

### Subcommands
- **Total:** $subcommand_total
- **Covered:** $subcommand_covered
- **Coverage:** ${subcommand_rate}%

### Options
- **Total:** $option_total
- **Covered:** $option_covered
- **Coverage:** ${option_rate}%

---

## ❌ Uncovered Areas

### Subcommands
EOF

    # 未カバーサブコマンド
    local uncovered_subcommands
    uncovered_subcommands=$(jq -r '.uncovered.subcommands[]? // empty' "$coverage_json" 2>/dev/null)

    if [[ -n "$uncovered_subcommands" ]]; then
        echo "$uncovered_subcommands" | while IFS= read -r subcmd; do
            echo "- \`$subcmd\`" >> "$output_md"
        done
    else
        echo "None" >> "$output_md"
    fi

    cat >> "$output_md" <<EOF

### Options
EOF

    # 未カバーオプション
    local uncovered_options
    uncovered_options=$(jq -r '.uncovered.options[]? // empty' "$coverage_json" 2>/dev/null)

    if [[ -n "$uncovered_options" ]]; then
        echo "$uncovered_options" | while IFS= read -r opt; do
            echo "- \`$opt\`" >> "$output_md"
        done
    else
        echo "None" >> "$output_md"
    fi

    cat >> "$output_md" <<EOF

---

## 💡 Recommendations
EOF

    # 改善提案
    local recommendations
    recommendations=$(jq -c '.recommendations[]?' "$coverage_json" 2>/dev/null)

    if [[ -n "$recommendations" ]]; then
        echo "$recommendations" | while IFS= read -r rec; do
            local priority
            priority=$(echo "$rec" | jq -r '.priority')

            local target
            target=$(echo "$rec" | jq -r '.target')

            local suggestion
            suggestion=$(echo "$rec" | jq -r '.suggestion')

            local priority_badge
            case "$priority" in
                high) priority_badge="🔴 HIGH" ;;
                medium) priority_badge="🟡 MEDIUM" ;;
                low) priority_badge="🟢 LOW" ;;
                *) priority_badge="⚪ $priority" ;;
            esac

            echo "- **$priority_badge** \`$target\`: $suggestion" >> "$output_md"
        done
    else
        echo "No recommendations" >> "$output_md"
    fi

    cat >> "$output_md" <<EOF

---

*Generated by CLI Testing Specialist v2.1.0*
EOF

    log INFO "Markdown coverage report generated: $output_md"
    return 0
}

# メイン実行（スクリプト直接実行時のみ）
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # 引数チェック
    if [[ $# -lt 3 ]]; then
        echo "Usage: $0 <coverage-json> <output-file> <format>" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  <coverage-json>  Coverage analysis result JSON" >&2
        echo "  <output-file>    Output report file" >&2
        echo "  <format>         Report format: html|json|markdown" >&2
        echo "" >&2
        echo "Examples:" >&2
        echo "  $0 ./coverage-result.json ./coverage-report.html html" >&2
        echo "  $0 ./coverage-result.json ./coverage-report.json json" >&2
        echo "  $0 ./coverage-result.json ./coverage-report.md markdown" >&2
        exit 1
    fi

    coverage_json="$1"
    output_file="$2"
    format="$3"

    case "$format" in
        html)
            generate_html_report "$coverage_json" "$output_file"
            ;;
        json)
            generate_json_report "$coverage_json" "$output_file"
            ;;
        markdown|md)
            generate_markdown_report "$coverage_json" "$output_file"
            ;;
        *)
            log ERROR "Unknown format: $format"
            log ERROR "Valid formats: html, json, markdown"
            exit 1
            ;;
    esac

    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        log INFO "Report generation completed successfully"
    else
        log ERROR "Report generation failed (exit code: $exit_code)"
    fi

    exit $exit_code
fi
