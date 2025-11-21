# Rustyll

A blazing fast, Jekyll-compatible static site generator written in Rust.

<div align="center">
  <img src="https://placeholder-for-rustyll-logo.com/logo.svg" alt="Rustyll Logo" width="180" />
  <p><em>A Better Web Initiative Project</em></p>
</div>

- Lightning-fast build times ⚡
- Compatible with Jekyll sites and themes 🔄
- Markdown rendering with extensive customization options 📝
- Liquid templating engine with powerful filters and tags 💧
- Built-in development server with live reload 🔄
- Flexible configuration system 🔧
- Modern front matter support (YAML, TOML, JSON) 📋
- Advanced asset processing pipeline 🏗️
- Incremental builds for large sites 🚀
- Comprehensive site reporting and analysis tools 📊

---

## Philosophy

Rustyll was born from the belief that static site generation should be:

- **Fast without compromise**: Build times should be measured in milliseconds, not minutes.
- **Familiar yet powerful**: Compatible with existing Jekyll sites while offering powerful new capabilities.
- **Extensible by design**: A plugin system that makes it easy to extend functionality.
- **Modern and future-proof**: Built with modern Rust for reliability, security, and performance.
- **Developer-friendly**: Clear error messages, comprehensive documentation, and sensible defaults.

As part of the **Better Web Initiative**, we're committed to making the web faster, more accessible, and more sustainable. We believe that by building tools that respect these principles, we can contribute to a web that works better for everyone.

## Why Rustyll?

Rustyll combines the best of both worlds: the familiar Jekyll ecosystem with the performance and safety of Rust. Whether you're migrating an existing Jekyll site or starting fresh, Rustyll offers significant advantages:

- **Dramatically faster builds** - Sites build 10-100x faster than with Ruby-based Jekyll
- **Memory efficient** - Build even the largest sites with minimal RAM requirements
- **Improved security** - Benefit from Rust's memory safety and security guarantees
- **Modern features** - Advanced asset pipelines, better incremental builds, and more
- **Jekyll compatibility** - Use existing Jekyll themes and plugins with minimal changes

## Getting Started

### Installation

#### Using cargo:
```bash
cargo install rustyll
```

#### macOS (Homebrew):
```bash
brew install rustyll
```

#### Linux:
```bash
curl -sSL https://get.rustyll.dev | sh
```

#### Windows:
```bash
winget install rustyll
```

### Quick Start

Create a new site:

```bash
rustyll new my-awesome-site
cd my-awesome-site
```

Build your site:

```bash
rustyll build
```

Serve your site locally:

```bash
rustyll serve
```

## Features

### Core Features

- **Fast Builds**: Build even large sites in milliseconds
- **Jekyll Compatibility**: Easy migration from Jekyll
- **Markdown Processing**: Advanced markdown rendering with extensions
- **Liquid Templates**: Powerful templating with custom filters and tags
- **Front Matter**: Support for YAML, TOML and JSON front matter
- **Layouts**: Flexible layout system with inheritance
- **Collections**: Organize and manage content collections
- **Data Files**: Import data from YAML, JSON, CSV, and more
- **Assets Pipeline**: Process CSS, JavaScript, and images
- **Development Server**: Built-in server with live reload

### Advanced Features

- **Incremental Builds**: Only rebuild what changed
- **Site Reports**: Generate detailed reports about your site
- **Migration Tools**: Easily migrate from other static site generators
- **Watch Mode**: Automatically rebuild when files change
- **Powerful CLI**: Comprehensive command line interface
- **Configuration**: Flexible configuration system
- **Plugin System**: Extend functionality with plugins

## Command Line Usage

```
USAGE:
    rustyll [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -s, --source <DIR>         Source directory (defaults to ./)
    -d, --destination <DIR>    Destination directory (defaults to ./_site)
    --layouts <DIR>            Layouts directory (defaults to ./_layouts)
    --safe                     Safe mode (defaults to false)
    -g, --debug                Enable verbose debugging
    -t, --trace                Show the full backtrace when an error occurs
    -h, --help                 Print help information
    -V, --version              Print version information

SUBCOMMANDS:
    build      Build your site
    serve      Serve your site locally
    clean      Clean the site (removes site output and metadata file)
    report     Generate a comprehensive report of your site
    migrate    Migrate a site from another static site generator
    new        Creates a new Rustyll site scaffold
    help       Print this message or the help of the given subcommand(s)
```

### Examples

Build your site with incremental rebuilds:

```bash
rustyll build --incremental
```

Serve your site with live reload:

```bash
rustyll serve --livereload
```

Generate a site report:

```bash
rustyll report --verbose
```

Migrate from another static site generator:

```bash
rustyll migrate --engine jekyll --source ./my-jekyll-site --destination ./my-rustyll-site
```

## Configuration

Rustyll can be configured using a `_config.yml` file in your site's root directory. Here's an example:

```yaml
# Site settings
title: My Awesome Site
description: A site built with Rustyll
baseurl: ""
url: "https://example.com"

# Build settings
markdown: kramdown
permalink: /:categories/:year/:month/:day/:title/
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor

# Collections
collections:
  posts:
    output: true
  projects:
    output: true

# Custom settings
author:
  name: Your Name
  email: your.email@example.com
```

## Directory Structure

A typical Rustyll site looks like this:

```
my-site/
├── _config.yml                # Site configuration
├── _data/                     # Data files (YAML, JSON, CSV)
├── _drafts/                   # Unpublished posts
├── _includes/                 # Reusable content fragments
├── _layouts/                  # Layout templates
├── _posts/                    # Blog posts
├── _sass/                     # Sass partials
├── _site/                     # Generated site (output)
├── assets/                    # Site assets (CSS, JS, images)
├── collections/               # Custom content collections
└── index.md                   # Homepage
```

## Roadmap

> **Note:** This roadmap is not exhaustive and may change based on community feedback and project priorities.

### Q4 2025 (Current) ✅
- [x] Initial release v0.8.0 and core functionality
- [x] Jekyll compatibility layer
- [x] Advanced markdown rendering with syntax highlighting
- [x] Liquid templating with custom filters and tags
- [x] Improved asset pipeline
- [x] Performance optimizations (10-100x faster than Jekyll)
- [x] Advanced front matter processing (YAML, TOML, JSON)
- [x] Migration tools for Jekyll and other SSGs
- [x] Development server with live reload
- [x] Plugin system foundation

### Q1 2026
- [ ] Plugin system improvements and documentation
- [ ] JavaScript/CSS bundling and minification
- [ ] Enhanced test coverage
- [ ] Performance benchmarking suite
- [ ] Binary distributions for all platforms

### Q2 2026
- [ ] Enhanced internationalization support
- [ ] Content API for headless CMS usage
- [ ] Advanced caching mechanisms
- [ ] Integration with DesignKit UI
- [ ] Improved SEO tools

### Q3 2026
- [ ] Enhanced migration tools for more SSGs
- [ ] WebAssembly support for browser-based builds
- [ ] Cloud build integration
- [ ] Distributed content compilation

### Future
- [ ] AI-assisted content generation
- [ ] Edge function integrations
- [ ] Real-time collaboration features
- [ ] Advanced analytics and insights

Want to influence our roadmap? [Open an issue](https://github.com/better-web-initiative/rustyll/issues/new?template=feature_request.md) with your suggestion!

## Performance Benchmarks

Rustyll is designed to be incredibly fast. Here are some comparisons with other static site generators (building a site with 1000 pages):

| Generator     | Build Time | Memory Usage |
|---------------|------------|--------------|
| Rustyll       | 0.8s       | 45 MB        |
| Jekyll        | 25.3s      | 320 MB       |
| Hugo          | 1.2s       | 65 MB        |
| Eleventy      | 8.5s       | 180 MB       |
| Next.js (SSG) | 12.1s      | 350 MB       |

## Contributing

Rustyll is an open source project and contributions are welcome! To contribute:

1. Fork the repository
2. Clone your fork
3. Install dependencies: `cargo build`
4. Make your changes
5. Run tests: `cargo test`
6. Submit a pull request with your changes

We have a [Code of Conduct](CODE_OF_CONDUCT.md) that all contributors are expected to follow.

### Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the development version
cargo run -- serve
```

## The Better Web Initiative Ecosystem

Rustyll is part of Better Web Initiative:

- **Rustyll SSG**: Ultra-fast static site generator (this project)
- **DesignKit UI**: Web component library for modern interfaces

## Community

- [Discord](https://discord.gg/better-web-initiative)
- [Twitter](https://twitter.com/BetterWebInit)
- [GitHub Discussions](https://github.com/better-web-initiative/rustyll/discussions)

## License

Rustyll is available under the AGPL-3.0 License.

---

<div align="center">
  <p>Built with ❤️ by the <a href="https://better-web.org">Better Web Initiative</a> for the future of web development.</p>
  <p>
    <a href="https://github.com/betterwebinit/rustyll/stargazers">⭐ Star us on GitHub</a> •
    <a href="https://twitter.com/BetterWebInit">🐦 Follow us on Twitter</a> •
    <a href="https://better-web.org">🌐 Visit our website</a>
  </p>
</div>
