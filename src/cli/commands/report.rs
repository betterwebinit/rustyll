use std::path::PathBuf;
use std::fs;
use crate::cli::types::Commands;
use crate::report::{self, ReportOptions};

pub async fn handle_report_command(
    command: &Commands,
    source_dir: Option<&PathBuf>
) {
    if let Commands::Report { source, verbose, console, output } = command {
        // Determine the source directory
        let source_dir = if let Some(s) = source {
            s.clone()
        } else if let Some(s) = source_dir {
            s.clone()
        } else {
            PathBuf::from("./_site")
        };
        
        if *verbose {
            log::info!("Generating report for {}", source_dir.display());
        }
        
        // Check if the source directory exists
        if !source_dir.exists() {
            log::error!("Source directory does not exist: {}", source_dir.display());
            log::error!("Make sure to build your site before generating a report.");
            return;
        }
        
        // Set up report options
        let options = ReportOptions {
            verbose: *verbose,
            console_output: *console,
        };
        
        // Generate the report
        match report::generate_report(&source_dir, options).await {
            Ok(report) => {
                if *console {
                    // Output to console
                    let console_report = report::generate_console_report(&report, *verbose);
                    println!("{}", console_report);
                } else {
                    // Generate HTML report
                    let html_report = report::generate_html_report(&report);
                    
                    // Write the report to the output file
                    match fs::write(output, html_report) {
                        Ok(_) => {
                            log::info!("Report generated successfully at {}", output.display());
                        },
                        Err(e) => {
                            log::error!("Failed to write report to {}: {}", output.display(), e);
                        }
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to generate report: {}", e);
            }
        }
    }
} 