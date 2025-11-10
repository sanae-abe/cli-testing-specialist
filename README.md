# CLI Testing Specialist Agent

**バージョン**: 1.1.0
**ステータス**: 開発中（Phase 1 Week 9-10）
**Claude Code専用**: セキュアで包括的なCLIツールテストフレームワーク

---

## 概要

CLI Testing Specialist Agentは、CLIツールの品質とセキュリティを自動検証する包括的なテストフレームワークです。

### 主要機能

- 🔒 **セキュリティテスト**: OWASP準拠の自動スキャン
- ✅ **包括的検証**: 11カテゴリ、100-120テストケース
- 🐚 **マルチシェル対応**: bash/zsh（v1.1でfish対応）
- 📊 **詳細レポート**: Markdown/JSON/HTML/JUnit XML
- 🔄 **CI/CD統合**: GitHub Actions & GitLab CI対応
- 🐳 **Docker統合**: 隔離環境でのテスト実行（オプション）

---

## クイックスタート

```bash
# 基本的な使用
cli-test your-cli

# セキュリティテストのみ
cli-test your-cli --category security

# 詳細レポート生成（全形式）
cli-test your-cli --report all --output ./reports/

# HTMLレポート生成
bash core/run-tests.sh ./generated-tests html ./reports
```

---

## インストール

```bash
# Claude Code経由で自動インストール（推奨）
# Agentが自動的にセットアップを実行

# または手動インストール
git clone <repository-url>
cd cli-testing-specialist
./bin/cli-test --version
```

---

## 機能一覧（v1.1）

| カテゴリ | 説明 | テスト数 |
|---------|------|---------|
| 基本動作検証 | ヘルプ、バージョン、終了コード | 10 |
| サブコマンドヘルプ | 全サブコマンドの網羅的検証 | 動的 |
| セキュリティ | インジェクション、機密漏洩、TOCTOU | 25 |
| パス処理 | 特殊文字、深い階層、Unicode | 20 |
| マルチシェル | bash/zsh互換性 | 12 |
| 入力検証 | 不正値、エッジケース | 12 |
| 出力検証 | フォーマット、カラー出力 | 8 |
| 環境依存 | OS、環境変数 | 10 |
| パフォーマンス | 起動時間、メモリ使用量 | 6 |
| ドキュメント整合性 | README vs ヘルプ | 5 |
| **レポート** | **4形式（Markdown/JSON/HTML/JUnit）** | - |

**合計**: 約100-120テストケース

---

## レポート形式

### 1. Markdown形式 (`.md`)
GitHub/GitLabで直接表示可能な人間が読みやすい形式

```bash
bash core/run-tests.sh ./generated-tests markdown ./reports
```

### 2. JSON形式 (`.json`)
CI/CD統合とプログラム処理に最適

```bash
bash core/run-tests.sh ./generated-tests json ./reports

# jqで成功率を取得
jq -r '.summary.success_rate' reports/test-report.json
```

### 3. HTML形式 (`.html`) - **新機能**
インタラクティブなブラウザ表示、GitHub Pages公開対応

```bash
bash core/run-tests.sh ./generated-tests html ./reports
open reports/test-report.html  # ブラウザで開く
```

**HTML機能**:
- Bootstrap 5によるモダンなデザイン
- リアルタイム検索・フィルタリング
- アニメーション付き成功率グラフ
- Shell互換性マトリクス表示
- レスポンシブデザイン対応

### 4. 全形式一括生成 (`all`)

```bash
bash core/run-tests.sh ./generated-tests all ./reports
```

詳細は [`docs/REPORT-FORMATS.md`](docs/REPORT-FORMATS.md) を参照してください。

---

## CI/CD統合

### GitHub Actions

`.github/workflows/cli-test.yml` で自動テスト＆レポート公開

**機能**:
- Ubuntu/macOS × Bash/Zshマトリクステスト
- HTMLレポートをGitHub Pagesに自動デプロイ
- テスト結果をArtifactとして保存
- ShellCheck自動Lint

**使用方法**:
1. リポジトリ設定でGitHub Pagesを有効化
2. mainブランチにpushで自動実行
3. `https://[username].github.io/[repo]/` でレポート閲覧

### GitLab CI/CD

`.gitlab-ci.yml` でマルチShell環境テスト＆GitLab Pages公開

**機能**:
- Bash/Zsh/Dash互換性テスト
- レポート集約ステージ
- GitLab Pagesへの自動デプロイ
- スケジュール実行によるリグレッションテスト

**パイプラインステージ**:
1. `validate` - 構造検証＆ShellCheck
2. `test` - 複数Shell環境でテスト実行
3. `report` - レポート集約
4. `deploy` - GitLab Pagesデプロイ

---

## セキュリティ機能

### 入力バリデーション
- CLIバイナリパスの検証
- パストラバーサル攻撃防御
- コマンドインジェクション対策

### セキュアな実行環境
- 一時ファイルのumask 077
- TOCTOU攻撃対策（mktemp使用）
- Docker非rootユーザー実行

### セキュリティスキャン
- OWASP Top 10準拠
- 機密情報漏洩チェック
- 依存関係脆弱性スキャン

---

## 設定

### デフォルト設定ファイル

```yaml
# ~/.config/cli-test/config.yaml
cli-testing-specialist:
  version: "1.1.0"

  test_categories:
    enabled:
      - basic-validation
      - help-checker
      - security-scanner
      - path-handler
      - shell-compatibility

  docker:
    enabled: true
    fallback_to_native: true

  logging:
    level: "INFO"
    file: "/tmp/cli-test.log"
```

詳細は `config/schema.yaml` を参照してください。

---

## 開発状況

### 実装スケジュール

- ✅ **Week 0**: プロジェクト構造作成
- ✅ **Week 1**: 基盤構築（validator, logger, analyzer）
- ✅ **Week 2-3**: コアテストモジュール
- ✅ **Week 4-8**: 環境互換性・統合・最適化
- 🔄 **Week 9-10**: レポート拡張＋CI/CD統合（現在）
- ⏳ **Week 11-12**: ドキュメント・検証・リリース

### 完了した実装（Phase 1 Week 9-10）

- ✅ HTMLレポート生成 (`core/report-generator-html.sh`)
  - Bootstrap 5デザイン
  - インタラクティブ検索・フィルタリング
  - アニメーション付き成功率グラフ
  - Shell互換性マトリクス表示

- ✅ `run-tests.sh` 拡張
  - `html`オプション追加
  - `all`オプションで全形式一括生成
  - レポート形式: markdown|json|html|all

- ✅ CI/CD統合
  - GitHub Actions設定 (`.github/workflows/cli-test.yml`)
  - GitLab CI設定 (`.gitlab-ci.yml`)
  - マルチOS/Shellマトリクステスト
  - 自動レポート公開（GitHub/GitLab Pages）

### マイルストーン

- **2025-12-20**: Week 4終了（コア機能完成）
- **2026-01-10**: Week 8終了（β版）
- **2026-01-24**: Week 10終了（レポート拡張＋CI/CD統合完了）
- **2026-02-07**: v1.0.0リリース

---

## サンプルレポート

サンプルテストとレポートを生成して確認できます:

```bash
# サンプルテスト実行＆全形式レポート生成
bash core/run-tests.sh sample-tests all sample-reports

# 生成されたファイル
sample-reports/
├── test-report.html  # HTMLレポート (22KB)
├── test-report.json  # JSONレポート (255B)
└── test-report.md    # Markdownレポート (968B)

# HTMLレポートをブラウザで開く
open sample-reports/test-report.html
```

**サンプルレポート**: `/Users/sanae.abe/projects/cli-testing-specialist/sample-reports/test-report.html`

---

## ライセンス

MIT License

---

## 貢献

プルリクエストを歓迎します。大きな変更の場合は、まずissueを開いて変更内容を議論してください。

---

## サポート

- **ドキュメント**: `docs/` ディレクトリ
  - [`REPORT-FORMATS.md`](docs/REPORT-FORMATS.md) - レポート形式詳細ガイド
- **Issues**: GitHub Issues
- **実装計画書**: `~/.claude/docs/cli-testing-agent-implementation-plan-v1.1.md`
- **レビューレポート**: `~/.claude/docs/cli-testing-agent-review-report.md`

---

## ファイル構造

```
cli-testing-specialist/
├── core/
│   ├── cli-analyzer.sh           # CLI解析エンジン
│   ├── test-generator.sh         # BATS生成エンジン
│   ├── run-tests.sh              # テスト実行＆レポート生成
│   ├── report-generator-html.sh  # HTMLレポート生成（新規）
│   ├── shell-detector.sh         # Shell検出エンジン
│   └── validator.sh              # 入力検証エンジン
├── .github/
│   └── workflows/
│       └── cli-test.yml          # GitHub Actions設定（新規）
├── .gitlab-ci.yml                # GitLab CI設定（新規）
├── sample-tests/
│   └── demo.bats                 # サンプルテスト
├── sample-reports/               # サンプルレポート出力
├── docs/
│   └── REPORT-FORMATS.md         # レポート形式ガイド（新規）
└── README.md                     # このファイル
```
