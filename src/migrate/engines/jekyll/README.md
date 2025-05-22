# Jekyll Migrator for Rustyll

This module handles migration of Jekyll static sites to the Rustyll format.

## Features

* Converts Jekyll configuration files (_config.yml)
* Migrates content from Jekyll directories (_posts, root pages, collections)
* Transforms layouts to Rustyll templates
* Transfers includes to Rustyll partials
* Handles data files and assets 

## Detection

The Jekyll migrator is activated when the source directory contains one of:
- _config.yml file
- _layouts directory
- _includes directory

## Migration Process

1. **Configuration**: Converts Jekyll's _config.yml to Rustyll format
2. **Content**: 
   - Blog posts (from _posts) are moved to _posts/
   - Pages and other content are converted to appropriate Rustyll formats
   - Collections are migrated with their structure preserved
3. **Layouts**: Jekyll layouts are transformed into Rustyll templates
4. **Includes**: Jekyll includes are converted to Rustyll partials
5. **Data Files**: Data files are migrated to Rustyll's data structure
6. **Assets**: Static files and assets are copied to Rustyll's assets directory

## Known Limitations

- Some Liquid tags may need manual adjustment
- Plugins functionality may require custom implementation
- Jekyll-specific themes need to be adapted for Rustyll

## Implementation Notes

The migrator is implemented in modular fashion with separate components handling:
- Configuration migration (config.rs)
- Content migration (content.rs)
- Layout migration (layouts.rs)
- Includes migration (includes.rs)
- Data file migration (data.rs)
- Assets migration (assets.rs)
- Documentation generation (readme.rs) 