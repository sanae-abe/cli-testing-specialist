# cli-testing-specialist 適用対象ガイド

**Phase 2.7**: 適用範囲の明確化とベストプラクティス

---

## 📋 概要

cli-testing-specialistは**標準的なCLIツール**向けの自動テスト生成フレームワークです。
このドキュメントでは、適用に適したツールと適していないツールを明確にし、
最適な統合方法を提案します。

---

## ✅ 高適用度（成功率70-90%）

### 対象ツールの特徴

- **標準的なオプション体系**: `--help`, `--version`, `--verbose`, `--quiet` 等
- **設定ファイル不要**: コマンドライン引数のみで動作
- **シンプルな入出力**: 標準入力/出力/エラー出力の基本的な使用
- **非インタラクティブ**: 対話的プロンプトなし
- **POSIX互換**: bash/zshで標準的に動作

### 具体例

#### システムユーティリティ
- `curl`, `wget`: ネットワークツール
- `ls`, `cat`, `grep`: ファイル操作
- `git`: バージョン管理（基本コマンド）

#### 開発ツール
- **package-publisher** (Node.js CLI)
  - 標準的な `--help`, `--version` サポート
  - 設定ファイルはオプショナル
  - 非インタラクティブモード対応
  - **推定成功率**: 70-85%

#### 汎用CLIツール
- 引数パーサーライブラリ使用（clap, commander, yargs等）
- RESTful API クライアント
- データ変換ツール（jq, yq等）

### CI統合推奨設定

```yaml
- name: Run CLI tests
  run: |
    cli-testing-specialist run \
      cli-tests \
      --format all \
      --output cli-reports \
      --timeout 60

- name: Check test results (fail on errors)
  if: always()
  run: |
    if [ -f cli-reports/cli-tests-report.json ]; then
      FAILED=$(jq '[.suites[].tests[] | select(.status == "failed")] | length' cli-reports/cli-tests-report.json)
      if [ "$FAILED" -gt 0 ]; then
        echo "::error::❌ $FAILED CLI tests failed"
        exit 1
      fi
    fi
```

---

## ⚠️ 中適用度（成功率30-60%）

### 対象ツールの特徴

- **設定ファイル依存**: 動作に設定ファイルが必須
- **カスタムUI実装**: 独自のプロンプト・インタラクティブUI
- **i18n対応**: 多言語サポート
- **カスタムオプション体系**: 標準的でない独自オプション

### 具体例

#### 設定ファイル依存型
- **cmdrun**
  - `commands.toml` 設定ファイルが必須
  - `cmdrun run <command-name>` という使い方
  - 設定駆動型アーキテクチャ
  - **推定成功率**: 30-50%
  - **失敗カテゴリ**: security (0/4), destructive-ops (0/2), basic (3/5)

#### カスタムUI実装型
- **cldev**
  - dialoguer（インタラクティブプロンプト）使用
  - i18n/多言語サポート
  - カスタム実装が強い
  - **推定成功率**: 40-60%

- **backup-suite**
  - dialoguer使用
  - 日本語専用UI
  - 複雑な設定ファイル構造
  - **推定成功率**: 0-20%
  - **失敗原因**: 汎用テストがカスタムUIに対応していない

### CI統合推奨設定（情報提供モード）

```yaml
- name: Run CLI tests
  continue-on-error: true  # テスト失敗を許容
  run: |
    cli-testing-specialist run \
      cli-tests \
      --format all \
      --output cli-reports \
      --timeout 60

- name: Display test results summary
  if: always()
  run: |
    if [ -f cli-reports/cli-tests-report.json ]; then
      echo "📊 Test Results:"
      TOTAL=$(jq '[.suites[].tests | length] | add' cli-reports/cli-tests-report.json)
      PASSED=$(jq '[.suites[].tests[] | select(.status == "passed")] | length' cli-reports/cli-tests-report.json)
      FAILED=$(jq '[.suites[].tests[] | select(.status == "failed")] | length' cli-reports/cli-tests-report.json)
      echo "Total: $TOTAL"
      echo "Passed: $PASSED"
      echo "Failed: $FAILED"
    fi

- name: Check test results (informational)
  if: always()
  continue-on-error: true
  run: |
    echo "📋 CLI Testing Summary"
    echo "Note: Generic tests from cli-testing-specialist are for reference."
    echo "Tests may fail due to custom implementation (config-driven, i18n, dialoguer UI)."
    echo ""
    if [ -f cli-reports/cli-tests-report.md ]; then
      echo "Full report available in artifacts: cli-test-reports-${{ matrix.os }}"
    fi
```

### 補完的テスト戦略

カスタム実装ツールには、cli-testing-specialistを**補完的**に使用：

1. **既存テスト**: カスタム機能の詳細テスト（Jest, pytest, cargo test等）
2. **cli-testing-specialist**: 基本的なCLIインターフェーステスト（参考情報）
3. **手動テスト**: インタラクティブ機能の確認

---

## ❌ 低適用度（非推奨）

### 対象ツールの特徴

- **インタラクティブシェル**: REPL環境
- **プロトコル依存**: 特殊な通信プロトコル使用
- **コンテナ管理**: 複雑な環境依存
- **GUI統合**: グラフィカルインターフェース

### 具体例

#### インタラクティブシェル
- `mysql`, `psql`, `redis-cli`: データベースクライアント
- `python`, `node`, `irb`: REPL環境

#### コンテナ・仮想化ツール
- `docker`, `podman`: コンテナ管理
- `kubectl`: Kubernetes管理
- `vagrant`: 仮想マシン管理

#### ドメイン特化型
- `aws-cli`, `gcloud`: クラウドプロバイダーCLI
- `terraform`: インフラストラクチャ管理
- `ansible`: 構成管理

### 非推奨の理由

1. **環境依存が強い**: 特定のサーバー・サービスが必須
2. **複雑な状態管理**: セッション・接続状態の維持が必要
3. **特殊なプロトコル**: 標準入出力以外の通信
4. **テスト設計の複雑さ**: 汎用テストでは対応不可

---

## 📊 適用判定チェックリスト

以下のチェックリストで適用度を判定できます：

### ✅ 高適用度の条件（5つ以上該当）

- [ ] `--help` オプションでヘルプ表示
- [ ] `--version` オプションでバージョン表示
- [ ] 設定ファイルなしで基本動作が可能
- [ ] 非インタラクティブモード対応
- [ ] 標準入出力のみ使用
- [ ] エラー時に適切な終了コード返却
- [ ] POSIX互換シェルで動作
- [ ] 引数パーサーライブラリ使用（clap, commander等）

### ⚠️ 中適用度の条件（3-4つ該当）

- [ ] 設定ファイルが必須だがシンプル
- [ ] カスタムオプション体系
- [ ] インタラクティブだが非対話モードあり
- [ ] i18n対応
- [ ] カスタムUI実装（dialoguer等）

### ❌ 低適用度の条件（2つ以下該当）

- [ ] REPL/対話型シェル
- [ ] 特殊なプロトコル使用
- [ ] 環境依存が強い（サーバー接続必須等）
- [ ] GUI統合
- [ ] コンテナ・仮想化管理

---

## 🎯 ベストプラクティス

### 1. 段階的導入

```bash
# Step 1: ローカルで試す
cli-testing-specialist analyze ./your-cli -o analysis.json
cli-testing-specialist generate analysis.json -o tests -c basic,help

# Step 2: 基本テストのみ実行
cli-testing-specialist run tests -f markdown -o reports

# Step 3: 成功率を確認
cat reports/tests-report.md

# Step 4: 成功率が高ければCI統合
```

### 2. カテゴリ選択

**高適用度ツール**: 全カテゴリ
```yaml
--categories all --include-intensive
```

**中適用度ツール**: 基本カテゴリのみ
```yaml
--categories basic,help,security
```

### 3. タイムアウト調整

```yaml
# 標準的なツール
--timeout 60

# カスタム実装ツール
--timeout 120
```

### 4. レポート活用

- **高適用度**: JUnit XML をCI/CDに統合
- **中適用度**: Markdown/HTML を参考情報として保存
- **低適用度**: 適用しない

---

## 📈 統計データ

### プロジェクト別成功率

| プロジェクト | ツール種別 | 成功率 | モード | 備考 |
|-------------|-----------|--------|--------|------|
| **package-publisher** | Node.js CLI | 推定70-85% | 通常 | 標準的なインターフェース |
| **cmdrun** | 設定駆動型 | 推定30-50% | 情報提供 | commands.toml必須 |
| **cldev** | i18n/dialoguer | 推定40-60% | 情報提供 | カスタムUI |
| **backup-suite** | 複雑なカスタム | 0-20% | 情報提供 | 日本語専用UI |

### カテゴリ別成功率（標準的なツール）

| カテゴリ | 成功率 | 備考 |
|---------|--------|------|
| basic | 90-100% | --help, --version等 |
| help | 85-95% | サブコマンドヘルプ |
| security | 60-80% | インジェクション対策 |
| path | 70-85% | パス処理 |
| multi-shell | 80-90% | bash/zsh互換性 |
| input-validation | 65-80% | 入力検証 |
| destructive-ops | 50-70% | 確認プロンプト |
| performance | 80-95% | 起動時間 |

---

## 🔄 継続的改善

### フィードバック収集

1. **成功事例**: GitHub Issues でシェア
2. **失敗事例**: 適用範囲の見直し
3. **改善提案**: 新機能のリクエスト

### 今後の拡張予定

- **Phase 3.0**: カスタム実装ツール向けテンプレート
- **Phase 3.1**: 設定ファイル対応
- **Phase 3.2**: インタラクティブツール対応

---

## 📚 関連ドキュメント

- [README.md](../README.md): プロジェクト概要
- [ARCHITECTURE.md](./ARCHITECTURE.md): アーキテクチャ設計
- [TESTING-GUIDE.md](./TESTING-GUIDE.md): テスト実行ガイド
- [CI-INTEGRATION.md](./CI-INTEGRATION.md): CI/CD統合ガイド

---

## 💡 まとめ

cli-testing-specialistは**標準的なCLIツール**に最適化されており、
以下の原則に従って適用することで最大の効果を得られます：

1. **適用範囲の見極め**: チェックリストで判定
2. **段階的導入**: ローカル → 基本テスト → CI統合
3. **適切なモード選択**: 通常モード vs 情報提供モード
4. **補完的戦略**: 既存テストとの組み合わせ

カスタム実装が強いツールには無理に適用せず、**情報提供モード**で参考情報として活用することを推奨します。
