use std::path::Path;
use std::fs;
use toml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_preprocessors(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook preprocessors...");
    }

    // Read book.toml to find preprocessors
    let book_toml_path = source_dir.join("book.toml");
    if !book_toml_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&book_toml_path)
        .map_err(|e| format!("Failed to read book.toml: {}", e))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse book.toml: {}", e))?;

    // Create _plugins directory
    let plugins_dir = dest_dir.join("_plugins");
    fs::create_dir_all(&plugins_dir)
        .map_err(|e| format!("Failed to create _plugins directory: {}", e))?;

    // Handle each preprocessor
    if let Some(preprocessors) = toml_value.get("preprocessor") {
        if let Some(table) = preprocessors.as_table() {
            for (name, config) in table {
                migrate_preprocessor(name, config, &plugins_dir, verbose, result)?;
            }
        }
    }

    Ok(())
}

fn migrate_preprocessor(
    name: &str,
    config: &toml::Value,
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    match name {
        "links" => migrate_links_preprocessor(plugins_dir, verbose, result)?,
        "index" => migrate_index_preprocessor(plugins_dir, verbose, result)?,
        "mermaid" => migrate_mermaid_preprocessor(config, plugins_dir, verbose, result)?,
        "katex" => migrate_katex_preprocessor(config, plugins_dir, verbose, result)?,
        _ => {
            if verbose {
                log::warn!("Unsupported preprocessor: {}", name);
            }
            result.warnings.push(format!("Unsupported preprocessor: {}", name));
        }
    }

    Ok(())
}

fn migrate_links_preprocessor(
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Links preprocessor equivalent
Jekyll::Hooks.register :site, :post_render do |site|
  site.documents.each do |doc|
    doc.output.gsub!(/\[([^\]]+)\]\(([^)]+)\)/) do |match|
      text = $1
      link = $2
      if link.start_with?('./')
        link = link[2..]
      end
      "[#{text}](#{link})"
    end
  end
end
"#;

    fs::write(plugins_dir.join("links.rb"), content)
        .map_err(|e| format!("Failed to write links preprocessor: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/links.rb".into(),
        description: "Created links preprocessor plugin".into(),
    });

    Ok(())
}

fn migrate_index_preprocessor(
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Index preprocessor equivalent
module Jekyll
  class IndexGenerator < Generator
    safe true
    priority :low

    def generate(site)
      site.pages.each do |page|
        next unless page.content =~ /<!-- index -->/
        index = build_index(site.pages)
        page.content.gsub!(/<!-- index -->/, index)
      end
    end

    private

    def build_index(pages)
      index = []
      pages.each do |page|
        next unless page.data['title']
        index << "* [#{page.data['title']}](#{page.url})"
      end
      index.join("\n")
    end
  end
end
"#;

    fs::write(plugins_dir.join("index.rb"), content)
        .map_err(|e| format!("Failed to write index preprocessor: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/index.rb".into(),
        description: "Created index preprocessor plugin".into(),
    });

    Ok(())
}

fn migrate_mermaid_preprocessor(
    config: &toml::Value,
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# Mermaid preprocessor equivalent
module Jekyll
  class MermaidBlock < Liquid::Block
    def render(context)
      text = super
      "<div class=\"mermaid\">\n#{text}\n</div>"
    end
  end
end

Liquid::Template.register_tag('mermaid', Jekyll::MermaidBlock)
"#;

    fs::write(plugins_dir.join("mermaid.rb"), content)
        .map_err(|e| format!("Failed to write mermaid preprocessor: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/mermaid.rb".into(),
        description: "Created mermaid preprocessor plugin".into(),
    });

    Ok(())
}

fn migrate_katex_preprocessor(
    config: &toml::Value,
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"
# KaTeX preprocessor equivalent
module Jekyll
  class KatexBlock < Liquid::Block
    def render(context)
      text = super
      "<div class=\"katex\">\n#{text}\n</div>"
    end
  end
end

Liquid::Template.register_tag('katex', Jekyll::KatexBlock)
"#;

    fs::write(plugins_dir.join("katex.rb"), content)
        .map_err(|e| format!("Failed to write katex preprocessor: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_plugins/katex.rb".into(),
        description: "Created katex preprocessor plugin".into(),
    });

    Ok(())
} 