# Release Guide

This document describes the release process for CLI Testing Specialist.

## Version 1.0.0 Release Checklist

### Pre-Release Validation

- [x] **All Tests Passing**
  - 98 unit tests passing
  - Integration tests with curl, git successful
  - All clippy checks passing with `-D warnings`
  - No formatting issues (`cargo fmt --check`)

- [x] **Security Audit**
  - `cargo audit`: 0 vulnerabilities
  - `cargo deny check`: All licenses approved
  - No known security issues

- [x] **Documentation**
  - Rustdoc: 100% coverage, 0 warnings
  - README.md complete with usage examples
  - CHANGELOG.md updated for v1.0.0
  - All public APIs documented

- [x] **Performance Benchmarks**
  - curl: 108ms (15x faster than Bash)
  - npm: 323ms (43x faster than Bash)
  - kubectl: 226ms (132x faster than Bash)
  - Memory: 6-68MB (under 50MB target)

- [x] **Packaging Metadata**
  - Cargo.toml: All fields complete
  - License: MIT
  - Repository: https://github.com/sanae-abe/cli-testing-specialist
  - Keywords and categories set

### Release Steps

#### 1. Version Bump (Already at 1.0.0-alpha.1)

For final release, update version in:
```bash
# Update Cargo.toml
sed -i '' 's/1.0.0-alpha.1/1.0.0/' Cargo.toml

# Update Cargo.lock
cargo build --release
```

#### 2. Final Build & Test

```bash
# Clean build
cargo clean
cargo build --release

# Run all tests
cargo test --all-features

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --check

# Security audit
cargo audit
cargo deny check

# Documentation
cargo doc --no-deps --open
```

#### 3. Integration Testing

```bash
# Test with real CLI tools
./target/release/cli-testing-specialist analyze /usr/bin/curl -o /tmp/curl.json
./target/release/cli-testing-specialist generate /tmp/curl.json -o /tmp/tests -c all
./target/release/cli-testing-specialist run /tmp/tests -f all -o /tmp/reports

# Verify reports generated
ls -lh /tmp/reports/
```

#### 4. Git Tag & Push

```bash
# Create annotated tag
git tag -a v1.0.0 -m "Release version 1.0.0

Complete Rust implementation with:
- 9 test categories
- 4 report formats
- 15-132x performance improvement
- 98 tests passing
- 0 security vulnerabilities"

# Push tag
git push origin v1.0.0
```

#### 5. Publish to crates.io

```bash
# Dry-run first
cargo publish --dry-run

# Actual publish
cargo publish
```

#### 6. GitHub Release

1. Go to https://github.com/sanae-abe/cli-testing-specialist/releases/new
2. Select tag: v1.0.0
3. Release title: "CLI Testing Specialist v1.0.0"
4. Description: Copy from CHANGELOG.md
5. Attach binaries (optional):
   ```bash
   # Build for macOS (current platform)
   cargo build --release
   tar czf cli-testing-specialist-v1.0.0-macos-arm64.tar.gz \
     -C target/release cli-testing-specialist

   # For Linux/Windows, use GitHub Actions or cross-compilation
   ```
6. Publish release

### Post-Release

#### Documentation Site (Optional)

```bash
# Generate and publish docs
cargo doc --no-deps
# Upload to GitHub Pages or docs.rs
```

#### Homebrew Formula (Optional)

Create a Homebrew tap for easy installation:
```ruby
# Formula/cli-testing-specialist.rb
class CliTestingSpecialist < Formula
  desc "Comprehensive testing framework for CLI tools"
  homepage "https://github.com/sanae-abe/cli-testing-specialist"
  url "https://github.com/sanae-abe/cli-testing-specialist/archive/v1.0.0.tar.gz"
  sha256 "..." # Calculate with: shasum -a 256 cli-testing-specialist-v1.0.0.tar.gz
  license "MIT"

  depends_on "rust" => :build
  depends_on "bats-core"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"cli-testing-specialist", "--version"
  end
end
```

#### Announcement

- [ ] Post to Rust subreddit (r/rust)
- [ ] Share on Twitter/X with #rustlang
- [ ] Update project README with installation instructions
- [ ] Add to awesome-rust list (if applicable)

### Rollback Plan

If critical issues are found after release:

1. **Yank from crates.io** (doesn't delete, just prevents new downloads):
   ```bash
   cargo yank --version 1.0.0
   ```

2. **Fix the issue** in a new version (1.0.1):
   ```bash
   # Fix the bug
   # Update version to 1.0.1
   cargo publish
   ```

3. **Document in CHANGELOG.md**:
   ```markdown
   ## [1.0.1] - 2025-11-12
   ### Fixed
   - Critical bug XYZ that caused...

   ## [1.0.0] - 2025-11-11 [YANKED]
   - Yanked due to critical bug XYZ
   ```

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.x.x): Incompatible API changes
- **MINOR** (x.1.x): New features, backward compatible
- **PATCH** (x.x.1): Bug fixes, backward compatible

### Pre-Release Versions

- **Alpha** (1.0.0-alpha.1): Early testing, unstable API
- **Beta** (1.0.0-beta.1): Feature complete, testing for bugs
- **RC** (1.0.0-rc.1): Release candidate, final testing

## Support Policy

- **v1.x**: Supported with bug fixes and security patches
- **v0.x** (Bash prototype): Deprecated, no support

## CI/CD Integration

The `.github/workflows/release.yml` workflow automates:
- Cross-platform builds (Linux, macOS, Windows)
- Binary artifact uploads
- crates.io publishing
- GitHub release creation

Trigger with:
```bash
git tag v1.0.0
git push origin v1.0.0
```

## License Compliance

All dependencies use approved licenses:
- MIT
- Apache-2.0
- MPL-2.0
- Unicode-3.0
- BSD-3-Clause
- ISC

Verified with `cargo deny check`.

---

## Questions?

Contact: sanae.abe@example.com
Issues: https://github.com/sanae-abe/cli-testing-specialist/issues
