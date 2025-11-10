# CLI Testing Specialist Agent

**バージョン**: 1.1.0
**ステータス**: 開発中（Phase 1）
**Claude Code専用**: セキュアで包括的なCLIツールテストフレームワーク

---

## 概要

CLI Testing Specialist Agentは、CLIツールの品質とセキュリティを自動検証する包括的なテストフレームワークです。

### 主要機能

- 🔒 **セキュリティテスト**: OWASP準拠の自動スキャン
- ✅ **包括的検証**: 11カテゴリ、100-120テストケース
- 🐚 **マルチシェル対応**: bash/zsh（v1.1でfish対応）
- 📊 **詳細レポート**: Markdown/JSON/HTML/JUnit XML
- 🐳 **Docker統合**: 隔離環境でのテスト実行（オプション）

---

## クイックスタート

```bash
# 基本的な使用
cli-test your-cli

# セキュリティテストのみ
cli-test your-cli --category security

# 詳細レポート生成
cli-test your-cli --report markdown,html --output ./reports/
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

## 機能一覧（v1.0）

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
| レポート | 4形式（Markdown/JSON/HTML/JUnit） | - |

**合計**: 約100-120テストケース

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
- 🔄 **Week 1**: 基盤構築（validator, logger, analyzer）
- ⏳ **Week 2-3**: コアテストモジュール
- ⏳ **Week 4**: 環境互換性
- ⏳ **Week 5-6**: 追加モジュール・レポート
- ⏳ **Week 7-8**: 統合・最適化
- ⏳ **Week 9**: ドキュメント
- ⏳ **Week 10-12**: 検証・リリース

### マイルストーン

- **2025-12-20**: Week 4終了（コア機能完成）
- **2026-01-10**: Week 8終了（β版）
- **2026-02-07**: v1.0.0リリース

---

## ライセンス

MIT License

---

## 貢献

プルリクエストを歓迎します。大きな変更の場合は、まずissueを開いて変更内容を議論してください。

---

## サポート

- **ドキュメント**: `docs/` ディレクトリ
- **Issues**: GitHub Issues
- **実装計画書**: `~/.claude/docs/cli-testing-agent-implementation-plan-v1.1.md`
- **レビューレポート**: `~/.claude/docs/cli-testing-agent-review-report.md`
