# レポート形式ガイド

CLI Testing Specialist Agentは複数のレポート形式をサポートしています。

## サポートされている形式

### 1. Markdown形式 (`.md`)

**用途**: ドキュメント、GitHub/GitLab表示、人間が読むための形式

**生成コマンド**:
```bash
bash core/run-tests.sh ./generated-tests markdown ./reports
```

**特徴**:
- GitHubやGitLabで直接表示可能
- 絵文字を使った視覚的な表示
- テスト結果の概要をテーブル形式で表示
- TAP出力をコードブロックに含める

**サンプル出力**:
```markdown
# CLI Testing Report

**Generated:** 2025-11-10 10:40:51
**Agent Version:** 1.1.0-dev

## Test Summary

| Metric | Value |
|--------|-------|
| **Total Tests** | 15 |
| **Passed** | ✅ 14 |
| **Failed** | ❌ 0 |
| **Skipped** | ⏭️  1 |
| **Success Rate** | 93.33% |
```

---

### 2. JSON形式 (`.json`)

**用途**: CI/CDパイプライン統合、プログラム処理、データ解析

**生成コマンド**:
```bash
bash core/run-tests.sh ./generated-tests json ./reports
```

**特徴**:
- 機械可読形式
- CI/CDツールとの統合が容易
- jqコマンドで簡単に解析可能
- 失敗したテストの詳細情報を含む

**JSON構造**:
```json
{
  "report_type": "cli_testing_results",
  "generated_at": "2025-11-10T01:40:51Z",
  "agent_version": "1.1.0-dev",
  "summary": {
    "total": 15,
    "passed": 14,
    "failed": 0,
    "skipped": 1,
    "success_rate": 93.33
  },
  "failed_tests": []
}
```

**使用例**:
```bash
# 成功率を取得
jq -r '.summary.success_rate' test-report.json

# 失敗したテスト数を取得
jq -r '.summary.failed' test-report.json

# 失敗したテストの名前一覧
jq -r '.failed_tests[].test_name' test-report.json
```

---

### 3. HTML形式 (`.html`) - 新機能

**用途**: インタラクティブな閲覧、GitHub Pages公開、チーム共有

**生成コマンド**:
```bash
bash core/run-tests.sh ./generated-tests html ./reports
```

**特徴**:
- Bootstrap 5を使用したモダンなデザイン
- レスポンシブデザイン対応
- インタラクティブな機能:
  - テスト結果の検索機能
  - ステータスによるフィルタリング (All/Passed/Failed/Skipped)
  - アニメーション付き成功率円グラフ
- Shell互換性マトリクス表示
- ブラウザで直接開いて閲覧可能

**インタラクティブ機能**:
1. **検索機能**: テスト名でリアルタイム検索
2. **フィルタリング**: ステータス別にテストを表示/非表示
3. **視覚的表示**: カラフルなバッジと統計カード
4. **成功率グラフ**: SVGアニメーション付き円グラフ

**ブラウザで開く**:
```bash
open sample-reports/test-report.html  # macOS
xdg-open sample-reports/test-report.html  # Linux
```

---

### 4. 全形式一括生成 (`all`)

**用途**: 完全なレポートセットを一度に生成

**生成コマンド**:
```bash
bash core/run-tests.sh ./generated-tests all ./reports
```

**生成されるファイル**:
- `test-report.md` - Markdown形式
- `test-report.json` - JSON形式
- `test-report.html` - HTML形式

---

## レポート生成の詳細設定

### HTMLレポート生成スクリプト

HTMLレポートは専用スクリプト `core/report-generator-html.sh` で生成されます。

**直接実行する場合**:
```bash
bash core/report-generator-html.sh <tap-file> <summary-json> <output-html>
```

**パラメータ**:
- `<tap-file>`: TAP形式のテスト結果ファイル
- `<summary-json>`: JSON形式の集計結果ファイル
- `<output-html>`: 出力するHTMLファイルパス

---

## CI/CD統合

### GitHub Actions

`.github/workflows/cli-test.yml` で自動的にHTML/JSON/Markdownレポートを生成し、GitHub Pagesに公開できます。

**主な機能**:
- 複数OS (Ubuntu/macOS) でテスト実行
- 複数Shell (Bash/Zsh) でテスト実行
- テスト結果をArtifactとして保存
- HTMLレポートをGitHub Pagesに自動デプロイ

**使用方法**:
1. リポジトリ設定でGitHub Pagesを有効化
2. mainブランチにpushすると自動実行
3. `https://[username].github.io/[repo]/` でレポート閲覧

### GitLab CI/CD

`.gitlab-ci.yml` でBash/Zsh/Dash環境でのテストと、GitLab Pagesへの自動公開に対応。

**主な機能**:
- 複数Shell環境でのテスト実行
- レポート集約ジョブ
- GitLab Pagesへの自動デプロイ
- スケジュール実行によるリグレッションテスト

**使用方法**:
1. GitLabリポジトリにpush
2. CI/CDパイプラインが自動実行
3. `https://[username].gitlab.io/[repo]/` でレポート閲覧

---

## Shell互換性マトリクス

HTMLレポートには以下のShell互換性マトリクスが含まれます:

| Feature | Bash | Zsh | Dash | Ksh | Fish |
|---------|------|-----|------|-----|------|
| Basic Commands | ✓ | ✓ | ✓ | ✓ | ~ |
| POSIX Compliance | ✓ | ✓ | ✓ | ✓ | ✗ |
| Advanced Arrays | ✓ | ✓ | ✗ | ~ | ✓ |
| Process Substitution | ✓ | ✓ | ✗ | ✓ | ~ |
| Extended Globbing | ✓ | ✓ | ✗ | ✓ | ✗ |

**凡例**:
- ✓ = Full Support (完全サポート)
- ~ = Partial Support (部分的サポート)
- ✗ = Not Supported (非サポート)

---

## ベストプラクティス

### 1. 開発時
```bash
# 素早くMarkdownレポートを生成
bash core/run-tests.sh ./generated-tests markdown ./reports
```

### 2. CI/CD統合時
```bash
# 全形式を生成してアーティファクトに保存
bash core/run-tests.sh ./generated-tests all ./reports
```

### 3. チーム共有時
```bash
# HTML形式を生成してブラウザで開く
bash core/run-tests.sh ./generated-tests html ./reports
open reports/test-report.html
```

### 4. プログラム解析時
```bash
# JSON形式を生成して解析
bash core/run-tests.sh ./generated-tests json ./reports
jq '.summary' reports/test-report.json
```

---

## トラブルシューティング

### HTMLレポートが生成されない

**原因**: `report-generator-html.sh` が実行可能でない

**解決策**:
```bash
chmod +x core/report-generator-html.sh
```

### jqコマンドが見つからない

**原因**: jqがインストールされていない

**解決策**:
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# CentOS/RHEL
sudo yum install jq
```

### 日本語が文字化けする

**原因**: ターミナルのエンコーディング設定

**解決策**:
```bash
export LANG=ja_JP.UTF-8
```

---

## サンプルレポート

サンプルレポートは `sample-reports/` ディレクトリに生成されます:

```bash
sample-reports/
├── test-report.html  # HTMLレポート (22KB)
├── test-report.json  # JSONレポート (255B)
└── test-report.md    # Markdownレポート (968B)
```

**サンプル生成コマンド**:
```bash
bash core/run-tests.sh sample-tests all sample-reports
```

---

## 今後の拡張予定

- PDF形式エクスポート機能
- テストカバレッジ統計
- 時系列比較グラフ
- 複数レポートの統合・比較機能
- カスタムテンプレート対応
