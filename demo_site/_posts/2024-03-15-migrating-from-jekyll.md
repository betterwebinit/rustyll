---
layout: post
title: "Migrating from Jekyll: A Complete Guide"
date: 2024-03-15 09:15:00 +0000
categories: [tutorials, migration]
tags: [jekyll, migration, guide]
author: "Migration Team"
---

Step-by-step guide to migrating your existing Jekyll site to Rustyll without losing any functionality.

## Pre-Migration Checklist

Before starting your migration, ensure you have:

1. A working Jekyll site
2. Rust installed on your system
3. A backup of your current site

## Installation

```bash
# Install Rustyll via Cargo
cargo install rustyll

# Or build from source
git clone https://github.com/better-web-initiative/rustyll
cd rustyll && cargo build --release
```

## Basic Migration

Most Jekyll sites will work immediately with Rustyll:

```bash
cd your-jekyll-site
rustyll build
```

## Common Issues and Solutions

### Plugin Compatibility
Some Jekyll plugins aren't supported yet. Here's how to handle common cases:

- **jekyll-feed**: Built into Rustyll
- **jekyll-sitemap**: Automatically generated
- **jekyll-seo-tag**: Supported with slight modifications

### Liquid Template Differences
Rustyll's Liquid implementation is very close to Jekyll's, but there are minor differences in edge cases.

## Performance Gains

After migration, you'll typically see:
- 10-50x faster build times
- 5-10x lower memory usage
- Instant hot reloading during development