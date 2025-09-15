use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook content...");
    }

    // Create content directories
    let content_dir = dest_dir.join("content");
    fs::create_dir_all(&content_dir)
        .map_err(|e| format!("Failed to create content directory: {}", e))?;

    // Process SUMMARY.md first to understand the book structure
    let summary_path = source_dir.join("src/SUMMARY.md");
    if summary_path.exists() {
        migrate_summary(&summary_path, &content_dir, verbose, result)?;
    }

    // Migrate all markdown files from src directory
    let src_dir = source_dir.join("src");
    if src_dir.exists() {
        for entry in WalkDir::new(&src_dir).min_depth(1) {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "md") {
                // Skip SUMMARY.md as it's handled separately
                if path.file_name().map_or(false, |name| name == "SUMMARY.md") {
                    continue;
                }

                let relative_path = path.strip_prefix(&src_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = content_dir.join(relative_path);

                // Create parent directories if needed
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                // Convert and copy the markdown file
                migrate_markdown_file(path, &dest_path, verbose, result)?;
            }
        }
    }

    Ok(())
}

fn migrate_summary(
    summary_path: &Path,
    content_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(summary_path)
        .map_err(|e| format!("Failed to read SUMMARY.md: {}", e))?;

    // Convert SUMMARY.md to _data/toc.yml
    let toc_content = convert_summary_to_toc(&content)?;
    let data_dir = content_dir.parent().unwrap().join("_data");
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create _data directory: {}", e))?;

    fs::write(data_dir.join("toc.yml"), toc_content)
        .map_err(|e| format!("Failed to write toc.yml: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/toc.yml".into(),
        description: "Created table of contents from SUMMARY.md".into(),
    });

    Ok(())
}

fn convert_summary_to_toc(content: &str) -> Result<String, String> {
    let mut toc_lines = Vec::new();
    toc_lines.push("# Table of contents converted from SUMMARY.md".to_string());
    toc_lines.push("toc:".to_string());

    for line in content.lines() {
        if let Some(link) = parse_markdown_link(line) {
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let spaces = "  ".repeat(indent / 2 + 1);
            toc_lines.push(format!("{}title: {}", spaces, link.title));
            toc_lines.push(format!("{}url: {}", spaces, link.url));
        }
    }

    Ok(toc_lines.join("\n"))
}

fn migrate_markdown_file(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read markdown file: {}", e))?;

    // Convert MDBook-specific markdown features
    let converted_content = convert_mdbook_markdown(&content)?;

    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write markdown file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: dest_path.to_string_lossy().into(),
        description: format!("Converted markdown file from {}", source_path.display()),
    });

    Ok(())
}

fn convert_mdbook_markdown(content: &str) -> Result<String, String> {
    let mut converted = String::new();
    let mut in_frontmatter = false;
    let mut has_frontmatter = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !has_frontmatter {
                has_frontmatter = true;
                in_frontmatter = !in_frontmatter;
                converted.push_str("---\n");
                if !in_frontmatter {
                    converted.push_str("layout: page\n");
                }
            } else {
                in_frontmatter = !in_frontmatter;
                converted.push_str("---\n");
            }
            continue;
        }

        if !has_frontmatter && !line.trim().is_empty() {
            // Add frontmatter if not present
            converted.push_str("---\nlayout: page\n---\n\n");
            has_frontmatter = true;
        }

        // Convert MDBook-specific syntax
        let processed_line = if in_frontmatter {
            line.to_string()
        } else {
            convert_mdbook_line(line)
        };

        converted.push_str(&processed_line);
        converted.push('\n');
    }

    Ok(converted)
}

fn convert_mdbook_line(line: &str) -> String {
    let mut result = line.to_string();

    // Convert MDBook-specific syntax to Jekyll/Rustyll equivalents
    result = result.replace("{{#playground", "{% highlight rust %}");
    result = result.replace("{{/playground}}", "{% endhighlight %}");
    result = result.replace("{{#include", "{% include_relative");
    result = result.replace("}}", "%}");

    result
}

struct MarkdownLink {
    title: String,
    url: String,
}

fn parse_markdown_link(line: &str) -> Option<MarkdownLink> {
    let link_start = line.find('[')?;
    let link_end = line.find(']')?;
    let url_start = line.find('(')?;
    let url_end = line.find(')')?;

    if link_start < link_end && url_start < url_end && link_end < url_start {
        Some(MarkdownLink {
            title: line[link_start + 1..link_end].to_string(),
            url: line[url_start + 1..url_end].to_string(),
        })
    } else {
        None
    }
} 