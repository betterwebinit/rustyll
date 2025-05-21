use axum::{
    Router,
    http::{StatusCode, header},
    body::Body,
    response::{Response, IntoResponse, Html},
};
use tower_http::services::ServeDir;
use std::path::{Path as FilePath, PathBuf};
use log::{debug, error};
use std::fs;
use std::io::ErrorKind;

/// Jekyll-compatible directory index file names
const DIRECTORY_INDEX: [&str; 9] = [
    "index.htm",
    "index.html",
    "index.rhtml",
    "index.xht",
    "index.xhtml",
    "index.cgi",
    "index.xml",
    "index.json",
    "index.md",
];

/// Create a router for serving static files from a directory
pub fn create_static_files_handler(
    directory: PathBuf, 
    show_dir_listing: bool
) -> Router {
    let serve_dir = ServeDir::new(&directory)
        .precompressed_gzip()
        .precompressed_br()
        .append_index_html_on_directories(!show_dir_listing);
    
    Router::new().nest_service("/", serve_dir)
}

/// Find an index file in the directory, trying all Jekyll-compatible index names
pub async fn find_index_file(directory: &FilePath) -> Option<Response> {
    for index_name in &DIRECTORY_INDEX {
        let index_path = directory.join(index_name);
        if index_path.exists() && index_path.is_file() {
            debug!("Found index file: {}", index_path.display());
            match fs::read(&index_path) {
                Ok(content) => {
                    let mime_type = mime_guess::from_path(&index_path)
                        .first_or_text_plain();
                    
                    let response = Response::builder()
                        .header(header::CONTENT_TYPE, mime_type.as_ref())
                        .body(Body::from(content))
                        .unwrap();
                    
                    return Some(response);
                }
                Err(e) => {
                    error!("Error reading index file: {}", e);
                }
            }
        }
    }
    None
}

/// Handle 404 errors with a custom error page if available
pub fn handle_not_found(root_dir: &FilePath) -> Response {
    // Try to serve a custom 404.html file if it exists
    let custom_404 = root_dir.join("404.html");
    if custom_404.exists() {
        debug!("Using custom 404 page: {}", custom_404.display());
        match fs::read(&custom_404) {
            Ok(content) => {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(content))
                    .unwrap();
            }
            Err(e) => {
                error!("Error reading 404.html: {}", e);
            }
        }
    }
    
    // Fall back to a simple 404 message
    debug!("No custom 404 page found, using default response");
    (StatusCode::NOT_FOUND, "Page not found").into_response()
}

/// Handle directory listing requests
pub fn create_directory_listing(dir: &FilePath, site_root: &FilePath) -> Result<Response, std::io::Error> {
    if !dir.is_dir() {
        return Ok((StatusCode::NOT_FOUND, "Directory not found").into_response());
    }
    
    let entries = fs::read_dir(dir).map_err(|e| {
        error!("Failed to read directory {}: {}", dir.display(), e);
        std::io::Error::new(ErrorKind::Other, "Failed to read directory")
    })?;
    
    // Build simple HTML directory listing
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<title>Directory listing</title>\n");
    html.push_str("<style>body {font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px;} ");
    html.push_str("h1 {border-bottom: 1px solid #ddd; padding-bottom: 10px;} ");
    html.push_str("ul {list-style-type: none; padding: 0;} ");
    html.push_str("li {margin: 5px 0; padding: 5px; border-radius: 3px;} ");
    html.push_str("li:hover {background-color: #f5f5f5;} ");
    html.push_str("a {text-decoration: none; color: #0366d6;}</style>\n");
    html.push_str("</head>\n<body>\n");
    
    // Get relative path from site root
    let rel_path = dir.strip_prefix(site_root).unwrap_or(dir);
    html.push_str(&format!("<h1>Directory: /{}</h1>\n", rel_path.display()));
    
    html.push_str("<ul>\n");
    
    // Add parent directory link if we're not at the root
    if rel_path != FilePath::new("") {
        html.push_str("<li><a href=\"../\">../</a> (Parent Directory)</li>\n");
    }
    
    // Sort entries: directories first, then files
    let mut entries_vec = entries
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    
    entries_vec.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
    
    // Add entries to the listing
    for entry in entries_vec {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let display_name = if is_dir {
            format!("{}/", file_name_str)
        } else {
            file_name_str.to_string()
        };
        
        html.push_str(&format!(
            "<li><a href=\"{}\">{}{}",
            file_name_str,
            display_name,
            if is_dir {
                "</a> (Directory)"
            } else {
                "</a>"
            }
        ));
        
        if !is_dir {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                let size_str = if size < 1024 {
                    format!("{} bytes", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                };
                html.push_str(&format!(" - {}", size_str));
            }
        }
        
        html.push_str("</li>\n");
    }
    
    html.push_str("</ul>\n");
    html.push_str("</body>\n</html>");
    
    Ok(Html(html).into_response())
} 