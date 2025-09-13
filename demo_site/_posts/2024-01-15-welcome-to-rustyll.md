---
layout: post
title: "Welcome to Rustyll"
date: 2024-01-15 10:00:00 +0000
categories: [announcement, news]
tags: [rustyll, static-site-generator, rust]
author: "Rustyll Team"
---

We're excited to announce Rustyll, a new static site generator that combines the speed and safety of Rust with the familiarity of Jekyll.

## Why Rustyll?

After years of working with various static site generators, we noticed a pattern: as sites grew larger, build times became increasingly painful. We built Rustyll to solve this problem once and for all.

### Key Benefits

1. **Speed**: Rustyll builds sites 10-100x faster than Ruby-based Jekyll
2. **Memory Efficiency**: Build large sites with minimal RAM usage
3. **Compatibility**: Drop-in replacement for most Jekyll sites
4. **Modern Features**: Advanced asset pipeline, incremental builds, and more

## Getting Started

Migrating from Jekyll is straightforward:

```bash
# Install Rustyll
cargo install rustyll

# In your Jekyll site directory
rustyll build

# That's it!
```

## Performance Comparison

Here's how Rustyll compares to other popular static site generators:

| Generator | 1000 Pages | 10000 Pages |
|-----------|------------|-------------|
| Rustyll   | 0.8s       | 7.2s        |
| Jekyll    | 25.3s      | 312.5s      |
| Hugo      | 1.2s       | 11.8s       |
| Eleventy  | 8.5s       | 89.3s       |

## What's Next?

We're just getting started. Our roadmap includes:

- WebAssembly plugin support
- Built-in CMS integration
- Enhanced internationalization
- Cloud build integration

Join us in building the future of static site generation!

---

Want to contribute? Check out our [GitHub repository](https://github.com/better-web-initiative/rustyll) or join our [Discord community](https://discord.gg/rustyll).