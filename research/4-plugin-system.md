# Jekyll's Plugin System

One of Jekyll's strengths is its extensibility through plugins. Jekyll supports several **types of plugins** that hook into different parts of the static site generation pipeline. These include:

* **Generators:** Create additional content or pages at build time.
* **Converters:** Transform content from one format to another (e.g., Markdown to HTML).
* **Liquid Tags:** Custom Liquid tags (the `{% tag %}` syntax) to be used in templates/content.
* **Liquid Filters:** Custom filters for Liquid `{{ variable | filter }}` usage.
* **Commands:** Add new subcommands to the `jekyll` CLI (e.g., `jekyll somecommand`).
* **Hooks:** Register callbacks to run at certain points in the lifecycle (like after a post is rendered, before writing files, etc.).

## Installation and Loading

In Jekyll, plugins can be distributed as Ruby gems (which is common for reusable plugins) or placed as individual Ruby script files in the `_plugins/` directory for site-specific plugins. By default, Jekyll will auto-load all `.rb` files in `_plugins/` at build time (unless `plugins_dir` is changed). Gem plugins should be listed in the config (`plugins:`) or included via the Gemfile so Jekyll knows to require them. 

In safe mode (`--safe`), Jekyll will **not** load any plugins not in the "whitelist" (which historically was a config list of allowed gem names, used by GitHub Pages to permit only certain plugins).

For a Rust SSG, a direct analog might be difficult unless you implement a scripting interface or a way to load dynamic libraries. However, if the aim is full compatibility, one might consider parsing or handling at least known plugins by providing built-in support or re-implementing their functionalities.

## Generators

Generators run after the site's content is initially read but before the site is rendered. They can create additional in-memory pages, modify site data, etc. A generator is any Ruby class that inherits `Jekyll::Generator` and implements a `generate(site)` method. Within that method, the plugin has access to the entire `site` object (which includes lists of pages, posts, data, etc.). It can add new `Jekyll::Page` objects to `site.pages`, or new docs to collections, or even alter existing content. The return value of `generate` is ignored – generators are expected to have side effects on the `site`.

For example, the popular **jekyll-feed** plugin is a generator that generates an Atom feed XML page from the list of posts, adding it to `site.pages`. Another example is **jekyll-archives**, which generates category and tag archive pages.

Here's a simplified example of a custom generator from Jekyll docs that creates pages for each category:

```ruby
module SamplePlugin
  class CategoryPageGenerator < Jekyll::Generator
    safe true

    def generate(site)
      site.categories.each do |category, posts|
        site.pages << CategoryPage.new(site, category, posts)
      end
    end
  end

  # Define a custom Page subclass to represent category index pages
  class CategoryPage < Jekyll::Page
    def initialize(site, category, posts)
      @site = site
      @base = site.source
      @dir  = category    # directory for the category page (e.g., "mycategory")

      @basename = "index" # filename "index.html"
      @ext      = ".html"
      @name     = "index.html"

      # Set up page data
      @data = {
        "title" => "Posts in #{category}",
        "posts" => posts    # pass list of posts to the page
      }

      # Apply front matter defaults for collections of type 'categories'
      data.default_proc = proc do |_, key|
        site.frontmatter_defaults.find(relative_path, :categories, key)
      end
    end

    # Optional: define URL placeholders if using permissive permalinks
    def url_placeholders
      {
        :path       => @dir,
        :category   => @dir,
        :basename   => basename,
        :output_ext => output_ext,
      }
    end
  end
end
```

This generator iterates over `site.categories` (which is a hash of category => posts list) and creates a new page for each category by instantiating a `CategoryPage`. That page's content might be rendered via a layout to list the posts. Notice `safe true` – a flag indicating the plugin can run in safe mode (doesn't execute unsafe code). Also, the plugin sets up `data.default_proc` to supply default front matter (so if the config has defaults for type `categories`, it will apply).

For Rust, implementing generators means after reading all content, you run custom routines that might add or modify the content list. You'd need to replicate at least the effects of known critical generators (like pagination, feed, sitemap if those are used).

## Converters

Converters handle file content conversion, such as Markdown to HTML, or Sass to CSS. A converter in Jekyll is a class inheriting `Jekyll::Converter` that defines at least three methods: `matches(ext)` (should we use this converter for a file with extension ext?), `output_ext(ext)` (what extension to use for the output file), and `convert(content)` (the actual transformation from input text to output text). Jekyll uses these for Markdown (`.md` -> `.html`), Textile, etc., and you can add your own (e.g., for `.rst` reStructuredText files or any custom format).

An example from docs is a trivial converter that uppercases text for files ending in `.upcase`:

```ruby
module Jekyll
  class UpcaseConverter < Converter
    safe true
    priority :low

    def matches(ext)
      ext =~ /^\.upcase$/i
    end

    def output_ext(ext)
      ".html"
    end

    def convert(content)
      content.upcase
    end
  end
end
```

This tells Jekyll: for any file with extension `.upcase`, use this converter to produce an HTML file with the content uppercased. The `priority :low` means this converter runs after others with higher priority (Jekyll allows ordering of converters and generators via a `priority` flag: values `:lowest, :low, :normal, :high, :highest` – highest runs first). The Markdown converter, for example, might have `priority :lowest` since it should convert after other things, or vice versa.

Your SSG will need to incorporate the Markdown and other conversions by default. If you support plugins, you'd need to allow new converters to register. At minimum, ensure to replicate the behavior of:

* **Markdown**: Files with extensions in the `markdown_ext` list (by default `.md`, `.markdown`, etc.) should be converted to HTML using the configured Markdown engine.

* **Liquid pre-processing**: Jekyll actually first processes Liquid tags in pages, then runs converters like Markdown on the result. Except, note that assets (like Sass files with front matter) are also processed by Liquid, meaning if you have any Liquid syntax in them it would be evaluated unless wrapped in `{% raw %}` tags. It's important to follow the order: for each file with front matter, Jekyll reads content -> applies Liquid (with the variables) -> passes the output to the converter (Markdown, etc.) -> then wraps in layout. For Sass, the `jekyll-sass-converter` plugin integrates with the build.

* **Sass/SCSS**: Sass conversion is handled by the `jekyll-sass-converter` (a built-in plugin). Any file with extension `.scss` or `.sass` in your regular assets (not in `_sass`) with front matter will be compiled to CSS. The converter ensures that `@import` directives look into the `sass_dir` (defaults `_sass/`). It also enforces that files in `_sass` should **not** have front matter (if they do, Jekyll will skip processing them as Sass partials). Configuration like `sass: style: compressed` can be set to control output style. All these details imply the SSG's Sass handling should mimic Jekyll's: skip `_sass` files (only use them as imports), compile SCSS with a library equivalent to Ruby Sass (Jekyll uses the SassC library via the plugin), and allow config of Sass output style.

The plugin system flags `safe` and `priority` shown above should be noted:

* **`safe` flag:** If a plugin class calls `safe true`, it declares it's safe to run in restricted mode (no arbitrary code execution). GitHub Pages only runs a curated set of plugins regardless, but if one of those had `safe false` it wouldn't run on GH Pages. For your SSG, if you have a concept of safe mode, you could respect this flag.
* **`priority` flag:** Determines ordering. Jekyll processes all converters in order of priority (highest first) for each file. Similarly, generators are ordered. Typically core converters have priorities set such that, e.g., Sass is high (to generate CSS early?), or others normal. For compatibility, ensure that if multiple converters could match, the one with highest priority wins.

## Liquid Tags (Custom Tags)

Jekyll's markdown and HTML content is processed by the Liquid templating engine (see Section 5). Users (or plugins) can extend Liquid with new tags – for example, a `{% youtube ... %}` tag to embed a YouTube video, or `{% mermaid %}` to render a diagram. A Liquid tag plugin in Ruby is usually created by subclassing `Liquid::Tag` and implementing an `render(context)` method. Jekyll will automatically register any subclasses of `Liquid::Tag` that are loaded. For instance, a plugin might do:

```ruby
module Jekyll
  class YoutubeTag < Liquid::Tag
    def initialize(tag_name, id, tokens)
      super
      @video_id = id.strip
    end
    def render(context)
      %Q{<iframe src="https://www.youtube.com/embed/#{@video_id}"></iframe>}
    end
  end
end

Liquid::Template.register_tag('youtube', Jekyll::YoutubeTag)
```

This would enable users to write `{% youtube dQw4w9WgXcQ %}` in content. Your SSG to be fully compatible would need to either implement some common custom tags or provide an interface to define tags. At minimum, be aware of core Jekyll tags that are **not** part of base Liquid:

* **Include tag:** `{% include file.ext %}` and `{% include_relative path %}` are implemented by Jekyll (not default Liquid). They allow including partial files from `_includes` (or relative to current file). Your engine needs to handle these tags: find the file, read it, and render it in place (with its own scope, accessible via an `include` variable for passed parameters).

* **Highlight tag:** `{% highlight LANG %} code {% endhighlight %}` – Jekyll provides this for code highlighting using Rouge (if user doesn't use triple backticks). It's basically a wrapper that outputs `<pre><code class="language-LANG">...</code></pre>` with proper escaping. This tag should be supported if you aim to handle all Jekyll sites (though many use Markdown fenced code instead).

* **Link and PostURL tags:** `{% link path/to/file.md %}` and `{% post_url 2019-01-01-post-title %}` – these are Jekyll tags to link to other pages/posts by their relative path or post slug. They ensure if URLs change (by permalink structure) the links remain correct. Implementing these requires looking up the target page in the site and outputting its URL. E.g. `{% post_url 2009-04-26-barcamp-boston-4-roundup %}` should produce the URL to that post (this requires knowing the post's generated URL from its front matter/permalink).

* **Asset tag (in some plugins):** Not built-in to Jekyll core, but some themes/plugins add tags like `{% asset_path something %}` or `{% image %}` – these would only need support if those plugins are to be mimicked.

* **Others:** Jekyll also has tags like `include_cached` (from jekyll-include-cache plugin), `render` (for rendering layouts within Markdown, introduced via jekyll-compose maybe), etc. Full compatibility implies covering these.

If a Rust SSG cannot dynamically load new tag code, one approach is to predefine widely-used tags (include, link, post\_url, highlight, etc.) and possibly allow some configurable shortcodes.

## Liquid Filters (Custom Filters)

Similar to tags, filters extend Liquid's output transformations. Jekyll adds many filters of its own on top of base Liquid. For example: `relative_url`, `absolute_url` (to prepend baseurl/site url); date filters like `date_to_xmlschema`, `date_to_rfc822`, `date_to_string` (these format dates in various ways); array filters like `where`, `where_exp` (to filter an array of objects by an expression), `group_by` and `group_by_exp` (to group items by a property or expression), `sort` and `sort_natural` (to sort an array by some key, alphabetically or naturally), `markdownify` (to render a Markdown string as HTML within a template), `inspect` (to output a debug string representation), `slugify` (turn a string into a URL-friendly slug), `jsonify` (convert an object to JSON string) – and more. The Jekyll documentation enumerates these Jekyll-specific filters. Your SSG should implement these filters in the template engine to achieve the same output. For instance:

* **Where filter:** `{{ site.posts | where:"category","news" }}` returns an array of posts whose `category` property equals "news". `where_exp` is more powerful: `{{ site.members | where_exp:"m","m.age < 30" }}` can use a Liquid expression to filter. This requires parsing the expression and applying it to each item.
* **Date filters:** If using a Rust date library, ensure the formatting (RFC822, etc.) matches Ruby's output (e.g., month abbreviations, time zone formats).
* **URL filters:** `relative_url` simply prepends the configured baseurl, whereas `absolute_url` prepends the full site URL. Make sure to handle edge cases like double slashes.
* **Misc:** `markdownify` should run the Markdown converter on a string (useful if a page's excerpt or data from a data file contains Markdown that needs rendering). `slugify` should replicate Jekyll's slugification (which lowercases and replaces spaces with `-`, and strips certain characters, with mode options if given).

If full plugin support is desired, you would allow new filter definitions. But an easier route is implementing the known Jekyll filters so that most sites (which rarely define their own filters) work out of the box.

## Plugin Hooks (Lifecycle Events)

Jekyll exposes a hook API where plugins can register to run code at specific events: e.g., after a site is initialized, before a page is rendered, after a post is written, etc.. This is done via `Jekyll::Hooks.register <owner>, <event> do |obj| ... end` in Ruby. Hook owners include `:site`, `:pages`, `:posts` (for the posts collection specifically), `:documents` (all collections including posts), and `:clean` (for cleanup of old files). Events include:

* `:after_init` (site just initialized),
* `:post_read` (after all files are read from disk),
* `:pre_render` (just before rendering a page/post),
* `:post_convert` (after content is converted, before layout rendering),
* `:post_render` (after the page/post is fully rendered, before writing to disk),
* `:post_write` (after the file is written to disk),
* For site as a whole, `:pre_render` and `:post_render` events pass in the entire site payload (all data),
* For cleaning, `:on_obsolete` event triggers for each obsolete file about to be deleted.

Plugins like **jemoji** (GitHub emoji support) use hooks: jemoji registers a `:posts, :post_render` hook to replace shortcode `:smile:` with actual emoji images after posts are rendered. Your SSG would need to either incorporate similar functionality directly or allow a way to register such transformations. It's complex to design a full hook system in Rust, but for compatibility, identify critical behaviors implemented via hooks and include them. For example:

* The **jekyll-mentions** plugin (for `@user` to GitHub profile links) likely uses a hook to transform text post-render.
* **jekyll-readme-index** uses a hook to possibly treat README.md as index.
* **jekyll-redirect-from** might use a hook post\_write to generate stub HTML files for redirects.

If not implementing a dynamic hook system, you might hardcode or mimic these specific plugin behaviors when their config is present (especially since GitHub Pages supports certain plugins – see Section 9 – you might want to support those in particular).

## Commands

Jekyll plugins can add new CLI commands (subclasses of `Jekyll::Command`). For example, **jekyll-compose** adds `jekyll draft` and `jekyll post` commands to create new posts. While not affecting site generation output, if you aim to replace Jekyll entirely, you might implement similar utility commands for a better developer experience (but they are not strictly necessary for generating the site correctly).

## Plugin Priority in Build Process

1. **Initialization:** Jekyll loads config, then requires all plugins, then reads files.
2. **Reading phase:** Jekyll reads all pages, posts, data, static files into memory. During this, **converters** may be consulted for handling certain files (e.g., Front matter and excerpt extraction rely on knowing markdown extensions, etc.).
3. **Post-read hooks:** After reading, any `:post_read` hooks run.
4. **Generator phase:** All `Jekyll::Generator` plugins run their `generate` methods, potentially adding/altering content.
5. **Pre-render hooks:** Right before rendering each item, run `:pre_render` hooks (both global and per-page). Then:
6. **Conversion & Rendering:** For each page/post:

   * Determine layout (from front matter or defaults).
   * Pass content through converters (e.g., Markdown to HTML) – at this point, the `:post_convert` hook fires.
   * Then wrap with layout (Liquid rendering occurs, including resolving includes and inserting content). After the final HTML is ready, `:post_render` hooks fire for that page/post.
7. **Site post\_render hook:** After all pages are processed, a hook `:site, :post_render` fires with the whole site payload (for sitemap or feed generation perhaps).
8. **Writing files:** Output files are written to disk in `_site`. After each write, relevant `:post_write` hooks fire. Then a `:site, :post_write` could fire after all are written (though not explicitly listed, site has only certain events).
9. **Cleanup:** If removing old files, the `:clean, :on_obsolete` hook triggers for each file to be deleted.

An SSG following this flow will produce the same results as Jekyll. For example, the pagination generator (if present) would have run in the generator step to create pagination pages *before* rendering. Or a hook like `:pre_render` could inject a variable for use during rendering.

## Example – How Plugins Tie Together

The **jekyll-paginate** plugin (v1) is technically a generator that reads `site.posts` and creates paginator pages, but it also relies on Jekyll core handling of the `paginator` variable in templates. **jekyll-paginate-v2** is more advanced (and not supported on GH Pages). The **jekyll-sitemap** plugin registers as a generator that creates a `sitemap.xml` page after all posts/pages are known. The **jekyll-seo-tag** is a Liquid tag plugin (provides `{% seo %}` tag to insert SEO meta tags based on page variables). The **jekyll-github-metadata** plugin hooks into site init to set `site.github` data (like repository name, etc.). These illustrate the variety: as the Rust SSG author, you might implement these specific plugins' functionality natively for compatibility (especially the ones GitHub Pages uses, see Section 9).

## Safe Mode and Whitelisting

In safe mode, Jekyll will **not** load plugins from `_plugins` and will only run the gem plugins that are considered safe (the set included in the GitHub Pages gem). It also disables caching to disk and disallows symlinks for security. If your SSG has a safe flag, it should skip custom plugin execution. For instance, on GitHub Pages, if a user has any custom Ruby in `_plugins`, it's ignored.

## Implementation Summary

Full Jekyll compatibility in plugins is a large undertaking. If targeting that, you'd have to implement a mini plugin architecture or bake in support for popular plugins. Prioritize matching output: e.g., ensure pagination, SEO tags, sitemaps, feeds, and other common features are present (either via config or automatic). Possibly provide a way for advanced users to write their own extensions in Rust or via scripts, but that's beyond Jekyll's own scope (Jekyll is Ruby, so plugins are Ruby). At minimum, implement:

* All core behavior that Jekyll core or default plugins cover (as described above).
* Support for theme gems (loading includes/layouts and maybe assets from them).
* The same Liquid tags/filters as Jekyll (so theme templates don't break).

Finally, note the plugin flags mentioned earlier:

* The `priority` and `safe` flags on plugin classes.
* Your SSG's build process should respect the relative order of operations that Jekyll uses for any equivalent plugin implementations.

By matching the plugin system's effects, a site with custom plugins might still need adjustments to run on a Rust SSG, but any site using the officially supported plugins (or none) would generate correctly. 