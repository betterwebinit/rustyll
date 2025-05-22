# Middleman Migrator for Rustyll

This module handles migration of Middleman static sites to the Rustyll format.

## Features

* Converts Middleman Ruby configuration to Rustyll YAML configuration
* Migrates content from various Middleman formats including Markdown and ERB
* Transforms Middleman layouts and templates to Rustyll Liquid templates
* Migrates partials to Rustyll includes
* Transfers data files from data/ to _data/
* Preserves static assets with appropriate directory structure

## Detection

The Middleman migrator is activated when the source directory contains one of:
- config.rb file (Middleman configuration)
- Gemfile with middleman dependency
- source/ directory (standard Middleman content directory)

## Migration Process

1. **Configuration**: Converts Middleman Ruby config to Rustyll YAML format
2. **Content**: 
   - Content files from source/ are migrated to appropriate Rustyll formats
   - Front matter is preserved and adjusted as needed
   - ERB templates are converted to Liquid where possible
3. **Layouts**: 
   - Layout templates are converted to Rustyll's _layouts format
   - ERB/Slim/Haml syntax is converted to Liquid where possible
4. **Partials**: Partials are migrated to Rustyll's _includes directory
5. **Data**: Data files are migrated to Rustyll's _data structure
6. **Static Assets**: Files from source/ are copied to the assets directory

## Known Limitations

- Ruby helpers need manual conversion to Liquid filters/tags
- Complex ERB/Slim/Haml templates may require manual adjustment
- Custom Middleman extensions need manual reimplementation
- Ruby in templates needs to be converted to Liquid equivalents

## Implementation Notes

The migrator is implemented in modular fashion with separate components handling:
- Configuration migration (config.rs)
- Content migration (content.rs)
- Layouts migration (layouts.rs)
- Partials migration (partials.rs)
- Data migration (data.rs)
- Static asset migration (static_assets.rs)
- Documentation generation (readme.rs) 