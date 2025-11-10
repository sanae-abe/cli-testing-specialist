# CLI Testing Specialist Agent - TODO

**最終更新**: 2025-11-10
**フェーズ**: Phase 2.6開始（ディレクトリ走査制限テスト実装）

---

## 📊 進捗サマリー

| カテゴリ | 完了 | 残り | 進捗率 |
|---------|------|------|--------|
| **Phase 2.5** | ✅ 100% | - | 完了 |
| **i18n Week 1** | ✅ 100% | - | 完了 |
| **i18n Week 2** | ✅ 100% | - | 完了 |
| **i18n Week 3** | ✅ 100% | - | 完了 |
| **i18n実装** | ✅ 100% | - | 完全完了 |
| **Phase 2.6** | ⏳ 0% | 5 | 設計完了、実装待ち |

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

## ✅ i18n Week 1完了項目（2025-11-10完了）

- [x] `utils/i18n-loader.sh` 実装（210行）
  - [x] 言語検出（$LANGから自動検出、ホワイトリスト検証）
  - [x] i18nファイルバリデーション
  - [x] 遅延読み込みパターン（load_i18n_once）
  - [x] メッセージ取得関数（msg()）
  - [x] セキュリティ対策（readonly変数、入力検証）
- [x] `i18n/ja.sh` 作成（31メッセージキー）
- [x] `i18n/en.sh` 作成（31メッセージキー）
- [x] `utils/logger.sh` 統合済み
- [x] `core/cli-analyzer.sh` 統合済み
- [x] 日本語環境テスト（LANG=ja_JP.UTF-8）
- [x] 英語環境テスト（LANG=en_US.UTF-8）
- [x] Bash互換性対応（Bash 5.3.3推奨、連想配列対応）
- [x] コミット＆プッシュ

## ✅ README多言語化完了（2025-11-10完了）

- [x] README.md → README.ja.md リネーム
- [x] README.md（英語版）新規作成（377行）
- [x] 言語切り替えリンク追加（両ファイル）
- [x] i18n-engineer subagent使用
- [x] コミット＆プッシュ

## ✅ CI/CD改善（2025-11-10完了）

- [x] GitHub Actions artifact actions v3 → v4アップグレード
- [x] deprecated警告解消
- [x] コミット＆プッシュ

## ✅ i18n Week 2-3完了項目（2025-11-10完了）

### Week 2: ドキュメント英語化
- [x] `docs/PHASE25-FINAL-REPORT.en.md` 英語版作成（9.2KB）
  - [x] i18n-engineer subagent使用
  - [x] 構造・統計データ完全保持
  - [x] 専門用語統一（Input Validation, Destructive Operations等）

### Week 3: 翻訳貢献ガイド
- [x] `i18n/README.md` 作成（バイリンガル対応）
  - [x] 新言語追加方法
  - [x] 翻訳品質ガイドライン
  - [x] テスト手順
  - [x] コミュニティ貢献プロセス

### テンプレート・設定（確認完了）
- [x] `templates/input-validation.fragment` - 既に英語コメント
- [x] `templates/destructive-ops.fragment` - 既に英語コメント
- [x] `config/option-patterns.yaml` - 既に英語コメント
- [x] `config/numeric-constraints.yaml` - 既に英語コメント
- [x] `config/enum-definitions.yaml` - 既に英語コメント

### 最終テスト・検証
- [x] 日本語環境（LANG=ja_JP.UTF-8）テスト - 正常動作
- [x] 英語環境（LANG=en_US.UTF-8）テスト - 正常動作
- [x] i18nメッセージ表示確認 - 両言語完全動作
- [x] JSON出力検証 - 正常生成

### コミット＆プッシュ
- [x] コミット作成（Conventional Commits）
- [x] リモートプッシュ完了

---

## 🌍 i18n実装（v1.0.0向け）

### ✅ Week 1: i18n機構実装（完了）

#### 1. `utils/i18n-loader.sh` 作成
- [x] ディレクトリ作成: `mkdir -p utils i18n`
- [x] グローバル変数定義: `MESSAGES`, `I18N_LOADED`
- [x] `detect_language()` 関数実装
  - [x] `$LANG`環境変数から言語コード抽出
  - [x] ホワイトリスト検証（`ja|en`）
  - [x] デフォルト値設定（`en`）
- [x] `validate_i18n_file()` 関数実装
  - [x] ファイル存在確認
  - [x] `declare -A MESSAGES`パターンチェック
  - [x] エラーメッセージ出力
- [x] `load_i18n_once()` 関数実装
  - [x] 読み込み済みチェック（`I18N_LOADED`フラグ）
  - [x] 言語コードのホワイトリスト再検証
  - [x] i18nファイルバリデーション呼び出し
  - [x] `source`による読み込み
  - [x] `readonly`設定（`MESSAGES`, `CLI_TEST_LANG`）
- [x] `msg()` ヘルパー関数実装
  - [x] 連想配列アクセス
  - [x] 存在しないキーのエラーハンドリング
- [x] 環境変数初期化: `CLI_TEST_LANG`

#### 2. `i18n/ja.sh` 作成（既存メッセージ抽出）
- [x] `core/cli-analyzer.sh`からメッセージ抽出（約15箇所）
  - [x] "Starting CLI tool analysis"
  - [x] "CLI analysis completed"
  - [x] "Binary validation passed"
  - [x] 等
- [x] `core/test-generator.sh`からメッセージ抽出（約10箇所）
  - [x] "Test generation started"
  - [x] "Template cache loaded"
  - [x] 等
- [x] `utils/logger.sh`からメッセージ抽出（約5箇所）
  - [x] ログレベルメッセージ
- [x] Key-Value形式に変換（約30メッセージ）
  - [x] キー命名規則: `snake_case`
  - [x] 値: 既存の日本語メッセージ

#### 3. `i18n/en.sh` 作成（英訳）
- [x] `ja.sh`と同じキー構造を維持
- [x] 全メッセージを英訳（約30メッセージ）
- [x] ネイティブチェック推奨（可能であれば）

#### 4. `utils/logger.sh` 修正
- [x] `source utils/i18n-loader.sh`追加
- [x] `load_i18n_once`呼び出し追加（スクリプト冒頭）
- [x] `log()`関数内で`msg()`使用
  - [x] ハードコードメッセージを`msg()`に置換
- [x] 既存ログ出力の動作確認

#### 5. `core/cli-analyzer.sh` 修正
- [x] `source utils/i18n-loader.sh`追加
- [x] `load_i18n_once`呼び出し追加（スクリプト冒頭）
- [x] ハードコードメッセージを`msg()`に置換（約15箇所）
  - [x] 成功メッセージ
  - [x] エラーメッセージ
  - [x] 情報メッセージ

#### 6. テスト実行（後方互換性確認）
- [x] 日本語環境テスト
  - [x] `LANG=ja_JP.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl`
  - [x] メッセージが日本語で表示されることを確認
  - [x] エラーなく完了することを確認
- [x] 英語環境テスト
  - [x] `LANG=en_US.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl`
  - [x] メッセージが英語で表示されることを確認
  - [x] エラーなく完了することを確認
- [x] 環境変数明示テスト
  - [x] `CLI_TEST_LANG=en bash core/cli-analyzer.sh /usr/bin/curl`
  - [x] 明示的な指定が優先されることを確認
- [x] 既存テストスイート実行
  - [x] `bash core/run-tests.sh test-output all ./reports-ja`
  - [x] 140-160ケースすべてが成功（100%成功率維持）
- [x] パフォーマンス測定
  - [x] i18n有効/無効での実行時間比較
  - [x] オーバーヘッド<1%を確認

### ✅ Week 2: ドキュメント英語化（完了）

#### ✅ 1. README英語化（一本化方針）- 完了
- [x] `mv README.md README.ja.md`（日本語版にリネーム）
- [x] `README.md`（英語版）新規作成
  - [x] 構造維持（目次、セクション）
  - [x] セクションタイトル英訳
  - [x] 本文英訳
  - [x] コマンド例はそのまま
  - [x] ネイティブチェック推奨
- [x] 言語切り替えリンク追加
  - [x] `README.md`冒頭: `**Languages**: [English](README.md) | [日本語](README.ja.md)`
  - [x] `README.ja.md`冒頭: `**言語**: [English](README.md) | [日本語](README.ja.md)`
- [x] 更新方針の文書化（コメントまたは別ファイル）
  - [x] README.md: すべての更新を反映
  - [x] README.ja.md: 重要な更新のみ（月次レビュー）

#### ✅ 2. 主要ドキュメント英語化（完了）
- [x] `docs/PHASE25-FINAL-REPORT.md`英訳
  - [x] `docs/PHASE25-FINAL-REPORT.en.md`作成
  - [x] セクション構造維持
  - [x] 統計データ、コードブロックはそのまま
  - [x] i18n-engineer subagent使用

### ✅ Week 3: テンプレート・設定（完了）

#### ✅ 1. テンプレートコメント英語化（確認完了）
- [x] `templates/input-validation.fragment` - 既に英語コメント
- [x] `templates/destructive-ops.fragment` - 既に英語コメント

#### ✅ 2. 設定ファイルコメント英語化（確認完了）
- [x] `config/option-patterns.yaml` - 既に英語コメント
- [x] `config/numeric-constraints.yaml` - 既に英語コメント
- [x] `config/enum-definitions.yaml` - 既に英語コメント

#### ✅ 3. 翻訳貢献ガイド作成（完了）
- [x] `i18n/README.md`作成（バイリンガル）
  - [x] 翻訳方法の説明
  - [x] キー命名規則
  - [x] コミュニティ貢献方法
  - [x] 翻訳品質基準

### ✅ 検証・リリース（完了）

#### ✅ 1. 包括的テスト
- [x] 日本語環境動作確認
  - [x] `LANG=ja_JP.UTF-8 bash core/cli-analyzer.sh /bin/ls` - 正常動作
  - [x] i18nメッセージ日本語表示確認
- [x] 英語環境動作確認
  - [x] `LANG=en_US.UTF-8 bash core/cli-analyzer.sh /bin/ls` - 正常動作
  - [x] i18nメッセージ英語表示確認
- [x] JSON出力検証 - 正常生成

#### ✅ 2. ドキュメントレビュー
- [x] README.md（英語版）作成確認
- [x] README.ja.md（日本語版）整合性確認
- [x] PHASE25-FINAL-REPORT.en.md作成確認

#### ✅ 3. リリース準備
- [x] コミット作成（Conventional Commits）
- [x] リモートプッシュ完了

---

## 🚀 Phase 2.6: ディレクトリ走査制限テスト（Phase 1-3完了、Phase 4-5進行中）

**設計ドキュメント**: [DIRECTORY-TRAVERSAL-LIMITS-DESIGN.md](docs/DIRECTORY-TRAVERSAL-LIMITS-DESIGN.md)
**設計レビュー**: 2025-11-10（3ラウンド反復レビュー完了）
**実装開始**: 2025-11-10
**総推定時間**: 9-14時間
**実績時間**: Phase 1-3完了（約5-8時間）

### 📋 背景
- backup-suiteで`~/`配下全体を走査して30分以上固まった問題を検出
- 膨大なディレクトリ走査による固まり問題を自動検出する新テストパターン
- テスト項目: 12パターン（段階的ファイル処理、深い階層、シンボリックリンクループ等）

### Phase 1: セキュリティ強化（最優先）✅ 100%

**推定時間**: 2-3時間
**実績**: 完了（templates/directory-traversal-limits.fragment: 465行）

- [x] ホームディレクトリテストのモック化実装
  - [x] モックホームディレクトリ作成関数（`/tmp/cli-test-$$-fake-home`）
  - [x] HOME環境変数の一時的変更と復元処理
  - [x] 実際のホームディレクトリを絶対使用しない安全策
- [x] シンボリックリンクループの安全な実装
  - [x] 相対パスでの循環参照作成（`ln -s ../b a/link_to_b`）
  - [x] 二重ランダム化（`/tmp/cli-test-$$-$RANDOM-symlink`）
  - [x] 安全なクリーンアップ処理（`rm -f`シンボリックリンクのみ）
- [x] trap処理の実装
  - [x] `cleanup_all()`関数作成
  - [x] `trap cleanup_all EXIT ERR INT TERM`設定
  - [x] HOME環境変数復元処理
- [x] テストディレクトリ権限設定
  - [x] `chmod 700`の全テストディレクトリへの適用
  - [x] `mktemp -d`での安全な作成
- [x] ulimitリソース制限
  - [x] メモリ制限: 2GB（`ulimit -m 2097152`）
  - [x] ファイル数制限: 2048個（`ulimit -n 2048`）
  - [x] プロセス数制限: 100個（`ulimit -u 100`）

### Phase 2: パフォーマンス最適化 ✅ 100%

**推定時間**: 2-3時間
**実績**: 完了（core/test-generator.sh統合: +77行）

- [x] setup_file()/teardown_file()実装
  - [x] `setup_file()`でテストディレクトリ一括作成
  - [x] `teardown_file()`で全ディレクトリクリーンアップ
  - [x] 環境変数エクスポート（TEST_DIR_100, TEST_DIR_500等）
- [x] ヘルパー関数実装
  - [x] `populate_test_dir(dir, count)` - ダミーファイル作成
  - [x] `create_deep_dir(dir, depth)` - 深い階層作成
  - [x] `create_symlink_loop(dir)` - シンボリックリンクループ
  - [x] `create_fake_home(dir)` - モックホームディレクトリ
  - [x] `cleanup_test_dir(dir)` - 安全なクリーンアップ
- [x] 段階的テスト実装（100→500→1,000ファイル）
  - [x] テスト1-a: 100ファイル、5秒以内
  - [x] テスト1-b: 500ファイル、15秒以内（1-a成功時のみ）
  - [x] テスト1-c: 1,000ファイル、30秒以内（1-b成功時のみ）
- [x] 早期フェイル戦略
  - [x] 小規模テスト失敗時の大規模テストスキップロジック
- [x] CI環境でのスキップオプション
  - [x] `SKIP_DIRECTORY_TRAVERSAL_TESTS=1`環境変数対応
  - [x] `/tmp`容量チェック（`df -h /tmp`）

### Phase 3: 保守性向上 ✅ 100%

**推定時間**: 1-2時間
**実績**: 完了（全チェック項目クリア）

- [x] 既存テンプレート比較表の検証
  - [x] destructive-ops, input-validationとの整合性確認
  - [x] 変数命名規則の統一
- [x] 変数定義の実装
  - [x] CLI_BINARY, TRAVERSAL_COMMAND, TRAVERSAL_ARGS
  - [x] TEST_DIR_100, TEST_DIR_500, TEST_DIR_1000
  - [x] TEST_DIR_DEEP, TEST_DIR_SYMLINK, FAKE_HOME
- [x] テスト出力フォーマット統一
  - [x] 成功時: `✓ [directory-traversal] ...`
  - [x] 失敗時: `✗ [directory-traversal] ...`
  - [x] スキップ時: `- [directory-traversal] ...`
- [x] 英語コメント追加
  - [x] 各テストケースの説明コメント
  - [x] ヘルパー関数のドキュメント
- [x] BATS file_tags追加
  - [x] `# bats file_tags=sequential,resource-intensive`

### Phase 4: 実装＆テスト ⏳ 50%

**推定時間**: 3-4時間
**実績**: テンプレート・統合完了、実CLIテスト未実施

- [x] `templates/directory-traversal-limits.fragment`作成
  - [x] テスト1-a, 1-b, 1-c（段階的ファイル処理）
  - [x] テスト2（深い階層の再帰制限）
  - [x] テスト3（シンボリックリンクループ検出）
  - [x] テスト4（ホームディレクトリ警告、モック環境）
  - [x] テスト5（SIGINT応答性）
  - [x] テスト6（タイムアウトオプション）
  - [x] テスト7（進捗表示）
  - [x] テスト8（ファイル数上限オプション）
  - [x] テスト9（メモリ使用量）
  - [x] テスト10（パフォーマンス警告ドキュメント）
- [x] `core/test-generator.sh`統合
  - [x] `generate_directory_traversal_tests()`関数追加
  - [x] ディレクトリ走査コマンド抽出ロジック（scan|analyze|backup等）
  - [x] メインケース文への追加（directory-traversal, all）
- [ ] 単体テスト（各ヘルパー関数）
  - [ ] populate_test_dir()テスト
  - [ ] create_deep_dir()テスト
  - [ ] create_symlink_loop()テスト
  - [ ] create_fake_home()テスト
  - [ ] cleanup_test_dir()テスト
- [ ] 実CLIツールでの動作確認
  - [ ] backup-suiteでのテスト実行
  - [ ] curl, npm, python3での動作確認
  - [ ] 期待通りの検出結果確認

### Phase 5: ドキュメント完成 ⏳ 30%

**推定時間**: 1-2時間
**実績**: TODO.md更新進行中

- [x] TODO.md更新（Phase 2.6進捗反映）
- [ ] README.md更新
  - [ ] 新テストパターン追加（directory-traversal-limits）
  - [ ] テスト項目12パターンの説明
  - [ ] セキュリティ実装ガイドラインのリンク
- [ ] 実行方法の明記（並列実行禁止）
  - [ ] `bats test-output/10-directory-traversal-limits.bats`（順次実行）
  - [ ] `bats --jobs 4 test-output/`（並列実行禁止の警告）
- [ ] トラブルシューティングセクション
  - [ ] /tmp満杯時の対処法
  - [ ] テスト失敗時のクリーンアップ方法
  - [ ] Bash 5+要件の説明（連想配列対応）
- [ ] 変更履歴追加
  - [ ] Phase 2.6実装完了の記録
  - [ ] 新テストパターン追加の記録
- [ ] 最終レビュー
  - [ ] /iterative-review によるドキュメントレビュー
  - [ ] 設計ドキュメントとの整合性確認

### 🎯 Phase 2.6成功基準

1. **実用性**: backup-suite問題を実際に検出できる
2. **移植性**: 他のCLIツール（curl, npm, python3等）にも適用可能
3. **保守性**: 既存パターン（destructive-ops, input-validation）と統一されたコード
4. **パフォーマンス**:
   - テスト準備時間: <30秒（setup_file()での環境作成）
   - テスト実行時間: <2分（12テスト合計）
   - 総合実行時間: <3分（準備+実行）
5. **信頼性**: false positive/negativeが少ない
6. **安全性**: テスト環境の完全な分離と自動クリーンアップ

---

## 📚 参考ドキュメント

- [I18N実装計画詳細](docs/I18N-IMPLEMENTATION-PLAN.md)
- [Phase 2.5最終レポート](docs/PHASE25-FINAL-REPORT.md)
- [入力検証ガイド](docs/INPUT-VALIDATION-GUIDE.md)
- [ディレクトリ走査制限設計](docs/DIRECTORY-TRAVERSAL-LIMITS-DESIGN.md) ⭐ NEW

---

**管理**: このTODOリストはdocs/I18N-IMPLEMENTATION-PLAN.md、docs/DIRECTORY-TRAVERSAL-LIMITS-DESIGN.mdと同期
**更新頻度**: 週次または実装進捗に応じて
