# CLI Testing Specialist Agent - i18n実装計画

**作成日**: 2025-11-10
**対象バージョン**: v1.0.0
**優先度**: Medium

---

## 📋 目次

- [概要](#概要)
- [目標](#目標)
- [対応言語](#対応言語)
- [i18n機構設計](#i18n機構設計)
- [実装範囲](#実装範囲)
- [ディレクトリ構造](#ディレクトリ構造)
- [実装手順](#実装手順)
- [検証基準](#検証基準)
- [リスク評価](#リスク評価)
- [TODO](#todo)

---

## 概要

CLI Testing Specialist Agentを国際化（i18n）し、英語対応を実装する。
現在は完全に日本語専用のため、以下を多言語化する：

- ドキュメント（README, docs/）
- ログメッセージ
- エラーメッセージ
- テストテンプレート
- 設定ファイルのコメント

---

## 目標

### 主要目標
1. **ドキュメントの英語化**: README.md, 主要ドキュメント
2. **ログ・エラーメッセージの多言語化**: Bash i18n機構の実装
3. **後方互換性の維持**: 既存の日本語環境を壊さない

### 副次目標
- 将来的な他言語対応の基盤構築
- 国際的なOSS貢献の促進

---

## 対応言語

### Phase 1（初期実装）
- **日本語** (ja): 既存（デフォルト）
- **英語** (en): 新規追加

### Phase 2（将来計画）
- 中国語 (zh)
- 韓国語 (ko)
- その他（コミュニティ貢献ベース）

---

## i18n機構設計

### 1. Bash i18n戦略

Bashスクリプトの国際化には以下の手法を採用：

#### オプションA: gettext利用（推奨）
```bash
# i18n/ja/messages.po
msgid "CLI analysis completed"
msgstr "CLI解析が完了しました"

# コード内
. gettext.sh
TEXTDOMAIN=cli-test
export TEXTDOMAIN

log INFO "$(gettext "CLI analysis completed")"
```

**利点**:
- 標準的な国際化手法
- 翻訳管理ツールが豊富
- POT/PO/MOファイル形式で管理

**欠点**:
- 依存関係が増加（gettext）
- インストール必須

#### オプションB: シンプルなKey-Valueファイル（軽量）
```bash
# i18n/ja.sh
declare -A MESSAGES=(
    [cli_analysis_completed]="CLI解析が完了しました"
    [test_generation_started]="テスト生成を開始します"
)

# i18n/en.sh
declare -A MESSAGES=(
    [cli_analysis_completed]="CLI analysis completed"
    [test_generation_started]="Starting test generation"
)

# コード内
source "i18n/${LANG}.sh"
log INFO "${MESSAGES[cli_analysis_completed]}"
```

**利点**:
- 依存関係なし
- シンプル
- 高速

**欠点**:
- 標準的な手法ではない
- 翻訳管理ツール非対応

### 2. 採用方針

**Phase 1**: オプションB（軽量Key-Value）を採用
- 理由: 依存関係を増やさない、シンプル、十分な機能
- 将来的にgettextへの移行パスを残す

### 3. 言語検出ロジック（セキュアな実装）

```bash
# utils/i18n-loader.sh

# グローバル変数（1回のみ初期化）
declare -g -A MESSAGES
declare -g I18N_LOADED=false

# 言語検出（環境変数から安全に抽出）
detect_language() {
    local lang="${LANG:-en_US.UTF-8}"
    local lang_code="${lang%%_*}"  # en_US.UTF-8 -> en

    # ホワイトリスト方式で検証
    case "$lang_code" in
        ja) echo "ja" ;;
        en) echo "en" ;;
        *) echo "en" ;;  # Default to English
    esac
}

# i18nファイルバリデーション
validate_i18n_file() {
    local file="$1"

    # ファイル存在確認
    if [[ ! -f "$file" ]]; then
        log ERROR "i18n file not found: $file"
        return 1
    fi

    # 正しい形式のみ許可（declare -A MESSAGES）
    if ! grep -q '^declare -A MESSAGES=' "$file"; then
        log ERROR "Invalid i18n file format: $file"
        return 1
    fi

    return 0
}

# i18nファイル読み込み（起動時1回のみ）
load_i18n_once() {
    # 既に読み込み済みならスキップ（パフォーマンス最適化）
    if [[ "$I18N_LOADED" == "true" ]]; then
        return 0
    fi

    # 環境変数から言語コード取得（ホワイトリスト検証済み）
    local lang_code="${CLI_TEST_LANG:-$(detect_language)}"

    # ホワイトリスト再検証（セキュリティ対策）
    case "$lang_code" in
        ja|en) ;;
        *)
            log ERROR "Invalid language code: $lang_code"
            lang_code="en"
            ;;
    esac

    local i18n_file="i18n/${lang_code}.sh"

    # ファイルバリデーション
    validate_i18n_file "$i18n_file" || return 1

    # 安全な読み込み
    source "$i18n_file"
    readonly MESSAGES
    readonly CLI_TEST_LANG="$lang_code"
    I18N_LOADED=true

    return 0
}

# メッセージ取得ヘルパー関数
msg() {
    local key="$1"
    echo "${MESSAGES[$key]:-[Missing: $key]}"
}

# 初期化（環境変数設定）
CLI_TEST_LANG="${CLI_TEST_LANG:-$(detect_language)}"
```

---

## 実装範囲

### 1. ドキュメント（最優先）

#### Phase 1: 必須ドキュメント
- [ ] `README.md` → `README.md` (英語) + `README.ja.md` (日本語)
- [ ] `docs/PHASE25-FINAL-REPORT.md` → 英語版作成

#### Phase 2: 詳細ドキュメント
- [ ] `docs/INPUT-VALIDATION-GUIDE.md`
- [ ] `docs/REPORT-FORMATS.md`
- [ ] `docs/I18N-IMPLEMENTATION-PLAN.md` (このファイル)

### 2. コード内メッセージ

#### 優先順位: 高
- [ ] `core/cli-analyzer.sh`: 解析プロセスのログ
- [ ] `core/test-generator.sh`: テスト生成ログ
- [ ] `utils/logger.sh`: ログ関数

#### 優先順位: 中
- [ ] `core/validator.sh`: バリデーションエラー
- [ ] `core/coverage-tracker.sh`: カバレッジメッセージ

#### 優先順位: 低
- [ ] その他のユーティリティスクリプト

### 3. テストテンプレート

#### コメントの多言語化
- [ ] `templates/input-validation.fragment`: テストコメント
- [ ] `templates/destructive-ops.fragment`: テストコメント

**方針**: テストコードは英語コメント、説明文のみ多言語化

### 4. 設定ファイル

#### YAMLコメントの多言語化
- [ ] `config/option-patterns.yaml`: 説明コメント
- [ ] `config/numeric-constraints.yaml`: 説明コメント
- [ ] `config/enum-definitions.yaml`: 説明コメント

---

## ディレクトリ構造

```
cli-testing-specialist/
├── i18n/                          # 新規ディレクトリ
│   ├── ja.sh                      # 日本語メッセージ
│   ├── en.sh                      # 英語メッセージ
│   ├── messages.pot               # POTテンプレート（将来用）
│   └── README.md                  # 翻訳貢献ガイド
├── utils/
│   └── i18n-loader.sh             # 新規: i18n読み込みユーティリティ
├── docs/
│   ├── I18N-IMPLEMENTATION-PLAN.md  # このファイル
│   └── (既存ドキュメント)
├── README.md                      # 英語版（メイン）
├── README.ja.md                   # 日本語版（新規）
└── (既存ファイル)
```

---

## 実装手順

### Phase 1: 基盤構築（Week 1）

#### Step 1: i18n機構の実装
```bash
# 1. utils/i18n-loader.sh作成
- 言語検出関数
- メッセージファイル読み込み
- msg() ヘルパー関数

# 2. i18n/ja.sh作成
- 既存の日本語メッセージを抽出
- Key-Value形式に変換

# 3. i18n/en.sh作成
- 日本語メッセージを英訳
- 同じキー構造を維持
```

#### Step 2: コア機能への統合
```bash
# 1. utils/logger.sh修正
- i18n-loader.shを読み込み
- log関数でmsg()を使用

# 2. core/cli-analyzer.sh修正
- ハードコードされた日本語を msg() に置き換え
```

#### Step 3: テスト・検証
```bash
# 1. 日本語環境でテスト
LANG=ja_JP.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl

# 2. 英語環境でテスト
LANG=en_US.UTF-8 bash core/cli-analyzer.sh /usr/bin/curl

# 3. 明示的な言語指定
CLI_TEST_LANG=en bash core/cli-analyzer.sh /usr/bin/curl
```

### Phase 2: ドキュメント英語化（Week 2）

#### Step 1: README英語化（一本化方針）
```bash
# 1. 現在のREADME.md（日本語）をREADME.ja.mdにリネーム
mv README.md README.ja.md

# 2. README.md（英語版）を新規作成
# - 構造は維持
# - セクションタイトル、本文を英訳
# - コマンド例はそのまま
# - ネイティブチェック推奨

# 3. 今後の更新方針
# - README.md（英語）: すべての更新を反映（メイン）
# - README.ja.md（日本語）: 重要な更新のみ（月次レビュー推奨）
```

#### Step 2: 言語切り替えリンク追加
```markdown
# README.md (英語版) - 冒頭に追加
**Languages**: [English](README.md) | [日本語](README.ja.md)

# README.ja.md (日本語版) - 冒頭に追加
**言語**: [English](README.md) | [日本語](README.ja.md)
```

#### Step 3: 主要ドキュメント英語化（優先度順）

**優先度判断基準**:
1. **アクセス頻度**: GitHub閲覧数、検索流入
2. **ユーザー影響**: 新規ユーザー向け > 詳細ドキュメント
3. **保守コスト**: ページ数、更新頻度

**英語化リスト**:
```bash
# 最優先: README.md（既にStep 1で実施）

# 高優先度（Week 2で実施）
1. docs/PHASE25-FINAL-REPORT.md → docs/PHASE25-FINAL-REPORT.en.md
   - 実績アピール、外部向け

# 中優先度（Week 3または将来）
2. docs/INPUT-VALIDATION-GUIDE.md → docs/INPUT-VALIDATION-GUIDE.en.md
   - 機能ガイド、利用者向け

# 低優先度（将来計画）
3. docs/I18N-IMPLEMENTATION-PLAN.md → docs/I18N-IMPLEMENTATION-PLAN.en.md
   - 開発者向け、更新頻度低
```

### Phase 3: テンプレート・設定の多言語化（Week 3）

#### テンプレートコメント
```bash
# templates/input-validation.fragment
# Before:
# 数値オプション検証テスト

# After (日本語環境):
# 数値オプション検証テスト
# Numeric option validation tests

# 英語コメントを追加（バイリンガル対応）
```

---

## 検証基準

### 機能要件

| 項目 | 目標 | 測定方法 |
|------|------|----------|
| **日本語メッセージ表示** | 100% | `LANG=ja_JP.UTF-8`で全スクリプト実行、メッセージ確認 |
| **英語メッセージ表示** | 100% | `LANG=en_US.UTF-8`で全スクリプト実行、メッセージ確認 |
| **後方互換性** | 100% | 既存テスト140-160ケースの成功率維持 |
| **メッセージカバレッジ** | 90%+ | 主要スクリプト（core/*, utils/*）のメッセージ数 |
| **環境変数指定** | 100% | `CLI_TEST_LANG=en`での動作確認 |

### 非機能要件

| 項目 | 目標 | 測定方法 |
|------|------|----------|
| **パフォーマンス影響** | <1% | i18n有効/無効での実行時間比較 |
| **メモリオーバーヘッド** | <100KB | プロセスメモリ使用量測定（`ps aux`） |
| **起動時間影響** | <10ms | `time bash core/cli-analyzer.sh --version` |
| **翻訳品質** | ネイティブ確認 | 英語話者によるレビュー |

### パフォーマンス測定手順

```bash
# 1. i18n無効（ベースライン）
time bash core/cli-analyzer.sh /usr/bin/curl

# 2. i18n有効（日本語）
LANG=ja_JP.UTF-8 time bash core/cli-analyzer.sh /usr/bin/curl

# 3. i18n有効（英語）
LANG=en_US.UTF-8 time bash core/cli-analyzer.sh /usr/bin/curl

# 4. 比較
# - 実行時間差が1%未満であることを確認
# - メモリ使用量の差が100KB未満であることを確認
```

### 後方互換性テスト

```bash
# 1. 既存テストスイート実行（日本語環境）
LANG=ja_JP.UTF-8 bash core/run-tests.sh test-output all ./reports-ja

# 2. 成功率確認
# - 140-160テストケースすべてが成功
# - JSON/HTML/Markdownレポート生成成功

# 3. 実CLIツールテスト
LANG=ja_JP.UTF-8 bash core/test-generator.sh test-data/curl-analysis.json test-output-ja all
bats test-output-ja/*.bats
```

---

## リスク評価

### 🔒 セキュリティリスク: 🟢 低（対策実装済み）

**リスク**:
- i18nファイルの読み込み処理でのコードインジェクション
- 環境変数`$LANG`, `$CLI_TEST_LANG`の悪意のある値

**軽減策（実装済み）**:
- ✅ ホワイトリスト方式の言語コード検証（`ja|en`のみ許可）
- ✅ i18nファイルのバリデーション（`declare -A MESSAGES`パターンチェック）
- ✅ 環境変数の読み取り専用化（`readonly CLI_TEST_LANG`）
- ✅ ファイル存在確認とエラーハンドリング

### ⚙️ 技術的リスク: 🟡 中

**リスク**:
- 既存コードへの影響（後方互換性破壊）
- 翻訳の品質・一貫性

**軽減策**:
- 既存の日本語をデフォルトに設定
- 段階的な移行（logger.shから開始）
- テストスイートで動作確認

### 📊 開発効率リスク: 🟡 中

**リスク**:
- ドキュメント二重メンテナンス
- 翻訳作業の工数増加

**軽減策**:
- README.mdを英語版に一本化、README.ja.mdは重要な更新のみ
- コミュニティ翻訳貢献を促進

---

## TODO

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

## 参考資料

### Bash i18n
- [GNU gettext](https://www.gnu.org/software/gettext/)
- [Bash-i18n Best Practices](https://www.gnu.org/software/bash/manual/html_node/Locale-Translation.html)

### ドキュメント多言語化
- [GitHub README多言語化ベストプラクティス](https://github.com/github/docs/blob/main/contributing/translations.md)
- [Markdown i18n Patterns](https://github.com/i18n/i18n-guide)

---

**作成者**: CLI Testing Specialist Agent Development Team
**レビュー**: 2025-11-10 反復レビュー実施（セキュリティ/パフォーマンス/保守性）
**修正版**: 2025-11-10 レビュー結果を反映（19件の改善実施）
