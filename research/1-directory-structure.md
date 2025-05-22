# Jekyll Directory Structure and Theming

A standard Jekyll project follows a specific directory structure. At minimum, a Jekyll site contains a configuration file (`_config.yml`) and special folders prefixed with an underscore. For example, a basic Jekyll site might look like this:

```plaintext
.
├── _config.yml
├── _data/
│   └── members.yml
├── _drafts/
│   ├── begin-with-the-crazy-ideas.md
│   └── on-simplicity-in-technology.md
├── _includes/
│   ├── footer.html
│   └── header.html
├── _layouts/
│   ├── default.html
│   └── post.html
├── _posts/
│   ├── 2007-10-29-why-every-programmer-should-play-nethack.md
│   └── 2009-04-26-barcamp-boston-4-roundup.md
├── _sass/
│   ├── _base.scss
│   └── _layout.scss
├── _site/
├── .jekyll-cache/
│   └── Jekyll/Cache/[...]
├── .jekyll-metadata
└── index.html  (or index.md with front matter)
```

## Top-level elements

* **`_config.yml`:** Main configuration file for site-wide settings.
* **`_posts/`:** Blog posts, named with the date prefix `YYYY-MM-DD-title.md`. Jekyll treats files here specially: it parses the date and title from the filename and generates each post page accordingly. Posts are sorted by date and accessible via site variables.
* **`index.html` (or `index.md`):** The homepage (or any page) with YAML front matter is processed by Jekyll. Pages can live at the root or in any non-underscored directory. Pages use layouts and can utilize the Liquid template system like posts.

## Special directories (all prefixed with `_`)

* **`_drafts/`:** Unpublished draft posts (no date in filename). Jekyll will only process these if building with the `--drafts` flag. Drafts do not appear in site output unless explicitly included.
* **`_layouts/`:** Layout templates that wrap pages/posts. Layout files are referenced by name (sans extension) in a page's front matter (e.g. `layout: post`). The layout file must reside in this folder (or in a theme). Layouts typically inject the page's rendered content via the `{{ content }}` tag.
* **`_includes/`:** Reusable partial snippets (HTML/Liquid) that can be inserted into layouts or pages. Includes are injected with the Liquid `{% include %}` tag, referencing a file in this directory. For example, `{% include footer.html %}` will include `_includes/footer.html`.
* **`_sass/`:** Sass partials for stylesheets. Files in `_sass` (e.g. `_base.scss`) are meant to be imported into your main Sass file. The main SCSS files (with front matter) live outside this folder (commonly in an `assets` or `css` directory) and import the partials. Jekyll config allows setting a custom `sass_dir` (defaults to `_sass`). Sass partials **should not** have front matter; only the primary Sass files (which output to CSS) include the YAML triple-dashes at the top to trigger processing.
* **`_data/`:** Data files (YAML, JSON, CSV, TSV) for site-wide data. Jekyll auto-loads these so they can be accessed via the Liquid `site.data` namespace. For example, `_data/members.yml` is exposed as `site.data.members` in templates. Data files allow storing structured content (like lists of team members, site settings, etc.) that can be looped over in pages without writing them in front matter repeatedly. Subfolders under `_data` become nested keys (e.g. `_data/clients/list.yml` is accessed as `site.data.clients.list`).
* **`_site/`:** The output directory where the static site is generated. Jekyll writes processed files here (HTML, CSS, JS, etc.). By default, Jekyll will **clean** this folder on each build (deleting old files), so it's recommended to never edit `_site` manually and to add it to your `.gitignore`. You can change the output directory via the `destination` setting in config or CLI flag.
* **`.jekyll-cache/` and `.jekyll-metadata`:** Auxiliary files for performance. `.jekyll-cache` stores cached content (like parsed templates) to speed up incremental serving. `.jekyll-metadata` tracks file modification times for **incremental builds**. These are created during `jekyll serve` or builds with incremental mode, and they should be git-ignored. They are not included in the generated site.

## Static files and assets

Any files and folders **not** starting with `_`, `.` (dot), `#`, or `~` are treated as static files and are copied to `_site` without modification (except those excluded by config). For example, an `assets/` or `images/` folder, or any `.css`/`.js`/`.jpg` files in the project root, will be copied over verbatim. 

One caveat: Jekyll will ignore files/folders beginning with `_` (or the other special characters) unless you explicitly whitelist them in the config via the `include:` setting. For instance, to include a folder `_special` or a dotfile like `.htaccess` in the site output, you can add:

```yaml
include:
  - _special
  - .htaccess
```

in your `_config.yml`.

## Gem-based themes

Modern Jekyll sites (since v3.2) often use gem-based themes (like the default **Minima** theme). When a theme gem is used, certain directories (layouts, includes, sass, assets) may not exist in the site folder because they are provided by the theme gem. For example, a fresh `jekyll new` site using Minima will not have a `_layouts/` directory locally – the layouts are in the gem. 

You can override any theme file by creating a file of the same path in your project. For instance, if the theme provides `/_layouts/default.html`, adding your own `_layouts/default.html` will supersede the theme's version. Gem-based themes lighten repository clutter, but your SSG must be able to **load theme-provided files**. This means searching the theme gem's directories if a requested layout/include isn't found in the user's site.

Jekyll's `theme` config option or the `Gemfile` reference to the theme gem dictates which theme to use. In a compatible SSG, implementing theme support requires reading the theme gem contents (which are basically another Jekyll project scaffold) and merging them with the user's site content.

## Implementation Notes

Your Rust SSG should treat these special directories and files exactly as Jekyll does – processing Markdown/HTML with front matter, copying static assets, supporting an optional theme overlay, and ignoring or including files according to the same rules. A correct directory handling ensures that content is correctly recognized (e.g., only Markdown files **with** front matter are transformed) and that the output in `_site` is structured the same way Jekyll would produce it. 