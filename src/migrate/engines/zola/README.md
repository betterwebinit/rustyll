# Zola Migrator for Rustyll

This module handles migration of Zola static sites to the Rustyll format.

## Features

* Converts Zola configuration files (config.toml)
* Migrates content from the `content/` directory to appropriate Rustyll formats
* Transforms Zola templates to Rustyll templates
* Transfers static assets while preserving directory structure

## Detection

The Zola migrator is activated when the source directory contains all of:
- config.toml file
- templates directory
- content directory

## Migration Process

1. **Configuration**: Converts Zola's config.toml to Rustyll format
2. **Content**: 
   - Section pages and content pages are converted to Rustyll format
   - Taxonomy pages are migrated appropriately
3. **Templates**: 
   - Tera templates are transformed to Liquid templates
   - Base templates, macros, and partials are all converted
4. **Static Assets**: Files from the static directory are copied to Rustyll's assets

## Known Limitations

- Zola shortcodes need manual conversion to Rustyll template tags
- Tera-specific templating features may require adjustments
- Some Zola-specific features (like automatic section pages) may need configuration

## Implementation Notes

The migrator is implemented in modular fashion with separate components handling:
- Configuration migration (config.rs)
- Content migration (content.rs)
- Templates migration (templates.rs)
- Static asset migration (static_assets.rs)
- Documentation generation (readme.rs) 