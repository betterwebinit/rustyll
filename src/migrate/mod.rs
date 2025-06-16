use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::Write;
use chrono::Local;

// Import engines module
mod engines;

// Re-export engines
pub use engines::jekyll::JekyllMigrator;
pub use engines::hugo::HugoMigrator;
pub use engines::zola::ZolaMigrator;
pub use engines::eleventy::EleventyMigrator;
pub use engines::gatsby::GatsbyMigrator;
pub use engines::docsy::DocsyMigrator;
pub use engines::mdbook::MdBookMigrator;
pub use engines::mkdocs::MkdocsMigrator;
pub use engines::gitbook::GitbookMigrator;
pub use engines::slate::SlateMigrator;
pub use engines::pelican::PelicanMigrator;
pub use engines::nanoc::NanocMigrator;
pub use engines::middleman::MiddlemanMigrator;
pub use engines::assemble::AssembleMigrator;
pub use engines::bridgetown::BridgetownMigrator;
pub use engines::cobalt::CobaltMigrator;
pub use engines::fresh::FreshMigrator;
pub use engines::harp::HarpMigrator;
pub use engines::jigsaw::JigsawMigrator;
pub use engines::metalsmith::MetalsmithMigrator;
pub use engines::nikola::NikolaMigrator;
pub use engines::octopress::OctopressMigrator;

// Migration options
pub struct MigrationOptions {
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub verbose: bool,
    pub clean: bool,
}

// Migration result containing changes made
pub struct MigrationResult {
    pub engine_name: String,
    pub changes: Vec<MigrationChange>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

// Individual migration change
pub struct MigrationChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub description: String,
}

// Types of changes that can occur during migration
pub enum ChangeType {
    Created,
    Modified,
    Converted,
    Copied,
    Ignored,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Created => write!(f, "Created"),
            ChangeType::Modified => write!(f, "Modified"),
            ChangeType::Converted => write!(f, "Converted"),
            ChangeType::Copied => write!(f, "Copied"),
            ChangeType::Ignored => write!(f, "Ignored"),
        }
    }
}

// Engine trait that all migrators must implement
pub trait EngineMigrator {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detect(&self, source_dir: &Path) -> bool;
    fn migrate(&self, options: &MigrationOptions) -> Result<MigrationResult, String>;
}

// Function to get the appropriate migrator based on engine name
pub fn get_migrator(engine: &str) -> Option<Box<dyn EngineMigrator>> {
    match engine.to_lowercase().as_str() {
        "jekyll" => Some(Box::new(JekyllMigrator::new())),
        "hugo" => Some(Box::new(HugoMigrator::new())),
        "zola" => Some(Box::new(ZolaMigrator::new())),
        "eleventy" | "11ty" => Some(Box::new(EleventyMigrator::new())),
        "gatsby" => Some(Box::new(GatsbyMigrator::new())),
        "docsy" => Some(Box::new(DocsyMigrator::new())),
        "mdbook" => Some(Box::new(MdBookMigrator::new())),
        "mkdocs" => Some(Box::new(MkdocsMigrator::new())),
        "gitbook" => Some(Box::new(GitbookMigrator::new())),
        "slate" => Some(Box::new(SlateMigrator::new())),
        "pelican" => Some(Box::new(PelicanMigrator::new())),
        "nanoc" => Some(Box::new(NanocMigrator::new())),
        "middleman" => Some(Box::new(MiddlemanMigrator::new())),
        "assemble" => Some(Box::new(AssembleMigrator::new())),
        "bridgetown" => Some(Box::new(BridgetownMigrator::new())),
        "cobalt" => Some(Box::new(CobaltMigrator::new())),
        "fresh" => Some(Box::new(FreshMigrator::new())),
        "harp" => Some(Box::new(HarpMigrator::new())),
        "jigsaw" => Some(Box::new(JigsawMigrator::new())),
        "metalsmith" => Some(Box::new(MetalsmithMigrator::new())),
        "nikola" => Some(Box::new(NikolaMigrator::new())),
        "octopress" => Some(Box::new(OctopressMigrator::new())),
        _ => None,
    }
}

// Function to detect the engine type from the source directory
pub fn detect_engine(source_dir: &Path) -> Option<String> {
    let engines: Vec<Box<dyn EngineMigrator>> = vec![
        Box::new(JekyllMigrator::new()),
        Box::new(HugoMigrator::new()),
        Box::new(ZolaMigrator::new()),
        Box::new(EleventyMigrator::new()),
        Box::new(GatsbyMigrator::new()),
        Box::new(DocsyMigrator::new()),
        Box::new(MdBookMigrator::new()),
        Box::new(MkdocsMigrator::new()),
        Box::new(GitbookMigrator::new()),
        Box::new(SlateMigrator::new()),
        Box::new(PelicanMigrator::new()),
        Box::new(NanocMigrator::new()),
        Box::new(MiddlemanMigrator::new()),
        Box::new(AssembleMigrator::new()),
        Box::new(BridgetownMigrator::new()),
        Box::new(CobaltMigrator::new()),
        Box::new(FreshMigrator::new()),
        Box::new(HarpMigrator::new()),
        Box::new(JigsawMigrator::new()),
        Box::new(MetalsmithMigrator::new()),
        Box::new(NikolaMigrator::new()),
        Box::new(OctopressMigrator::new()),
    ];

    for engine in engines {
        if engine.detect(source_dir) {
            return Some(engine.name().to_string());
        }
    }

    None
}

// Helper function to create a directory if it doesn't exist
pub fn create_dir_if_not_exists(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;
    }
    Ok(())
}

// Helper function to copy a file
pub fn copy_file(src: &Path, dest: &Path) -> Result<(), String> {
    fs::copy(src, dest)
        .map_err(|e| format!("Failed to copy file from {} to {}: {}", 
                            src.display(), dest.display(), e))?;
    Ok(())
}

// Helper function to write README files for directories
pub fn write_readme(dir: &Path, content: &str) -> Result<(), String> {
    let readme_path = dir.join("README.md");
    let mut file = File::create(&readme_path)
        .map_err(|e| format!("Failed to create README file at {}: {}", readme_path.display(), e))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to README file at {}: {}", readme_path.display(), e))?;
    
    Ok(())
}

// Function to generate the migration report
pub fn generate_migration_report(result: &MigrationResult, dest_dir: &Path) -> Result<PathBuf, String> {
    let report_path = dest_dir.join("MIGRATION.md");
    let mut file = File::create(&report_path)
        .map_err(|e| format!("Failed to create migration report: {}", e))?;
    
    let datetime = Local::now().format("%Y-%m-%d %H:%M:%S");
    let report_content = format!(
        r#"# Migration Report

## Overview
- **Source Engine**: {}
- **Migration Date**: {}
- **Total Changes**: {}
- **Warnings**: {}

## Changes

| File | Type | Description |
|------|------|-------------|
{}

## Warnings

{}

## Next Steps

1. Review the migrated content to ensure everything was converted correctly.
2. Run `rustyll build` to build your site.
3. Address any warnings listed above.
4. Check the README.md files in each directory for specific guidance about the migrated components.

"#,
        result.engine_name,
        datetime,
        result.changes.len(),
        result.warnings.len(),
        result.changes.iter()
            .map(|c| format!("| {} | {} | {} |", c.file_path, c.change_type, c.description))
            .collect::<Vec<String>>()
            .join("\n"),
        result.warnings.iter()
            .map(|w| format!("- {}", w))
            .collect::<Vec<String>>()
            .join("\n")
    );
    
    file.write_all(report_content.as_bytes())
        .map_err(|e| format!("Failed to write migration report: {}", e))?;
    
    Ok(report_path)
}

/// Return the list of supported engine names
pub fn list_engines() -> Vec<&'static str> {
    vec![
        "jekyll", "hugo", "zola", "eleventy", "gatsby", "docsy", "mdbook",
        "mkdocs", "gitbook", "slate", "pelican", "nanoc", "middleman",
        "assemble", "bridgetown", "cobalt", "fresh", "harp", "jigsaw",
        "metalsmith", "nikola", "octopress",
    ]
}
