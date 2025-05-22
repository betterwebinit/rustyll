# Configuration (`_config.yml` and Command-Line Flags)

Jekyll's configuration system is flexible. Users can set most options in a `_config.yml` file (written in YAML; `_config.toml` is also supported as an alternative), and/or override or supplement these via command-line flags when running `jekyll build` or `jekyll serve`. To be Jekyll-compatible, the Rust SSG must support the same configuration keys and CLI flags, producing the same effects.

## Configuration File: `_config.yml`

This file (by default in the site root) contains site settings, default values, and plugin/theme configs. Jekyll loads this once at startup (changes to it during `serve` won't take effect until restart). Common settings include site metadata (title, description, etc.), as well as build directives. Notable configuration options and their default values are:

### Site Source and Destination

By default, Jekyll considers the current directory as the source (where your content is) and writes the site to `./_site`. These can be changed with `source: <dir>` and `destination: <dir>` in config or via `--source` (`-s`) and `--destination` (`-d`) CLI flags. For example, `jekyll build -d public_html` would output to `public_html` instead of `_site`. If your SSG offers similar flags, they should mirror this behavior.

### Directory Names

Other special directories can be configured:

* `plugins_dir`: which directory Jekyll will load plugins from (default `_plugins`). CLI flag: `-p` or `--plugins DIR1,DIR2,...` to specify one or more plugin directories.
* `layouts_dir`: default `_layouts`, can be set to a custom path (flag `--layouts DIR`).
* `data_dir`: default `_data`.
* `includes_dir`: default `_includes`.
* `collections_dir`: by default, collections (including posts and drafts) are at project root. Jekyll lets you set `collections_dir: <folder>` to gather all collections under a subdirectory. For example, if `collections_dir: my_collections`, then Jekyll will expect `_posts` to be in `my_collections/_posts`, etc.. Your SSG should honor this if implemented (noting to adjust where it looks for `_posts`, `_drafts`, and other `_<collection>` folders).

### Include/Exclude Patterns

`exclude` can list files or glob patterns to exclude from processing. By default, Jekyll excludes certain files like node\_modules, Gemfile, etc. (the default list is shown when you do `jekyll new` and is merged with any additional excludes). Similarly, `include` can whitelist otherwise ignored files (like dotfiles or underscore-prefixed files). The SSG should allow similar patterns (likely using glob matching) to filter input files.

### Site URL and Base URL

`url` is the base URL of the site (e.g. `https://example.com`) and `baseurl` is a path subdirectory if the site is published in a subfolder of the domain (e.g. `/blog`). These are used by certain Liquid filters (`absolute_url` and `relative_url`) and by plugins like sitemap or feed to construct full URLs. In your SSG, you'll want to store these and ensure filters or any link generation logic uses them consistently.

### Build Settings

* `safe`: if true, disables user plugins (only whitelisted built-in plugins allowed) and disallows custom Ruby code execution. GitHub Pages builds with `safe: true` by default (only certain plugins can run). For Rust SSG, "safe mode" might not be as relevant unless you allow user scripting, but for full compatibility, you might include a safe mode that, for example, ignores custom plugins or shell commands in templates.

* `watch`: enable file watching to rebuild on changes (default off in build, on in `serve`). Controlled by CLI `-w/--watch`.

* `incremental`: enable incremental regeneration (default false). This creates the `.jekyll-metadata` and only rebuilds files that changed or depend on changed files. It's an experimental feature in Jekyll (can break in some cases), but dramatically speeds up rebuilds for large sites. If implementing, you would track dependency graph of pages to know what needs updating. At least, support the `--incremental` flag to mimic Jekyll, even if the actual mechanism differs.

* `future`: by default, Jekyll does **not** publish posts dated in the future. Setting `future: true` (or using `--future` flag) will include future-dated posts in the build. Your SSG should filter out posts whose `date` is greater than now unless this is true.

* `unpublished`: similarly, Jekyll excludes any post with `published: false` in front matter. `unpublished: true` (or `--unpublished` CLI) will include them in the build (useful for previewing).

* `show_drafts`: defaults to false; if true or `--drafts` is passed, it will process files in `_drafts/` as if they were posts (assigning them a date of build time). Implement by reading `_drafts` only when this flag is on.

* `limit_posts`: if set to an integer N, or `--limit_posts N` used, Jekyll will only process the first N posts (after sorting by date). This is mostly for debugging or generation speed. For compatibility, you can support this by slicing the posts list.

* `lsi`: if true/`--lsi`, use the classifier-reborn plugin to generate related posts with Latent Semantic Indexing. GitHub Pages doesn't support `lsi` and it requires an extra gem, so many avoid it. If not implementing LSI, ensure that setting is ignored or documented.

* `verbose`/`quiet`: `--verbose` increases logging (Jekyll prints more info on file reading, regeneration); `--quiet` silences normal output. These are about console output and not the site content, but your SSG could offer similar flags for user experience.

* `profile`: `--profile` generates a Liquid rendering profile report to help optimize templates. You likely don't need to replicate this unless you want debugging tools; it doesn't affect the site output.

* `trace`: `--trace` shows full stack traces on errors (useful for debugging). Again, not content-related, but can be offered.

### Markdown & Rendering Settings

`markdown: kramdown` (the default Markdown engine) or other options (see Section 6 for details). `highlighter: rouge` is default for code syntax highlighting. These can be configured or changed by user. The SSG should either integrate the default (kramdown-like CommonMark with GFM support and Rouge for highlighting) or at least parse with equivalent features if using a Rust library.

### Plugins

In config, `plugins:` (or the deprecated `gems:`) can list gem names of plugins to use. Jekyll auto-requires those gems on build. For a Rust SSG, this might not apply directly unless you implement a plugin system (see Section 4) with a concept of external modules. But to mimic Jekyll, you may still support a `plugins_dir` (for loading custom plugin files from a folder) and a means to specify which plugins are enabled. 

Jekyll also has a `whitelist` (in older versions) or allowed list for plugins in safe mode – if safe mode is on, only plugins named in `whitelist` run. Additionally, `ignore_theme_config: true` is a newer option (Jekyll 4.0+) to ignore a theme gem's `_config.yml` if it causes conflicts. The Rust SSG should decide how to handle theme configurations (for compatibility, probably merge theme's config by default, unless an equivalent flag is set to ignore it).

## Default Configuration

Jekyll's **default configuration** provides a baseline for many of these values (which is useful as a reference). For example, by default: `safe: false`, `include: [".htaccess"]`, `exclude: ["Gemfile", "node_modules", etc.]`, `keep_files: [".git", ".svn"]`, `encoding: "utf-8"`, `markdown_ext: "markdown,mkdown,mkdn,mkd,md"` (all file extensions that will be treated as Markdown), `strict_front_matter: false`, `show_drafts: null` (meaning false unless `--drafts` is set), `future: false`, `unpublished: false`, `markdown: kramdown`, `highlighter: rouge`, `permalink: date` (which corresponds to `/:categories/:year/:month/:day/:title.html` by default), `paginate_path: /page:num`, etc.

Your SSG should implement similar defaults and then override with user config values. Notably, if multiple config files are provided (Jekyll supports `--config file1,file2` to merge configs), later files override earlier ones.

## Command-line Flags Summary

Ensure to support (with Jekyll-equivalent names and behavior):

* `-s, --source DIR` and `-d, --destination DIR` to set source/output directories.
* `--config FILE1,FILE2,...` to specify config files (and ignore the default `_config.yml` unless it's included explicitly in the list).
* `-w, --watch/--no-watch` to toggle watching (file system monitoring for changes).
* `-D, --drafts` to include drafts.
* `--future`, `--unpublished` to include future posts and unpublished posts.
* `--limit_posts N` to limit processing to N posts.
* `-I, --incremental` for incremental build mode.
* `--safe` to enable safe mode (no custom plugins, no symbolic links, etc.).
* `--profile`, `-t/--trace`, `-V/--verbose`, `-q/--quiet` for the logging and debug options.
* `-H, --host` and `-P, --port` for the development server settings (if you implement a dev server like `jekyll serve`). Also `-l, --livereload` and related livereload options, which Jekyll added in v4 for auto-refresh in browser.

By matching these config and flag options, a user should be able to take an existing Jekyll site's config and use it with your SSG without changes to get an equivalent result. 