# Coverage Analysis Design Document
# CLI Testing Specialist v2.1.0

**作成日**: 2025-11-10
**対象機能**: カバレッジ分析エンジン

---

## 🎯 概要

CLI Testing Specialistのテストがどれだけの機能をカバーしているかを可視化し、未テスト領域を特定する機能。

### 主要機能
1. **コマンド使用追跡**: テスト実行時のオプション/サブコマンド使用を記録
2. **カバレッジ計算**: 全機能に対する使用率を計算
3. **未カバー領域特定**: テストされていないオプション/パスを検出
4. **レポート生成**: HTML/JSON形式でカバレッジレポートを出力

---

## 📐 アーキテクチャ

### コンポーネント構成

```
┌─────────────────────────────────────────────────────────┐
│                    cli-test (main)                      │
│  - カバレッジモード起動 (--coverage フラグ)              │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│             core/coverage-tracker.sh                    │
│  - テスト実行をフック                                    │
│  - オプション/サブコマンド使用を記録                     │
│  - リアルタイムで coverage.db (SQLite) に保存           │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│             core/coverage-analyzer.sh                   │
│  - analysis.json (全機能定義) 読み込み                   │
│  - coverage.db (使用履歴) と突合                        │
│  - カバレッジ率計算                                      │
│  - 未カバー領域特定                                      │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│             core/coverage-reporter.sh                   │
│  - HTMLレポート生成 (D3.jsヒートマップ)                 │
│  - JSONレポート生成                                      │
│  - 改善提案の生成                                        │
└─────────────────────────────────────────────────────────┘
```

### データフロー

```
1. テスト開始
   └─> coverage-tracker.sh: カバレッジモード初期化
       └─> coverage.db: テーブル作成 (command_usage, option_usage)

2. テスト実行中
   └─> BATS tests: run "$CLI_BINARY" --option subcommand
       └─> coverage-tracker.sh: フック関数でキャプチャ
           └─> coverage.db: INSERT INTO command_usage VALUES (...)

3. テスト完了後
   └─> coverage-analyzer.sh: 分析開始
       ├─> analysis.json: 全機能定義読み込み
       ├─> coverage.db: 使用履歴読み込み
       └─> coverage-result.json: カバレッジ結果出力

4. レポート生成
   └─> coverage-reporter.sh: レポート作成
       ├─> coverage-result.json: 読み込み
       └─> coverage-report.html: ヒートマップ生成
```

---

## 🗄️ データモデル

### SQLite Database Schema (coverage.db)

#### テーブル: command_usage
```sql
CREATE TABLE IF NOT EXISTS command_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    command TEXT NOT NULL,           -- 実行コマンド全体
    subcommand TEXT,                 -- サブコマンド（あれば）
    exit_code INTEGER,               -- 終了コード
    test_name TEXT,                  -- テスト名（BATS）
    test_file TEXT                   -- テストファイル
);

CREATE INDEX idx_subcommand ON command_usage(subcommand);
CREATE INDEX idx_test_file ON command_usage(test_file);
```

#### テーブル: option_usage
```sql
CREATE TABLE IF NOT EXISTS option_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    option_name TEXT NOT NULL,       -- オプション名（例: --verbose）
    option_value TEXT,               -- オプション値（あれば）
    command_id INTEGER NOT NULL,     -- command_usageへの外部キー
    FOREIGN KEY (command_id) REFERENCES command_usage(id)
);

CREATE INDEX idx_option_name ON option_usage(option_name);
```

### JSON Schema (coverage-result.json)

```json
{
  "generated_at": "2025-11-10T10:00:00Z",
  "cli_binary": "/bin/ls",
  "total_features": 50,
  "covered_features": 40,
  "coverage_rate": 80.0,
  "summary": {
    "subcommands": {
      "total": 10,
      "covered": 8,
      "coverage_rate": 80.0
    },
    "options": {
      "total": 40,
      "covered": 32,
      "coverage_rate": 80.0
    }
  },
  "uncovered": {
    "subcommands": ["subcommand-x", "subcommand-y"],
    "options": ["--option-a", "--option-b", "--option-c"]
  },
  "coverage_matrix": {
    "subcommand-1": {
      "covered": true,
      "usage_count": 5,
      "options_covered": ["--opt1", "--opt2"],
      "options_uncovered": ["--opt3"]
    }
  },
  "recommendations": [
    {
      "type": "uncovered_subcommand",
      "target": "subcommand-x",
      "priority": "high",
      "suggestion": "Add test case for 'subcommand-x' to improve coverage"
    }
  ]
}
```

---

## 🔧 実装詳細

### core/coverage-tracker.sh

#### 主要関数

##### 1. initialize_coverage_db()
```bash
initialize_coverage_db() {
    local db_path="$1"

    sqlite3 "$db_path" <<EOF
CREATE TABLE IF NOT EXISTS command_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    command TEXT NOT NULL,
    subcommand TEXT,
    exit_code INTEGER,
    test_name TEXT,
    test_file TEXT
);

CREATE TABLE IF NOT EXISTS option_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    option_name TEXT NOT NULL,
    option_value TEXT,
    command_id INTEGER NOT NULL,
    FOREIGN KEY (command_id) REFERENCES command_usage(id)
);

CREATE INDEX IF NOT EXISTS idx_subcommand ON command_usage(subcommand);
CREATE INDEX IF NOT EXISTS idx_option_name ON option_usage(option_name);
EOF

    log INFO "Coverage database initialized: $db_path"
}
```

##### 2. track_command_execution()
```bash
track_command_execution() {
    local command="$1"
    local exit_code="$2"
    local test_name="${3:-unknown}"
    local test_file="${4:-unknown}"
    local db_path="${COVERAGE_DB_PATH}"

    # コマンド解析
    local subcommand=$(extract_subcommand "$command")
    local options=$(extract_options "$command")

    # タイムスタンプ
    local timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

    # command_usageに挿入
    local command_id=$(sqlite3 "$db_path" \
        "INSERT INTO command_usage (timestamp, command, subcommand, exit_code, test_name, test_file) \
         VALUES ('$timestamp', '$command', '$subcommand', $exit_code, '$test_name', '$test_file'); \
         SELECT last_insert_rowid();")

    # option_usageに挿入
    while IFS= read -r option; do
        local option_name=$(echo "$option" | cut -d'=' -f1)
        local option_value=$(echo "$option" | cut -d'=' -f2-)

        sqlite3 "$db_path" \
            "INSERT INTO option_usage (timestamp, option_name, option_value, command_id) \
             VALUES ('$timestamp', '$option_name', '$option_value', $command_id);"
    done <<< "$options"

    log DEBUG "Tracked command execution: $command (exit: $exit_code)"
}
```

##### 3. extract_subcommand()
```bash
extract_subcommand() {
    local command="$1"

    # コマンドからサブコマンドを抽出
    # 例: "/bin/git commit -m 'message'" → "commit"
    local parts=($command)

    # 最初の非オプション引数を探す
    for part in "${parts[@]:1}"; do
        if [[ ! "$part" =~ ^- ]]; then
            echo "$part"
            return 0
        fi
    done

    echo ""
}
```

##### 4. extract_options()
```bash
extract_options() {
    local command="$1"

    # オプションを抽出（改行区切り）
    # 例: "--verbose --output=file.txt" → "--verbose\n--output=file.txt"
    echo "$command" | grep -oE '(-{1,2}[a-zA-Z0-9_-]+(=[^ ]+)?)' || echo ""
}
```

### core/coverage-analyzer.sh

#### 主要関数

##### 1. analyze_coverage()
```bash
analyze_coverage() {
    local analysis_json="$1"
    local coverage_db="$2"
    local output_json="$3"

    log INFO "Starting coverage analysis"

    # analysis.jsonから全機能を読み込み
    local all_subcommands=$(jq -r '.subcommands[]' "$analysis_json")
    local all_options=$(jq -r '.options[]' "$analysis_json")

    # coverage.dbから使用履歴を読み込み
    local used_subcommands=$(sqlite3 "$coverage_db" \
        "SELECT DISTINCT subcommand FROM command_usage WHERE subcommand IS NOT NULL;")
    local used_options=$(sqlite3 "$coverage_db" \
        "SELECT DISTINCT option_name FROM option_usage;")

    # カバレッジ計算
    local coverage_result=$(calculate_coverage \
        "$all_subcommands" "$used_subcommands" \
        "$all_options" "$used_options")

    # 結果をJSON出力
    echo "$coverage_result" > "$output_json"

    log INFO "Coverage analysis completed: $output_json"
}
```

##### 2. calculate_coverage()
```bash
calculate_coverage() {
    local all_subcommands="$1"
    local used_subcommands="$2"
    local all_options="$3"
    local used_options="$4"

    # 総数
    local total_subcommands=$(echo "$all_subcommands" | grep -v '^$' | wc -l)
    local total_options=$(echo "$all_options" | grep -v '^$' | wc -l)

    # カバー数
    local covered_subcommands=0
    while IFS= read -r subcmd; do
        [[ -z "$subcmd" ]] && continue
        if echo "$used_subcommands" | grep -qx "$subcmd"; then
            ((covered_subcommands++))
        fi
    done <<< "$all_subcommands"

    local covered_options=0
    while IFS= read -r opt; do
        [[ -z "$opt" ]] && continue
        if echo "$used_options" | grep -qx "$opt"; then
            ((covered_options++))
        fi
    done <<< "$all_options"

    # カバレッジ率計算
    local subcommand_rate=0
    [[ $total_subcommands -gt 0 ]] && \
        subcommand_rate=$(awk -v c="$covered_subcommands" -v t="$total_subcommands" \
            'BEGIN {printf "%.2f", (c / t) * 100}')

    local option_rate=0
    [[ $total_options -gt 0 ]] && \
        option_rate=$(awk -v c="$covered_options" -v t="$total_options" \
            'BEGIN {printf "%.2f", (c / t) * 100}')

    # JSON構築
    jq -n \
        --arg timestamp "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --argjson total_sub "$total_subcommands" \
        --argjson covered_sub "$covered_subcommands" \
        --arg sub_rate "$subcommand_rate" \
        --argjson total_opt "$total_options" \
        --argjson covered_opt "$covered_options" \
        --arg opt_rate "$option_rate" \
        '{
            generated_at: $timestamp,
            summary: {
                subcommands: {
                    total: $total_sub,
                    covered: $covered_sub,
                    coverage_rate: ($sub_rate | tonumber)
                },
                options: {
                    total: $total_opt,
                    covered: $covered_opt,
                    coverage_rate: ($opt_rate | tonumber)
                }
            }
        }'
}
```

### core/coverage-reporter.sh

#### 主要関数

##### 1. generate_html_report()
```bash
generate_html_report() {
    local coverage_json="$1"
    local output_html="$2"

    log INFO "Generating HTML coverage report"

    # JSONデータ読み込み
    local coverage_data=$(cat "$coverage_json")

    # HTMLテンプレート処理
    local template_file="$TEMPLATE_DIR/coverage-report.html"

    # JavaScriptにJSONデータを埋め込み
    sed -e "s|{{COVERAGE_DATA}}|$coverage_data|g" \
        "$template_file" > "$output_html"

    log INFO "HTML coverage report generated: $output_html"
}
```

---

## 🎨 UI/UX設計

### HTML Coverage Report

#### レイアウト
```
┌─────────────────────────────────────────────────────────┐
│ CLI Testing Specialist - Coverage Report               │
│ Generated: 2025-11-10 10:00:00                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  📊 Overall Coverage                                    │
│  ┌──────────────────────────────────────┐              │
│  │  Total Coverage: 80.0%               │              │
│  │  ████████████████████░░░░░░░░        │              │
│  └──────────────────────────────────────┘              │
│                                                         │
│  📈 Breakdown                                           │
│  ┌──────────────────────────────────────┐              │
│  │  Subcommands: 8/10 (80%)             │              │
│  │  Options: 32/40 (80%)                │              │
│  └──────────────────────────────────────┘              │
│                                                         │
│  🗺️  Coverage Heatmap (D3.js)                          │
│  [Interactive heatmap showing coverage by module]      │
│                                                         │
│  ❌ Uncovered Areas                                     │
│  - subcommand-x                                        │
│  - subcommand-y                                        │
│  - --option-a                                          │
│                                                         │
│  💡 Recommendations                                     │
│  1. Add test for 'subcommand-x' (high priority)       │
│  2. Test '--option-a' with edge cases                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

#### D3.js ヒートマップ仕様
- **X軸**: サブコマンド
- **Y軸**: オプション
- **色**: カバレッジ率（緑: 100%, 黄: 50-99%, 赤: 0-49%）
- **ツールチップ**: 使用回数、最終使用日時
- **インタラクティブ**: クリックで詳細表示

---

## 🔌 統合ポイント

### cli-test統合

#### 新規フラグ: --coverage
```bash
# 使用例
./cli-test --coverage -o ./test-output /bin/ls

# 内部動作
1. coverage-tracker.sh初期化
2. 環境変数設定: COVERAGE_MODE=true
3. BATS tests実行（フックで追跡）
4. テスト完了後、coverage-analyzer.sh実行
5. coverage-reporter.sh実行
6. coverage-report.html生成
```

#### BATSテストフック

**setup()関数に追加**:
```bash
setup() {
    # 既存のsetup処理
    CLI_BINARY="/bin/ls"

    # カバレッジモード時のフック
    if [[ "${COVERAGE_MODE:-false}" == "true" ]]; then
        # runコマンドをラップする関数を定義
        run() {
            builtin run "$@"  # BATSのrun実行
            track_command_execution "$*" "$status" "$BATS_TEST_NAME" "$BATS_TEST_FILENAME"
        }
    fi
}
```

---

## 🛡️ エラーハンドリング

### SQLite エラー
- **データベースロック**: リトライ機構（最大3回、1秒間隔）
- **ディスク容量不足**: 早期警告（容量 < 100MB）
- **破損検出**: PRAGMA integrity_checkで検証

### パフォーマンス対策
- **バッチ挿入**: 100件ごとにトランザクション
- **インデックス**: 頻繁な検索カラムに作成
- **定期クリーンアップ**: 30日以上古いデータは削除オプション

---

## ✅ 検証計画

### Unit Tests
- [ ] `extract_subcommand()` - サブコマンド抽出の正確性
- [ ] `extract_options()` - オプション抽出の正確性
- [ ] `calculate_coverage()` - カバレッジ計算の正確性
- [ ] SQLite挿入・検索処理

### Integration Tests
- [ ] `/bin/ls`での完全テスト
- [ ] カバレッジ率90%以上の検証
- [ ] HTMLレポート生成成功
- [ ] 未カバー領域の正確な特定

### Performance Tests
- [ ] 1,000テスト実行での測定
- [ ] データベースサイズ < 10MB（1,000テスト）
- [ ] レポート生成 < 3秒

---

## 📅 実装タイムライン

### Day 1-2: coverage-tracker.sh
- [ ] SQLiteスキーマ実装
- [ ] track_command_execution()実装
- [ ] extract_*()関数実装

### Day 3-4: coverage-analyzer.sh
- [ ] analyze_coverage()実装
- [ ] calculate_coverage()実装
- [ ] JSON出力実装

### Day 5-6: coverage-reporter.sh
- [ ] HTMLテンプレート作成
- [ ] D3.jsヒートマップ実装
- [ ] generate_html_report()実装

### Day 7: 統合・テスト
- [ ] cli-test統合
- [ ] BATSフック実装
- [ ] エンドツーエンドテスト

---

## 🚀 次のステップ

1. **プロトタイプ実装**: coverage-tracker.sh基本版
2. **動作確認**: /bin/echoでの簡易テスト
3. **レビュー**: 設計の妥当性確認
4. **本実装**: 全機能実装

この設計により、Phase 2 Week 1の目標達成を目指します。
