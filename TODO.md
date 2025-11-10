# CLI Testing Specialist Agent - TODO

**最終更新**: 2025-11-10
**フェーズ**: v1.0.0リリース準備（Phase 2.5完了、i18n実装予定）

---

## 📊 進捗サマリー

| カテゴリ | 完了 | 残り | 進捗率 |
|---------|------|------|--------|
| **Phase 2.5** | ✅ 100% | - | 完了 |
| **i18n実装** | 0% | 100% | 未着手 |

---

## ✅ Phase 2.5完了項目（2025-11-10完了）

- [x] 入力検証テスト実装（25パターン）
- [x] 破壊的操作テスト実装（16パターン）
- [x] データ駆動設計（YAML設定500行）
- [x] jq JSON破損問題修正
- [x] セキュリティ監査（0脆弱性）
- [x] パフォーマンス測定（2.79秒、目標<5秒達成）
- [x] 実CLIツールテスト（curl, npm, python3）
- [x] Phase 2.5最終レポート作成
- [x] README.md更新・クリーンアップ
- [x] コミット＆プッシュ

---

## 🌍 i18n実装（v1.0.0向け）

### Week 1: i18n機構実装

#### 1. `utils/i18n-loader.sh` 作成
- [ ] ディレクトリ作成: `mkdir -p utils i18n`
- [ ] グローバル変数定義: `MESSAGES`, `I18N_LOADED`
- [ ] `detect_language()` 関数実装
  - [ ] `$LANG`環境変数から言語コード抽出
  - [ ] ホワイトリスト検証（`ja|en`）
  - [ ] デフォルト値設定（`en`）
- [ ] `validate_i18n_file()` 関数実装
  - [ ] ファイル存在確認
  - [ ] `declare -A MESSAGES`パターンチェック
  - [ ] エラーメッセージ出力
- [ ] `load_i18n_once()` 関数実装
  - [ ] 読み込み済みチェック（`I18N_LOADED`フラグ）
  - [ ] 言語コードのホワイトリスト再検証
  - [ ] i18nファイルバリデーション呼び出し
  - [ ] `source`による読み込み
  - [ ] `readonly`設定（`MESSAGES`, `CLI_TEST_LANG`）
- [ ] `msg()` ヘルパー関数実装
  - [ ] 連想配列アクセス
  - [ ] 存在しないキーのエラーハンドリング
- [ ] 環境変数初期化: `CLI_TEST_LANG`

#### 2. `i18n/ja.sh` 作成（既存メッセージ抽出）
- [ ] `core/cli-analyzer.sh`からメッセージ抽出（約15箇所）
  - [ ] "Starting CLI tool analysis"
  - [ ] "CLI analysis completed"
  - [ ] "Binary validation passed"
  - [ ] 等
- [ ] `core/test-generator.sh`からメッセージ抽出（約10箇所）
  - [ ] "Test generation started"
  - [ ] "Template cache loaded"
  - [ ] 等
- [ ] `utils/logger.sh`からメッセージ抽出（約5箇所）
  - [ ] ログレベルメッセージ
- [ ] Key-Value形式に変換（約30メッセージ）
  - [ ] キー命名規則: `snake_case`
  - [ ] 値: 既存の日本語メッセージ

#### 3. `i18n/en.sh` 作成（英訳）
- [ ] `ja.sh`と同じキー構造を維持
- [ ] 全メッセージを英訳（約30メッセージ）
- [ ] ネイティブチェック推奨（可能であれば）

#### 4. `utils/logger.sh` 修正
- [ ] `source utils/i18n-loader.sh`追加
- [ ] `load_i18n_once`呼び出し追加（スクリプト冒頭）
- [ ] `log()`関数内で`msg()`使用
  - [ ] ハードコードメッセージを`msg()`に置換
- [ ] 既存ログ出力の動作確認

#### 5. `core/cli-analyzer.sh` 修正
- [ ] `source utils/i18n-loader.sh`追加
- [ ] `load_i18n_once`呼び出し追加（スクリプト冒頭）
- [ ] ハードコードメッセージを`msg()`に置換（約15箇所）
  - [ ] 成功メッセージ
  - [ ] エラーメッセージ
  - [ ] 情報メッセージ

#### 6. テスト実行（後方互換性確認）
- [ ] 日本語環境テスト
  - [ ] `LANG=ja_JP.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl`
  - [ ] メッセージが日本語で表示されることを確認
  - [ ] エラーなく完了することを確認
- [ ] 英語環境テスト
  - [ ] `LANG=en_US.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl`
  - [ ] メッセージが英語で表示されることを確認
  - [ ] エラーなく完了することを確認
- [ ] 環境変数明示テスト
  - [ ] `CLI_TEST_LANG=en bash core/cli-analyzer.sh /usr/bin/curl`
  - [ ] 明示的な指定が優先されることを確認
- [ ] 既存テストスイート実行
  - [ ] `bash core/run-tests.sh test-output all ./reports-ja`
  - [ ] 140-160ケースすべてが成功（100%成功率維持）
- [ ] パフォーマンス測定
  - [ ] i18n有効/無効での実行時間比較
  - [ ] オーバーヘッド<1%を確認

### Week 2: ドキュメント英語化

#### 1. README英語化（一本化方針）
- [ ] `mv README.md README.ja.md`（日本語版にリネーム）
- [ ] `README.md`（英語版）新規作成
  - [ ] 構造維持（目次、セクション）
  - [ ] セクションタイトル英訳
  - [ ] 本文英訳
  - [ ] コマンド例はそのまま
  - [ ] ネイティブチェック推奨
- [ ] 言語切り替えリンク追加
  - [ ] `README.md`冒頭: `**Languages**: [English](README.md) | [日本語](README.ja.md)`
  - [ ] `README.ja.md`冒頭: `**言語**: [English](README.md) | [日本語](README.ja.md)`
- [ ] 更新方針の文書化（コメントまたは別ファイル）
  - [ ] README.md: すべての更新を反映
  - [ ] README.ja.md: 重要な更新のみ（月次レビュー）

#### 2. 主要ドキュメント英語化
- [ ] `docs/PHASE25-FINAL-REPORT.md`英訳
  - [ ] `docs/PHASE25-FINAL-REPORT.en.md`作成
  - [ ] セクション構造維持
  - [ ] 統計データ、コードブロックはそのまま
  - [ ] ネイティブチェック推奨

### Week 3: テンプレート・設定

#### 1. テンプレートコメント英語化
- [ ] `templates/input-validation.fragment`
  - [ ] 日本語コメントの英訳を追加（バイリンガル対応）
  - [ ] テストコード自体は変更なし
- [ ] `templates/destructive-ops.fragment`
  - [ ] 日本語コメントの英訳を追加（バイリンガル対応）
  - [ ] テストコード自体は変更なし

#### 2. 設定ファイルコメント英語化
- [ ] `config/option-patterns.yaml`
  - [ ] YAMLコメントに英訳追加
- [ ] `config/numeric-constraints.yaml`
  - [ ] YAMLコメントに英訳追加
- [ ] `config/enum-definitions.yaml`
  - [ ] YAMLコメントに英訳追加

#### 3. 翻訳貢献ガイド作成
- [ ] `i18n/README.md`作成
  - [ ] 翻訳方法の説明
  - [ ] キー命名規則
  - [ ] コミュニティ貢献方法
  - [ ] 翻訳品質基準

### 検証・リリース

#### 1. 包括的テスト
- [ ] 全テストケースで日本語動作確認
  - [ ] `LANG=ja_JP.UTF-8 bats test-output/*.bats`
  - [ ] 140-160ケース成功
- [ ] 全テストケースで英語動作確認
  - [ ] `LANG=en_US.UTF-8 bats test-output/*.bats`
  - [ ] 140-160ケース成功
- [ ] 実CLIツールテスト（curl, npm）
  - [ ] 日本語環境でテスト生成・実行
  - [ ] 英語環境でテスト生成・実行

#### 2. ドキュメントレビュー
- [ ] README.md（英語版）レビュー
- [ ] README.ja.md（日本語版）整合性確認
- [ ] 翻訳品質チェック（可能であればネイティブ）

#### 3. リリース準備
- [ ] `.gitignore`更新（必要に応じて）
- [ ] コミット作成（Conventional Commits）
- [ ] v1.0.0リリースノート準備
- [ ] GitHub Releasesページ作成（英語/日本語）

---

## 📚 参考ドキュメント

- [I18N実装計画詳細](docs/I18N-IMPLEMENTATION-PLAN.md)
- [Phase 2.5最終レポート](docs/PHASE25-FINAL-REPORT.md)
- [入力検証ガイド](docs/INPUT-VALIDATION-GUIDE.md)

---

**管理**: このTODOリストはdocs/I18N-IMPLEMENTATION-PLAN.mdと同期
**更新頻度**: 週次または実装進捗に応じて
