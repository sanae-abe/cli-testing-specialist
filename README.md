# CLI Testing Specialist Agent

**開発ステータス**: Phase 2.5 完了（2025-11-10）
**最終更新**: 2025-11-10
**リリース予定**: v1.0.0 (2026-02-07)
**Claude Code専用**: セキュアで包括的なCLIツールテストフレームワーク

---

## 📑 目次

- [概要](#概要)
- [クイックスタート](#クイックスタート)
- [インストール](#インストール)
- [機能一覧](#機能一覧phase-25完了時点)
- [レポート形式](#レポート形式)
- [CI/CD統合](#cicd統合)
- [セキュリティ機能](#セキュリティ機能)
- [設定](#設定)
- [ファイル構造](#ファイル構造)
- [サンプルレポート](#サンプルレポート)
- [ライセンス](#ライセンス)
- [貢献](#貢献)
- [サポート](#サポート)

---

## 概要

CLI Testing Specialist Agentは、CLIツールの品質とセキュリティを自動検証する包括的なテストフレームワークです。

### 主要機能

- 🔒 **セキュリティテスト**: OWASP準拠の自動スキャン
- ✅ **包括的検証**: 11カテゴリ、140-160テストケース
- 🎯 **入力検証テスト** (Phase 2.5): 数値/パス/列挙型オプション自動検証
- 🛡️ **破壊的操作テスト** (Phase 2.5): 確認プロンプト・安全性検証
- 🐚 **マルチシェル対応**: bash/zsh（将来対応予定: fish）
- 📊 **詳細レポート**: Markdown/JSON/HTML/JUnit XML
- 🔄 **CI/CD統合**: GitHub Actions & GitLab CI対応
- 🐳 **Docker統合**: 隔離環境でのテスト実行（オプション）
- ⚡ **高速化** (Phase 2.5): テスト生成5-10倍高速化

---

## クイックスタート

```bash
# 1. CLIツールを解析
bash core/cli-analyzer.sh /usr/local/bin/your-cli

# 2. テストを生成（全カテゴリ）
bash core/test-generator.sh cli-analysis.json test-output all

# 3. テストを実行
bats test-output/*.bats

# 4. HTMLレポートを生成
bash core/run-tests.sh test-output html ./reports

# 5. ブラウザで開く
open reports/test-report.html  # macOS
# xdg-open reports/test-report.html  # Linux
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

### 依存関係

CLI Testing Specialist Agentは以下のツールに依存しています：

#### 必須（コア機能）
- **Bash 4.0+**: テストエンジンの実行環境
- **jq**: JSON処理（CLIメタデータ解析、レポート生成）
- **BATS**: テスト実行フレームワーク
  ```bash
  # macOS
  brew install bats-core

  # Ubuntu/Debian
  apt-get install bats

  # Manual installation
  git clone https://github.com/bats-core/bats-core.git
  cd bats-core
  sudo ./install.sh /usr/local
  ```

#### Phase 2.5+ 必須（入力検証機能）
- **yq v4.x**: YAML処理（オプション型推論、制約定義）
  ```bash
  # macOS
  brew install yq

  # Ubuntu/Debian (snap)
  snap install yq

  # Linux (binary)
  wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
  chmod +x yq_linux_amd64
  sudo mv yq_linux_amd64 /usr/local/bin/yq

  # Verify installation
  yq --version  # Should show: yq (https://github.com/mikefarah/yq/) version 4.x
  ```

#### オプション（拡張機能）
- **SQLite3**: カバレッジトラッキング（Phase 2機能）
  ```bash
  # macOS
  brew install sqlite3

  # Ubuntu/Debian
  apt-get install sqlite3
  ```
- **Docker**: 隔離環境でのテスト実行
- **envsubst** (gettext): テンプレート変数置換（Bashフォールバックあり）

#### 依存関係チェック

```bash
# 必須ツールの確認
command -v bash && echo "✓ Bash"
command -v jq && echo "✓ jq"
command -v bats && echo "✓ BATS"
command -v yq && echo "✓ yq (Phase 2.5+)"
command -v sqlite3 && echo "✓ SQLite3 (Phase 2)"

# yq version check (must be v4.x for Phase 2.5)
yq --version 2>&1 | grep -q "version 4" && echo "✓ yq v4.x" || echo "⚠ yq v4.x required"
```

---

## 機能一覧（Phase 2.5完了時点）

| カテゴリ | 説明 | テスト数 |
|---------|------|---------|
| 基本動作検証 | ヘルプ、バージョン、終了コード | 10 |
| サブコマンドヘルプ | 全サブコマンドの網羅的検証 | 動的 |
| セキュリティ | インジェクション、機密漏洩、TOCTOU | 25 |
| パス処理 | 特殊文字、深い階層、Unicode | 20 |
| マルチシェル | bash/zsh互換性 | 12 |
| 入力検証（基本） | 不正値、エッジケース | 12 |
| **入力検証（拡張）** 🆕 | **数値/パス/列挙型オプション検証** | **25** |
| **破壊的操作** 🆕 | **確認プロンプト、--yes/--forceフラグ** | **16** |
| 出力検証 | フォーマット、カラー出力 | 8 |
| 環境依存 | OS、環境変数 | 10 |
| パフォーマンス | 起動時間、メモリ使用量 | 6 |
| ドキュメント整合性 | README vs ヘルプ | 5 |
| **レポート** | **4形式（Markdown/JSON/HTML/JUnit）** | - |

**合計**: 約140-160テストケース（Phase 2.5で41テストケース追加）

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

**サンプルレポート**: [`sample-reports/test-report.html`](sample-reports/test-report.html)

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
  - [`INPUT-VALIDATION-GUIDE.md`](docs/INPUT-VALIDATION-GUIDE.md) - 入力検証ガイド
  - [`INPUT-VALIDATION-PLAN-v2.md`](docs/INPUT-VALIDATION-PLAN-v2.md) - Phase 2.5実装計画
  - [`PHASE2-PLAN.md`](docs/PHASE2-PLAN.md) - Phase 2実装計画
  - [`PHASE25-FINAL-REPORT.md`](docs/PHASE25-FINAL-REPORT.md) - Phase 2.5最終レポート
- **Issues**: GitHub Issues

---

## ファイル構造

```
cli-testing-specialist/
├── core/
│   ├── cli-analyzer.sh            # CLI解析エンジン
│   ├── test-generator.sh          # BATS生成エンジン（Phase 2.5拡張）
│   ├── option-analyzer.sh         # オプション型推論エンジン（Phase 2.5新規）
│   ├── coverage-tracker.sh        # カバレッジトラッキング（Phase 2）
│   ├── run-tests.sh               # テスト実行＆レポート生成
│   ├── report-generator-html.sh   # HTMLレポート生成
│   ├── shell-detector.sh          # Shell検出エンジン
│   └── validator.sh               # 入力検証エンジン
├── config/                        # Phase 2.5新規
│   ├── option-patterns.yaml       # オプション型パターン定義
│   ├── numeric-constraints.yaml   # 数値制約定義
│   └── enum-definitions.yaml      # 列挙型定義
├── templates/                     # Phase 2.5新規
│   ├── bats-test.template         # BATSテンプレート
│   ├── input-validation.fragment  # 入力検証テストフラグメント
│   └── destructive-ops.fragment   # 破壊的操作テストフラグメント
├── docs/
│   ├── REPORT-FORMATS.md          # レポート形式ガイド
│   ├── INPUT-VALIDATION-GUIDE.md  # 入力検証ガイド（Phase 2.5新規）
│   ├── PHASE2-PLAN.md             # Phase 2実装計画
│   └── INPUT-VALIDATION-PLAN-v2.md # Phase 2.5実装計画
├── .github/workflows/cli-test.yml # GitHub Actions設定
├── .gitlab-ci.yml                 # GitLab CI設定
├── sample-tests/demo.bats         # サンプルテスト
├── sample-reports/                # サンプルレポート出力
└── README.md                      # このファイル
```

---

