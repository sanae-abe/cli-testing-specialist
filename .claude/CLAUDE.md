# CLI Testing Specialist - Project Configuration

## 🔧 Development Workflow

### Git Hooks Configuration

#### pre_commit
- **cargo fmt** (自動フォーマット)
  - 実行時間: 1-2秒
  - 目的: コミット前にコードを自動整形

#### pre_push
1. **cargo clippy --all-features -- -D warnings** (先に実行)
   - 実行時間: 5-10秒
   - 目的: 型エラーを早期検出（早期失敗）

2. **cargo test --all-features --lib --bins** (統合テスト除外)
   - 実行時間: 10-30秒
   - 目的: 単体テスト・バイナリテストの実行
   - 除外理由: proptest (237秒) などの時間のかかる統合テストを除外

### 設計の根拠

- **Clippy 優先**: 型エラーを数秒で検出し、テスト実行前に問題を発見
- **統合テスト除外**: 開発速度を優先し、時間のかかる統合テストはCIに委譲
- **CI で全テスト**: GitHub Actions で統合テストを含む全テストを実行
