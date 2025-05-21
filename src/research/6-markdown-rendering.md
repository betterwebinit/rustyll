# Markdown Rendering

By default, Jekyll uses the **Kramdown** Markdown engine (a Ruby library) to convert Markdown files to HTML. Kramdown is configured to use **GitHub-Flavored Markdown (GFM)** as its input syntax, which means it supports features like fenced code blocks, tables, strikethrough, autolink URLs, etc., similar to GitHub's markdown. To be Jekyll-compatible, the Rust SSG's Markdown processing should match Kramdown (with GFM) output as closely as possible, including support for various extensions and configuration options.

## Default Engine and Extension

Jekyll's `_config.yml` has `markdown: kramdown` by default, and `kramdown` itself has sub-configurations under the `kramdown:` key. Notably, Jekyll sets `kramdown.input: GFM` by default, meaning the Kramdown GFM parser is used. This brings Kramdown's syntax close to CommonMark. For your SSG, you may choose a Markdown library (like pulldown-cmark or comrak in Rust) that supports GFM extensions.

## Kramdown (GFM) Features

These include:

* Fenced code blocks with syntax highlighting (`lang ...`).
* Tables with pipes.
* Strike-through text using `~~strike~~`.
* Autolinking of URLs (http... becomes `<a href="...">...</a>` automatically).
* Task lists (`- [ ]` checkboxes) if enabled.
* Footnotes (`[^1]` references to `[^1]: footnote text`).
* Emoji shorthand (if jemoji plugin is used, otherwise `:smile:` stays as text).
* Smart quotes conversion (depending on config).

Kramdown also supports an inline attribute syntax (e.g., `{:.class}` to add classes to elements), which some Jekyll users utilize. Not mandatory to implement unless heavily used.

## Configuration Options for Kramdown

Jekyll exposes a variety of Kramdown options via `_config.yml` under `kramdown:`. Some default values from Jekyll's default config:

* `auto_ids: true` – automatically generate `id` attributes for headings based on text (useful for linking to sections).
* `toc_levels: [1,2,3,4,5,6]` – which heading levels are included when generating a table of contents.
* `entity_output: as_char` – how to output special characters (as raw char vs named entities).
* `smart_quotes: lsquo,rsquo,ldquo,rdquo` – convert straight quotes to curly quotes, etc.
* `input: GFM` – as mentioned, use GitHub-Flavored Markdown parser.
* `hard_wrap: false` – in GFM, newlines are not converted to `<br>` by default (as opposed to original Markdown where a double space or newline may break lines).
* `footnote_nr: 1` – starting number for footnotes.
* `show_warnings: false` – whether to show warnings during conversion (not relevant to output).

These can be overridden by the user's config. For example, one might set `kramdown: { hard_wrap: true }` to treat every newline as a `<br>`.

If you aim for maximum fidelity, pick a Markdown library that allows configuring these or pre/post-process to simulate them. For instance, auto\_ids: a Rust Markdown might generate IDs for headings differently – ensure they match Kramdown's algorithm (Kramdown downcases text, strips certain characters, and appends a number if needed for uniqueness).

## Syntax Highlighting

Jekyll by default uses **Rouge** (a Ruby gem) for highlighting fenced code blocks (`highlighter: rouge` in config). The `{% highlight %}` Liquid tag also uses Rouge. For Rust SSG, you can integrate a syntax highlighter (like Syntect or similar) to highlight code. The key is to match class names and markup format that Rouge produces:

* Rouge wraps code in `<pre><code class="language-XYZ">...</code></pre>`.
* It might include `<span class="k">for</span>` etc., where classes (k = keyword, nf = function name, s = string, c = comment, etc.) follow Pygments style classes. Many Jekyll themes rely on these class names for styling.
* Ensure that your highlighter either uses the same class naming or you supply custom CSS to mimic Rouge output.

Alternatively, since many use GitHub's style, you could consider outputting code with classes similar to GitHub's, but that might break themes expecting Rouge's classes. Better to emulate Rouge classes or allow custom CSS.

## Alternate Markdown Engines

Jekyll also supports setting `markdown` to other values if the gem is present:

* `commonmark` or `commonmark-ghpages` (via gems like jekyll-commonmark and jekyll-commonmark-ghpages which wrap the C CommonMark library). If a user set `markdown: commonmark`, Jekyll would use that gem. On GitHub Pages, `jekyll-commonmark-ghpages` is available (which supports CommonMark + GFM). For your SSG, you might not implement multiple markdown engines; but being aware is useful. You could choose one that is closest to GFM output (CommonMark should be fine, since Kramdown GFM aims for CommonMark compliance).
* In older times, Redcarpet (another C extension) or other processors could be used. Kramdown replaced most of those.
* If someone configures a Markdown engine unknown, Jekyll tries to require that library. Likely out of scope to allow arbitrary pluggable Markdown in Rust, so you may document that only the default (CommonMark/GFM) is supported, which covers most cases.

## GitHub-Flavored Markdown Specifics

* GitHub's table syntax: e.g.,

  ```
  | Column1 | Column2 |
  |---------|---------|
  | value1  | value2  |
  ```

  This should produce a `<table>` with proper `<thead>` and `<tbody>` etc. Kramdown GFM does it.
* Strikethrough: `~~text~~` → `<del>text</del>`.
* Fenced code with triple backticks: make sure to parse info string for language (like "\`\`\`ruby") and apply `language-ruby` class to code element.
* Task lists: `- [x] Done` (with `gfm_parser: true`, Kramdown will output checkboxes). Actually, Kramdown's GFM parser does support parsing task list items, but one might need to explicitly enable an option. The plugin jekyll-commonmark might handle it by default. If implementing, turn `- [ ]` into `<ul><li><input type="checkbox" disabled> ...`.
* Autolinks: e.g., `http://example.com` in text becomes clickable link. Kramdown does this in GFM mode.
* Tables of contents: Kramdown has a `{::toc}` special tag for table of contents insertion. However, Jekyll doesn't enable the Kramdown parser's default for that (some users use plugins or do it manually by scanning headings).

## Base-level Paragraphs and Line Breaks

Kramdown treats a blank line as a paragraph break, single newline as just a space (when hard\_wrap is false). Some differences with CommonMark might exist (like how it handles underscores in words, etc.). It might not be critical except for edge cases (like emphasis rules). Aim to match typical content.

## HTML in Markdown

Kramdown allows raw HTML in Markdown, and by default it passes it through unaltered (unless options to sanitize are set, which Jekyll doesn't do by default). So if a user has `<div class="note">...</div>` in their .md, it should appear as-is in output. Most Markdown libs preserve HTML by default, so that should be fine.

## Error Handling

If Markdown has an unclosed tag or something, Kramdown might throw a warning. Jekyll's `show_warnings` is false, meaning it won't show those warnings. Your SSG can ignore minor MD warnings similarly, focusing on producing output.

## Implementing in Rust

A good approach might be using a library like Comrak (CommonMark in Rust) with GitHub extensions enabled (it has options for strikethrough, tables, autolinks, tagfilter, tasklist, footnotes, etc.). Comrak with `ComrakExtensionOptions { strikethrough: true, table: true, autolink: true, tasklist: true, footnotes: true, ... }` would get pretty close to GFM. You'd need to ensure heading IDs generation matches Kramdown's:

* Kramdown generates IDs from text by:

  1. Removing HTML tags from heading text.
  2. Unescaping HTML entities.
  3. Replacing spaces and non-alphanumeric characters with hyphens (it keeps Unicode letters I think, but lowercases everything).
  4. If it ends up empty, it uses "section".
  5. If an ID is already used, appends a number (starting with 1, then 2, etc.) to make it unique.

Comrak can generate IDs or you might do a pass to generate them yourself. Many Jekyll themes rely on these IDs for linking or JS (e.g., anchor links next to headings). So try to match that algorithm.

## Syntax Highlighting in Markdown vs Highlight Tag

* Fenced code blocks can either be left alone and highlighted by a client-side library or highlighted during build. Jekyll uses Rouge during build to output styled HTML. If replicating that, you'd detect the info string (like "\`\`\`js") and run a highlighter to output `<pre><code class="language-js highlight"><span class="...">...`.
* The `{% highlight lang %} code {% endhighlight %}` tag should produce similar output. This tag also supports an optional `linenos` option to number lines (`{% highlight lang linenos %}`) – Rouge outputs a table with line numbers in a separate column for that.

If not implementing line numbering, you might at least not break if `linenos` is present (maybe just ignore it or ensure the class "linenos" is set so theme can style differently if needed).

## GitHub Pages Considerations

GitHub Pages by default enables kramdown + GFM parser and also has `jekyll-commonmark-ghpages` which one can opt into. They support both to some extent. But since GH Pages primarily defaults to kramdown, matching that is sufficient. Also note GH Pages might restrict certain HTML (like `<script>` tags might be removed by a sanitizer – though I think GH only sanitizes user content in some contexts like via the `github-pages` gem's HTML pipeline for certain markdown includes or user content injection, but for Jekyll sites it usually leaves HTML alone, except possibly for things like the `emoji` or `mentions` pipeline which deliberately filters some tags). 

Actually, the `html-pipeline` gem listed in GH Pages dependencies suggests they do run the output through HTML::Pipeline (used by jekyll-mentions, jemoji, etc., which are implemented via HTML filters). So, if replicating GH Pages exactly, one would implement jemoji (replace \:emoji: with images) and jekyll-mentions (replace @user with link) which actually are done *after* markdown conversion via a filter.

## Markdown in Other Places

Jekyll allows rendering Markdown in:

* **Includes**: If an include file has front matter, it will be processed as a standalone (which is unusual). Typically includes are just fragments. If an include has Markdown and you want it to be converted, one approach is to set `markdown="1"` in the include tag or use the `markdownify` filter on include content. E.g., some includes are used like: `{% capture note_content %}...markdown text...{% endcapture %}{{ note_content | markdownify }}` to convert a snippet to HTML. So implementing the `markdownify` filter (which likely just calls the Markdown converter on a string) is needed.
* **Data files**: If data strings contain Markdown and are output with markdownify or some filter.
* **Post Excerpts**: Jekyll generates `page.excerpt` for posts, usually by taking the text before `<!--more-->` or before two newlines and strip HTML tags. By default, Jekyll's `page.excerpt` is **unrendered** text (it's the raw Markdown snippet) but if used in a template, often you see `{{ page.excerpt | markdownify }}` in themes to render it properly. Your SSG should mimic excerpt generation: by default, excerpt is the content up to the first blank line or `excerpt_separator` (which by default is `\n\n` but user can set an explicit marker). And no HTML tags in excerpt by default, just text. Implementing excerpt logic ensures compatibility with themes that show post summaries.

Given these, ensure:

* Use the same `excerpt_separator` (configurable, default two newlines) to split excerpts.
* Provide `page.excerpt` in Liquid and have `{{ page.excerpt | markdownify }}` produce the right HTML.

## Implementation Summary

The Markdown conversion in the SSG should handle all typical Jekyll Markdown files with:

* GFM features,
* Kramdown's default behaviors (smart quotes, etc. as configured),
* heading ID generation,
* footnotes and other extras (footnotes are somewhat niche but supported by Kramdown and likely needed by some blogs),
* output HTML consistent enough that any theme's CSS will style it correctly (especially for code blocks and tables).

Test with sample posts from a Jekyll site to verify minimal differences (maybe minor whitespace differences are tolerable, but structure should be the same). For instance, check that a simple markdown list or blockquote outputs as the same tags structure. 