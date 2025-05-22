# MkDocs Migrator for Rustyll

This module handles migration of MkDocs documentation sites to the Rustyll format.

## Features

* Converts MkDocs YAML configuration to Rustyll YAML configuration
* Migrates Markdown content files while preserving front matter
* Transforms MkDocs templates to Rustyll Liquid templates
* Preserves the navigation structure
* Transfers static assets with appropriate directory structure
* Handles MkDocs-specific features like admonitions

## Detection

The MkDocs migrator is activated when the source directory contains one of:
- mkdocs.yml file (MkDocs configuration)
- docs/ directory (standard MkDocs content directory)
- requirements.txt with mkdocs dependency

## Migration Process

1. **Configuration**: Converts MkDocs YAML config to Rustyll YAML format
2. **Content**: 
   - Markdown files from docs/ are migrated to appropriate Rustyll formats
   - Front matter is preserved and adjusted as needed
   - MkDocs-specific syntax is converted to Rustyll-compatible syntax
3. **Layouts**: 
   - Theme templates are converted to Rustyll's _layouts format
   - Navigation structure is preserved
4. **Static Assets**: 
   - CSS, JavaScript, and images are copied to the assets directory
   - Theme assets are properly handled

## Known Limitations

- MkDocs plugins need to be manually reimplemented
- Custom theme extensions may require adjustment
- Python-specific features need manual conversion
- Some advanced MkDocs features may not have direct equivalents in Rustyll

## Implementation Notes

The migrator is implemented in modular fashion with separate components handling:
- Configuration migration (config.rs)
- Content migration (content.rs)
- Layouts migration (layouts.rs)
- Static asset migration (static_assets.rs)
- Documentation generation (readme.rs) 