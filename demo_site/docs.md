---
layout: page
title: Documentation
permalink: /docs/
---

# Documentation

Complete guide to using Rustyll for building fast static sites.

## Quick Start

### Installation

```bash
# Install from crates.io
cargo install rustyll

# Or build from source
git clone https://github.com/better-web-initiative/rustyll
cd rustyll
cargo build --release
```

### Your First Site

```bash
# Create a new site
rustyll new my-site
cd my-site

# Build the site
rustyll build

# Start development server
rustyll serve
```

## Configuration

Rustyll uses a `_config.yml` file for site configuration:

```yaml
title: "My Rustyll Site"
description: "A fast static site built with Rustyll"
author: "Your Name"
email: "your.email@example.com"

# Build settings
destination: "_site"
source: "."
layouts_dir: "_layouts"
includes_dir: "_includes"
data_dir: "_data"

# Jekyll compatibility
markdown: kramdown
highlighter: rouge
permalink: /:categories/:year/:month/:day/:title/

# Collections
collections:
  posts:
    output: true
    permalink: /:collection/:name/
  projects:
    output: true
    permalink: /:collection/:name/
```

## Directory Structure

```
my-site/
├── _config.yml          # Site configuration
├── _layouts/            # Page templates
│   ├── default.html
│   ├── post.html
│   └── page.html
├── _includes/           # Reusable components
│   ├── header.html
│   ├── footer.html
│   └── sidebar.html
├── _posts/              # Blog posts
│   └── 2024-01-01-hello-world.md
├── _data/               # Data files
│   └── navigation.yml
├── assets/              # Static assets
│   ├── css/
│   ├── js/
│   └── images/
├── index.md             # Homepage
└── about.md             # About page
```

## Liquid Templates

Rustyll supports Jekyll-compatible Liquid templating:

### Variables
```liquid
{{ site.title }}
{{ page.title }}
{{ content }}
```

### Filters
```liquid
{{ "hello world" | capitalize }}
{{ site.time | date_to_string }}
{{ content | markdownify }}
```

### Tags
```liquid
{% raw %}{% for post in site.posts %}
  <h2>{{ post.title }}</h2>
{% endfor %}

{% include header.html %}
{% include_relative sidebar.md %}{% endraw %}
```

## Performance

Rustyll is designed for speed:

- **Parallel Processing:** Uses all CPU cores for faster builds
- **Incremental Builds:** Only rebuilds changed files
- **Optimized Output:** Compressed HTML, CSS, and JavaScript
- **Fast Serving:** Built-in development server with live reload

## Migration from Jekyll

Rustyll is designed to be Jekyll-compatible:

1. Copy your Jekyll site files
2. Update `_config.yml` if needed
3. Run `rustyll build`

Most Jekyll sites work without modification!

## API Reference

### Commands

- `rustyll build` - Build the site
- `rustyll serve` - Start development server
- `rustyll new <name>` - Create new site
- `rustyll clean` - Clean build artifacts

### Options

- `--source <dir>` - Source directory
- `--destination <dir>` - Output directory
- `--verbose` - Verbose output
- `--watch` - Watch for changes

## Troubleshooting

### Common Issues

**Build fails with template errors:**
- Check Liquid syntax in templates
- Ensure all included files exist

**Pages not appearing:**
- Verify front matter is present
- Check file extensions (.md, .html)

**Slow build times:**
- Use `--incremental` flag
- Optimize large collections

### Getting Help

- [GitHub Issues](https://github.com/better-web-initiative/rustyll/issues)
- [Community Forum](https://github.com/better-web-initiative/rustyll/discussions)
- [Documentation Wiki](https://github.com/better-web-initiative/rustyll/wiki)