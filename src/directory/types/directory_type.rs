/// Different types of directories in a Jekyll site
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryType {
    /// Layout templates directory
    Layouts,
    
    /// Include snippets directory
    Includes,
    
    /// Blog posts directory
    Posts,
    
    /// Draft posts directory
    Drafts,
    
    /// Data files directory
    Data,
    
    /// Custom collections directory
    Collections,
    
    /// SASS/SCSS files directory
    Sass,
    
    /// Jekyll plugins directory
    Plugins,
    
    /// Output site directory
    Site,
    
    /// Static files directory (source)
    Static,
} 