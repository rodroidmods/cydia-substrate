# Publishing to Crates.io - Checklist

## ✅ Completed

- [x] Complete Rust implementation
- [x] Clean build (0 warnings, 0 errors)
- [x] Comprehensive README.md with examples
- [x] Working code examples (5 Rust + 2 C/C++)
- [x] C header file (substrate.h)
- [x] Cargo.toml with full metadata
- [x] Error handling examples
- [x] Multi-architecture support documented

## 📋 Before Publishing

### 1. Update Repository URLs

In `Cargo.toml`, replace placeholders:
```toml
homepage = "https://github.com/YOURUSERNAME/substrate-rs"
repository = "https://github.com/YOURUSERNAME/substrate-rs"
```

### 2. Create LICENSE Files

```bash
# LGPL-3.0 License
cp /path/to/LGPL-3.0.txt LICENSE

# Or download:
wget https://www.gnu.org/licenses/lgpl-3.0.txt -O LICENSE
```

### 3. Test Examples

```bash
# Test all Rust examples
cargo run --example basic_hook
cargo run --example library_hook
cargo run --example android_game_hook
cargo run --example multi_arch
cargo run --example error_handling

# Test C examples (optional)
cd examples
gcc example.c -o example -L../target/release -lsubstrate -ldl
g++ example.cpp -o example_cpp -L../target/release -lsubstrate -ldl -std=c++11
```

### 4. Generate Documentation

```bash
# Generate local docs
cargo doc --no-deps --open

# Check for documentation warnings
cargo doc 2>&1 | grep warning
```

### 5. Run Tests

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Check for unused dependencies
cargo machete  # Install with: cargo install cargo-machete
```

### 6. Verify Package

```bash
# Dry run to check what will be published
cargo package --list

# Build the package
cargo package

# Check package size
ls -lh target/package/substrate-rs-0.1.0.crate
```

### 7. Security Audit

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit
```

### 8. Cross-Platform Build Test

```bash
# Test on different architectures (if possible)
cargo build --target x86_64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu
cargo build --target armv7-unknown-linux-gnueabihf

# Or use cross (install with: cargo install cross)
cross build --target aarch64-linux-android
cross build --target armv7-linux-androideabi
```

## 🚀 Publishing Steps

### 1. Login to Crates.io

```bash
# Get your API token from https://crates.io/me
cargo login YOUR_API_TOKEN
```

### 2. Publish (Dry Run)

```bash
# Test publish without actually uploading
cargo publish --dry-run
```

### 3. Publish for Real

```bash
# Actually publish to crates.io
cargo publish
```

### 4. Verify Publication

- Visit: https://crates.io/crates/substrate-rs
- Check documentation: https://docs.rs/substrate-rs
- Test installation: `cargo add substrate-rs`

## 📝 Post-Publication

### 1. Create Git Tag

```bash
git tag -a v0.1.0 -m "Initial release"
git push origin v0.1.0
```

### 2. Create GitHub Release

- Go to: https://github.com/YOURUSERNAME/substrate-rs/releases
- Create new release from tag v0.1.0
- Add release notes
- Attach prebuilt binaries (optional)

### 3. Update README Badges

Ensure badges work:
- [![Crates.io](https://img.shields.io/crates/v/substrate-rs.svg)](https://crates.io/crates/substrate-rs)
- [![Documentation](https://docs.rs/substrate-rs/badge.svg)](https://docs.rs/substrate-rs)

## 📦 Building Release Binaries

For users who want to use the C API:

```bash
# Linux x86-64
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Linux ARMv7
cargo build --release --target armv7-unknown-linux-gnueabihf

# Android ARM64
cargo build --release --target aarch64-linux-android

# Android ARMv7
cargo build --release --target armv7-linux-androideabi
```

Libraries will be in `target/<TRIPLE>/release/libsubstrate.so`

Package them with the header:
```bash
mkdir -p release/{include,lib}
cp substrate.h release/include/
cp target/*/release/libsubstrate.so release/lib/libsubstrate-<arch>.so
zip -r substrate-rs-v0.1.0.zip release/
```

## 🔧 Maintenance

### Updating Version

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Commit changes
4. Create new tag
5. Publish: `cargo publish`

### Yanking a Version

```bash
# If you need to yank a broken release
cargo yank --vers 0.1.0

# To un-yank
cargo yank --vers 0.1.0 --undo
```

## 📊 Metrics to Track

After publication, monitor:
- Downloads: https://crates.io/crates/substrate-rs/reverse_dependencies
- GitHub stars
- Issues and PRs
- Documentation views
- Community feedback

## 🎯 Success Criteria

- [x] Clean build with zero warnings
- [ ] All examples compile and run
- [ ] Documentation is complete
- [ ] README is comprehensive
- [ ] License file exists
- [ ] Repository URLs are correct
- [ ] Package size < 10MB
- [ ] Successful `cargo publish --dry-run`

## ⚠️ Common Issues

### Issue: Package too large
**Solution:** Add `.cargo-ok` and unnecessary files to `.gitignore`

### Issue: Documentation warnings
**Solution:** Add `#![warn(missing_docs)]` and document all public items

### Issue: Examples don't compile
**Solution:** Test each example individually before publishing

### Issue: License missing
**Solution:** Add LICENSE file to repository root

## 📚 Resources

- [Crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Semantic Versioning](https://semver.org/)
