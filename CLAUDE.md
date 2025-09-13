# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rustyll is a blazing fast, Jekyll-compatible static site generator written in Rust. It aims to provide Jekyll compatibility while offering 10-100x faster build times through Rust's performance advantages.

## Development Commands

### Build and Run
```bash
# Build the project (debug mode)
cargo build

# Build for release with optimizations
cargo build --release

# Run the development version
cargo run -- [COMMAND]

# Build a site (default command)
cargo run -- build
# Or with explicit paths
cargo run -- build -s ./source -d ./destination

# Serve site with development server
cargo run -- serve
# With live reload
cargo run -- serve --livereload

# Clean build artifacts
cargo run -- clean

# Generate site report
cargo run -- report --verbose
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter with suggestions
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

## Architecture Overview

### Core Module Structure

The codebase is organized into distinct modules, each handling specific responsibilities:

- **`builder/`**: Core site building logic
  - `site/builder.rs`: Main build orchestration, processes all content and generates the static site
  - `site/processor.rs`: Content processing pipeline (markdown, liquid, front matter)
  - `page/`: Individual page building and rendering
  - `watcher/`: File system watching for incremental builds

- **`liquid/`**: Liquid templating engine integration
  - `tags/`: Custom Liquid tags (include, highlight, link, raw)
  - `filters/`: Custom Liquid filters (markdownify, relative_url, absolute_url, date_to_string)
  - `preprocess.rs`: Template preprocessing and optimization

- **`markdown/`**: Markdown processing
  - `renderer/`: Markdown to HTML rendering with syntax highlighting support
  - `toc/`: Table of contents generation and parsing
  - `engine/comrak.rs`: Comrak markdown parser integration

- **`collections/`**: Content collections management
  - `document/`: Document model and loader with front matter parsing
  - `data/`: Data file loading (YAML, JSON, CSV)

- **`front_matter/`**: Front matter extraction and parsing
  - `parser/`: YAML/TOML/JSON front matter parsers
  - `extractor.rs`: Front matter extraction from content files
  - `defaults.rs`: Default front matter values and merging

- **`config/`**: Configuration management
  - `loader.rs`: Config file loading and validation
  - `types.rs`: Configuration data structures
  - `defaults/`: Default configuration values

- **`server/`**: Development server
  - `core/server.rs`: Axum-based HTTP server
  - `middleware/`: Security, caching, compression middleware

- **`plugins/`**: Plugin system for extensibility
  - `hooks.rs`: Hook system for plugin integration points
  - `loader.rs`: Dynamic plugin loading
  - `registry.rs`: Plugin registration and management

### Key Design Patterns

1. **Jekyll Compatibility**: The architecture maintains Jekyll's directory structure (_posts, _layouts, _includes) and Liquid templating to ensure easy migration.

2. **Parallel Processing**: Uses Rayon for parallel content processing, significantly improving build times for large sites.

3. **Incremental Builds**: Tracks file modifications to rebuild only changed content.

4. **Plugin Architecture**: Extensible through a hook-based plugin system that allows customizing the build process at various stages.

## Important Implementation Details

### Content Processing Pipeline
1. Load configuration from `_config.yml`
2. Scan source directory for content files
3. Extract and parse front matter
4. Process Liquid templates (with includes resolution)
5. Render Markdown to HTML
6. Apply layouts hierarchically
7. Write to destination directory

### Liquid Template Resolution
- Templates are resolved from `_includes/` and `_layouts/` directories
- Custom tags and filters are registered during initialization
- Context includes site data, page variables, and collection data

### Performance Optimizations
- Parallel file processing using Rayon
- Lazy loading of large data files
- Efficient front matter extraction without full file parsing
- Cached template compilation

## Directory Conventions

The project follows Jekyll's directory structure:
- `_config.yml`: Site configuration
- `_posts/`: Blog posts with date-prefixed filenames
- `_layouts/`: Page layout templates
- `_includes/`: Reusable template fragments
- `_data/`: Data files (YAML, JSON, CSV)
- `_site/`: Generated output (default destination)
- `assets/`: Static assets (CSS, JS, images)

## Testing Approach

Tests are embedded within modules using `#[test]` attributes. Key test areas:
- Front matter parsing (`src/front_matter/`)
- Markdown rendering (`src/markdown/`)
- Plugin system (`src/plugins/`)
- Middleware (`src/server/middleware/`)

Run tests for specific modules:
```bash
cargo test front_matter
cargo test markdown
cargo test plugins
```