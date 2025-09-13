# Configuration in Rustyll

Rustyll's configuration system is designed to be both Jekyll-compatible and Rust-performant. It supports YAML (`_config.yml`) and TOML (`_config.toml`) formats, with the same configuration keys and command-line flags as Jekyll, while adding Rust-specific optimizations and features.

## Core Configuration System

### File Formats and Loading

Rustyll supports both YAML and TOML configuration formats:

```yaml
# _config.yml example
title: "My Site"
baseurl: "/blog"
```

```toml
# _config.toml example
title = "My Site"
baseurl = "/blog"
```

Configuration is loaded once at startup by default. Changes require a restart, except for data files which are reloaded during `serve` with auto-regeneration. The loading process:

1. Load default configuration
2. Merge theme configuration (if a theme is used and not disabled)
3. Merge site's `_config.yml` or `_config.toml`
4. Apply command-line overrides

For multiple config files, use:
```bash
rustyll build --config _config.yml,_config_dev.yml
```

Later files override settings in earlier ones.

### Site Directories

These configuration options control where Rustyll looks for content:

```yaml
# Core directories
source: .                # Where to find site content
destination: _site       # Where to output the site
plugins_dir: _plugins    # Plugin directory
layouts_dir: _layouts    # Layout templates directory
data_dir: _data          # Data files directory
includes_dir: _includes  # Partial includes directory
collections_dir: .       # Root directory for all collections 

# Collection directories are relative to collections_dir
```

All directory paths are resolved relative to the site root, not the config file location. Rustyll implements these with Rust's `std::path` for cross-platform compatibility.

### Include/Exclude Patterns

Control which files are processed:

```yaml
# Files/patterns to exclude from processing
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor/bundle/
  - vendor/cache/
  - vendor/gems/
  - vendor/ruby/
  - .git/
  - .github/
  - .gitignore
  
# Force inclusion of files that would otherwise be excluded
include:
  - .htaccess
  - .well-known
```

Rustyll implements these patterns using Rust's `globset` crate, offering faster matching than Ruby's `Dir.glob`. By optimizing the glob compilation, Rustyll achieves significant performance improvements for large sites with many exclude/include patterns.

### URL Configuration

```yaml
url: "https://example.com"   # The base hostname & protocol
baseurl: "/blog"           # The subpath of your site, e.g. /blog
permalink: date            # Posts URL format (date, pretty, ordinal, none)
                          # or a custom template like /:categories/:year/:month/:day/:title.html
```

Unlike Jekyll, Rustyll uses a custom URL parser optimized for static site generation, avoiding the overhead of Ruby's URI parsing for better performance.

## Build Settings

### Core Build Options

```yaml
# Build control options
safe: false               # Disable plugins and ignore symbolic links
strict_front_matter: false # Fail on invalid front matter
liquid:
  strict_filters: false   # Strict error checking for Liquid filters
  strict_variables: false # Strict error checking for Liquid variables
encoding: utf-8           # Default encoding for files
future: false             # Show future-dated posts
unpublished: false        # Render posts marked as unpublished
watch: false              # Watch for changes (default: true in serve)
show_drafts: false        # Process drafts
limit_posts: 0            # Limit the number of posts to parse
incremental: false        # Only re-generate changed files
```

While Jekyll performs these checks at runtime, Rustyll implements many validations during the initial parsing phase, reducing the need for multiple passes and improving build times.

### Markdown Processing

```yaml
markdown: kramdown       # Markdown processor to use
markdown_ext: "markdown,mkdown,mkdn,mkd,md" # Markdown file extensions
kramdown:                # Kramdown-specific options
  input: GFM
  auto_ids: true
  hard_wrap: false
  syntax_highlighter: rouge
```

Rustyll implements Markdown processing using Rust crates like `pulldown-cmark` and `syntect` for syntax highlighting, offering up to 50x faster Markdown rendering than Ruby-based kramdown. It maintains compatibility with the same configuration options while benefiting from Rust's performance.

### Plugin System

```yaml
plugins:                  # List of plugins to load
  - jekyll-feed
  - jekyll-seo-tag
whitelist:                # Plugins allowed in safe mode
  - jekyll-feed
```

Rustyll's plugin system is reimplemented in Rust with two key components:

1. **Built-in compatible plugins**: Native Rust implementations of common Jekyll plugins
2. **Plugin API**: A Rust API for building custom plugins with Rustyll-specific features

Unlike Jekyll's Ruby-based plugins that run in the same process, Rustyll's plugin architecture can isolate plugins for better stability and security when needed.

### Theme Configuration

```yaml
theme: minima            # Theme to use
ignore_theme_config: false # Whether to ignore theme's _config.yml
```

Rustyll handles themes differently than Jekyll:

1. Themes are packaged as `.tar.gz` or directory references
2. Rust's memory-mapped I/O enables faster theme asset loading
3. A theme cache minimizes repeated file operations

## Rustyll Performance Optimizations

Rustyll adds performance-focused configuration options:

```yaml
# Rustyll-specific performance options
performance:
  threads: auto          # Thread count for parallelism (auto or specific number)
  cache:
    enabled: true        # Enable build caching
    strategy: smart      # Cache strategy: minimal, normal, aggressive
    directory: .rustyll-cache # Cache directory location
  memory_limit: 1024     # Memory usage limit in MB (0 for unlimited)
  profiling: false       # Enable performance profiling
```

### Multi-threading Model

Unlike Jekyll's primarily single-threaded operation, Rustyll implements a work-stealing thread pool using Rust's `rayon` crate. This enables parallel processing of:

- Markdown rendering
- Template parsing
- Asset transformation
- File I/O operations

The `threads` option lets users control the thread count based on their system's capabilities.

### Smart Caching System

Rustyll's caching system is more sophisticated than Jekyll's `.jekyll-metadata`:

1. **Content hashing**: Track content changes, not just file modification times
2. **Dependency tracking**: Build a graph of template dependencies to only rebuild affected files
3. **Persistent caching**: Store rendered partials, templates, and intermediate data
4. **Memory caching**: Keep frequently used objects in memory for faster access

This results in incremental builds that are often 10-50x faster than Jekyll's.

## Asset Processing

Rustyll extends Jekyll's basic asset handling with integrated processing:

```yaml
# Asset pipeline configuration
assets:
  compression: true       # Enable asset compression
  cache_busting: true     # Add content hash to static assets
  sources:                # Asset source directories
    - _assets/images
    - _assets/javascripts
    - _assets/stylesheets
  destination:            # Output directories
    images: assets/images
    javascripts: assets/js
    stylesheets: assets/css
  js:
    bundle: true         # Bundle JS files
    minify: true         # Minify JS files
  css:
    bundle: true         # Bundle CSS files
    minify: true         # Minify CSS files
  images:
    optimize: true       # Optimize images
    formats:
      webp: true         # Generate WebP versions
      avif: true         # Generate AVIF versions
```

Rustyll's asset pipeline uses native Rust libraries for image processing, minification, and transpilation, avoiding the need for external tools or Node.js dependencies.

## Development Server

```yaml
# Development server settings
server:
  port: 4000             # Server port
  host: 127.0.0.1        # Server host
  livereload: true       # Enable live reload
  livereload_port: 35729 # Live reload port
  open_url: false        # Open browser automatically
  show_dir_listing: false # Show directory listing for missing index pages
  error_pages:           # Custom error pages
    404: 404.html
  headers:               # Custom HTTP headers
    "Access-Control-Allow-Origin": "*"
  quiet: false           # Silence output
  verbose: false         # Verbose output
```

Rustyll's development server is built on Rust's async I/O libraries, providing faster response times and lower resource usage than Jekyll's WEBrick-based server.

## Advanced Features

### Collections Configuration

```yaml
# Collections configuration
collections:
  projects:               # Collection name
    output: true          # Generate individual pages
    sort_by: date         # Field to sort by
    permalink: /projects/:path/ # URL template
    paginate: 5           # Enable pagination
    pagination_path: /projects/page:num/ # Pagination path pattern
    filter:               # Filter collection items
      featured: true      # Only include featured items
```

Rustyll's collections implementation uses optimized data structures for faster lookup and rendering, with a modular design allowing for type-safe collection access.

### Advanced Pagination

```yaml
# Pagination settings
paginate: 5               # Posts per page
paginate_path: /page:num/ # Pagination URL path
pagination:
  enabled: true          # Enable advanced pagination
  per_page: 5            # Items per page
  title: ':title - page :num' # Page title format
  sort_field: 'date'     # Field to sort by
  sort_reverse: true     # Reverse sort order
  trail:                 # Pagination navigation trail
    before: 2            # Links before current page
    after: 2             # Links after current page
  extension: html        # Output file extension
```

Rustyll's pagination engine efficiently pre-computes pagination metadata during the site loading phase, reducing overhead during template rendering.

## Migration from Jekyll

Rustyll is designed as a drop-in replacement for Jekyll, but users should be aware of these differences:

1. **Configuration Loading**: Rustyll loads configuration slightly faster and supports TOML natively
2. **Plugin Behavior**: While API-compatible, plugins are implemented in Rust
3. **Performance Options**: Additional configuration for multi-threading and caching
4. **Error Handling**: More detailed error messages with suggestions

To migrate from Jekyll to Rustyll:

1. Install Rustyll: `cargo install rustyll`
2. Run in the same directory: `rustyll build` or `rustyll serve`
3. Optionally add Rustyll-specific optimizations to your config

## Command-line Interface

Rustyll maintains the same CLI flags as Jekyll for compatibility:

```
rustyll build|b : Build your site
  -s, --source SOURCE            Source directory (default: .)
  -d, --destination DESTINATION  Destination directory (default: ./_site)
  --config CONFIG_FILE[,...]     Configuration file (default: _config.yml)
  --future                       Publish posts with a future date
  --unpublished                  Render posts that were marked as unpublished
  --drafts                       Render posts in the _drafts folder
  --lsi                          Use LSI for improved related posts
  -q, --quiet                    Silence output
  -V, --verbose                  Print verbose output
  --trace                        Show full backtrace on errors
  --profile                      Generate timing report

rustyll serve|s : Serve your site locally
  (includes all build options plus:)
  -H, --host HOST                Host to bind to (default: 127.0.0.1)
  -P, --port PORT                Port to listen on (default: 4000)
  -l, --livereload               Enable LiveReload
  --open-url                     Launch your site in a browser
  --detach                       Detach the server from the terminal
  --watch|--no-watch             Enable/disable auto-regeneration
  -i, --incremental              Rebuild only modified posts and pages
```

Rustyll adds some additional flags for its advanced features:

```
  --threads NUM                  Set number of threads for parallel processing
  --cache-dir PATH               Set cache directory
  --disable-cache                Disable the build cache
  --strict                       Strict mode (extra validation)
```

This implementation creates a familiar environment for Jekyll users while offering substantial performance improvements. 