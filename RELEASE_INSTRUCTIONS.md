# Release Instructions for Rustyll v0.8.0

This document provides step-by-step instructions for publishing Rustyll v0.8.0 to crates.io, GitHub, and Homebrew.

## Prerequisites

Before starting the release process, ensure you have:

1. ✅ A crates.io account with API token
2. ✅ GitHub CLI (`gh`) installed and authenticated
3. ✅ Write access to the Rustyll repository
4. ✅ Write access to a Homebrew tap repository (optional)

## Step 1: Publish to crates.io

### 1.1 Login to crates.io

If you haven't already, login to crates.io:

```bash
cargo login
```

You'll be prompted to enter your API token from https://crates.io/me

### 1.2 Dry run (recommended)

Test the publish process without actually uploading:

```bash
cargo publish --dry-run
```

This will:
- Package your crate
- Check all files are included
- Verify metadata is correct
- Ensure there are no errors

### 1.3 Publish to crates.io

Once the dry run succeeds, publish for real:

```bash
cargo publish
```

**Note**: Once published to crates.io, you cannot delete or replace a version. Make sure everything is correct!

### 1.4 Verify publication

Check your package on crates.io:
- Visit: https://crates.io/crates/rustyll
- Verify version 0.8.0 appears
- Check that README and documentation render correctly

## Step 2: Push changes and create GitHub Release

### 2.1 Push commit and tags

```bash
# Push the commit
git push origin main

# Create and push the tag
git tag -a v0.8.0 -m "Release v0.8.0"
git push origin v0.8.0
```

### 2.2 Create GitHub Release

You can create the release using the GitHub CLI:

```bash
gh release create v0.8.0 \
  --title "Rustyll v0.8.0 - Jekyll-Compatible Static Site Generator" \
  --notes-file RELEASE_NOTES_v0.8.0.md \
  --draft
```

The `--draft` flag creates a draft release for review. Remove it to publish immediately.

Alternatively, manually create the release:

1. Go to https://github.com/betterwebinit/rustyll/releases/new
2. Choose tag: `v0.8.0`
3. Release title: `Rustyll v0.8.0 - Jekyll-Compatible Static Site Generator`
4. Copy content from `RELEASE_NOTES_v0.8.0.md` to the description
5. Check "Set as the latest release"
6. Click "Publish release"

### 2.3 Upload release binaries (optional)

For better user experience, you can build and upload binaries for different platforms:

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu
tar -czf rustyll-v0.8.0-linux-x86_64.tar.gz -C target/x86_64-unknown-linux-gnu/release rustyll

# macOS x86_64
cargo build --release --target x86_64-apple-darwin
tar -czf rustyll-v0.8.0-macos-x86_64.tar.gz -C target/x86_64-apple-darwin/release rustyll

# macOS ARM64
cargo build --release --target aarch64-apple-darwin
tar -czf rustyll-v0.8.0-macos-arm64.tar.gz -C target/aarch64-apple-darwin/release rustyll

# Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc
zip rustyll-v0.8.0-windows-x86_64.zip target/x86_64-pc-windows-msvc/release/rustyll.exe
```

Then upload these files to the GitHub release.

## Step 3: Create/Update Homebrew Formula

### 3.1 Calculate SHA256 hash

After creating the GitHub release, download the source tarball and calculate its SHA256:

```bash
curl -L https://github.com/betterwebinit/rustyll/archive/refs/tags/v0.8.0.tar.gz -o rustyll-0.8.0.tar.gz
shasum -a 256 rustyll-0.8.0.tar.gz
```

### 3.2 Update the Homebrew formula

Update the `sha256` field in `rustyll.rb` with the hash from step 3.1.

### 3.3 Option A: Homebrew Core (long-term goal)

To submit to Homebrew core:

1. Fork https://github.com/Homebrew/homebrew-core
2. Add `rustyll.rb` to `Formula/r/rustyll.rb`
3. Test the formula: `brew install --build-from-source ./Formula/r/rustyll.rb`
4. Submit a pull request

Requirements for Homebrew core:
- Project must be stable and notable
- Must have 75+ GitHub stars
- Must have 30+ forks
- Must be actively maintained

### 3.4 Option B: Create a Homebrew Tap (immediate)

Create your own tap for easier installation:

```bash
# Create a tap repository
gh repo create betterwebinit/homebrew-rustyll --public

# Clone it
git clone https://github.com/betterwebinit/homebrew-rustyll
cd homebrew-rustyll

# Create Formula directory
mkdir -p Formula

# Copy the formula
cp /path/to/rustyll/rustyll.rb Formula/rustyll.rb

# Commit and push
git add Formula/rustyll.rb
git commit -m "Add Rustyll v0.8.0 formula"
git push origin main
```

Users can then install with:

```bash
brew tap betterwebinit/rustyll
brew install rustyll
```

### 3.5 Test the formula

```bash
# Audit the formula
brew audit --strict --online rustyll

# Test installation
brew install rustyll

# Test the binary
rustyll --version

# Uninstall
brew uninstall rustyll
```

## Step 4: Announce the Release

### 4.1 Update documentation

- Update the website at rustyll.better-web.org with v0.8.0 information
- Update any installation guides
- Update the README if needed

### 4.2 Social media and community

Consider announcing on:
- Twitter/X
- Reddit (r/rust, r/webdev)
- Hacker News
- Dev.to
- Your blog or website

### 4.3 Update project status

- Update project status badges
- Mark milestone as complete
- Close related issues
- Update roadmap

## Verification Checklist

Before considering the release complete, verify:

- [ ] Package appears on https://crates.io/crates/rustyll
- [ ] `cargo install rustyll` works
- [ ] GitHub release is published at https://github.com/betterwebinit/rustyll/releases
- [ ] Git tag v0.8.0 exists
- [ ] Homebrew formula works (if published)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is accurate
- [ ] Release announcement is published

## Rollback (if needed)

If you need to rollback:

### crates.io
- You **cannot** delete a version from crates.io
- You can **yank** a version: `cargo yank --vers 0.8.0`
- Yanked versions won't be used for new projects but existing ones still work
- Publish a patch version (0.8.1) with fixes

### GitHub
- You can delete a release and tag
- Use `gh release delete v0.8.0` or delete from the web UI
- Delete the tag: `git push origin :refs/tags/v0.8.0`

### Homebrew
- Submit a new PR removing or updating the formula
- Or update your tap repository

## Troubleshooting

### "failed to verify the publication"
- Wait a few minutes and try again
- crates.io might be experiencing issues

### "the remote server responded with an error: exhausted"
- Your upload was interrupted
- Try again with a better internet connection

### Homebrew formula fails
- Check the SHA256 hash matches
- Ensure the tarball URL is accessible
- Test with `brew install --build-from-source`
- Check Homebrew logs: `brew gist-logs rustyll`

## Next Steps

After releasing v0.8.0:

1. Start planning v0.8.1 or v0.9.0
2. Address known issues mentioned in release notes
3. Respond to user feedback and issues
4. Continue development on the main branch

---

For questions or issues with the release process, contact the Rustyll team or open an issue.
