#!/bin/bash

# Script to monitor the release process

echo "=========================================="
echo "Rustyll v0.8.0 Release Status Monitor"
echo "=========================================="
echo ""

# Check GitHub Actions status
echo "📊 GitHub Actions Status:"
echo ""
gh run list --limit 5 --json workflowName,status,conclusion,createdAt,displayTitle | \
  jq -r '.[] | "  \(.workflowName): \(.status) \(if .conclusion then "(\(.conclusion))" else "" end)"'

echo ""
echo "=========================================="
echo ""

# Check if release exists
echo "🏷️  GitHub Release:"
gh release view v0.8.0 --json tagName,name,publishedAt,url 2>/dev/null | \
  jq -r '"  Tag: \(.tagName)\n  Name: \(.name)\n  URL: \(.url)"' || \
  echo "  Release not yet created"

echo ""
echo "=========================================="
echo ""

# Check crates.io
echo "📦 crates.io Status:"
if curl -s "https://crates.io/api/v1/crates/rustyll" | jq -e '.crate.max_version == "0.8.0"' > /dev/null 2>&1; then
  echo "  ✅ Version 0.8.0 published to crates.io"
  echo "  🔗 https://crates.io/crates/rustyll"
else
  echo "  ⏳ Not yet published or waiting for index update"
  echo "  🔗 Check: https://crates.io/crates/rustyll"
fi

echo ""
echo "=========================================="
echo ""

echo "💡 Tips:"
echo "  - Watch live: gh run watch"
echo "  - View release: gh release view v0.8.0"
echo "  - Check logs: gh run view --log"
echo "  - Install: cargo install rustyll"
echo ""
echo "🌐 Links:"
echo "  - Actions: https://github.com/betterwebinit/rustyll/actions"
echo "  - Releases: https://github.com/betterwebinit/rustyll/releases"
echo "  - crates.io: https://crates.io/crates/rustyll"
echo ""
