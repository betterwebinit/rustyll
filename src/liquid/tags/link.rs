use liquid_core::{Runtime, Error, ParseTag, Renderable, TagReflection, TagTokenIter};
use crate::config::Config;
use log::debug;

/// Jekyll-compatible link tag
#[derive(Debug, Clone)]
pub struct LinkTag {
    config: Config,
}

impl LinkTag {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    fn resolve_path(&self, path: &str) -> Result<String, Error> {
        // Clean up the path - strip quotes and whitespace
        let path = path.trim_matches('"').trim_matches('\'').trim();
        debug!("Link tag: resolving path '{}'", path);
        
        // Jekyll link tag handling:
        // 1. For paths starting with underscore (_), they're collection documents or special directories
        // 2. Need to convert .md/.markdown files to .html
        // 3. Need to apply site.baseurl
        
        // First, check if the file exists exactly as specified
        let source_path = self.config.source.join(path);
        if source_path.exists() {
            debug!("Link tag: found exact path: {}", source_path.display());
            // Convert file path to URL path
            return self.path_to_url(path);
        }
        
        // Try without leading underscore for collections (Jekyll behavior)
        if path.starts_with('_') {
            let alt_path = path.trim_start_matches('_');
            // Try with the underscore removed
            let alt_source_path = self.config.source.join(alt_path);
            if alt_source_path.exists() {
                debug!("Link tag: found alternate path without underscore: {}", alt_source_path.display());
                return self.path_to_url(alt_path);
            }
        }
        
        // Try with source/_posts for post files
        if path.contains("posts/") {
            let post_path = path.replace("posts/", "_posts/");
            let post_source_path = self.config.source.join(&post_path);
            if post_source_path.exists() {
                debug!("Link tag: found post path: {}", post_source_path.display());
                // For posts we need special handling for permalinks
                return self.path_to_url(&post_path);
            }
        }
        
        // Try with _docs and docs for documentation files
        if path.contains("docs/") && !path.starts_with('_') {
            let docs_path = format!("_{}", path);
            let docs_source_path = self.config.source.join(&docs_path);
            if docs_source_path.exists() {
                debug!("Link tag: found docs path with underscore: {}", docs_source_path.display());
                return self.path_to_url(&docs_path);
            }
        } else if path.starts_with("_docs/") {
            let no_underscore_path = path.trim_start_matches('_');
            let no_underscore_source_path = self.config.source.join(no_underscore_path);
            if no_underscore_source_path.exists() {
                debug!("Link tag: found docs path without underscore: {}", no_underscore_source_path.display());
                return self.path_to_url(no_underscore_path);
            }
        }
        
        // If we couldn't find the file, we'll still generate a URL (Jekyll does this)
        debug!("Link tag: couldn't find exact file, generating URL anyway from: {}", path);
        self.path_to_url(path)
    }
    
    fn path_to_url(&self, path: &str) -> Result<String, Error> {
        // Convert a path like "_docs/history.md" to a URL like "/docs/history.html"
        
        // Remove leading underscore if present (Jekyll collection convention)
        let path_without_underscore = if path.starts_with('_') && !path.starts_with("_posts/") {
            path.trim_start_matches('_')
        } else {
            path
        };
        
        // Get path relative to source
        let url_path = path_without_underscore.trim_start_matches('/');
        
        // Handle markdown files by changing extension to .html
        let url_path = if url_path.ends_with(".md") || url_path.ends_with(".markdown") {
            let without_ext = url_path.rsplit_once('.').map(|(base, _)| base).unwrap_or(url_path);
            format!("{}.html", without_ext)
        } else {
            url_path.to_string()
        };
        
        // Special handling for posts permalink
        let url_path = if path.starts_with("_posts/") {
            // Jekyll would use the post's permalink here, but for simplicity
            // we'll just use a basic transformation
            let filename = url_path.rsplit_once('/').map(|(_, file)| file).unwrap_or(&url_path);
            format!("posts/{}", filename)
        } else {
            url_path
        };
        
        // Add base_url if configured
        let base_url = &self.config.base_url;
        let url = if base_url.is_empty() || base_url == "/" {
            format!("/{}", url_path)
        } else {
            format!("{}/{}", base_url.trim_end_matches('/'), url_path)
        };
        
        debug!("Link tag: final URL: {}", url);
        Ok(url)
    }
}

struct LinkTagReflection;

impl TagReflection for LinkTagReflection {
    fn tag(&self) -> &str {
        "link"
    }

    fn description(&self) -> &str {
        "Creates a link to a page, post, or collection item"
    }
}

impl ParseTag for LinkTag {
    fn reflection(&self) -> &dyn TagReflection {
        &LinkTagReflection
    }
    
    fn parse(&self, mut arguments: TagTokenIter, _options: &liquid_core::parser::Language) -> Result<Box<dyn Renderable>, Error> {
        // For Jekyll-style link tags, we just need to get the path argument
        // This is typically just a simple string like "_docs/history.md"
        let token = arguments.next().ok_or_else(|| Error::with_msg("Link tag requires a path argument"))?;
        let path = token.as_str().trim();
        
        // Add a debug log to see what the path argument is
        debug!("Link tag: parsing path '{}'", path);
        
        Ok(Box::new(LinkTagRenderer {
            config: self.config.clone(),
            path: path.to_string(),
        }))
    }
}

/// Renderer for the link tag
#[derive(Debug)]
struct LinkTagRenderer {
    config: Config,
    path: String,
}

impl Renderable for LinkTagRenderer {
    fn render(&self, _runtime: &dyn Runtime) -> Result<String, Error> {
        let link_tag = LinkTag::new(self.config.clone());
        link_tag.resolve_path(&self.path)
    }

    fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn Runtime) -> Result<(), Error> {
        let s = self.render(runtime)?;
        writer.write_all(s.as_bytes()).map_err(|e| Error::with_msg(format!("Failed to write to output: {}", e)))?;
        Ok(())
    }
} 