use crate::config::Config;
use crate::directory::{DirectoryStructure, clean_destination};
use crate::collections::{load_collections, load_data_files, collections_to_liquid};
use crate::liquid::{create_jekyll_parser, create_site_object};
use crate::markdown::MarkdownRenderer;
use crate::builder::page::{Page, collect_pages};
use crate::builder::types::BoxResult;
use crate::builder::site::{
    load_layouts, 
    load_includes, 
    process_collections, 
    process_pages,
    data_to_liquid
};

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use log::{info, debug, warn, error};
use liquid::model::Value;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// The incremental build cache structure
#[derive(Debug, Default, Serialize, Deserialize)]
struct IncrementalCache {
    /// Map of file paths to their last modification time
    file_mtimes: HashMap<PathBuf, SystemTime>,
    /// Map of file paths to their dependencies
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    /// Map of template paths to their dependents
    dependents: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl IncrementalCache {
    /// Load the cache from disk
    fn load(config: &Config) -> Self {
        let cache_path = Path::new(&config.source).join(&config.cache_dir).join("cache.json");
        if cache_path.exists() {
            match fs::read_to_string(&cache_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(cache) => return cache,
                        Err(e) => {
                            warn!("Failed to parse cache file: {}", e);
                            Self::default()
                        }
                    }
                },
                Err(e) => {
                    warn!("Failed to read cache file: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Save the cache to disk
    fn save(&self, config: &Config) -> BoxResult<()> {
        let cache_dir = Path::new(&config.source).join(&config.cache_dir);
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        
        let cache_path = cache_dir.join("cache.json");
        let json = serde_json::to_string(self)?;
        fs::write(cache_path, json)?;
        Ok(())
    }

    /// Check if a file has been modified since the last build
    fn is_modified(&self, path: &Path) -> bool {
        match fs::metadata(path) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(mtime) => {
                        match self.file_mtimes.get(path) {
                            Some(cached_mtime) => mtime > *cached_mtime,
                            None => true, // File not in cache, consider it modified
                        }
                    },
                    Err(_) => true, // Can't get mtime, consider it modified
                }
            },
            Err(_) => true, // Can't get metadata, consider it modified
        }
    }

    /// Check if a file or any of its dependencies have been modified
    fn needs_rebuild(&self, path: &Path) -> bool {
        // Check if the file itself has been modified
        if self.is_modified(path) {
            return true;
        }

        // Check if any of its dependencies have been modified
        if let Some(deps) = self.dependencies.get(path) {
            for dep in deps {
                if self.is_modified(dep) {
                    return true;
                }
            }
        }

        false
    }

    /// Update the file's modification time in the cache
    fn update_mtime(&mut self, path: &Path) {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(mtime) = metadata.modified() {
                self.file_mtimes.insert(path.to_path_buf(), mtime);
            }
        }
    }

    /// Add a dependency relationship between a file and its dependency
    fn add_dependency(&mut self, file: &Path, dependency: &Path) {
        self.dependencies
            .entry(file.to_path_buf())
            .or_insert_with(HashSet::new)
            .insert(dependency.to_path_buf());

        self.dependents
            .entry(dependency.to_path_buf())
            .or_insert_with(HashSet::new)
            .insert(file.to_path_buf());
    }

    /// Get all files that depend on the given file (directly or indirectly)
    fn get_affected_files(&self, path: &Path) -> HashSet<PathBuf> {
        let mut result = HashSet::new();
        let mut to_process = vec![path.to_path_buf()];

        while let Some(current) = to_process.pop() {
            if let Some(deps) = self.dependents.get(&current) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        to_process.push(dep.clone());
                    }
                }
            }
        }

        result
    }
}

/// Build statistics
#[derive(Debug, Default)]
struct BuildStats {
    /// Number of pages processed
    pages_count: usize,
    /// Number of documents processed
    documents_count: usize,
    /// Number of static files copied
    static_files_count: usize,
    /// Number of errors encountered
    errors_count: usize,
    /// Build duration
    duration: std::time::Duration,
}

/// Build a Jekyll-compatible static site
pub fn build_site(config: &Config, _include_drafts: bool, _include_unpublished: bool) -> BoxResult<()> {
    let start_time = std::time::Instant::now();
    let incremental = config.incremental.unwrap_or(false);
    
    // Setup build statistics
    let mut stats = BuildStats::default();
    
    // Setup logging verbosity
    if let Some(quiet) = config.quiet {
        if quiet {
            log::set_max_level(log::LevelFilter::Warn);
        }
    }
    
    if let Some(verbose) = config.verbose {
        if verbose {
            log::set_max_level(log::LevelFilter::Debug);
        }
    }
    
    // Load incremental cache if enabled
    let mut cache = if incremental {
        info!("Incremental build enabled");
        IncrementalCache::load(config)
    } else {
        IncrementalCache::default()
    };

    // Create directory structure
    let dirs = DirectoryStructure::from_config(config);
    info!("Using source directory: {}", dirs.source.display());
    info!("Output will be generated in: {}", dirs.destination.display());

    // Only clean destination if not doing incremental build
    if !incremental {
        clean_destination(config)?;
    }

    // Create destination directory and other required directories
    dirs.create_site_directories()?;

    // Load collections (includes posts)
    info!("Loading collections...");
    let mut collections = load_collections(config)?;
    
    // Get posts collection for convenience
    let _posts = collections.get("posts")
        .ok_or("Posts collection not found")?
        .clone();
    
    // Count documents in all collections
    let total_documents: usize = collections.values()
        .map(|collection| collection.documents.len())
        .sum();
    debug!("Loaded {} documents from {} collections", total_documents, collections.len());
    stats.documents_count = total_documents;

    // Load pages
    info!("Loading pages...");
    let mut pages = collect_pages(&dirs)?;
    stats.pages_count = pages.len();
    debug!("Loaded {} pages", pages.len());

    // Load layouts
    info!("Loading layouts...");
    let layouts = load_layouts(&dirs)?;
    debug!("Loaded {} layouts", layouts.len());

    // Load includes
    info!("Loading includes...");
    let includes = load_includes(&dirs)?;
    debug!("Loaded {} includes", includes.len());

    // Load data files
    info!("Loading data files...");
    let data = load_data_files(config)?;
    debug!("Loaded {} data files", data.len());

    // Update dependency tracking for includes and layouts
    if incremental {
        for (name, _) in &includes {
            let path = dirs.includes_dir.join(format!("{}.html", name));
            cache.update_mtime(&path);
        }

        for (name, _layout) in &layouts {
            let path = dirs.layouts_dir.join(format!("{}.html", name));
            cache.update_mtime(&path);

            // Track dependencies between layouts (for layout inheritance)
            if let Some(layout_info) = layouts.get(name) {
                // Check if layout has a parent layout in its front matter
                if let Some(parent_layout) = &layout_info.front_matter.layout {
                    let parent_path = dirs.layouts_dir.join(format!("{}.html", parent_layout));
                    cache.add_dependency(&path, &parent_path);
                }
            }
        }
    }

    // Copy static files (skip in incremental mode if not modified)
    info!("Copying static files...");
    #[allow(unused_assignments)]
    let mut copied_count = 0;

    // Use a thread pool for parallel file copying
    if !incremental {
        // Non-incremental: copy all static files
        copied_count = dirs.copy_static_files()?;
    } else {
        // Incremental: only copy modified files
        let static_files = dirs.get_static_files()?;
        
        // Use rayon for parallel processing of static files
        let counter = Arc::new(Mutex::new(0));
        let cache_arc = Arc::new(Mutex::new(&mut cache));
        
        static_files.par_iter().for_each(|(source, dest)| {
            let is_modified = {
                let cache_guard = cache_arc.lock().unwrap();
                cache_guard.is_modified(source)
            };
            
            if is_modified {
                if let Some(parent) = dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                
                if let Err(e) = fs::copy(source, dest) {
                    error!("Failed to copy static file from {} to {}: {}", 
                           source.display(), dest.display(), e);
                } else {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    
                    let mut cache_guard = cache_arc.lock().unwrap();
                    cache_guard.update_mtime(source);
                }
            }
        });
        
        copied_count = *counter.lock().unwrap();
    }
    
    stats.static_files_count = copied_count;
    info!("Copied {} static files", copied_count);

    // Create the Liquid parser with custom tags and filters
    info!("Setting up template engine...");
    let parser = create_jekyll_parser(config, includes)?;

    // Create the Markdown renderer
    let markdown_renderer = MarkdownRenderer::new(config);

    // Create the site object with all collections and data
    let mut site_data = create_site_object(config);
    
    // Add collections to site object
    let collections_data = collections_to_liquid(&collections, config);
    site_data.insert("collections".into(), Value::Object(collections_data));
    
    // Add pages to site object
    let pages_array = pages.iter()
        .map(|page| crate::builder::site::page_to_liquid(page))
        .collect::<Vec<Value>>();
    site_data.insert("pages".into(), Value::Array(pages_array));
    
    // Add posts directly to site object
    if let Some(posts_collection) = collections.get("posts") {
        // Sort the posts by date (newest first)
        let mut sorted_posts = posts_collection.documents.clone();
        sorted_posts.sort_by(|a, b| {
            let a_date = a.date.unwrap_or_else(|| Utc::now());
            let b_date = b.date.unwrap_or_else(|| Utc::now());
            b_date.cmp(&a_date) // Reverse chronological order
        });
        
        // Apply limit_posts if configured
        if let Some(limit) = config.limit_posts {
            if limit > 0 && limit < sorted_posts.len() {
                sorted_posts.truncate(limit);
                debug!("Limited posts to {} of {}", limit, posts_collection.documents.len());
            }
        }
        
        // Convert to liquid values
        let posts_array = sorted_posts.iter()
            .map(|doc| crate::collections::document_to_liquid(doc))
            .collect::<Vec<Value>>();
        site_data.insert("posts".into(), Value::Array(posts_array.clone()));
        
        // Add HTML pages subset
        let html_pages = pages.iter()
            .filter(|page| {
                if let Some(ext) = page.path.extension() {
                    return ext == "html";
                }
                false
            })
            .map(|page| crate::builder::site::page_to_liquid(page))
            .collect::<Vec<Value>>();
        site_data.insert("html_pages".into(), Value::Array(html_pages));
        
        // Collect categories and tags from posts
        let mut categories = std::collections::HashMap::new();
        let mut tags = std::collections::HashMap::new();
        
        for doc in &posts_collection.documents {
            // Process categories
            for category in &doc.categories {
                let category_posts = categories.entry(category.clone())
                    .or_insert_with(Vec::new);
                category_posts.push(crate::collections::document_to_liquid(doc));
            }
            
            // Process tags
            for tag in &doc.tags {
                let tag_posts = tags.entry(tag.clone())
                    .or_insert_with(Vec::new);
                tag_posts.push(crate::collections::document_to_liquid(doc));
            }
        }
        
        // Convert categories and tags to Liquid objects
        let mut categories_obj = liquid::Object::new();
        for (category, posts) in categories {
            categories_obj.insert(category.into(), Value::Array(posts));
        }
        site_data.insert("categories".into(), Value::Object(categories_obj));
        
        let mut tags_obj = liquid::Object::new();
        for (tag, posts) in tags {
            tags_obj.insert(tag.into(), Value::Array(posts));
        }
        site_data.insert("tags".into(), Value::Object(tags_obj));
        
        // Handle related posts
        process_related_posts(&mut collections, config)?;
    }
    
    // Add data files to site object
    let data_object = data_to_liquid(&data);
    site_data.insert("data".into(), Value::Object(data_object));

    // Process collections
    // First sort collections for consistent output
    for (_, collection) in &mut collections {
        collection.sort_documents();
        collection.set_next_prev_links();
    }
    
    // First collect all documents that need URLs
    for (_name, collection) in &mut collections {
        let docs_count = collection.documents.len();
        let mut urls = Vec::with_capacity(docs_count);
        
        // Generate URLs without mutating documents
        for doc in &collection.documents {
            let url = collection.generate_url(doc, config);
            urls.push(url);
        }
        
        // Now assign the URLs to documents
        for (i, doc) in collection.documents.iter_mut().enumerate() {
            doc.url = urls.get(i).cloned().flatten();
            
            // Generate excerpts if needed
            doc.generate_excerpt(&config.excerpt_separator);
        }
    }
    
    // Process pagination if enabled
    if let Some(paginate) = config.paginate {
        if paginate > 0 {
            info!("Processing pagination (per_page: {})", paginate);
            process_pagination(&mut pages, &collections, paginate, config)?;
        }
    }
    
    // Process and render collections (including posts)
    let collections_result = process_collections(&mut collections, &layouts, &parser, &site_data, &markdown_renderer, &dirs, config);
    if let Err(e) = collections_result {
        error!("Error processing collections: {}", e);
        stats.errors_count += 1;
    }
    
    // Process and render pages
    let pages_result = process_pages(pages, &layouts, &parser, &site_data, &markdown_renderer, config);
    if let Err(e) = pages_result {
        error!("Error processing pages: {}", e);
        stats.errors_count += 1;
    }

    // Save the incremental cache if enabled
    if incremental {
        if let Err(e) = cache.save(config) {
            warn!("Failed to save incremental cache: {}", e);
        }
    }

    let elapsed = start_time.elapsed();
    stats.duration = elapsed;
    
    info!("Site built in {:.2?}", elapsed);
    info!("Pages: {}, Documents: {}, Static files: {}, Errors: {}", 
          stats.pages_count, stats.documents_count, stats.static_files_count, stats.errors_count);

    // Generate build report if configured
    if config.build_report.unwrap_or(false) {
        info!("Generating build report...");
        match crate::report::generate_build_report(config, elapsed) {
            Ok(_) => info!("Build report generated successfully"),
            Err(e) => warn!("Failed to generate build report: {}", e),
        }
    }

    Ok(())
}

/// Process related posts for all collections
fn process_related_posts(collections: &mut HashMap<String, crate::collections::Collection>, config: &Config) -> BoxResult<()> {
    if let Some(posts) = collections.get_mut("posts") {
        // Only process if there are enough posts
        if posts.documents.len() < 2 {
            return Ok(());
        }
        
        // Check if LSI is enabled
        let use_lsi = config.lsi.unwrap_or(false);
        
        if use_lsi {
            // LSI implementation would go here - requires NLP libraries
            warn!("LSI related posts not implemented yet, falling back to date-based related posts");
            generate_date_based_related_posts(posts)?;
        } else {
            // Use date-based related posts
            generate_date_based_related_posts(posts)?;
        }
    }
    
    Ok(())
}

/// Generate date-based related posts
fn generate_date_based_related_posts(posts: &mut crate::collections::Collection) -> BoxResult<()> {
    // Sort posts by date
    let mut sorted_posts = posts.documents.clone();
    sorted_posts.sort_by(|a, b| {
        let a_date = a.date.unwrap_or_else(|| Utc::now());
        let b_date = b.date.unwrap_or_else(|| Utc::now());
        b_date.cmp(&a_date) // Reverse chronological order
    });
    
    // For each post, find 4 related posts
    for i in 0..posts.documents.len() {
        let mut related = Vec::new();
        let current = &posts.documents[i];
        
        // Consider tags for better matching
        let current_tags: HashSet<String> = current.tags.iter().cloned().collect();
        
        // Score other posts based on tag overlap and date proximity
        let mut scored_posts: Vec<(usize, f64)> = Vec::new();
        
        for (j, post) in sorted_posts.iter().enumerate() {
            if i == j {
                continue; // Skip self
            }
            
            // Base score on date proximity
            let score = if let (Some(current_date), Some(post_date)) = (current.date, post.date) {
                let diff = (current_date - post_date).num_days().abs() as f64;
                1.0 / (1.0 + diff / 30.0) // Scale by month
            } else {
                0.1 // Default low score
            };
            
            // Bonus for tag overlap
            let post_tags: HashSet<String> = post.tags.iter().cloned().collect();
            let tag_overlap = current_tags.intersection(&post_tags).count();
            let tag_bonus = tag_overlap as f64 * 0.2;
            
            scored_posts.push((j, score + tag_bonus));
        }
        
        // Sort by score
        scored_posts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 4 related posts
        for (j, _) in scored_posts.iter().take(4) {
            related.push(sorted_posts[*j].id.clone());
        }
        
        // Update the document with related posts
        posts.documents[i].related = related;
    }
    
    Ok(())
}

/// Process pagination for index pages
fn process_pagination(
    pages: &mut Vec<Page>, 
    collections: &HashMap<String, crate::collections::Collection>,
    per_page: usize,
    config: &Config
) -> BoxResult<()> {
    // Find index pages with pagination enabled
    let mut paginated_pages = Vec::new();
    
    for (i, page) in pages.iter().enumerate() {
        if page.front_matter.pagination.is_some() {
            paginated_pages.push(i);
        }
    }
    
    if paginated_pages.is_empty() {
        return Ok(());
    }
    
    // Get the posts collection
    let posts = match collections.get("posts") {
        Some(posts) => posts,
        None => {
            warn!("Pagination enabled but no posts collection found");
            return Ok(());
        }
    };
    
    // For each paginated page, create additional pages
    let mut new_pages = Vec::new();
    
    for &index in &paginated_pages {
        let page = &pages[index];
        
        // Get the posts to paginate
        let mut paginate_posts = posts.documents.clone();
        
        // Sort posts by date (newest first)
        paginate_posts.sort_by(|a, b| {
            let a_date = a.date.unwrap_or_else(|| Utc::now());
            let b_date = b.date.unwrap_or_else(|| Utc::now());
            b_date.cmp(&a_date)
        });
        
        // Filter posts as needed
        // TODO: Implement category and tag filtering based on pagination config
        
        // Calculate the number of pages needed
        let total_posts = paginate_posts.len();
        let num_pages = (total_posts + per_page - 1) / per_page; // Ceiling division
        
        if num_pages <= 1 {
            // No pagination needed
            continue;
        }
        
        // Get pagination path pattern
        let paginate_path = config.paginate_path.clone();
        
        // Create pages for each chunk of posts
        for page_num in 1..num_pages {
            // Clone the original page
            let mut new_page = page.clone();
            
            // Calculate post range for this page
            let start = page_num * per_page;
            let end = std::cmp::min(start + per_page, total_posts);
            let page_posts = &paginate_posts[start..end];
            
            // Update pagination info in front matter
            if let Some(ref mut pagination) = new_page.front_matter.pagination {
                pagination.per_page = per_page;
                // Use different variables to represent pagination state
                
                // Add the post references
                pagination.posts = page_posts.iter()
                    .map(|doc| doc.id.clone())
                    .collect();
            }
            
            // Update the output path
            let page_path = paginate_path.replace(":num", &(page_num + 1).to_string());
            let dest_path = Path::new(&config.destination).join(page_path.trim_start_matches('/'));
            
            // Ensure directory exists
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            
            // Set the new output path
            new_page.output_path = Some(dest_path);
            
            // Add the new page
            new_pages.push(new_page);
        }
        
        // Update the first page (original page)
        if let Some(ref mut pagination) = pages[index].front_matter.pagination {
            pagination.per_page = per_page;
            // Use different variables to represent pagination state
            
            // Add the post references for the first page
            let end = std::cmp::min(per_page, total_posts);
            let page_posts = &paginate_posts[0..end];
            
            pagination.posts = page_posts.iter()
                .map(|doc| doc.id.clone())
                .collect();
        }
    }
    
    // Add all new pages to the pages vector
    pages.extend(new_pages);
    
    Ok(())
}

/// Check if a file is excluded based on config
pub fn is_excluded(path: &Path, config: &Config) -> bool {
    config.is_excluded(path)
} 