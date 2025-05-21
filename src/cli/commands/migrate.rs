use std::path::PathBuf;
use crate::cli::types::Commands;
use crate::migrate;

pub async fn handle_migrate_command(
    command: &Commands,
    source_dir: Option<&PathBuf>,
    destination_dir: Option<&PathBuf>
) {
    if let Commands::Migrate { source, destination, engine, verbose, clean } = command {
        // Determine source and destination directories
        let source_dir = if let Some(s) = source {
            s.clone()
        } else if let Some(s) = source_dir {
            s.clone()
        } else {
            PathBuf::from("./")
        };
        
        let destination_dir = if let Some(d) = destination {
            d.clone()
        } else if let Some(d) = destination_dir {
            d.clone()
        } else {
            PathBuf::from("./rustyll-site")
        };
        
        if *verbose {
            log::info!("Migrating site from {} engine", engine);
            log::info!("Source directory: {}", source_dir.display());
            log::info!("Destination directory: {}", destination_dir.display());
        }
        
        // Check if the source directory exists
        if !source_dir.exists() {
            log::error!("Source directory does not exist: {}", source_dir.display());
            return;
        }
        
        // Get the appropriate migrator based on the engine
        match migrate::get_migrator(engine) {
            Some(migrator) => {
                // Set up migration options
                let options = migrate::MigrationOptions {
                    source_dir: source_dir.clone(),
                    destination_dir: destination_dir.clone(),
                    verbose: *verbose,
                    clean: *clean,
                };
                
                // Perform the migration
                match migrator.migrate(&options) {
                    Ok(result) => {
                        // Generate the migration report
                        match migrate::generate_migration_report(&result, &destination_dir) {
                            Ok(report_path) => {
                                if *verbose {
                                    log::info!("Changes made: {}", result.changes.len());
                                    log::info!("Warnings: {}", result.warnings.len());
                                }
                                
                                log::info!("Migration completed successfully.");
                                log::info!("Migration report generated at: {}", report_path.display());
                                log::info!("Review the report for details on the migration.");
                            },
                            Err(e) => log::error!("Failed to generate migration report: {}", e),
                        }
                    },
                    Err(e) => log::error!("Migration failed: {}", e),
                }
            },
            None => {
                log::error!("Unsupported engine: {}", engine);
                log::info!("Try checking for the engine in the source directory...");
                
                // Try to detect the engine
                match migrate::detect_engine(&source_dir) {
                    Some(detected_engine) => {
                        log::info!("Detected engine: {}. Please run the command with --engine={}", 
                            detected_engine, detected_engine.to_lowercase());
                    },
                    None => {
                        log::error!("Could not detect the source engine. Please specify the engine with --engine");
                        log::info!("Supported engines: jekyll, hugo, zola, eleventy, gatsby, docsy, mdbook, mkdocs, gitbook, slate, pelican, nanoc, middleman, assemble, bridgetown, cobalt, fresh, harp, jigsaw, metalsmith, nikola, octopress, sphinx");
                    }
                }
            }
        }
    }
} 