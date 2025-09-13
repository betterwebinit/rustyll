mod lighthouse;
mod accessibility;
mod links;
mod seo;
mod performance;


use std::path::Path;
use std::time::Duration;
use crate::config::Config;

pub struct ReportOptions {
    pub verbose: bool,
    pub console_output: bool,
}

/// Site report structure
pub struct SiteReport {
    /// Site title
    pub title: String,
    /// Build time
    pub build_time: Duration,
    /// Number of pages
    pub num_pages: usize,
    /// Number of posts
    pub num_posts: usize,
    /// Number of collections
    pub num_collections: usize,
    pub lighthouse_score: Option<f32>,
    pub accessibility_issues: Vec<String>,
    pub broken_links: Vec<String>,
    pub seo_issues: Vec<String>,
    pub performance_issues: Vec<(String, String)>, // (file_path, issue)
}

pub async fn generate_report(source_dir: &Path, options: ReportOptions) -> Result<SiteReport, String> {
    let mut report = SiteReport {
        title: String::new(),
        build_time: Duration::new(0, 0),
        num_pages: 0,
        num_posts: 0,
        num_collections: 0,
        lighthouse_score: None,
        accessibility_issues: Vec::new(),
        broken_links: Vec::new(),
        seo_issues: Vec::new(),
        performance_issues: Vec::new(),
    };

    // Run all checks
    if let Ok(score) = lighthouse::check_lighthouse(source_dir, options.verbose).await {
        report.lighthouse_score = Some(score);
    }
    
    if let Ok(issues) = accessibility::check_accessibility(source_dir, options.verbose).await {
        report.accessibility_issues = issues;
    }
    
    if let Ok(links) = links::check_broken_links(source_dir, options.verbose).await {
        report.broken_links = links;
    }
    
    if let Ok(issues) = seo::check_seo(source_dir, options.verbose).await {
        report.seo_issues = issues;
    }
    
    if let Ok(issues) = performance::check_performance(source_dir, options.verbose).await {
        report.performance_issues = issues;
    }

    Ok(report)
}

pub fn generate_html_report(report: &SiteReport) -> String {
    // Simple HTML report
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rustyll Site Report</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 1200px; margin: 0 auto; padding: 20px; }
        h1, h2, h3 { color: #333; }
        .section { margin-bottom: 30px; border: 1px solid #ddd; border-radius: 5px; padding: 15px; }
        .issue { background-color: #f8f8f8; padding: 10px; margin-bottom: 5px; border-left: 4px solid #ff9800; }
        .good { background-color: #e8f5e9; padding: 10px; margin-bottom: 5px; border-left: 4px solid #4caf50; }
        .score { font-size: 2em; font-weight: bold; }
        .high { color: green; }
        .medium { color: orange; }
        .low { color: red; }
        table { width: 100%; border-collapse: collapse; }
        th, td { text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <h1>Rustyll Site Report</h1>
"#);

    // Lighthouse score
    html.push_str("<div class='section'><h2>Lighthouse Score</h2>");
    if let Some(score) = report.lighthouse_score {
        let score_class = if score >= 90.0 {
            "high"
        } else if score >= 70.0 {
            "medium"
        } else {
            "low"
        };
        html.push_str(&format!("<p>Overall score: <span class='score {}'>{:.1}</span></p>", score_class, score));
    } else {
        html.push_str("<p>Lighthouse score not available</p>");
    }
    html.push_str("</div>");

    // Accessibility
    html.push_str("<div class='section'><h2>Accessibility Issues</h2>");
    if report.accessibility_issues.is_empty() {
        html.push_str("<p class='good'>No accessibility issues found!</p>");
    } else {
        html.push_str("<ul>");
        for issue in &report.accessibility_issues {
            html.push_str(&format!("<li class='issue'>{}</li>", issue));
        }
        html.push_str("</ul>");
    }
    html.push_str("</div>");

    // Broken Links
    html.push_str("<div class='section'><h2>Broken Links</h2>");
    if report.broken_links.is_empty() {
        html.push_str("<p class='good'>No broken links found!</p>");
    } else {
        html.push_str("<ul>");
        for link in &report.broken_links {
            html.push_str(&format!("<li class='issue'>{}</li>", link));
        }
        html.push_str("</ul>");
    }
    html.push_str("</div>");

    // SEO Issues
    html.push_str("<div class='section'><h2>SEO Issues</h2>");
    if report.seo_issues.is_empty() {
        html.push_str("<p class='good'>No SEO issues found!</p>");
    } else {
        html.push_str("<ul>");
        for issue in &report.seo_issues {
            html.push_str(&format!("<li class='issue'>{}</li>", issue));
        }
        html.push_str("</ul>");
    }
    html.push_str("</div>");

    // Performance Issues
    html.push_str("<div class='section'><h2>Performance Issues</h2>");
    if report.performance_issues.is_empty() {
        html.push_str("<p class='good'>No performance issues found!</p>");
    } else {
        html.push_str("<table><tr><th>File</th><th>Issue</th></tr>");
        for (file, issue) in &report.performance_issues {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", file, issue));
        }
        html.push_str("</table>");
    }
    html.push_str("</div>");

    html.push_str("</body></html>");
    html
}

pub fn generate_console_report(report: &SiteReport, verbose: bool) -> String {
    let mut output = String::from("Rustyll Site Report\n==================\n\n");

    // Lighthouse score
    output.push_str("Lighthouse Score:\n");
    if let Some(score) = report.lighthouse_score {
        output.push_str(&format!("  Overall score: {:.1}\n", score));
    } else {
        output.push_str("  Lighthouse score not available\n");
    }
    output.push_str("\n");

    // Accessibility
    output.push_str("Accessibility Issues:\n");
    if report.accessibility_issues.is_empty() {
        output.push_str("  No accessibility issues found!\n");
    } else {
        for issue in &report.accessibility_issues {
            output.push_str(&format!("  - {}\n", issue));
        }
    }
    output.push_str("\n");

    // Broken Links
    output.push_str("Broken Links:\n");
    if report.broken_links.is_empty() {
        output.push_str("  No broken links found!\n");
    } else {
        for link in &report.broken_links {
            output.push_str(&format!("  - {}\n", link));
        }
    }
    output.push_str("\n");

    // SEO Issues
    output.push_str("SEO Issues:\n");
    if report.seo_issues.is_empty() {
        output.push_str("  No SEO issues found!\n");
    } else {
        for issue in &report.seo_issues {
            output.push_str(&format!("  - {}\n", issue));
        }
    }
    output.push_str("\n");

    // Performance Issues
    output.push_str("Performance Issues:\n");
    if report.performance_issues.is_empty() {
        output.push_str("  No performance issues found!\n");
    } else {
        for (file, issue) in &report.performance_issues {
            output.push_str(&format!("  - {}: {}\n", file, issue));
        }
    }

    output
}

/// Generate a build report
pub fn generate_build_report(config: &Config, elapsed: std::time::Duration) -> Result<(), Box<dyn std::error::Error>> {
    let report = SiteReport {
        title: config.title.clone(),
        build_time: elapsed,
        num_pages: 0, // These would need to be passed in as parameters
        num_posts: 0,
        num_collections: config.collections.items.len(),
        lighthouse_score: None,
        accessibility_issues: Vec::new(),
        broken_links: Vec::new(),
        seo_issues: Vec::new(),
        performance_issues: Vec::new(),
    };
    
    let html = generate_html_report(&report);
    
    let report_path = Path::new(&config.destination).join("build-report.html");
    std::fs::write(report_path, html)?;
    
    Ok(())
} 