use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_search(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook search functionality...");
    }

    // Create search plugin directory
    let search_dir = dest_dir.join("_plugins/search");
    fs::create_dir_all(&search_dir)
        .map_err(|e| format!("Failed to create search directory: {}", e))?;

    // Create search plugin
    create_search_plugin(&search_dir, result)?;

    // Create search layout
    create_search_layout(dest_dir, result)?;

    // Create search assets
    create_search_assets(dest_dir, result)?;

    Ok(())
}

fn create_search_plugin(
    search_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Search plugin equivalent to MDBook's search functionality
require 'json'

module Jekyll
  class SearchIndexGenerator < Generator
    safe true
    priority :low

    def generate(site)
      # Generate search index
      index = []
      
      site.pages.each do |page|
        next if page.data['exclude_from_search']
        
        index << {
          'title' => page.data['title'] || page.name,
          'url' => page.url,
          'content' => extract_searchable_content(page.content),
          'summary' => page.data['description'] || extract_summary(page.content)
        }
      end

      # Write search index to a JSON file
      index_file = File.join(site.dest, 'assets', 'search_index.json')
      FileUtils.mkdir_p(File.dirname(index_file))
      File.write(index_file, JSON.generate(index))
    end

    private

    def extract_searchable_content(content)
      # Remove HTML tags and normalize whitespace
      content.gsub(/<[^>]+>/, ' ')
             .gsub(/\s+/, ' ')
             .strip
    end

    def extract_summary(content)
      # Extract first paragraph or first 200 characters
      first_para = content.split(/\n\n/).first || ''
      summary = extract_searchable_content(first_para)
      summary.length > 200 ? summary[0...197] + '...' : summary
    end
  end
end
"#;

    fs::write(search_dir.join("search_index_generator.rb"), content)
        .map_err(|e| format!("Failed to write search plugin: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/search/search_index_generator.rb".into(),
        description: "Created search index generator plugin".into(),
    });

    Ok(())
}

fn create_search_layout(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"---
layout: default
---
<div class="search-container">
  <input type="text" id="search-input" placeholder="Search..." aria-label="Search">
  <div id="search-results"></div>
</div>

<script src="{{ '/assets/js/search.js' | relative_url }}"></script>
<script>
  window.searchIndex = '{{ "/assets/search_index.json" | relative_url }}';
</script>
"#;

    let layouts_dir = dest_dir.join("_layouts");
    fs::create_dir_all(&layouts_dir)
        .map_err(|e| format!("Failed to create layouts directory: {}", e))?;

    fs::write(layouts_dir.join("search.html"), content)
        .map_err(|e| format!("Failed to write search layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/search.html".into(),
        description: "Created search page layout".into(),
    });

    Ok(())
}

fn create_search_assets(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let js_content = r#"
// Search functionality
class BookSearch {
  constructor() {
    this.searchInput = document.getElementById('search-input');
    this.searchResults = document.getElementById('search-results');
    this.searchIndex = [];
    
    this.loadSearchIndex();
    this.bindEvents();
  }

  async loadSearchIndex() {
    try {
      const response = await fetch(window.searchIndex);
      this.searchIndex = await response.json();
    } catch (error) {
      console.error('Failed to load search index:', error);
    }
  }

  bindEvents() {
    this.searchInput.addEventListener('input', () => this.performSearch());
  }

  performSearch() {
    const query = this.searchInput.value.toLowerCase();
    if (query.length < 2) {
      this.searchResults.innerHTML = '';
      return;
    }

    const results = this.searchIndex.filter(item => {
      const titleMatch = item.title.toLowerCase().includes(query);
      const contentMatch = item.content.toLowerCase().includes(query);
      return titleMatch || contentMatch;
    });

    this.displayResults(results);
  }

  displayResults(results) {
    if (results.length === 0) {
      this.searchResults.innerHTML = '<p>No results found</p>';
      return;
    }

    const html = results.map(item => `
      <div class="search-result">
        <h3><a href="${item.url}">${item.title}</a></h3>
        <p>${item.summary}</p>
      </div>
    `).join('');

    this.searchResults.innerHTML = html;
  }
}

document.addEventListener('DOMContentLoaded', () => {
  new BookSearch();
});
"#;

    let css_content = r#"
/* Search styles */
.search-container {
  margin: 2rem auto;
  max-width: 800px;
  padding: 0 1rem;
}

#search-input {
  width: 100%;
  padding: 0.5rem;
  font-size: 1.1rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.search-result {
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid #eee;
  border-radius: 4px;
}

.search-result h3 {
  margin: 0 0 0.5rem 0;
}

.search-result p {
  margin: 0;
  color: #666;
}

.search-result a {
  color: #0366d6;
  text-decoration: none;
}

.search-result a:hover {
  text-decoration: underline;
}
"#;

    // Create assets directories
    let js_dir = dest_dir.join("assets/js");
    let css_dir = dest_dir.join("assets/css");
    fs::create_dir_all(&js_dir)
        .map_err(|e| format!("Failed to create js directory: {}", e))?;
    fs::create_dir_all(&css_dir)
        .map_err(|e| format!("Failed to create css directory: {}", e))?;

    // Write JavaScript file
    fs::write(js_dir.join("search.js"), js_content)
        .map_err(|e| format!("Failed to write search.js: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/js/search.js".into(),
        description: "Created search JavaScript".into(),
    });

    // Write CSS file
    fs::write(css_dir.join("search.css"), css_content)
        .map_err(|e| format!("Failed to write search.css: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/search.css".into(),
        description: "Created search styles".into(),
    });

    Ok(())
} 