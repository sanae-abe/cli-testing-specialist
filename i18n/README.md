# Internationalization (i18n) Translation Guide
# 国際化（i18n）翻訳ガイド

**Languages / 言語**: [English](#english) | [日本語](#日本語)

---

<a name="english"></a>
## English

### Overview

This directory contains internationalization (i18n) message files for the CLI Testing Specialist Agent. We use a lightweight key-value approach for multi-language support.

### Supported Languages

- **Japanese** (`ja.sh`) - Default language
- **English** (`en.sh`) - International support

### File Structure

```
i18n/
├── README.md          # This file (Contribution guide)
├── ja.sh             # Japanese messages (31 keys)
└── en.sh             # English messages (31 keys)
```

### Message File Format

Each language file follows this structure:

```bash
#!/usr/bin/env bash
#
# [language].sh - [Language Name] Messages for CLI Testing Specialist
#

# Message Key = "Message Value"
MESSAGES[cli_analysis_started]="Starting CLI analysis"
MESSAGES[cli_analysis_completed]="CLI analysis completed"
# ... more messages
```

### Key Naming Convention

- **Format**: `snake_case`
- **Pattern**: `[component]_[action]_[status]`
- **Examples**:
  - `cli_analysis_started`
  - `logger_initialized`
  - `failed_to_get_help`

### How to Contribute

#### Adding a New Language

1. **Create a new language file**:
   ```bash
   cp i18n/en.sh i18n/[lang_code].sh
   ```

2. **Update the file header**:
   ```bash
   # [lang_code].sh - [Language Name] Messages for CLI Testing Specialist
   ```

3. **Translate all message values**:
   - Keep the same keys as `ja.sh` and `en.sh`
   - Translate only the values
   - Preserve `%s` placeholders for printf formatting

4. **Update `utils/i18n-loader.sh`**:
   Add your language code to the whitelist:
   ```bash
   case "$lang_code" in
       ja|en|[your_lang]) ;;  # Add your language code
       *) echo "en" ;;
   esac
   ```

5. **Test your translation**:
   ```bash
   CLI_TEST_LANG=[your_lang] bash core/cli-analyzer.sh /usr/bin/curl
   ```

#### Updating Existing Translations

1. **Check for missing keys**:
   ```bash
   # Compare key counts
   grep -c "^MESSAGES\[" i18n/ja.sh
   grep -c "^MESSAGES\[" i18n/en.sh
   grep -c "^MESSAGES\[" i18n/[your_lang].sh
   ```

2. **Add missing keys** from `ja.sh` or `en.sh`

3. **Improve existing translations**:
   - Better wording
   - Technical accuracy
   - Natural phrasing

4. **Test the changes**:
   ```bash
   CLI_TEST_LANG=[your_lang] bash core/cli-analyzer.sh /usr/bin/curl
   ```

### Translation Quality Guidelines

#### Must-Have

- ✅ All keys from `ja.sh` and `en.sh` present
- ✅ `%s` placeholders preserved in correct positions
- ✅ Technical terms consistent
- ✅ No syntax errors (test with `bash -n i18n/[lang].sh`)

#### Recommended

- 📝 Natural phrasing for native speakers
- 📝 Consistent terminology across all messages
- 📝 Technical accuracy
- 📝 Cultural appropriateness

### Testing Your Translation

```bash
# 1. Syntax check
bash -n i18n/[your_lang].sh

# 2. Load test
source utils/i18n-loader.sh
CLI_TEST_LANG=[your_lang] load_i18n_once

# 3. Functional test
CLI_TEST_LANG=[your_lang] bash core/cli-analyzer.sh /usr/bin/curl

# 4. All messages displayed test
CLI_TEST_LANG=[your_lang] bash core/cli-analyzer.sh /usr/bin/curl 2>&1 | grep -v "\[Missing"
```

### Current Message Coverage

| Component | Message Keys | Coverage |
|-----------|--------------|----------|
| CLI Analyzer | 20 | ✅ 100% |
| Logger | 5 | ✅ 100% |
| Test Generator | 0 | ⏳ Future |
| **Total** | **31** | - |

### Contribution Process

1. **Fork** the repository
2. **Create** your translation file or update existing one
3. **Test** thoroughly (see "Testing Your Translation")
4. **Submit** a Pull Request with:
   - Translation file (`i18n/[lang].sh`)
   - Updated whitelist in `utils/i18n-loader.sh`
   - Test results (screenshots or logs)

### Community

- **Issues**: Report translation errors or suggestions
- **Discussions**: Propose new languages or improvements
- **Reviews**: Native speaker reviews highly appreciated

---

<a name="日本語"></a>
## 日本語

### 概要

このディレクトリには、CLI Testing Specialist Agentの国際化（i18n）メッセージファイルが含まれています。軽量なキー・バリュー方式で多言語対応を実現しています。

### 対応言語

- **日本語** (`ja.sh`) - デフォルト言語
- **英語** (`en.sh`) - 国際対応

### ファイル構造

```
i18n/
├── README.md          # このファイル（貢献ガイド）
├── ja.sh             # 日本語メッセージ（31キー）
└── en.sh             # 英語メッセージ（31キー）
```

### メッセージファイル形式

各言語ファイルは以下の構造に従います：

```bash
#!/usr/bin/env bash
#
# [language].sh - CLI Testing Specialist用[言語名]メッセージ
#

# メッセージキー = "メッセージ値"
MESSAGES[cli_analysis_started]="CLI解析を開始します"
MESSAGES[cli_analysis_completed]="CLI解析が完了しました"
# ... 他のメッセージ
```

### キー命名規則

- **形式**: `snake_case`
- **パターン**: `[コンポーネント]_[アクション]_[ステータス]`
- **例**:
  - `cli_analysis_started`
  - `logger_initialized`
  - `failed_to_get_help`

### 貢献方法

#### 新しい言語の追加

1. **新しい言語ファイルを作成**:
   ```bash
   cp i18n/en.sh i18n/[言語コード].sh
   ```

2. **ファイルヘッダーを更新**:
   ```bash
   # [言語コード].sh - CLI Testing Specialist用[言語名]メッセージ
   ```

3. **すべてのメッセージ値を翻訳**:
   - `ja.sh`と`en.sh`と同じキーを保持
   - 値のみを翻訳
   - `%s`プレースホルダーをprintf形式用に保持

4. **`utils/i18n-loader.sh`を更新**:
   ホワイトリストに言語コードを追加:
   ```bash
   case "$lang_code" in
       ja|en|[your_lang]) ;;  # 言語コードを追加
       *) echo "en" ;;
   esac
   ```

5. **翻訳をテスト**:
   ```bash
   CLI_TEST_LANG=[言語コード] bash core/cli-analyzer.sh /usr/bin/curl
   ```

#### 既存翻訳の更新

1. **不足キーを確認**:
   ```bash
   # キー数を比較
   grep -c "^MESSAGES\[" i18n/ja.sh
   grep -c "^MESSAGES\[" i18n/en.sh
   grep -c "^MESSAGES\[" i18n/[言語コード].sh
   ```

2. **`ja.sh`または`en.sh`から不足キーを追加**

3. **既存翻訳を改善**:
   - より良い表現
   - 技術的正確性
   - 自然な言い回し

4. **変更をテスト**:
   ```bash
   CLI_TEST_LANG=[言語コード] bash core/cli-analyzer.sh /usr/bin/curl
   ```

### 翻訳品質ガイドライン

#### 必須事項

- ✅ `ja.sh`と`en.sh`のすべてのキーが存在
- ✅ `%s`プレースホルダーが正しい位置に保持
- ✅ 技術用語の一貫性
- ✅ 構文エラーなし（`bash -n i18n/[lang].sh`でテスト）

#### 推奨事項

- 📝 ネイティブスピーカーにとって自然な表現
- 📝 すべてのメッセージで一貫した用語
- 📝 技術的正確性
- 📝 文化的適切性

### 翻訳のテスト

```bash
# 1. 構文チェック
bash -n i18n/[言語コード].sh

# 2. 読み込みテスト
source utils/i18n-loader.sh
CLI_TEST_LANG=[言語コード] load_i18n_once

# 3. 機能テスト
CLI_TEST_LANG=[言語コード] bash core/cli-analyzer.sh /usr/bin/curl

# 4. すべてのメッセージ表示テスト
CLI_TEST_LANG=[言語コード] bash core/cli-analyzer.sh /usr/bin/curl 2>&1 | grep -v "\[Missing"
```

### 現在のメッセージカバレッジ

| コンポーネント | メッセージキー数 | カバレッジ |
|---------------|-----------------|-----------|
| CLI Analyzer | 20 | ✅ 100% |
| Logger | 5 | ✅ 100% |
| Test Generator | 0 | ⏳ 今後 |
| **合計** | **31** | - |

### 貢献プロセス

1. **Fork** リポジトリをフォーク
2. **Create** 翻訳ファイルを作成または既存ファイルを更新
3. **Test** 徹底的にテスト（「翻訳のテスト」を参照）
4. **Submit** 以下を含むPull Requestを提出:
   - 翻訳ファイル（`i18n/[lang].sh`）
   - `utils/i18n-loader.sh`の更新されたホワイトリスト
   - テスト結果（スクリーンショットまたはログ）

### コミュニティ

- **Issues**: 翻訳エラーや提案を報告
- **Discussions**: 新言語や改善を提案
- **Reviews**: ネイティブスピーカーレビューを歓迎

---

**Generated by CLI Testing Specialist Agent**
**Last Updated**: 2025-11-10
