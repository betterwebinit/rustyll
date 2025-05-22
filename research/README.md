# Jekyll Compatibility Research

This directory contains comprehensive research on building a Jekyll-compatible Static Site Generator (SSG) in Rust. The goal is to document all aspects of Jekyll's functionality to enable implementing a 100% compatible alternative.

## Research Files

1. [**Directory Structure and Theming**](1-directory-structure.md) - Covers Jekyll's standard directory structure, special directories, and how themes are organized.

2. [**YAML Front Matter System**](2-front-matter.md) - Details Jekyll's front matter parsing, variable usage, and defaults.

3. [**Configuration System**](3-configuration.md) - Documents Jekyll's `_config.yml` options, command-line flags, and how configuration affects site generation.

4. [**Plugin System**](4-plugin-system.md) - Explores Jekyll's plugin types, hooks, and extension points.

5. [**Liquid Templates**](5-liquid-templates.md) - Covers Jekyll's use of Liquid templating, including variables, tags, filters, and security model.

6. [**Markdown Rendering**](6-markdown-rendering.md) - Details Jekyll's Markdown processing, including Kramdown options, syntax highlighting, and GitHub-Flavored Markdown support.

7. [**Pagination**](7-pagination.md) - Explains Jekyll's pagination system, including configuration, limitations, and the advanced pagination plugin.

8. [**Collections and Data Handling**](8-collections-data.md) - Documents Jekyll's collections feature and data file handling.

9. [**GitHub Pages Compatibility**](9-github-pages.md) - Covers GitHub Pages' build environment, constraints, and special features.

10. [**Advanced Features**](10-advanced-features.md) - Explores incremental builds, internationalization, asset pipelines, and other advanced Jekyll capabilities.

## Implementation Strategy

To build a Jekyll-compatible SSG in Rust, we recommend the following implementation priorities:

### Core Features (High Priority)
- Directory structure handling
- YAML front matter parsing
- Configuration system
- Markdown rendering with GitHub-Flavored Markdown
- Liquid templating (or compatible alternative)
- Collections support (especially for posts)
- Data file handling

### Important Features (Medium Priority)
- GitHub Pages compatibility
- Basic pagination
- Asset processing (at least for Sass/SCSS)
- SEO tag generation
- Feed generation
- Front matter defaults

### Advanced Features (Lower Priority)
- Incremental builds
- Internationalization support
- Advanced pagination
- Related posts with better algorithms than Jekyll
- Plugin system (if desired)

## Rust Libraries to Consider

For a Jekyll-compatible implementation, these Rust libraries may be useful:

- **Template Rendering**: [liquid-rust](https://github.com/cobalt-org/liquid-rust) or [tera](https://github.com/Keats/tera)
- **Markdown**: [comrak](https://github.com/kivikakk/comrak) or [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
- **YAML**: [serde_yaml](https://github.com/dtolnay/serde-yaml)
- **Syntax Highlighting**: [syntect](https://github.com/trishume/syntect)
- **Sass Compilation**: [grass](https://github.com/connorskees/grass)
- **File Watching**: [notify](https://github.com/notify-rs/notify)

## Feature Comparison with Jekyll

The table below summarizes key Jekyll features and their implementation status in this project:

| Feature | Jekyll | Implementation Status | Notes |
|---------|--------|----------------------|-------|
| Core Functionality | ✅ | 🔄 In progress | Basic site generation |
| Liquid Templates | ✅ | 🔄 In progress | Using liquid-rust |
| Markdown (GFM) | ✅ | 🔄 In progress | Using comrak |
| Collections | ✅ | 🔄 In progress | Including posts |
| Data Files | ✅ | 🔄 In progress | YAML, JSON, CSV, TSV |
| Pagination | ✅ | 🔄 Planned | Basic pagination |
| Sass Processing | ✅ | 🔄 Planned | Using grass |
| Front Matter Defaults | ✅ | 🔄 Planned | |
| Incremental Builds | ✅ | 📅 Future | |
| Internationalization | ❌ (plugins) | 📅 Future | |
| GitHub Pages | ✅ | 📅 Future | |
| Custom Plugins | ✅ | ❓ TBD | May use a different approach |

## Project Goals

The primary goals of this research and implementation are:

1. **100% Jekyll Compatibility**: Allow users to migrate existing Jekyll sites without modification.
2. **Performance Improvements**: Leverage Rust's performance to provide faster build times.
3. **Modern Developer Experience**: Provide a more modern CLI and development experience.
4. **Documentation**: Ensure comprehensive documentation for easy migration and new user onboarding.

## Contributing

This research is open for contributions. If you identify gaps in the documentation or have insights into Jekyll's behavior that aren't covered, please submit improvements. 