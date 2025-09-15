use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_renderers(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook renderers...");
    }

    // Create renderers directory
    let renderers_dir = dest_dir.join("_plugins/renderers");
    fs::create_dir_all(&renderers_dir)
        .map_err(|e| format!("Failed to create renderers directory: {}", e))?;

    // Migrate HTML renderer
    migrate_html_renderer(&renderers_dir, verbose, result)?;

    // Check for additional renderers in book.toml
    let book_toml_path = source_dir.join("book.toml");
    if book_toml_path.exists() {
        let content = fs::read_to_string(&book_toml_path)
            .map_err(|e| format!("Failed to read book.toml: {}", e))?;

        let toml_value: toml::Value = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse book.toml: {}", e))?;

        if let Some(renderers) = toml_value.get("output") {
            if let Some(table) = renderers.as_table() {
                for (name, config) in table {
                    match name.as_str() {
                        "html" => (), // Already handled
                        "markdown" => migrate_markdown_renderer(&renderers_dir, config, verbose, result)?,
                        "manpage" => migrate_manpage_renderer(&renderers_dir, config, verbose, result)?,
                        _ => {
                            if verbose {
                                log::warn!("Unsupported renderer: {}", name);
                            }
                            result.warnings.push(format!("Unsupported renderer: {}", name));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn migrate_html_renderer(
    renderers_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# HTML renderer equivalent
module Jekyll
  class HTMLRenderer < Converter
    safe true
    priority :low

    def matches(ext)
      ext =~ /^\.html$/i
    end

    def output_ext(ext)
      ".html"
    end

    def convert(content)
      content
    end
  end
end
"#;

    fs::write(renderers_dir.join("html.rb"), content)
        .map_err(|e| format!("Failed to write HTML renderer: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/renderers/html.rb".into(),
        description: "Created HTML renderer plugin".into(),
    });

    Ok(())
}

fn migrate_markdown_renderer(
    renderers_dir: &Path,
    config: &toml::Value,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Markdown renderer equivalent
module Jekyll
  class MarkdownRenderer < Converter
    safe true
    priority :high

    def matches(ext)
      ext =~ /^\.md$/i
    end

    def output_ext(ext)
      ".html"
    end

    def convert(content)
      Jekyll::Converters::Markdown.new(@config).convert(content)
    end
  end
end
"#;

    fs::write(renderers_dir.join("markdown.rb"), content)
        .map_err(|e| format!("Failed to write Markdown renderer: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/renderers/markdown.rb".into(),
        description: "Created Markdown renderer plugin".into(),
    });

    Ok(())
}

fn migrate_manpage_renderer(
    renderers_dir: &Path,
    config: &toml::Value,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Manpage renderer equivalent
module Jekyll
  class ManpageRenderer < Converter
    safe true
    priority :low

    def matches(ext)
      ext =~ /^\.(1|2|3|4|5|6|7|8|9)$/i
    end

    def output_ext(ext)
      ".html"
    end

    def convert(content)
      # Convert manpage format to HTML
      # This is a basic implementation that should be enhanced based on needs
      "<pre class=\"manpage\">\n#{content}\n</pre>"
    end
  end
end
"#;

    fs::write(renderers_dir.join("manpage.rb"), content)
        .map_err(|e| format!("Failed to write Manpage renderer: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/renderers/manpage.rb".into(),
        description: "Created Manpage renderer plugin".into(),
    });

    Ok(())
} 