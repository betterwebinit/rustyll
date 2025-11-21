# GitHub Actions Workflows

This directory contains automated workflows for Rustyll CI/CD.

## Workflows

### 1. CI (`ci.yml`)

**Trigger**: On push to `main` or `develop`, and on pull requests

**Purpose**: Continuous Integration testing

**Jobs**:
- **Test**: Runs tests on multiple platforms (Ubuntu, macOS, Windows) and Rust versions (stable, nightly)
- **Build**: Builds release binary and checks size
- **Security Audit**: Runs `cargo audit` to check for security vulnerabilities
- **Check Dependencies**: Checks for outdated dependencies

**Usage**: Automatically runs on every push and PR

### 2. Release (`release.yml`)

**Trigger**: On pushing a version tag (e.g., `v0.8.0`)

**Purpose**: Automated release to crates.io, build binaries, and create GitHub release

**Jobs**:
1. **publish-crate**: Publishes the crate to crates.io
2. **build-binaries**: Builds binaries for multiple platforms:
   - Linux: x86_64, aarch64 (GNU and musl)
   - macOS: x86_64, aarch64 (Apple Silicon)
   - Windows: x86_64
3. **create-release**: Creates a GitHub release with all binaries
4. **update-homebrew**: Updates the Homebrew formula with correct SHA256

**Usage**:
```bash
# Create and push a tag
git tag -a v0.8.0 -m "Release v0.8.0"
git push origin v0.8.0

# The workflow will automatically:
# 1. Verify version matches tag
# 2. Publish to crates.io
# 3. Build binaries for all platforms
# 4. Create GitHub release
# 5. Attach binaries to release
# 6. Update Homebrew formula
```

### 3. Manual Publish (`publish-manual.yml`)

**Trigger**: Manual workflow dispatch

**Purpose**: Manually publish to crates.io with dry-run option

**Usage**:
1. Go to Actions tab in GitHub
2. Select "Manual Publish to crates.io"
3. Click "Run workflow"
4. Choose dry-run mode or real publish
5. Click "Run workflow" button

**Options**:
- **Dry run** (default: true): Test the publish without actually uploading

## Secrets Required

The following secrets must be configured in your GitHub repository:

- `CARGO_REGISTRY_TOKEN`: Your crates.io API token
  - Get from: https://crates.io/me
  - Add to: Repository Settings → Secrets and variables → Actions → New repository secret

## Release Process

### Automatic Release (Recommended)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Release v0.8.0"
   git push origin main
   ```
4. Create and push tag:
   ```bash
   git tag -a v0.8.0 -m "Release v0.8.0"
   git push origin v0.8.0
   ```
5. Watch the Actions tab - everything else is automatic!

### Manual Release

Use the manual publish workflow if you need more control or if the automatic release failed.

## Platform Support

### Built Binaries

The release workflow builds binaries for:

| Platform | Target | Binary Name |
|----------|--------|-------------|
| Linux x86_64 (GNU) | x86_64-unknown-linux-gnu | rustyll-linux-x86_64.tar.gz |
| Linux x86_64 (musl) | x86_64-unknown-linux-musl | rustyll-linux-x86_64-musl.tar.gz |
| Linux ARM64 | aarch64-unknown-linux-gnu | rustyll-linux-aarch64.tar.gz |
| macOS x86_64 | x86_64-apple-darwin | rustyll-macos-x86_64.tar.gz |
| macOS ARM64 | aarch64-apple-darwin | rustyll-macos-aarch64.tar.gz |
| Windows x86_64 | x86_64-pc-windows-msvc | rustyll-windows-x86_64.exe.zip |

### Installation from Binaries

Users can download pre-built binaries from the GitHub releases page:

```bash
# Linux/macOS
curl -L https://github.com/betterwebinit/rustyll/releases/download/v0.8.0/rustyll-linux-x86_64.tar.gz | tar xz
sudo mv rustyll /usr/local/bin/

# Or use cargo
cargo install rustyll

# Or Homebrew (macOS)
brew tap betterwebinit/rustyll
brew install rustyll
```

## Troubleshooting

### Release workflow fails at publish step

- Check that `CARGO_REGISTRY_TOKEN` is correctly set
- Verify the version in `Cargo.toml` hasn't been published before
- Check crates.io status: https://status.crates.io/

### Binary build fails for a specific platform

- Check the Actions logs for that platform
- May need to update cross-compilation dependencies
- Some platforms may require additional setup

### Version mismatch error

The workflow checks that the git tag version matches `Cargo.toml`. Ensure they match:
- Tag: `v0.8.0`
- Cargo.toml: `version = "0.8.0"`

### Homebrew formula SHA mismatch

The SHA is calculated automatically after release creation. If it fails:
- Download the release tarball manually
- Calculate SHA: `shasum -a 256 rustyll-0.8.0.tar.gz`
- Update `rustyll.rb` manually

## Monitoring

- **CI Status**: Check the Actions tab for test results
- **Release Progress**: Watch the release workflow in real-time
- **crates.io**: https://crates.io/crates/rustyll
- **GitHub Releases**: https://github.com/betterwebinit/rustyll/releases

## Contributing

When contributing:
- All PRs trigger CI tests automatically
- Ensure tests pass before merging
- Update version and changelog before creating release tags
- Follow semantic versioning for tags

## Security

- Never commit API tokens or secrets
- Use GitHub Secrets for sensitive data
- Audit workflow changes carefully
- Review security audit results regularly
