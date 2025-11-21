# Changelog

All notable changes to Rustyll will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2025-11-20

### Added
- Jekyll-compatible Liquid templating engine with custom filters and tags
- Advanced markdown rendering with syntax highlighting support
- Complete front matter support (YAML, TOML, JSON)
- Built-in development server with live reload capabilities
- Collection management system for organizing content
- Data file loading support (YAML, JSON, CSV)
- Incremental build system for faster rebuilds
- Comprehensive site reporting and analysis tools
- Migration tools for Jekyll and other static site generators
- Flexible configuration system compatible with Jekyll
- Custom Liquid filters: `markdownify`, `relative_url`, `absolute_url`, `date_to_string`, `number_with_delimiter`
- Custom Liquid tags: `include`, `highlight`, `link`, `raw`
- Table of contents generation
- Parallel content processing using Rayon
- Security middleware for the development server
- MIME type handling for static assets
- Plugin system with hooks for extensibility

### Changed
- Optimized build performance for 10-100x faster builds compared to Jekyll
- Improved memory efficiency for large sites
- Enhanced error messages and debugging output
- Modernized CLI interface with better user experience

### Fixed
- Liquid parser handling for includes with complex paths
- Proper MIME type detection for various file types
- Front matter extraction and parsing edge cases
- Development server asset serving issues

## [Unreleased]

### Planned
- JavaScript/CSS bundling and minification
- Enhanced internationalization support
- Content API for headless CMS usage
- Advanced caching mechanisms
- Integration with DesignKit UI
- WebAssembly support

[0.8.0]: https://github.com/betterwebinit/rustyll/releases/tag/v0.8.0
