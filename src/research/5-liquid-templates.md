# Liquid Templates in Jekyll

Jekyll uses the **Liquid** templating language (originated from Shopify) to empower its themes and content. Liquid is the layer that allows **variable interpolation** (`{{ ... }}`), **logic** (`{% if ... %}`), **loops** (`{% for ... %}`), and inclusion of dynamic content in an otherwise static site. To be Jekyll-compatible, the Rust SSG must integrate a templating engine that behaves like Liquid – including all Jekyll's additions.

## Basic Liquid Syntax

In Jekyll (Liquid), double curly braces output a variable's value, e.g. `{{ page.title }}` prints the page's title. Curly braces with percent signs denote tags that perform logic or control flow, e.g. `{% if user %}Hello {{ user.name }}{% endif %}`, or `{% for post in site.posts %}...{% endfor %}`. Comments in Liquid are `{% comment %} this will be ignored {% endcomment %}` or one-line `{% # comment text %}`.

## Standard Liquid Tags and Control Flow

Liquid supports:

* **Conditional**: `{% if condition %} ... {% elsif other %} ... {% else %} ... {% endif %}`.
* **Case**: `{% case var %}{% when value %}...{% endcase %}`.
* **Loop**: `{% for item in collection %} ... {% endfor %}`. Inside for-loops, Liquid provides `forloop` object with properties like `forloop.index` (1-based index), `forloop.first`, `forloop.last`, etc.
* **Assign**: `{% assign foo = "bar" %}` to set a variable in the template context.
* **Capture**: `{% capture var %} ... {% endcapture %}` to capture output of the inner content into a variable.
* **Include**: `{% include file.html %}` – Jekyll's include tag (discussed in Section 4 and below).
* **Raw**: `{% raw %} ... {% endraw %}` to disable Liquid parsing of the content within (useful when writing code examples or when including JS templating syntax that conflicts with Liquid).
* Others like `comment` (to comment out blocks), `break` and `continue` (inside loops), etc.

Your templating engine should handle all these similarly.

## Liquid Variables and Objects

Jekyll populates a variety of objects that templates can use:

* **`site`** – The global site data (from config, plus content lists). For example, `site.posts` (list of all posts), `site.pages` (all pages), `site.data` (data files content), `site.collections` (all collections metadata), `site.time` (build timestamp), `site.documents` (all collection items), `site.categories` and `site.tags` (categories and tags indices), etc. Jekyll's documentation enumerates many of these:

  * `site.posts` is a reverse-chronological array of posts (by default, only posts in `_posts` collection).
  * `site.pages` is all pages (each page is like a `Jekyll::Page` instance, excluding posts).
  * `site.collections` is an array of collections; each has `label`, `docs`, etc..
  * `site.<collection_name>` for each custom collection (e.g., `site.authors` if an `_authors` collection exists) – this is a list of documents in that collection, similar to how posts is a built-in collection.
  * `site.data` is a map of the data files content.
  * `site.time` is the time of build (a `Time` object).
  * `site.posts` is a `posts` collection which is "hard-coded" to exist even if empty.
  * `site.categories.CATEGORY` gives array of posts in that category; similarly `site.tags.TAG` for tags.
  * `site.baseurl`, `site.url` reflect the config settings; `site.title`, etc., reflect whatever user put in config for metadata.
  * For GitHub Pages integration, a `site.github` object is provided by the `jekyll-github-metadata` plugin with repository info (like `site.github.project_title`, `site.github.owner_name`, etc.). If compatibility with GH Pages environment is needed, you might need to simulate some of these (see Section 9).

* **`page`** – The **current page/post** being rendered. This includes all front matter variables of that page, plus some standard ones Jekyll defines. For example, `page.title`, `page.content` (the rendered HTML content of the page, available in layouts), `page.excerpt` (if an excerpt was auto-generated or defined), `page.url` (the computed URL path of the page), `page.date` (for posts, the date), `page.id` (a unique identifier, often something like `/YEAR/MONTH/DAY/slug` for posts), `page.previous` / `page.next` for adjacent posts in chronological order (if on a post and pagination of posts is enabled). Everything in the page's YAML front matter is accessible, e.g., `page.categories`, `page.tags`, custom ones like `page.author`, etc.. In includes, the context is slightly different: includes use an `include` variable (see below).

* **`layout`** – In a layout file, there's a `layout` variable referring to the current layout's front matter variables. This is rarely used, but for example, if a layout file has `layout: none` in its front matter (for nesting, which is unusual), or any metadata, it can be accessed via `layout.someVar`.

* **`content`** – In a layout, `{{ content }}` is a special variable that outputs the rendered content of the child page/post being wrapped. **Important**: In Jekyll, layouts themselves can also have content if one layout uses another (layout nesting), so `{{ content }}` bubbles down. But in pages/posts, `content` is not a predefined variable accessible (the page's content is available in Liquid as either the raw Markdown via `page.content` before conversion, or after conversion you normally wouldn't include a page within itself). Typically only layouts use the `content` variable.

* **`include`** – When using the `{% include %}` tag to include a file, the included file has access to an `include` variable that is a map of any parameters passed to the include. For example: `{% include note.html content="This is a note." %}` – inside `_includes/note.html`, one can use `{{ include.content }}` to get the string passed. Also, the include has access to the parent page's variables (page, site, etc.), but any variables it creates (like with assign) are scoped to that include and do not affect the parent. Your SSG should mimic this: includes are essentially partial templates evaluated in a scope that has `include` plus references up to site/page.

* **`paginator`** – If pagination is enabled (via `jekyll-paginate` plugin's `paginate` config), then on paginated index pages, a `paginator` object is available. It contains attributes like `paginator.page`, `paginator.per_page`, `paginator.posts` (the posts on that page), `paginator.total_pages`, `paginator.next_page` & `paginator.previous_page` and their paths. The SSG must populate this on the appropriate pages (usually the index and subsequent pageN) when doing pagination (see Section 7 for details). Themes like Minima use `paginator` to render older/newer post navigation if enabled.

* **Other objects:** Jekyll sometimes exposes `{% assign collections = site.collections | where:"label","my_collection" %}` as in docs to filter a specific collection from the list, but typically you use `site.my_collection` directly. Also `site.pages` includes all pages **except** those in collections (like posts). There is also a `site.static_files` listing static files (with properties `path`, `modified_time`, etc.). And a `site.html_files` (subset of pages that end in .html).

## Scoping and Variable Precedence

Variables in Liquid have lexical scoping. The `site`, `page`, etc. are global for the page render. Inside a `for` loop, you can use the loop variable (e.g., `post` in `{% for post in site.posts %}`) and it won't override a same-named variable outside. If you do `{% assign title = "New Title" %}` inside a page, it sets page's `title` variable only within that rendering context (Liquid doesn't actually mutate the original page's data, it just stores it in the context). Includes get their own scope: any `assign` or captured variables inside an include do not leak out. Conversely, includes can see the outer scope's variables (unless a variable name is shadowed by something in the include's own locals, like an include param). One difference: Jekyll's `include` allows you to pass in variables, which become `include.varname` inside, but you cannot directly access variables defined inside the include from the outside.

## Liquid's Security Model

Liquid was designed to be secure – meaning template authors can't break out and execute arbitrary code on the host. It doesn't allow direct system access, only what you expose through variables and filters. In Jekyll's context, because the site templates are trusted (usually authored by the site builder), security isn't about multi-tenant safety but rather about not executing malicious includes or something. However, GitHub Pages relies on this safety since it runs Jekyll on user-provided code. Therefore:

* No direct Ruby eval or shell execution is possible through Liquid.
* All filters and tags are whitelisted – you can't call an undefined filter.
* Jekyll's safe mode (when `safe: true`) further ensures no plugins with custom tags/filters are loaded unless whitelisted.
* Your SSG, if considering security, should ensure that template evaluation doesn't allow injection that could read files beyond the site, etc. For example, the `{% include_relative ../outside.html %}` is prevented – Jekyll forbids `..` in include\_relative to not climb out of the source directory. Similarly, symlinks are not processed in safe mode to avoid directory traversal.

## Core Jekyll Tags & Filters to Implement

* **Tags:** `include`, `include_relative`, `highlight`/`endhighlight`, `liquid` (used to write raw Liquid in a page by wrapping in a tag), `gist` (via jekyll-gist plugin, for embedding GitHub gists), `post_url`, `link`, possibly `render` (used by jekyll-compose to render a file with a given layout in a page, not widely used).
* **Filters:** All those documented by Jekyll: `abs_url`/`absolute_url`, `rel_url`/`relative_url`, `date_to_string`, `date_to_xmlschema`, `date_to_rfc822`, `xml_escape`, `cgi_escape` (Liquid has a default `url_encode` as well), `strip_html`, `newline_to_br`, `markdownify`, `smartify` (smart quotes via typography, maybe alias to smart quotes conversion), `slugify`, `capitalize`/`upcase`/`downcase` (Liquid defaults), `sort`/`sort_natural`, `where`/`where_exp`, `group_by`/`group_by_exp`, `filter` (there is a Liquid `filter` to filter an array by truthiness of an attribute), `sample` (to get a random item), `inspect` (debug output), `printf` (Liquid core has a format one), etc. The Jekyll docs list standard Liquid filters separately for reference, which you might not need to implement if using an existing Liquid library.

## Layout and Rendering Flow with Liquid

Typically:

* Jekyll reads a page's Markdown (for example), separates front matter and content.
* It builds the Liquid context (site, page, etc.) and then processes the content with Liquid. During this, includes are processed, and any Liquid placeholders replaced with appropriate output or left as HTML.
* After Liquid in the content, Jekyll then passes the content to the converter (e.g., Markdown converter) to produce HTML.
* Then it takes the chosen layout (if `layout: default` etc. is specified). It reads the layout file, which itself likely contains Liquid code (like `{{ content }}` and perhaps references to site/page vars in header/footer).
* It sets `content` = the page's rendered HTML, and then renders the layout's Liquid in the context of the page (so the page's variables are still accessible, plus now `content` yields the HTML).
* Layouts can nest (a layout can specify `layout: base` in its front matter). In that case, after rendering the child layout, Jekyll assigns that result to `content` and then renders the base layout with it. This continues until a layout with no parent layout. Your SSG must implement layout cascading similarly.
* Any includes in layouts are processed at their turn.

## Example to Illustrate Liquid Use

In a theme like Minima's `post.html` layout, you might see:

```html
<article class="post">
  <h1>{{ page.title }}</h1>
  <div class="post-content">
    {{ content }}
  </div>
  {% if page.tags %}
    <p>Tagged: 
      {% for tag in page.tags %}
        <a href="{{ '/tags/' | relative_url }}{{ tag | slugify }}/">{{ tag }}</a>{% unless forloop.last %}, {% endunless %}
      {% endfor %}
    </p>
  {% endif %}
</article>
```

This snippet shows:

* Using `page.title`.
* Inserting the rendered content of the post via `{{ content }}`.
* A conditional: if the page has tags, it loops through them and generates links to tag archive pages (assuming such pages exist at `/tags/<tag>/`). It uses `relative_url` to prepend baseurl properly, and `slugify` to create the URL-friendly tag name.

All those Liquid operations must behave as they would in Jekyll:

* The `forloop.last` in the loop to avoid trailing comma in tag list.
* The `relative_url` filter to prepend baseurl.
* The ability to access `page.tags` which could be a string or array and iterate accordingly (Jekyll ensures in the site payload that `tags` is an array of strings even if front matter was a comma-separated string originally).

## Internationalization Considerations in Liquid

If a site is multilingual (not natively supported by Jekyll, but via plugins or manual setup), templates might have logic like:

```liquid
{% if page.lang == 'en' %}
  // English content
{% endif %}
```

or they may include different files based on language. Also, `site.data` often is used to store translation strings. There isn't built-in i18n support in Liquid beyond what the site dev implements. However, some plugins (jekyll-multiple-languages, jekyll-polyglot) add filters or variables like `site.languages`, `page.lang`, and might automatically generate multiple sites. If aiming to support i18n plugin usage, you'd incorporate those features as described in Section 10.

## Sandboxing and Security Recap

Jekyll's use of Liquid is safe by design – if your SSG uses an off-the-shelf Liquid interpreter (or an analogous templating engine), ensure it doesn't allow dangerous operations. For example, do not allow includes of files from outside the site source (unless explicitly allowed by user). Do not allow reading arbitrary system variables unless Jekyll does (Jekyll does allow environment variables via the `ENV` filter in Liquid, I think not by default – though in some contexts `{{ ENV.VAR }}` might be blocked in safe mode). Generally, match Jekyll: e.g., in safe mode, external data access (like the `jekyll-data` plugin that could fetch remote content) should be off.

## Implementation Summary

Implementing a Liquid-compatible templating system means:

* Provide all the predefined objects (`site`, `page`, etc.) with the same structure/fields Jekyll would.
* Implement or use a Liquid engine that supports tags (if not all, at least ones Jekyll uses in its default stack) and filters similarly.
* Ensure the order of processing (Liquid then Markdown, includes processed at the right time).
* Ensure that things like `{{ }}` vs `{% %}` are distinguished and output properly escaped (Liquid by default HTML-escapes variables unless they are marked `| safe` or are used in `{{ }}`? Actually, in Liquid, output is not auto-escaped in Jekyll context because Jekyll passes all content raw to be inserted; the `escape_once` or `escape` filters can be used explicitly. But shopify's Liquid sometimes has an "object called html" that it might autoescape in certain contexts – likely not in Jekyll usage).
* Pass through unknown tags/filters? Jekyll would error if it sees an unknown tag/filter, unless it's within a raw block. Your engine should similarly throw an error for unknown tags to alert the user (so they know a plugin might be missing).
* Contain any malicious content: for example, if a user somehow put `{{ site.posts | eval }}` – there's no eval filter, but if they attempted something like that or using the `| inspect` to reveal internal structures, it's not harmful but might show site data. Should be fine since it's their own data.

Implementing these will allow existing Jekyll templates and includes to render the same with your SSG. 