# GitHub Pages Compatibility

GitHub Pages is one of the most popular hosting platforms for Jekyll sites, offering free static site hosting directly from a GitHub repository. For a Jekyll-compatible SSG, supporting GitHub Pages deployment is crucial, as many users would expect to use their Rust-based static site generator with the same GitHub Pages workflow they're accustomed to with Jekyll.

## GitHub Pages Build Environment

When a repository is configured for GitHub Pages, GitHub uses Jekyll to build the site automatically whenever changes are pushed to the repository. This build environment has certain constraints and configurations:

### Available Jekyll Plugins

GitHub Pages only allows a specific set of Jekyll plugins (gems) to be used when building sites. These are listed in the [GitHub Pages gem](https://github.com/github/pages-gem) and include:

* **jekyll-coffeescript**: CoffeeScript converter
* **jekyll-default-layout**: Auto-selects layout
* **jekyll-gist**: GitHub Gist tag
* **jekyll-github-metadata**: GitHub metadata access
* **jekyll-optional-front-matter**: Makes front matter optional
* **jekyll-paginate**: Pagination support (v1 only)
* **jekyll-readme-index**: Use README as index
* **jekyll-titles-from-headings**: Auto-generate page title from heading
* **jekyll-relative-links**: Convert relative links to Jekyll pages
* **jekyll-feed**: Atom feed generator
* **jekyll-sitemap**: XML sitemap generator
* **jekyll-seo-tag**: SEO meta tags

GitHub Pages **will not build** sites that use plugins outside this approved list. Many Jekyll users maintain two workflows: one for local development (with custom plugins) and one for GitHub Pages deployment (limiting to approved plugins).

### Safe Mode

GitHub Pages builds Jekyll sites in **safe mode** (`safe: true`), which:

* Disables custom plugins
* Disables arbitrary code execution in Liquid templates
* Prevents symbolic links from being followed
* Restricts access to certain directories

This is a security measure to prevent malicious code execution in user-submitted content.

### Configuration Defaults and Overrides

GitHub Pages sets certain configuration values that may override user settings:

* `lsi: false` (Latent Semantic Indexing disabled)
* `safe: true` (as mentioned above)
* `incremental: false` (incremental builds disabled)
* `highlighter: rouge` (syntax highlighting with Rouge)
* `gist.noscript: false` (Gist JavaScript embedding)
* `kramdown.math_engine: mathjax` (MathJax for math rendering)
* `kramdown.syntax_highlighter: rouge` (Rouge for syntax highlighting)

Your SSG would need to respect these constraints when generating GitHub Pages-compatible sites.

### The github-pages Gem

Many Jekyll users who deploy to GitHub Pages install the `github-pages` gem locally, which sets up the exact same environment as GitHub Pages, including the approved plugins and default settings. Your SSG could consider a similar compatibility mode (e.g., a `--github-pages` flag) that mimics these restrictions.

## jekyll-github-metadata

A key component of GitHub Pages integration is the **jekyll-github-metadata** plugin, which populates the Liquid templating environment with information about the repository, including:

* `site.github.owner_name` - Repository owner username
* `site.github.repository_name` - Repository name
* `site.github.project_title` - Project title
* `site.github.project_tagline` - Project description
* `site.github.url` - GitHub Pages URL
* `site.github.contributors` - List of repository contributors
* `site.github.public_repositories` - User's public repositories
* `site.github.organization_members` - Organization members (for org repos)

For a Jekyll-compatible SSG, implementing a similar metadata provider would be valuable for GitHub Pages users. This could involve:

1. Detecting a GitHub repository context
2. Using the GitHub API to fetch repository metadata
3. Providing this data to templates in the same structure

## Project Pages vs. User/Organization Pages

GitHub Pages supports two types of sites:

1. **Project Pages** - Sites for specific repositories, served at `username.github.io/repository-name`
2. **User/Organization Pages** - Sites for a user or organization, served at `username.github.io`

These differ in how they're configured:

* **Project Pages**:
  * Source is typically the `gh-pages` branch or a `/docs` folder in the `main` branch
  * The site is at a subpath: `username.github.io/repository-name`
  * `baseurl` is typically set to `/repository-name`

* **User/Organization Pages**:
  * Source must be the `main` branch of a special repository named `username.github.io`
  * The site is at the root domain: `username.github.io`
  * `baseurl` is typically empty

Your SSG should handle both cases, particularly regarding URL generation with `baseurl` prefixes.

## Common Jekyll Features Used with GitHub Pages

Several Jekyll features are commonly used with GitHub Pages:

### Relative URLs and Baseurl Handling

Most GitHub Pages sites use these common filters to handle URLs correctly:

```liquid
{{ "/assets/style.css" | relative_url }}
{{ "https://example.com" | absolute_url }}
```

These prepend `site.baseurl` and `site.url` appropriately. Your SSG should implement these filters to ensure themes work correctly.

### Jekyll Themes on GitHub Pages

GitHub Pages supports a set of built-in themes that can be enabled with a simple configuration:

```yaml
theme: minima
```

Supported themes include:

* minima
* jekyll-theme-architect
* jekyll-theme-cayman
* jekyll-theme-dinky
* jekyll-theme-hacker
* jekyll-theme-leap-day
* jekyll-theme-merlot
* jekyll-theme-midnight
* jekyll-theme-minimal
* jekyll-theme-modernist
* jekyll-theme-slate
* jekyll-theme-tactile
* jekyll-theme-time-machine

When using these themes, GitHub Pages loads them from GitHub-hosted gem repositories. Your SSG might implement a similar theme system or compatibility layer.

### Front Matter Defaults

GitHub Pages respects Jekyll's front matter defaults, which are commonly used to set layouts, permalinks, and other metadata for entire sections of a site:

```yaml
defaults:
  -
    scope:
      path: ""
      type: "posts"
    values:
      layout: "post"
      author: "Site Owner"
```

Implementing front matter defaults is important for compatibility with existing Jekyll sites.

## Common Deployment Configurations

Jekyll users deploying to GitHub Pages typically use one of these approaches:

1. **Direct GitHub Pages deployment**: Push to GitHub, let GitHub build the site (limited to approved plugins)
2. **Build locally, push output**: Build locally with any plugins, push only the generated `_site` directory
3. **CI/CD deployment**: Use GitHub Actions or similar to build with custom plugins and deploy to Pages

For a Jekyll-compatible SSG, supporting all these workflows would be ideal. This might involve:

* Documentation on configuring GitHub Pages with your SSG
* GitHub Actions examples for automated builds
* Tools for pushing only the generated site to a specific branch

## Implementing GitHub Pages Compatibility

To make your Rust SSG fully compatible with GitHub Pages workflow, consider:

### Compatibility Mode

Implement a `--github-pages` flag that:

* Turns on safe mode
* Disables unsupported features
* Sets the same defaults as GitHub Pages

### Metadata Access

Provide a `site.github` object with the same structure as jekyll-github-metadata:

```rust
struct GitHubMetadata {
    url: String,
    owner_name: String,
    repository_name: String,
    project_title: String,
    project_tagline: String,
    // Other fields...
}

// When in GitHub context (either detected or specified by user):
site.github = fetch_github_metadata(repo_url);
```

This could detect a GitHub repository by:
* Checking for a `.git` directory and parsing its config
* Looking for environment variables set by GitHub Actions
* Accepting explicit user configuration

### URL Handling

Implement the `relative_url` and `absolute_url` filters to handle `baseurl` correctly:

```rust
fn relative_url(path: &str, site: &Site) -> String {
    if path.starts_with('/') {
        format!("{}{}", site.baseurl, path)
    } else {
        format!("{}/{}", site.baseurl, path)
    }
}

fn absolute_url(path: &str, site: &Site) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        relative_url(path, site)
            .pipe(|rel_url| format!("{}{}", site.url, rel_url))
    }
}
```

### Theme Support

For GitHub Pages theme compatibility:

1. Support loading layouts and includes from theme gems
2. Implement the supported themes or document how to use equivalents
3. Honor theme configuration in `_config.yml`

This might involve bundling theme files or providing a way to download them.

### Plugin Compatibility

Implement the functionality of common GitHub Pages plugins:

* Basic pagination (jekyll-paginate)
* Feed generation (jekyll-feed)
* Sitemap generation (jekyll-sitemap)
* SEO tags (jekyll-seo-tag)
* README as index (jekyll-readme-index)

These don't need to be implemented as plugins per se; they could be built-in features that activate when the corresponding configuration is detected.

## Conclusion

GitHub Pages compatibility is a significant advantage for Jekyll users, providing free hosting and simple deployment. For a Jekyll-compatible Rust SSG, supporting GitHub Pages workflows will make migration much easier for users.

The key aspects to implement are:

1. Safe mode and GitHub Pages build constraints
2. The site.github metadata object
3. Proper URL handling with baseurl
4. Support for GitHub Pages themes
5. The functionality of common GitHub Pages plugins

With these features, users can deploy sites built with your SSG to GitHub Pages just as they would with Jekyll, using the same configuration patterns and expecting the same behavior. 