# Eleventy (11ty) Migrator for Rustyll

This module handles migration of Eleventy (11ty) static sites to the Rustyll format.

## Features

* Converts Eleventy configuration files (.eleventy.js, eleventy.config.js)
* Migrates content from various formats (Markdown, Nunjucks, Liquid, etc.)
* Transforms Eleventy templates to Rustyll templates
* Migrates data files and includes
* Transfers static assets while preserving directory structure

## Detection

The Eleventy migrator is activated when the source directory contains one of:
- .eleventy.js file
- eleventy.config.js file
- Both _includes and _data directories

## Migration Process

1. **Configuration**: Converts Eleventy config to Rustyll format
2. **Content**: 
   - Content files are converted to appropriate Rustyll formats
   - Front matter is preserved and adjusted as needed
3. **Includes**: 
   - Template includes are converted to Rustyll format
   - Shortcodes are transformed into equivalent Rustyll constructs
4. **Data**: Data files are migrated to Rustyll's data structure
5. **Static Assets**: Files from public or static directories are copied to Rustyll's assets

## Known Limitations

- JavaScript-based templates and functionality need manual adaptation
- Custom Eleventy plugins require manual conversion
- Some Nunjucks/Liquid syntax may need adjustment

## Implementation Notes

The migrator is implemented in modular fashion with separate components handling:
- Configuration migration (config.rs)
- Content migration (content.rs)
- Includes migration (includes.rs)
- Data migration (data.rs)
- Static asset migration (static_assets.rs)
- Documentation generation (readme.rs) 