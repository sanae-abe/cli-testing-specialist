# Directory Traversal Limits Test Pattern Design

**Version**: 1.1.0-draft (Updated after iterative review)
**Date**: 2025-11-10
**Status**: Design Phase - Review Complete
**Review Date**: 2025-11-10 (3-round iterative review)

## 📑 目次

- [目的](#-目的)
- [問題の背景](#問題の背景)
- [テスト項目設計（12パターン）](#-テスト項目設計12パターン)
  - [A. ディレクトリ走査制限](#a-ディレクトリ走査制限5パターン)
  - [B. 処理中断・タイムアウト](#b-処理中断タイムアウト3パターン)
  - [C. リソース制限](#c-リソース制限2パターン)
  - [D. ドキュメント](#d-ドキュメント1パターン)
- [既存テンプレートとの整合性](#-既存テンプレートとの整合性)
- [変数定義](#-変数定義)
- [テスト出力フォーマット](#-テスト出力フォーマット)
- [実装方針](#-実装方針)
- [セキュリティ実装ガイドライン](#-セキュリティ実装ガイドライン)
- [リスク評価](#-リスク評価)
- [成功基準](#-成功基準)
- [実装フェーズ](#-実装フェーズ)
- [変更履歴](#-変更履歴)

---

## 🎯 目的

backup-suiteで発生した「膨大なディレクトリ走査による固まり問題」を検出するための新しいテストパターン。

### 問題の背景

- **実際の事象**: backup-suite実行時に `~/` 配下全体を走査して30分以上固まった
- **根本原因**: 数万〜数十万のファイルを無制限に走査する実装
- **影響範囲**: ホームディレクトリ全体、node_modules、大規模プロジェクト等
- **検出目的**: 同様の問題を持つCLIツールを自動的に発見

---

## 📋 テスト項目設計（12パターン）

### A. ディレクトリ走査制限（5パターン）

#### 1-a. 小規模ファイル処理（早期フェイル）
```yaml
テスト名: "[directory-traversal] handles 100 files within 5 seconds"
準備環境:
  - 100個のダミーファイルを含むテストディレクトリ作成
  - /tmp/cli-test-$$-100files/（プロセスIDでユニーク化）
  - 権限: chmod 700（所有者のみアクセス可）
  - ファイルサイズ: 各1KB（合計100KB）
検証内容:
  - CLIツールに対してディレクトリ走査を実行
  - 5秒以内に完了するか
期待動作:
  - タイムアウト機能が動作
  - または高速処理（5秒以内完了）
失敗条件:
  - 5秒以上応答なし
  - プロセスが固まる
備考:
  - 早期フェイル戦略: 小規模で失敗なら大規模テストをスキップ
```

#### 1-b. 中規模ファイル処理
```yaml
テスト名: "[directory-traversal] handles 500 files within 15 seconds"
準備環境:
  - 500個のダミーファイル
  - /tmp/cli-test-$$-500files/
  - 権限: chmod 700
  - ファイルサイズ: 各1KB（合計500KB）
検証内容:
  - 15秒以内に完了するか
期待動作:
  - タイムアウト機能が動作
  - または高速処理（15秒以内完了）
失敗条件:
  - 15秒以上応答なし
備考:
  - テスト1-aが成功した場合のみ実行
```

#### 1-c. 大規模ファイル処理
```yaml
テスト名: "[directory-traversal] handles 1000 files within 30 seconds"
準備環境:
  - 1,000個のダミーファイル
  - /tmp/cli-test-$$-1000files/
  - 権限: chmod 700
  - ファイルサイズ: 各1KB（合計1MB）
検証内容:
  - 30秒以内に完了するか
期待動作:
  - タイムアウト機能が動作
  - または高速処理（30秒以内完了）
失敗条件:
  - 30秒以上応答なし
備考:
  - テスト1-bが成功した場合のみ実行
```

#### 2. 深い階層の再帰制限
```yaml
テスト名: "[directory-traversal] handles deep directory nesting"
準備環境:
  - 50階層のネストディレクトリ作成
  - /tmp/cli-test-$$-deep/a/b/c/.../50
  - 権限: chmod -R 700
検証内容:
  - 深いディレクトリ構造を走査
  - 無限ループに陥らないか
期待動作:
  - 深さ制限オプション（--max-depth）
  - または適切なエラーメッセージ
  - タイムアウト（30秒）
失敗条件:
  - スタックオーバーフロー
  - 無限ループ
  - タイムアウト
```

#### 3. シンボリックリンクループ検出
```yaml
テスト名: "[directory-traversal] detects symlink loops"
準備環境:
  - 循環参照ディレクトリの安全な作成
  - /tmp/cli-test-$$-$RANDOM-symlink/（二重ランダム化）
  - 作成手順（相対パス使用）:
      mkdir -p "$TEST_DIR"/{a,b}
      ln -s ../b "$TEST_DIR/a/link_to_b"
      ln -s ../a "$TEST_DIR/b/link_to_a"
  - 権限: chmod 700
検証内容:
  - シンボリックリンクループを適切に検出
  - タイムアウト（10秒）
期待動作:
  - ループ検出と停止
  - エラーメッセージ
  - またはタイムアウト
失敗条件:
  - 無限ループ
  - プロセスが固まる
安全策:
  - クリーンアップ: rm -f（シンボリックリンク先を追跡しない）
  - trap処理で強制削除
```

#### 4. ホームディレクトリ全体の警告（モック環境）
```yaml
テスト名: "[directory-traversal] warns for home directory traversal (mock)"
準備環境:
  - モックホームディレクトリ作成（実際の~/は絶対使用しない）
  - FAKE_HOME="/tmp/cli-test-$$-fake-home"
  - mkdir -p "$FAKE_HOME"/{Documents,Downloads,projects}
  - 各ディレクトリに数個のダミーファイル作成
  - 権限: chmod 700
  - 一時的にHOME環境変数を変更: export HOME="$FAKE_HOME"
検証内容:
  - $HOME または ~/ を引数に指定（モック環境）
  - 警告メッセージの有無
  - 確認プロンプトの有無
期待動作:
  - 警告メッセージ表示
  - 確認プロンプト（y/n）
  - または --force フラグ要求
失敗条件:
  - 警告なしで全走査開始
備考:
  - skip可能（小規模CLIでは不要な場合あり）
  - テスト後にHOME環境変数を復元
安全策:
  - 実際のホームディレクトリは絶対に使用しない
  - テストディレクトリが/tmp配下であることを検証
```

### B. 処理中断・タイムアウト（3パターン）

#### 5. 処理中断（SIGINT）の応答性
```yaml
テスト名: "[directory-traversal] responds to SIGINT within 5 seconds"
準備環境:
  - 1,000ファイルのテストディレクトリ（再利用）
  - バックグラウンド実行
検証内容:
  - CLIツールをバックグラウンドで実行
  - 1秒待機後にSIGINT送信: kill -INT $PID
  - プロセス終了を確認（wait + kill -0）
  - 5秒以内に停止するか
期待動作:
  - グレースフルシャットダウン
  - 部分結果の保存（オプション）
  - 終了コード: 128+2=130（SIGINT）
失敗条件:
  - 5秒以上応答なし
  - ゾンビプロセス化（kill -0で生存確認）
実装例:
  - "$CLI_BINARY" "$TEST_DIR_1000" & PID=$!
  - sleep 1
  - kill -INT $PID
  - for i in {1..50}; do
      kill -0 $PID 2>/dev/null || break
      sleep 0.1
    done
  - kill -0 $PID 2>/dev/null && fail "Process did not stop"
```

#### 6. タイムアウトオプションの有無
```yaml
テスト名: "[directory-traversal] supports timeout option"
検証内容:
  - --timeout=10s または --max-time=10 等のフラグ検出
  - "$CLI_BINARY" --help | grep -qE -- '--(timeout|max-time)'
  - 実際にタイムアウトするか
期待動作:
  - オプション認識
  - 指定時間で中断
失敗条件:
  - オプション未実装 → skip
  - タイムアウトしない
備考:
  - オプション未実装ならskip（"Reason: timeout option not supported"）
```

#### 7. 進捗表示の有無
```yaml
テスト名: "[directory-traversal] shows progress for long operations"
準備環境:
  - 1,000ファイルのテストディレクトリ（再利用）
検証内容:
  - 処理中の進捗表示（バー、パーセント、カウント等）
  - 出力に以下のパターンが含まれるか:
    - 進捗バー: "[=====>    ]"
    - パーセント: "50%", "75%"
    - カウント: "500/1000"
期待動作:
  - 進捗情報の出力
  - ユーザーへのフィードバック
  - 更新頻度: 1秒に1回程度
失敗条件:
  - 何も表示されず固まったように見える
備考:
  - ベストプラクティス推奨（必須ではない）
  - 進捗なしでもskip（失敗ではない）
```

### C. リソース制限（2パターン）

#### 8. ファイル数上限オプション
```yaml
テスト名: "[directory-traversal] supports max-files limit"
検証内容:
  - --max-files=100 または --limit=100 等のフラグ検出
  - "$CLI_BINARY" --help | grep -qE -- '--(max-files|limit)'
  - 実際に制限されるか（1,000ファイルに対して100で制限）
期待動作:
  - オプション認識
  - 指定数で停止
  - 処理ファイル数が100以下
失敗条件:
  - オプション未実装 → skip
  - 制限が機能しない
備考:
  - オプション未実装ならskip
```

#### 9. メモリ使用量の制限
```yaml
テスト名: "[directory-traversal] memory usage stays under 1GB"
準備環境:
  - 1,000ファイルのテストディレクトリ（再利用）
検証内容:
  - メモリ使用量測定
  - 1GB以下か
測定方法:
  - Linux: /usr/bin/time -v %M（KB単位）
  - macOS: /usr/bin/time -l（bytes単位）→ KB変換
  - ベースライン測定: CLI実行前のメモリ使用量を記録
  - 測定誤差: ±10%を許容
期待動作:
  - ストリーム処理
  - 適切なメモリ管理
  - 1GB = 1,048,576 KB以下
失敗条件:
  - メモリリーク
  - 1GB超過
備考:
  - GNU timeインストールがオプション依存（必須ではない）
  - time未対応環境ではskip
```

### D. ドキュメント（1パターン）

#### 10. 処理時間の警告・ドキュメント
```yaml
テスト名: "[directory-traversal] documents performance warnings"
検証内容:
  - --help での大量データ処理の注意書き
  - デフォルト制限値の明記
  - 以下のキーワード検索:
    - "large", "performance", "limit", "timeout", "warning"
期待動作:
  - パフォーマンス注意書き
  - 制限値の説明
  - 例: "Note: Processing large directories may take time"
失敗条件:
  - ドキュメントなし → skip（推奨だが必須ではない）
備考:
  - ベストプラクティス推奨
```

---

## 🔍 既存テンプレートとの整合性

| 項目 | destructive-ops | input-validation | directory-traversal（新規） |
|------|----------------|-----------------|---------------------------|
| **ファイル名** | 09-destructive-ops.bats | 08-input-validation.bats | 10-directory-traversal-limits.bats |
| **テスト数** | 16パターン | 25パターン | 12パターン |
| **主要変数** | `${DESTRUCTIVE_COMMAND}` | `${OPTION_NAME}` | `${TRAVERSAL_COMMAND}` |
| **setup処理** | なし（各テスト独立） | なし | **setup_file()必須**（環境作成） |
| **teardown処理** | 推奨 | 不要 | **teardown_file()必須**（クリーンアップ） |
| **skip条件** | オプション未実装 | オプション未実装 | オプション未実装 + CI環境 |
| **タイムアウト** | timeout 10 | timeout 5 | timeout 5-30（段階的） |
| **並列実行** | 可能 | 可能 | **禁止**（resource-intensive） |

### 統一すべき点

1. **テスト名プレフィックス**: `[directory-traversal]`（既存パターンに準拠）
2. **skip条件の判定方法**: `"$CLI_BINARY" --help | grep -qE -- '--option'`
3. **タイムアウトエラーの処理**: `[ $status -eq 124 ]`（BATSタイムアウト）
4. **コメントフォーマット**: 英語コメント（既存テンプレートに合わせる）
5. **BATS file_tags**: `# bats file_tags=sequential,resource-intensive`

---

## 📦 変数定義

| 変数名 | 説明 | 例 | デフォルト値 | 設定箇所 |
|--------|------|-----|-------------|---------|
| **CLI_BINARY** | テスト対象CLI | `/path/to/backup-suite` | （必須） | BATS環境変数 |
| **TRAVERSAL_COMMAND** | ディレクトリ走査コマンド | `scan`, `analyze`, `backup` | （CLI解析から抽出） | test-generator.sh |
| **TRAVERSAL_ARGS** | 追加引数 | `--recursive`, `--all` | `""` | test-generator.sh |
| **TEST_DIR_100** | 100ファイルテストディレクトリ | `/tmp/cli-test-$$-100files` | （setup_file()で作成） | setup_file() |
| **TEST_DIR_500** | 500ファイルテストディレクトリ | `/tmp/cli-test-$$-500files` | （setup_file()で作成） | setup_file() |
| **TEST_DIR_1000** | 1,000ファイルテストディレクトリ | `/tmp/cli-test-$$-1000files` | （setup_file()で作成） | setup_file() |
| **TEST_DIR_DEEP** | 50階層テストディレクトリ | `/tmp/cli-test-$$-deep` | （setup_file()で作成） | setup_file() |
| **TEST_DIR_SYMLINK** | シンボリックリンクテストディレクトリ | `/tmp/cli-test-$$-$RANDOM-symlink` | （setup_file()で作成） | setup_file() |
| **FAKE_HOME** | モックホームディレクトリ | `/tmp/cli-test-$$-fake-home` | （setup_file()で作成） | setup_file() |

### 実装例

```bash
# templates/directory-traversal-limits.fragment

# BATS setup_file()
setup_file() {
    # Environment validation
    CLI_BINARY="${CLI_BINARY:-}"
    if [[ -z "$CLI_BINARY" ]]; then
        echo "# ERROR: CLI_BINARY not set" >&3
        return 1
    fi

    TRAVERSAL_COMMAND="${TRAVERSAL_COMMAND:-scan}"
    TRAVERSAL_ARGS="${TRAVERSAL_ARGS:-}"

    # Create test directories (cached for all tests)
    TEST_DIR_100="$(mktemp -d /tmp/cli-test-$$-100files-XXXXXX)"
    TEST_DIR_500="$(mktemp -d /tmp/cli-test-$$-500files-XXXXXX)"
    TEST_DIR_1000="$(mktemp -d /tmp/cli-test-$$-1000files-XXXXXX)"
    TEST_DIR_DEEP="$(mktemp -d /tmp/cli-test-$$-deep-XXXXXX)"
    TEST_DIR_SYMLINK="$(mktemp -d /tmp/cli-test-$$-symlink-XXXXXX)"
    FAKE_HOME="$(mktemp -d /tmp/cli-test-$$-fake-home-XXXXXX)"

    # Set permissions
    chmod 700 "$TEST_DIR_100" "$TEST_DIR_500" "$TEST_DIR_1000" \
              "$TEST_DIR_DEEP" "$TEST_DIR_SYMLINK" "$FAKE_HOME"

    # Populate directories
    populate_test_dir "$TEST_DIR_100" 100
    populate_test_dir "$TEST_DIR_500" 500
    populate_test_dir "$TEST_DIR_1000" 1000
    create_deep_dir "$TEST_DIR_DEEP" 50
    create_symlink_loop "$TEST_DIR_SYMLINK"
    create_fake_home "$FAKE_HOME"

    export TEST_DIR_100 TEST_DIR_500 TEST_DIR_1000 TEST_DIR_DEEP TEST_DIR_SYMLINK FAKE_HOME
}

# BATS teardown_file()
teardown_file() {
    # Cleanup all test directories
    cleanup_test_dir "$TEST_DIR_100"
    cleanup_test_dir "$TEST_DIR_500"
    cleanup_test_dir "$TEST_DIR_1000"
    cleanup_test_dir "$TEST_DIR_DEEP"
    cleanup_test_dir "$TEST_DIR_SYMLINK"
    cleanup_test_dir "$FAKE_HOME"
}

# Helper: Cleanup function
cleanup_test_dir() {
    local dir="$1"
    if [[ -n "$dir" && "$dir" =~ ^/tmp/cli-test- ]]; then
        rm -rf "$dir" 2>/dev/null || true
    fi
}
```

---

## 📄 テスト出力フォーマット

### 成功時
```
✓ [directory-traversal] handles 100 files within 5 seconds (1.234s)
# Processed 100 files in 1.2 seconds
# Memory usage: 12MB
# Exit code: 0
```

### 失敗時
```
✗ [directory-traversal] handles 1000 files within 30 seconds (timeout after 30s)
# Expected: completion within 30 seconds
# Actual: timeout (process did not respond)
# Command: /path/to/cli scan /tmp/cli-test-12345-1000files
# PID: 12345 (killed due to timeout)
# Exit code: 124
```

### スキップ時
```
- [directory-traversal] supports timeout option (skipped)
# Reason: --timeout option not supported
# Detected via: /path/to/cli --help | grep -qE -- '--(timeout|max-time)'
# This is optional functionality
```

### BATS標準出力形式との統一
- BATS標準出力形式を使用
- `echo "# ..."` で詳細情報を追加
- `>&3` でデバッグ出力を分離（verbose mode）

---

## 🛠️ 実装方針

### テンプレートファイル
- **ファイル名**: `templates/directory-traversal-limits.fragment`
- **フォーマット**: BATS形式（既存テンプレートに準拠）
- **変数**: 上記「変数定義」セクション参照
- **BATS file_tags**: `sequential,resource-intensive`

### test-generator.sh統合

```bash
# core/test-generator.sh に追加

generate_directory_traversal_tests() {
    local cli_analysis="$1"
    local output_dir="$2"

    log INFO "Generating directory traversal limits tests..."

    # Extract traversal commands (commands that might traverse directories)
    local traversal_commands
    traversal_commands=$(jq -r '
        .subcommands[] |
        select(.name | test("scan|analyze|backup|sync|search|find|list|ls"))
    ' "$cli_analysis")

    if [[ -z "$traversal_commands" ]]; then
        log WARN "No traversal commands detected, using default"
        traversal_commands="scan"
    fi

    # Generate test file
    local output_file="$output_dir/10-directory-traversal-limits.bats"

    # Load template
    local template
    template=$(cat "$SCRIPT_DIR/../templates/directory-traversal-limits.fragment")

    # Substitute variables
    template="${template//\$\{CLI_BINARY\}/\$CLI_BINARY}"
    template="${template//\$\{TRAVERSAL_COMMAND\}/$traversal_commands}"

    # Write to file
    cat > "$output_file" <<EOF
#!/usr/bin/env bats
#
# Directory Traversal Limits Tests
# CLI Testing Specialist Agent v2.6.0
#
# This test suite verifies directory traversal performance and safety:
# - File count limits
# - Deep directory nesting
# - Symlink loop detection
# - Home directory warnings
# - SIGINT responsiveness
#
# WARNING: This test MUST run sequentially (not parallel)
# Reason: Creates large test directories, consumes /tmp space
#
# bats file_tags=sequential,resource-intensive

$template
EOF

    chmod +x "$output_file"
    log INFO "Generated: $output_file"
}

# メイン処理に追加
case "$test_type" in
    ...
    directory-traversal)
        generate_directory_traversal_tests "$cli_analysis" "$output_dir"
        ;;
    all)
        ...
        generate_directory_traversal_tests "$cli_analysis" "$output_dir"
        ;;
esac
```

### 依存関係
- **必須**: bash 4+, BATS, timeout, mktemp, chmod
- **オプション**: GNU time (macOS: brew install gnu-time) - メモリ測定用

---

## 🔒 セキュリティ実装ガイドライン

### 1. テストディレクトリの安全な作成

```bash
# ✅ 良い例
TEST_DIR="$(mktemp -d /tmp/cli-test-$$-XXXXXX)"
chmod 700 "$TEST_DIR"

# ❌ 悪い例
TEST_DIR="/tmp/cli-test"  # 固定名、競合リスク
mkdir "$TEST_DIR"         # 権限未設定、他ユーザーアクセス可能
```

### 2. trap処理による確実なクリーンアップ

```bash
# テストファイル全体のtrap設定
cleanup_all() {
    local exit_code=$?
    echo "# Cleaning up test directories..." >&3

    # 全テストディレクトリを削除
    for dir in "$TEST_DIR_100" "$TEST_DIR_500" "$TEST_DIR_1000" \
               "$TEST_DIR_DEEP" "$TEST_DIR_SYMLINK" "$FAKE_HOME"; do
        if [[ -n "$dir" && "$dir" =~ ^/tmp/cli-test- ]]; then
            rm -rf "$dir" 2>/dev/null || true
        fi
    done

    # HOME環境変数の復元
    if [[ -n "$ORIGINAL_HOME" ]]; then
        export HOME="$ORIGINAL_HOME"
    fi

    return $exit_code
}

trap cleanup_all EXIT ERR INT TERM
```

### 3. ホームディレクトリテストの安全な実装

```bash
# ✅ モック環境の使用（安全）
FAKE_HOME="$(mktemp -d /tmp/cli-test-$$-fake-home-XXXXXX)"
chmod 700 "$FAKE_HOME"
mkdir -p "$FAKE_HOME"/{Documents,Downloads,projects}

# HOME環境変数の一時的な変更（元の値を保存）
ORIGINAL_HOME="$HOME"
export HOME="$FAKE_HOME"

# テスト実行
"$CLI_BINARY" $TRAVERSAL_COMMAND "$HOME"

# HOME環境変数の復元
export HOME="$ORIGINAL_HOME"

# ❌ 実際のホームディレクトリの使用（危険）
# "$CLI_BINARY" $TRAVERSAL_COMMAND "$HOME"  # 絶対に使用しない
```

### 4. シンボリックリンクループの安全な作成

```bash
# ✅ 相対パスでの作成（安全）
TEST_DIR="$(mktemp -d /tmp/cli-test-$$-symlink-XXXXXX)"
chmod 700 "$TEST_DIR"
mkdir -p "$TEST_DIR"/{a,b}
ln -s ../b "$TEST_DIR/a/link_to_b"
ln -s ../a "$TEST_DIR/b/link_to_a"

# クリーンアップ（シンボリックリンクのみ削除）
rm -f "$TEST_DIR"/a/link_to_b "$TEST_DIR"/b/link_to_a
rmdir "$TEST_DIR"/{a,b}
rmdir "$TEST_DIR"

# ❌ 絶対パスでの作成（危険）
# ln -s /tmp/cli-test/b /tmp/cli-test/a/link_to_b  # 外部ファイル削除リスク
```

### 5. 並列実行の防止

```bash
# テストファイルヘッダーに明記
# bats file_tags=sequential,resource-intensive

# README.mdに実行方法を明記
## 実行方法
```bash
# ❌ 並列実行禁止
# bats --jobs 4 test-output/

# ✅ 順次実行
bats test-output/10-directory-traversal-limits.bats
```
```

### 6. リソース制限（ulimit）

```bash
# setup_file()でリソース制限を設定
setup_file() {
    # メモリ制限: 2GB（テストディレクトリ作成を考慮）
    ulimit -m 2097152 2>/dev/null || true

    # ファイル数制限: 2048個
    ulimit -n 2048 2>/dev/null || true

    # プロセス数制限: 100個
    ulimit -u 100 2>/dev/null || true

    # ... 残りのsetup処理
}
```

---

## 📊 リスク評価

### セキュリティリスク 🟡 中リスク（修正後）
- **懸念**:
  1. テスト用の大量ファイル作成が/tmpを圧迫
  2. ホームディレクトリテストでの実データ損失リスク
  3. シンボリックリンクループでのシステム不安定化
- **軽減策**:
  1. /tmp配下に限定、ファイル数1,000個程度に制限（合計約1MB）
  2. モックホームディレクトリの使用（実際の~/は絶対使用しない）
  3. 相対パスでのシンボリックリンク作成、安全なクリーンアップ
  4. trap処理による確実なクリーンアップ
  5. テストディレクトリ権限設定（chmod 700）
  6. 並列実行禁止（flock不要、BATS file_tags使用）
  7. ulimitによるリソース制限
- **残存リスク**: /tmp満杯時のテスト失敗（軽微）

### 技術的リスク 🟡 中リスク
- **懸念**: テスト実行時間が長くなる可能性
- **軽減策**:
  - タイムアウト値を適切に設定（5-30秒、段階的）
  - CI/CDでのスキップオプション提供（`SKIP_DIRECTORY_TRAVERSAL_TESTS=1`）
  - 並列実行禁止（BATS file_tags）
  - 早期フェイル戦略（100ファイルで失敗なら1,000をスキップ）
  - setup_file()での環境キャッシング

### CI/CD環境リスク 🟡 中リスク（新規追加）
- **懸念**: GitHub Actions等での/tmp容量制限（約14GB）
- **軽減策**:
  - テストディレクトリサイズを事前計算（約1MB/1,000ファイル）
  - 並列実行の明示的な禁止
  - CI環境変数での軽量版テスト切り替え
  - df -h /tmp でのディスク容量チェック

### 開発効率リスク 🟢 低リスク
- **懸念**: 既存テンプレートとの整合性
- **軽減策**:
  - 既存のdestructive-ops, input-validationパターンを参考
  - 変数命名規則の統一（上記「既存テンプレートとの整合性」参照）
  - テスト出力フォーマットの標準化
  - コメント・ドキュメントの充実

---

## 🎯 成功基準

1. **実用性**: backup-suite問題を実際に検出できる
2. **移植性**: 他のCLIツールにも適用可能
3. **保守性**: 既存パターンと統一されたコード
4. **パフォーマンス**（修正版）:
   - テスト準備時間: <30秒（setup_file()での環境作成）
   - テスト実行時間: <2分（12テスト合計）
   - 総合実行時間: <3分（準備+実行）
5. **信頼性**: false positive/negativeが少ない
6. **安全性**: テスト環境の完全な分離と自動クリーンアップ

---

## 🚀 実装フェーズ

### Phase 1: セキュリティ強化（最優先）
- [ ] ホームディレクトリテストのモック化
- [ ] シンボリックリンクループの安全な実装
- [ ] trap処理の実装
- [ ] テストディレクトリ権限設定（chmod 700）
- [ ] ulimitリソース制限

**推定時間**: 2-3時間

### Phase 2: パフォーマンス最適化
- [ ] setup_file()/teardown_file()実装
- [ ] 段階的テスト（100→500→1,000ファイル）
- [ ] 早期フェイル戦略
- [ ] CI環境でのスキップオプション

**推定時間**: 2-3時間

### Phase 3: 保守性向上
- [ ] 既存テンプレート比較表の検証
- [ ] 変数定義の実装
- [ ] テスト出力フォーマット統一
- [ ] 英語コメント追加
- [ ] BATS file_tags追加

**推定時間**: 1-2時間

### Phase 4: 実装＆テスト
- [ ] templates/directory-traversal-limits.fragment作成
- [ ] test-generator.sh統合
- [ ] 単体テスト（各ヘルパー関数）
- [ ] backup-suiteでの動作確認
- [ ] curl, npm, python3での動作確認

**推定時間**: 3-4時間

### Phase 5: ドキュメント完成
- [ ] README.md更新
- [ ] 実行方法の明記（並列実行禁止）
- [ ] トラブルシューティングセクション
- [ ] 変更履歴追加
- [ ] 最終レビュー

**推定時間**: 1-2時間

**総推定時間**: 9-14時間

---

## 📝 変更履歴

### v1.1.0-draft (2025-11-10) - レビュー反映版
- 3ラウンド反復レビュー実施（セキュリティ、パフォーマンス、保守性）
- 26件の発見事項（Critical 4, Important 14, Minor 8）
- **セキュリティ強化**:
  - ホームディレクトリテストをモック環境に変更（Critical #1）
  - シンボリックリンクループの安全な実装手順追加（Critical #2）
  - trap処理の詳細化、権限設定の明記
- **パフォーマンス最適化**:
  - テスト準備時間の成功基準追加（Critical #8）
  - 段階的テスト（100→500→1,000ファイル）に変更
  - setup_file()/teardown_file()の実装方針追加
- **保守性向上**:
  - 既存テンプレート比較表追加（Critical #15）
  - 変数定義セクション追加
  - テスト出力フォーマットセクション追加
  - 目次追加
- **テスト項目**: 10パターン → 12パターン（段階的テスト追加）

### v1.0.0-draft (2025-11-10) - 初版
- backup-suite問題に基づく設計
- 10パターンのテスト項目定義
- 基本的なリスク評価

---

**レビュー観点**:
- ✅ セキュリティ: テスト環境の安全性、クリーンアップ処理（Phase 1で対応）
- ✅ パフォーマンス: テスト実行時間、リソース使用量（Phase 2で最適化）
- ✅ 保守性: コード品質、既存パターンとの整合性（Phase 3で統一）
