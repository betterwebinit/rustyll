# Pagination

**Pagination** in Rustyll refers to splitting a list of posts (or other collection documents) across multiple pages, typically for a blog index. Rustyll provides built-in pagination capabilities for efficiently managing large collections of content.

## Rustyll Pagination

### Configuration

Enable pagination by setting `paginate: N` in `_config.yml` (where N is the number of posts per page). Optionally, set `paginate_path` for the URL structure of paginated pages. For example:

```yaml
paginate: 5
paginate_path: "/blog/page:num/"
```

If `paginate_path` is not set, Rustyll uses `"/page:num"` by default (which corresponds to `paginate_path: /page:num`). The `:num` token is replaced by the page number (2,3,...). Page 1 is the implicit main index page.

### How It Works

The pagination system expects you to have an HTML file (not Markdown) that will serve as the paginated index template. Usually, this is the `index.html` (or a specific index like `blog/index.html` if you want the blog on a subpath). Rustyll will read `index.html`, and seeing that `paginate` is configured, it will generate multiple copies of that page:

* The original `index.html` will get the first N posts in `paginator.posts`.
* It will then create new output files for page 2, page 3, ... as needed, using the content of the same index template each time but with a different subset of posts.
* For page 2 and onward, it places them in the path according to `paginate_path`. For example, with `paginate_path: /blog/page:num/`, Rustyll will create `_site/blog/page2/index.html`, `_site/blog/page3/index.html`, etc. (Page 1 remains at `_site/blog/index.html`).

### Linking Between Pages

The `paginator` object provides `paginator.previous_page`, `paginator.previous_page_path`, `paginator.next_page`, `paginator.next_page_path` for navigation. In templates, you can use:

```
{% if paginator.previous_page %}
  <a href="{{ paginator.previous_page_path }}">Newer Posts</a>
{% endif %}
{% if paginator.next_page %}
  <a href="{{ paginator.next_page_path }}">Older Posts</a>
{% endif %}
```

Also, `paginator.page` is the current page number, `paginator.total_pages` is the total count. Some themes will output "Page X of Y". The default `paginate_path` structure ensures page 1 doesn't get a `/page1/` (Rustyll does not create a directory for page1). It only creates from 2 upwards, treating the original index as page 1.

### Advanced Features

Rustyll's pagination system also supports:

* Pagination for collections other than posts
* Category and tag-based pagination
* Multiple pagination sections on one site
* Customizable title suffixes for paginated pages
* Sorting options for paginated content

### Configuration Example for Advanced Features

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

Rustyll also supports an **AutoPages** feature where it can generate pagination for every tag and category automatically, via configuration:

```yaml
paginate: 5
pagination:
  enabled: true
  auto: 
    categories: true
    tags: true
```

This would create pages like `/tags/TagName/page2/`, etc., for each tag with more than paginate count items.

## Templating with Pagination

Usually, the theme's index uses a loop:

```
{% for post in paginator.posts %}
  <!-- display post excerpt or title -->
{% endfor %}
```

This works with Rustyll's template context provided by the paginator. If no pagination is enabled, themes might instead loop over `site.posts`.

## Edge Cases

If `paginate: N` is set but you have fewer than N posts, Rustyll still sets up paginator on page1 (with total_pages=1, and no next page). It doesn't error out; it just means one page. So it handles 0 or few posts gracefully.

## Don't Set a Permalink on the Index

The docs warn that if you manually set a permalink in the front matter of the index page, it can conflict with how pagination writes files. Because if a permalink were set, Rustyll might output that index differently, which would affect pagination. If an index page has a permalink in front matter, pagination might not function as expected.

## Implementation Summary

Rustyll's pagination system:

* Activates when config `paginate` is present (and >0, and at least one post exists)
* Creates paginated pages for posts on the specified index
* Follows the `paginate_path` pattern for output paths (replacing :num with page number)
* Provides the `paginator` template object with all necessary attributes
* Ensures page1 doesn't get the extra path (only subsequent pages)
* Does not require any special front matter in the index for it to work
* Supports advanced options through the `pagination` configuration key 