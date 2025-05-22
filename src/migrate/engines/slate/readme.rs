use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn generate_readme(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create README.md in the root directory
    let readme_path = dest_dir.join("README.md");
    let readme_content = generate_root_readme();
    
    fs::write(&readme_path, readme_content)
        .map_err(|e| format!("Failed to create README.md: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "README.md".to_string(),
        description: "Created README.md for the Jekyll site".to_string(),
    });
    
    // Create a contributing guide
    let contributing_path = dest_dir.join("CONTRIBUTING.md");
    let contributing_content = generate_contributing_guide();
    
    fs::write(&contributing_path, contributing_content)
        .map_err(|e| format!("Failed to create CONTRIBUTING.md: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "CONTRIBUTING.md".to_string(),
        description: "Created CONTRIBUTING.md for the Jekyll site".to_string(),
    });
    
    Ok(())
}

fn generate_root_readme() -> String {
    r#"# Site Documentation

This site was migrated from a Slate API documentation site to Jekyll using [Rustyll](https://github.com/example/rustyll).

## Getting Started

### Prerequisites

- [Ruby](https://www.ruby-lang.org/en/downloads/) version 2.5.0 or higher
- [RubyGems](https://rubygems.org/pages/download)
- [Bundler](https://bundler.io/)

### Installation

1. Clone this repository
2. Install dependencies:
   ```
   bundle install
   ```

### Local Development

To serve the site locally:

```
bundle exec jekyll serve
```

The site will be available at [http://localhost:4000](http://localhost:4000).

### Building for Production

To build the site for production:

```
JEKYLL_ENV=production bundle exec jekyll build
```

The site will be generated in the `_site` directory.

## Project Structure

- `_layouts/`: Layout templates
- `_includes/`: Reusable components and partials
- `_sass/`: SCSS files
- `assets/`: CSS, JavaScript, images, and other static files
- `_config.yml`: Jekyll configuration file

## Making Changes

- Edit Markdown files to update content
- Modify files in `_sass/` to change styles
- Use `_layouts/` and `_includes/` to adjust page structure

## Documentation

For more information about Jekyll, see:

- [Jekyll Documentation](https://jekyllrb.com/docs/)
- [Liquid Template Guide](https://shopify.github.io/liquid/)

"#.to_string()
}

fn generate_contributing_guide() -> String {
    r#"# Contributing to this Documentation

Thank you for your interest in contributing to this documentation!

## Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature-name`)
3. Make your changes
4. Test your changes by running the site locally
5. Commit your changes (`git commit -am 'Add some feature'`)
6. Push to the branch (`git push origin feature/your-feature-name`)
7. Create a new Pull Request

## Local Development

To serve the site locally:

```
bundle exec jekyll serve
```

The site will be available at [http://localhost:4000](http://localhost:4000).

## Content Guidelines

### Markdown

All documentation pages are written in Markdown. Here are some basic formatting guidelines:

- Use `#` for headings, starting with `##` (as `#` is reserved for the page title)
- Use backticks for inline code: `code`
- Use triple backticks for code blocks:

```
code block
```

### Code Examples

When adding code examples:

- Use language-specific syntax highlighting by specifying the language after the opening triple backticks
- Keep examples concise and focused
- Include comments for complex sections

### Images

- Place images in the `assets/images/` directory
- Use relative paths in your Markdown: `![Alt text]({{ site.baseurl }}/assets/images/filename.png)`
- Optimize images before adding them to the repository

## Style Guide

- Use sentence case for headings
- Be concise and direct in your writing
- Use active voice when possible
- Break up long paragraphs for readability

"#.to_string()
} 