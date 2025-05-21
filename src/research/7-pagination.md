# Pagination

**Pagination** in Jekyll refers to splitting a list of posts (or other collection documents) across multiple pages, typically for a blog index. Jekyll's core has no built-in pagination after version 3 (in Jekyll 2 it had an integrated paginator for posts). Instead, it's provided via plugins. The standard plugin **jekyll-paginate** (v1) is very commonly used (and is enabled by default on GitHub Pages for user sites). There is also a more advanced plugin **jekyll-paginate-v2** which offers pagination of categories, tags, and collections, though GitHub Pages does not support v2 by default.

## jekyll-paginate (v1)

### Configuration

Enable by including `jekyll-paginate` in the plugins and setting `paginate: N` in `_config.yml` (where N is the number of posts per page). Optionally, set `paginate_path` for the URL structure of paginated pages. For example:

```yaml
paginate: 5
paginate_path: "/blog/page:num/"
```

If `paginate_path` is not set, Jekyll uses `"/page:num"` by default (which corresponds to `paginate_path: /page:num`). The `:num` token is replaced by the page number (2,3,...). Page 1 is the implicit main index page.

### How It Works

The plugin expects you to have an HTML file (not Markdown, per the docs) that will serve as the paginated index template. Usually, this is the `index.html` (or a specific index like `blog/index.html` if you want the blog on a subpath). Jekyll will read `index.html`, and seeing that `paginate` is configured, it will generate multiple copies of that page:

* The original `index.html` will get the first N posts in `paginator.posts`.
* It will then create new output files for page 2, page 3, ... as needed, using the content of the same index template each time but with a different subset of posts.
* For page 2 and onward, it places them in the path according to `paginate_path`. For example, with `paginate_path: /blog/page:num/`, Jekyll will create `_site/blog/page2/index.html`, `_site/blog/page3/index.html`, etc. (Page 1 remains at `_site/blog/index.html`).

### Linking Between Pages

The `paginator` object provides `paginator.previous_page`, `paginator.previous_page_path`, `paginator.next_page`, `paginator.next_page_path` for navigation. In Liquid, themes often do:

```liquid
{% if paginator.previous_page %}
  <a href="{{ paginator.previous_page_path }}">Newer Posts</a>
{% endif %}
{% if paginator.next_page %}
  <a href="{{ paginator.next_page_path }}">Older Posts</a>
{% endif %}
```

Also, `paginator.page` is the current page number, `paginator.total_pages` total count. Some themes will output "Page X of Y". The default `paginate_path` structure ensures page 1 doesn't get a `/page1/` (Jekyll does not create a directory for page1). It only creates from 2 upwards, treating the original index as page 1.

### Limitations of v1

It can only paginate the `posts` collection, and only one paginator per site (you can't have two different paginated sections in Jekyll v1). Also, it must be an HTML file (the logic doesn't trigger in a `.md` file due to how the plugin is implemented). If a user put `paginate: N` and the index was a Markdown file, it wouldn't paginate (so generally one uses .html for that page). Your SSG could choose to remove that quirk (allowing .md to paginate by converting it then paginating), but to match behavior exactly, you might require the template be HTML or at least treat it similarly.

### Example

Suppose 12 posts, `paginate: 5`. Output:

* `index.html` -> shows posts 1-5.
* `page2/index.html` -> shows posts 6-10.
* `page3/index.html` -> shows posts 11-12.

The `paginator` values:
* On page1: page=1, total\_pages=3, next\_page=2, next\_page\_path="/page2/", previous\_page (nil).
* On page2: page=2, previous\_page=1, previous\_page\_path="/", next\_page=3, next\_page\_path="/page3/".
* On page3: page=3, previous\_page=2, next\_page (nil).

Jekyll doesn't generate /page1 because it's not needed (the main index covers it). If you used `blog/index.html` with `paginate_path: /blog/page:num`, then:
* `/blog/index.html` (page1),
* `/blog/page2/index.html`, etc.

### Implementation

Likely you would:

1. After reading all posts and sorting them (newest first by date by default), check if `paginate` is set and the target template exists.

2. The template (page) to paginate is identified as the one with `paginate` in its front matter? Actually, in jekyll-paginate v1, you *don't* put anything in front matter of index. The plugin just finds the first HTML index in source? According to Jekyll docs, you just set config and ensure that the index page has no `permalink` (otherwise pagination may break). The plugin likely finds the "pager" by looking for `index.html` (or the first occurrence of a page that matches `paginate_path` structure maybe).

   * Typically, people use it on the main index or an index in a subdir.
   * The docs specifically say "Pagination works when called from within the HTML file, named `index.html`, which optionally may reside in and produce pagination from within a subdirectory". So either `/index.html` or e.g. `/blog/index.html` can be paginated.
   * The plugin might see if `paginate_path` contains a subdir and then find the index in that subdir. Or it could rely on the presence of `paginate: N` in config as the trigger and just assume there's an `index.html` at `paginate_path` base.
   * Likely easier: require user's index to have `paginator` in template to actually output something; if none found, no harm done.

3. Split the posts list into slices of N.

4. Render the index template multiple times with each slice:

   * The first time, with paginator context for page1, and write to its normal destination (e.g., `/blog/index.html`).
   * Then for each subsequent page number i from 2 to total\_pages, adjust variables and write to `/blog/page{i}/index.html`.

5. Provide `paginator` object in context when rendering those pages.

6. Possibly add `paginator` to `site` as well (Jekyll doesn't have a global `site.paginator` because it's only relevant in the index context, not globally).

### Don't Set a Permalink on the Index

The docs warn that if you manually set a permalink in the front matter of the index page, it can conflict with how paginate writes files. Because if a permalink were set, Jekyll might output that index differently, which pagination plugin doesn't expect. For SSG, you could either override any permalink on the page template when paginating or issue a warning. But probably just mimic Jekyll: if an index page has a permalink in YAML, pagination might not function (and often folks run into that on Jekyll).

## jekyll-paginate-v2

This is a more powerful plugin (not enabled on GH Pages by default) which allows:

* Pagination for any collection, not just posts.
* Pagination by category or tag (auto-creates category index pages).
* Multiple pagination on one site (like an archive page for each tag with pagination).
* Complex configurations like an `pagination` key with subkeys instead of just a number.

### Configuration Example

```yaml
pagination:
  enabled: true
  per_page: 5
  title_suffix: " - page :num"
  circular: false
  sort_field: "date"
  sort_reverse: true
  # If doing category or tag pagination:
  category: 'JavaScript'  # to paginate posts of one category
  # or:
  tag: 'tutorial'
```

But more commonly, v2 supports an **AutoPages** feature where it can generate pagination for every tag and category automatically, via configuration:

```yaml
paginate: 5
pagination:
  enabled: true
  auto: 
    categories: true
    tags: true
```

This would create pages like `/tags/TagName/page2/`, etc., for each tag with more than paginate count items.

Given the complexity, implementing full v2 is a project on its own. For compatibility, supporting v1 covers the majority of use-cases, especially for GH Pages sites. If a user uses v2, the site likely won't build on GH Pages anyway, but they might build locally. Up to you if you want to support that. Possibly out-of-scope unless specifically needed.

However, at least be aware:

* v2 uses `pagination:` config instead of `paginate:` and `paginate_path`. The presence of `pagination:` could hint the site intended to use v2. You could either not support it or try partial support (maybe treat it similar to v1 for posts).
* v2 also introduced `pagination.template` for customizing the template path, and it can output to different folders with placeholders like `:category` in path.

Since the question explicitly mentions `jekyll-paginate-v2, with examples`, they might expect some explanation:

* The docs mention jekyll-paginate-v2 is not supported by GitHub Pages.
* It supports categories, tags, collections.
* The plugin's README (and config snippet from StackOverflow we saw) shows keys like `collection: name`, `category: name` or arrays, `tag: name` or arrays to filter which items to paginate.
* It also supports an `offset`, `limit` etc., and even `reversed` order toggling.
* It can generate pagination in any page, not just index.html, by using a `paginate: collectionName` in that page's front matter (I think v2 allowed per-page front matter triggers too).

For thoroughness, we can describe how one *could* implement these:

* If `pagination.enabled`, and `pagination.collection` specified (like "posts" or "authors"), use that collection instead of posts.
* If `pagination.category` or `pagination.tag` specified, filter the collection to those items (or autopages mode would require generating pages for each category/tag).
* Titles can be given suffix (like " - page 2") with template `:num`.
* There are additional features like `pagination.extended: true` to generate "previous 1 2 3 ... next" page number links (some plugin features). That mostly affects templates (and they supply a `pagination.pages` list maybe).

Given time constraints, focusing on v1 in implementation and mentioning v2's capabilities is likely enough, since replicating v2 entirely is extensive.

## Templating with Pagination

Usually, the theme's index uses a loop:

```liquid
{% for post in paginator.posts %}
  <!-- display post excerpt or title -->
{% endfor %}
```

This should work with your SSG's context if paginator is set properly. If no pagination, theme might instead loop over `site.posts`.

One more difference: By default, Jekyll's `site.posts` includes all posts (future posts if not published, etc. except they filter out unpublished by default). The pagination uses the `posts` collection as loaded in `site.posts` (which is sorted newest first by default). If `future: false`, posts with future dates are already excluded from `site.posts`. If `unpublished: false`, unpublished are excluded. So `site.posts` used by paginator is already filtered appropriately. Your SSG should do the same: only include published, <= Time.now posts in the list to paginate (unless those flags are true).

## Edge Case

If `paginate: N` is set but you have fewer than N posts, Jekyll still sets up paginator on page1 (with total\_pages=1, and no next page) IIRC. It doesn't error out; it just means one page. So handle 0 or few posts gracefully (paginator still exists maybe with total\_pages=1, or perhaps it doesn't define paginator at all? I believe it does define it but next/prev are nil).

## Implementation Summary

Implementing Jekyll-like pagination means:

* If config `paginate` is present (and >0, and at least one posts exists), create paginated pages for posts on the specified index.
* Adhere to `paginate_path` pattern for output paths (replacing \:num with page number, note that \:num is by default no leading zero etc., just an integer).
* Provide the `paginator` Liquid object with all attributes Jekyll provides.
* Ensure page1 doesn't get the extra path (only subsequent pages).
* Do not require any front matter in the index for it to work (Jekyll v1 doesn't require adding anything to index front matter to trigger it).
* Possibly throw an error or warning if trying to use v1 but the index is Markdown (or just handle it by converting that Markdown to HTML first and then paginating? That could be a nice improvement but might diverge from Jekyll's requirement).
* Document that if someone sets up `pagination:` they might need to adapt for support (or ignore it if not implementing v2). 