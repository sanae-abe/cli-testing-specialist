#!/usr/bin/env bash
#
# report-generator-html.sh - HTMLレポート生成エンジン
# CLI Testing Specialist Agent v1.1.0
#
# 機能:
# - Bootstrap 5を使用したインタラクティブなHTMLレポート生成
# - テスト結果の視覚的表示（成功率グラフ、ステータスバッジ）
# - Shell互換性マトリクス表示
# - フィルタリング・検索機能（JavaScript）
# - レスポンシブデザイン対応

set -euo pipefail
IFS=$'\n\t'

# エラートラップ
trap 'echo "Error at line $LINENO in report-generator-html.sh" >&2' ERR

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 依存ファイルの読み込み
source "$SCRIPT_DIR/../utils/logger.sh" 2>/dev/null || true

# HTMLヘッダー生成
generate_html_header() {
    local title="$1"
    local timestamp="$2"

    cat <<'EOF'
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CLI Testing Report</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
    <link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css" rel="stylesheet">
    <style>
        :root {
            --success-color: #198754;
            --danger-color: #dc3545;
            --warning-color: #ffc107;
            --info-color: #0dcaf0;
        }

        body {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 2rem 0;
        }

        .report-container {
            background: white;
            border-radius: 15px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            padding: 2rem;
            max-width: 1200px;
            margin: 0 auto;
        }

        .header-section {
            border-bottom: 3px solid #667eea;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
        }

        .stat-card {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border-radius: 10px;
            padding: 1.5rem;
            text-align: center;
            transition: transform 0.3s ease;
        }

        .stat-card:hover {
            transform: translateY(-5px);
        }

        .stat-value {
            font-size: 2.5rem;
            font-weight: bold;
            margin: 0.5rem 0;
        }

        .stat-label {
            font-size: 0.9rem;
            opacity: 0.9;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .success-rate-container {
            position: relative;
            width: 200px;
            height: 200px;
            margin: 2rem auto;
        }

        .success-rate-circle {
            transform: rotate(-90deg);
        }

        .success-rate-text {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 2rem;
            font-weight: bold;
        }

        .test-item {
            border-left: 4px solid transparent;
            padding: 1rem;
            margin-bottom: 0.5rem;
            border-radius: 5px;
            transition: all 0.3s ease;
        }

        .test-item.passed {
            border-left-color: var(--success-color);
            background: rgba(25, 135, 84, 0.05);
        }

        .test-item.failed {
            border-left-color: var(--danger-color);
            background: rgba(220, 53, 69, 0.05);
        }

        .test-item.skipped {
            border-left-color: var(--warning-color);
            background: rgba(255, 193, 7, 0.05);
        }

        .test-item:hover {
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
        }

        .badge-custom {
            padding: 0.5rem 1rem;
            font-size: 0.9rem;
        }

        .filter-section {
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 10px;
            margin-bottom: 2rem;
        }

        .search-box {
            border-radius: 25px;
            padding: 0.5rem 1.5rem;
        }

        .shell-matrix {
            overflow-x: auto;
        }

        .shell-matrix table {
            min-width: 600px;
        }

        .compatibility-badge {
            display: inline-block;
            width: 30px;
            height: 30px;
            border-radius: 50%;
            text-align: center;
            line-height: 30px;
            font-weight: bold;
        }

        .compat-yes {
            background: var(--success-color);
            color: white;
        }

        .compat-no {
            background: var(--danger-color);
            color: white;
        }

        .compat-partial {
            background: var(--warning-color);
            color: black;
        }

        .footer-section {
            margin-top: 3rem;
            padding-top: 2rem;
            border-top: 1px solid #dee2e6;
            text-align: center;
            color: #6c757d;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="report-container">
EOF

    echo "            <div class=\"header-section\">"
    echo "                <h1 class=\"display-4\"><i class=\"bi bi-clipboard-check\"></i> $title</h1>"
    echo "                <p class=\"text-muted\"><i class=\"bi bi-clock\"></i> Generated: $timestamp</p>"
    echo "            </div>"
}

# サマリーセクション生成
generate_summary_section() {
    local summary_json="$1"

    local total passed failed skipped success_rate
    total=$(echo "$summary_json" | jq -r '.total')
    passed=$(echo "$summary_json" | jq -r '.passed')
    failed=$(echo "$summary_json" | jq -r '.failed')
    skipped=$(echo "$summary_json" | jq -r '.skipped')
    success_rate=$(echo "$summary_json" | jq -r '.success_rate')

    # 成功率の円グラフ用の値を計算
    local circumference=565.48  # 2 * π * 90
    local offset=$(awk -v rate="$success_rate" -v circ="$circumference" 'BEGIN {printf "%.2f", circ - (circ * rate / 100)}')

    cat <<EOF
            <div class="row mb-4">
                <div class="col-md-3 mb-3">
                    <div class="stat-card">
                        <div class="stat-label">Total Tests</div>
                        <div class="stat-value">$total</div>
                        <i class="bi bi-file-earmark-text" style="font-size: 2rem;"></i>
                    </div>
                </div>
                <div class="col-md-3 mb-3">
                    <div class="stat-card" style="background: linear-gradient(135deg, #198754 0%, #157347 100%);">
                        <div class="stat-label">Passed</div>
                        <div class="stat-value">$passed</div>
                        <i class="bi bi-check-circle" style="font-size: 2rem;"></i>
                    </div>
                </div>
                <div class="col-md-3 mb-3">
                    <div class="stat-card" style="background: linear-gradient(135deg, #dc3545 0%, #b02a37 100%);">
                        <div class="stat-label">Failed</div>
                        <div class="stat-value">$failed</div>
                        <i class="bi bi-x-circle" style="font-size: 2rem;"></i>
                    </div>
                </div>
                <div class="col-md-3 mb-3">
                    <div class="stat-card" style="background: linear-gradient(135deg, #ffc107 0%, #cc9a06 100%);">
                        <div class="stat-label">Skipped</div>
                        <div class="stat-value">$skipped</div>
                        <i class="bi bi-skip-forward-circle" style="font-size: 2rem;"></i>
                    </div>
                </div>
            </div>

            <div class="text-center mb-4">
                <h3>Success Rate</h3>
                <div class="success-rate-container">
                    <svg width="200" height="200">
                        <circle cx="100" cy="100" r="90" stroke="#e9ecef" stroke-width="20" fill="none" />
                        <circle class="success-rate-circle" cx="100" cy="100" r="90"
                                stroke="$([ "$failed" -eq 0 ] && echo "#198754" || echo "#ffc107")"
                                stroke-width="20" fill="none"
                                stroke-dasharray="$circumference"
                                stroke-dashoffset="$offset"
                                style="transition: stroke-dashoffset 1s ease;" />
                    </svg>
                    <div class="success-rate-text" style="color: $([ "$failed" -eq 0 ] && echo "#198754" || echo "#ffc107");">
                        ${success_rate}%
                    </div>
                </div>
            </div>
EOF
}

# テスト結果セクション生成
generate_test_results_section() {
    local test_results="$1"

    cat <<'EOF'
            <div class="filter-section">
                <div class="row align-items-center">
                    <div class="col-md-6 mb-2">
                        <input type="text" id="searchBox" class="form-control search-box" placeholder="Search tests...">
                    </div>
                    <div class="col-md-6 mb-2">
                        <div class="btn-group" role="group">
                            <input type="radio" class="btn-check" name="filter" id="filterAll" value="all" checked>
                            <label class="btn btn-outline-primary" for="filterAll">All</label>

                            <input type="radio" class="btn-check" name="filter" id="filterPassed" value="passed">
                            <label class="btn btn-outline-success" for="filterPassed">Passed</label>

                            <input type="radio" class="btn-check" name="filter" id="filterFailed" value="failed">
                            <label class="btn btn-outline-danger" for="filterFailed">Failed</label>

                            <input type="radio" class="btn-check" name="filter" id="filterSkipped" value="skipped">
                            <label class="btn btn-outline-warning" for="filterSkipped">Skipped</label>
                        </div>
                    </div>
                </div>
            </div>

            <h3 class="mb-3"><i class="bi bi-list-check"></i> Test Results</h3>
            <div id="testResultsContainer">
EOF

    # TAPをパースしてHTMLに変換
    local test_number=0
    while IFS= read -r line; do
        if [[ "$line" =~ ^ok\ ([0-9]+)\ (.+) ]]; then
            test_number="${BASH_REMATCH[1]}"
            local test_name="${BASH_REMATCH[2]}"
            local status="passed"
            local icon="check-circle-fill"
            local badge_class="success"

            if [[ "$line" =~ \#\ skip ]]; then
                status="skipped"
                icon="skip-forward-circle-fill"
                badge_class="warning"
            fi

            cat <<TESTITEM
                <div class="test-item $status" data-status="$status">
                    <div class="d-flex justify-content-between align-items-center">
                        <div>
                            <span class="badge bg-$badge_class badge-custom">
                                <i class="bi bi-$icon"></i> $(echo "$status" | tr '[:lower:]' '[:upper:]')
                            </span>
                            <strong>#$test_number</strong> $test_name
                        </div>
                    </div>
                </div>
TESTITEM
        elif [[ "$line" =~ ^not\ ok\ ([0-9]+)\ (.+) ]]; then
            test_number="${BASH_REMATCH[1]}"
            local test_name="${BASH_REMATCH[2]}"

            if [[ ! "$line" =~ \#\ skip ]]; then
                cat <<TESTITEM
                <div class="test-item failed" data-status="failed">
                    <div class="d-flex justify-content-between align-items-center">
                        <div>
                            <span class="badge bg-danger badge-custom">
                                <i class="bi bi-x-circle-fill"></i> FAILED
                            </span>
                            <strong>#$test_number</strong> $test_name
                        </div>
                    </div>
                </div>
TESTITEM
            fi
        fi
    done <<< "$test_results"

    echo "            </div>"
}

# Shell互換性マトリクス生成
generate_shell_matrix_section() {
    cat <<'EOF'
            <div class="shell-matrix mt-5">
                <h3 class="mb-3"><i class="bi bi-terminal"></i> Shell Compatibility Matrix</h3>
                <table class="table table-bordered table-hover">
                    <thead class="table-dark">
                        <tr>
                            <th>Feature</th>
                            <th>Bash</th>
                            <th>Zsh</th>
                            <th>Dash</th>
                            <th>Ksh</th>
                            <th>Fish</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Basic Commands</td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-partial">~</span></td>
                        </tr>
                        <tr>
                            <td>POSIX Compliance</td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-no">✗</span></td>
                        </tr>
                        <tr>
                            <td>Advanced Arrays</td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-no">✗</span></td>
                            <td><span class="compatibility-badge compat-partial">~</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                        </tr>
                        <tr>
                            <td>Process Substitution</td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-no">✗</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-partial">~</span></td>
                        </tr>
                        <tr>
                            <td>Extended Globbing</td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-no">✗</span></td>
                            <td><span class="compatibility-badge compat-yes">✓</span></td>
                            <td><span class="compatibility-badge compat-no">✗</span></td>
                        </tr>
                    </tbody>
                </table>
                <div class="mt-2">
                    <small class="text-muted">
                        <span class="compatibility-badge compat-yes">✓</span> Full Support &nbsp;
                        <span class="compatibility-badge compat-partial">~</span> Partial Support &nbsp;
                        <span class="compatibility-badge compat-no">✗</span> Not Supported
                    </small>
                </div>
            </div>
EOF
}

# JavaScriptとフッター生成
generate_footer_and_scripts() {
    local agent_version="${1:-1.1.0-dev}"

    cat <<'EOF'
            <div class="footer-section">
                <p>Generated by <strong>CLI Testing Specialist Agent</strong> v
EOF
    echo "$agent_version"
    cat <<'EOF'
</p>
                <p class="mb-0">
                    <i class="bi bi-github"></i>
                    <a href="https://github.com/your-org/cli-testing-specialist" target="_blank">GitHub Repository</a>
                </p>
            </div>
        </div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js"></script>
    <script>
        // Search functionality
        document.getElementById('searchBox').addEventListener('input', function(e) {
            const searchTerm = e.target.value.toLowerCase();
            filterTests();
        });

        // Filter functionality
        document.querySelectorAll('input[name="filter"]').forEach(radio => {
            radio.addEventListener('change', filterTests);
        });

        function filterTests() {
            const searchTerm = document.getElementById('searchBox').value.toLowerCase();
            const filterValue = document.querySelector('input[name="filter"]:checked').value;
            const testItems = document.querySelectorAll('.test-item');

            testItems.forEach(item => {
                const text = item.textContent.toLowerCase();
                const status = item.getAttribute('data-status');

                const matchesSearch = text.includes(searchTerm);
                const matchesFilter = filterValue === 'all' || status === filterValue;

                item.style.display = (matchesSearch && matchesFilter) ? 'block' : 'none';
            });
        }

        // Animate success rate circle on load
        window.addEventListener('load', function() {
            const circle = document.querySelector('.success-rate-circle');
            if (circle) {
                const offset = circle.getAttribute('stroke-dashoffset');
                circle.style.strokeDashoffset = '565.48';
                setTimeout(() => {
                    circle.style.strokeDashoffset = offset;
                }, 100);
            }
        });
    </script>
</body>
</html>
EOF
}

# メインHTML生成関数
generate_html_report() {
    local test_results="$1"
    local summary_json="$2"
    local output_file="$3"

    if command -v log &>/dev/null; then
        log INFO "Generating HTML report: $output_file"
    fi

    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    local agent_version
    agent_version=$(cat "$AGENT_ROOT/VERSION" 2>/dev/null || echo "1.1.0-dev")

    # HTMLファイルを生成
    {
        generate_html_header "CLI Testing Report" "$timestamp"
        generate_summary_section "$summary_json"
        generate_test_results_section "$test_results"
        generate_shell_matrix_section
        generate_footer_and_scripts "$agent_version"
    } > "$output_file"

    if command -v log &>/dev/null; then
        log INFO "HTML report generated successfully: $output_file"
    fi
}

# スクリプトを直接実行した場合
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [[ $# -lt 3 ]]; then
        echo "Usage: $0 <test-results-file> <summary-json-file> <output-html-file>" >&2
        echo "" >&2
        echo "Example:" >&2
        echo "  $0 test-results.tap summary.json report.html" >&2
        exit 1
    fi

    test_results_file="$1"
    summary_json_file="$2"
    output_file="$3"

    if [[ ! -f "$test_results_file" ]]; then
        echo "Error: Test results file not found: $test_results_file" >&2
        exit 1
    fi

    if [[ ! -f "$summary_json_file" ]]; then
        echo "Error: Summary JSON file not found: $summary_json_file" >&2
        exit 1
    fi

    test_results=$(cat "$test_results_file")
    summary_json=$(cat "$summary_json_file")

    generate_html_report "$test_results" "$summary_json" "$output_file"

    echo "HTML report generated: $output_file"
fi
