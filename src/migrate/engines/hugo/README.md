# Hugo Migration Engine for Rustyll

This module migrates Hugo static sites to the Rustyll static site generator format.

## Features

* Detects Hugo sites by looking for config files (`config.toml`, `config.yaml`, `config.json`) or the presence of both content/ and themes/ directories
* Converts configuration from Hugo's TOML/YAML/JSON to Rustyll's YAML format
* Extracts site parameters and menu structure to separate data files
* Migrates content with proper front matter conversion
* Processes blog posts with date-based filenames
* Converts Hugo shortcodes to Liquid tags
* Transforms Go templates to Liquid templates
* Converts Hugo partials to Rustyll includes
* Migrates static assets to appropriate locations

## Key Components

* `config.rs`: Converts Hugo config files to Rustyll's _config.yml
* `content.rs`: Migrates content with front matter and shortcode conversion
* `layouts.rs`: Transforms Hugo's Go templates to Liquid templates
* `partials.rs`: Converts Hugo partials to Rustyll includes
* `static_assets.rs`: Processes static assets
* `data.rs`: Handles data file conversion
* `readme.rs`: Generates documentation

## Conversion Details

### Configuration
- Hugo's configuration in TOML/YAML/JSON is converted to Rustyll's YAML format
- Site params are extracted to `_data/params.yml`
- Menu configuration is extracted to `_data/menu.yml`

### Content
- Blog posts are converted to the `_posts` directory with Jekyll-style date prefixes
- Regular content is converted to the `_pages` collection
- Front matter is preserved and normalized
- Hugo shortcodes are converted to Liquid tags and HTML where possible

### Templates
- Hugo's Go templates are converted to Liquid syntax
- `{{ .Title }}` becomes `{{ page.title }}`
- `{{ .Site.Title }}` becomes `{{ site.title }}`
- `{{ partial "header.html" . }}` becomes `{% include "header.html" %}`
- `{{ block "main" . }}{{ end }}` becomes `{{ content }}`

### Static Assets
- Static files are copied to appropriate directories in the Rustyll structure

## Known Limitations

- Hugo's advanced Go template features may require manual adjustment
- Some complex shortcodes may need manual conversion
- Taxonomy handling differs between Hugo and Rustyll
- Custom output formats in Hugo need manual implementation in Rustyll 