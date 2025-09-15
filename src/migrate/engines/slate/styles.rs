use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_styles(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Slate styles...");
    }

    // Create destination style directories
    let dest_css_dir = dest_dir.join("assets/css");
    create_dir_if_not_exists(&dest_css_dir)?;
    
    let dest_sass_dir = dest_dir.join("_sass");
    create_dir_if_not_exists(&dest_sass_dir)?;

    // In Slate, styles are typically in source/stylesheets
    let source_styles_dir = source_dir.join("source/stylesheets");
    if source_styles_dir.exists() && source_styles_dir.is_dir() {
        // Process style files
        for entry in WalkDir::new(&source_styles_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process stylesheet files
                if is_stylesheet_file(file_path) {
                    migrate_stylesheet_file(file_path, &source_styles_dir, &dest_css_dir, &dest_sass_dir, result)?;
                }
            }
        }
    } else {
        // Try alternative stylesheet locations
        let alt_styles_dirs = [
            source_dir.join("source/css"),
            source_dir.join("source/style"),
        ];
        
        let mut found = false;
        for alt_dir in alt_styles_dirs {
            if alt_dir.exists() && alt_dir.is_dir() {
                for entry in WalkDir::new(&alt_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        if is_stylesheet_file(file_path) {
                            migrate_stylesheet_file(file_path, &alt_dir, &dest_css_dir, &dest_sass_dir, result)?;
                            found = true;
                        }
                    }
                }
            }
        }
        
        if !found {
            // Create default stylesheet if no styles were found
            create_default_stylesheets(&dest_css_dir, &dest_sass_dir, result)?;
        }
    }

    Ok(())
}

fn migrate_stylesheet_file(
    file_path: &Path,
    source_dir: &Path,
    dest_css_dir: &Path,
    dest_sass_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the stylesheet file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read stylesheet file {}: {}", file_path.display(), e))?;
    
    // Get file extension to determine how to process
    let ext = file_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    
    // Determine if it's a partial (file starts with _)
    let file_name = file_path.file_name().unwrap().to_string_lossy();
    let is_partial = file_name.starts_with('_');
    
    // Convert and determine destination
    let (dest_path, converted_content) = if ext == "scss" || ext == "sass" {
        if is_partial {
            // Sass partials go into _sass directory
            let rel_path = file_path.strip_prefix(source_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            (dest_sass_dir.join(rel_path), content)
        } else {
            // Main Sass files go into assets/css with Jekyll front matter
            let rel_path = file_path.strip_prefix(source_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            let jekyll_front_matter = "---\n---\n\n";
            let converted = format!("{}{}", jekyll_front_matter, content);
            
            (dest_css_dir.join(rel_path), converted)
        }
    } else if ext == "css" {
        // CSS files go directly into assets/css
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        (dest_css_dir.join(rel_path), content)
    } else {
        // For other files, just copy to assets/css
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        (dest_css_dir.join(rel_path), content)
    };
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write stylesheet file {}: {}", dest_path.display(), e))?;
    
    // Determine relative path for change record
    let change_path = if dest_path.starts_with(dest_sass_dir) {
        format!("_sass/{}", dest_path.strip_prefix(dest_sass_dir).unwrap().display())
    } else {
        format!("assets/css/{}", dest_path.strip_prefix(dest_css_dir).unwrap().display())
    };
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: change_path,
        description: format!("Converted stylesheet from {}", file_path.display()),
    });
    
    Ok(())
}

fn create_default_stylesheets(
    dest_css_dir: &Path,
    dest_sass_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create main CSS file with Jekyll front matter
    let main_css_content = r#"---
---

@import "slate";
@import "syntax";
@import "normalize";
@import "custom";
"#;

    let main_css_path = dest_css_dir.join("style.css");
    fs::write(&main_css_path, main_css_content)
        .map_err(|e| format!("Failed to create main CSS file: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/style.css".into(),
        description: "Created main CSS file".into(),
    });

    // Create basic Sass partials
    let slate_scss_content = r#"// Slate theme for Jekyll
// Based on the original Slate theme style

// Variables
$main-bg: #fff;
$main-text: #333;
$nav-bg: #2E3336;
$nav-text: #fff;
$section-bg: #f5f5f5;
$heading-text: #333;
$link-color: #0099e5;
$sidebar-width: 230px;

// Basic layout
body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  color: $main-text;
  background-color: $main-bg;
}

a {
  color: $link-color;
  text-decoration: none;
  
  &:hover {
    text-decoration: underline;
  }
}

// Header styles
header {
  background-color: $nav-bg;
  color: $nav-text;
  padding: 15px 30px;
  
  .logo a {
    color: $nav-text;
    font-weight: bold;
    font-size: 24px;
    text-decoration: none;
  }
  
  nav {
    margin-top: 10px;
    
    a {
      color: $nav-text;
      margin-right: 15px;
      opacity: 0.8;
      
      &:hover {
        opacity: 1;
        text-decoration: none;
      }
    }
  }
}

// Content styles
.page-wrapper {
  padding: 30px;
  max-width: 1200px;
  margin: 0 auto;
}

.content {
  h1, h2, h3, h4, h5, h6 {
    color: $heading-text;
    margin-top: 1.5em;
    margin-bottom: 0.8em;
  }
  
  h1 { font-size: 2.5em; }
  h2 { font-size: 2em; }
  h3 { font-size: 1.5em; }
  
  code {
    background-color: rgba(0,0,0,0.05);
    padding: 3px 5px;
    border-radius: 3px;
  }
  
  pre code {
    display: block;
    overflow-x: auto;
    padding: 15px;
    background-color: #2E3336;
    color: #fff;
    border-radius: 5px;
  }
}

// Language selector
.lang-selector {
  background-color: darken($nav-bg, 5%);
  padding: 10px 30px;
  
  a {
    display: inline-block;
    color: $nav-text;
    padding: 5px 15px;
    margin-right: 10px;
    opacity: 0.7;
    
    &:hover, &.active {
      opacity: 1;
      text-decoration: none;
    }
    
    &.active {
      background-color: rgba(255,255,255,0.1);
      border-radius: 5px;
    }
  }
}

// Table of contents footer
.toc-footer {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid rgba(0,0,0,0.1);
  
  ul {
    padding-left: 20px;
    
    li {
      margin-bottom: 10px;
    }
  }
}

// Media queries for responsiveness
@media (min-width: 768px) {
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    
    nav {
      margin-top: 0;
    }
  }
}
"#;

    let slate_scss_path = dest_sass_dir.join("_slate.scss");
    fs::write(&slate_scss_path, slate_scss_content)
        .map_err(|e| format!("Failed to create _slate.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_sass/_slate.scss".into(),
        description: "Created Slate SCSS partial".into(),
    });

    // Create syntax highlighting stylesheet
    let syntax_scss_content = r#"// Syntax highlighting styles
.highlight {
  .c { color: #999988; font-style: italic } // Comment
  .err { color: #a61717; background-color: #e3d2d2 } // Error
  .k { font-weight: bold } // Keyword
  .o { font-weight: bold } // Operator
  .cm { color: #999988; font-style: italic } // Comment.Multiline
  .cp { color: #999999; font-weight: bold } // Comment.Preproc
  .c1 { color: #999988; font-style: italic } // Comment.Single
  .cs { color: #999999; font-weight: bold; font-style: italic } // Comment.Special
  .gd { color: #000000; background-color: #ffdddd } // Generic.Deleted
  .gd .x { color: #000000; background-color: #ffaaaa } // Generic.Deleted.Specific
  .ge { font-style: italic } // Generic.Emph
  .gr { color: #aa0000 } // Generic.Error
  .gh { color: #999999 } // Generic.Heading
  .gi { color: #000000; background-color: #ddffdd } // Generic.Inserted
  .gi .x { color: #000000; background-color: #aaffaa } // Generic.Inserted.Specific
  .go { color: #888888 } // Generic.Output
  .gp { color: #555555 } // Generic.Prompt
  .gs { font-weight: bold } // Generic.Strong
  .gu { color: #aaaaaa } // Generic.Subheading
  .gt { color: #aa0000 } // Generic.Traceback
  .kc { font-weight: bold } // Keyword.Constant
  .kd { font-weight: bold } // Keyword.Declaration
  .kp { font-weight: bold } // Keyword.Pseudo
  .kr { font-weight: bold } // Keyword.Reserved
  .kt { color: #445588; font-weight: bold } // Keyword.Type
  .m { color: #009999 } // Literal.Number
  .s { color: #d14 } // Literal.String
  .na { color: #008080 } // Name.Attribute
  .nb { color: #0086B3 } // Name.Builtin
  .nc { color: #445588; font-weight: bold } // Name.Class
  .no { color: #008080 } // Name.Constant
  .ni { color: #800080 } // Name.Entity
  .ne { color: #990000; font-weight: bold } // Name.Exception
  .nf { color: #990000; font-weight: bold } // Name.Function
  .nn { color: #555555 } // Name.Namespace
  .nt { color: #000080 } // Name.Tag
  .nv { color: #008080 } // Name.Variable
  .ow { font-weight: bold } // Operator.Word
  .w { color: #bbbbbb } // Text.Whitespace
  .mf { color: #009999 } // Literal.Number.Float
  .mh { color: #009999 } // Literal.Number.Hex
  .mi { color: #009999 } // Literal.Number.Integer
  .mo { color: #009999 } // Literal.Number.Oct
  .sb { color: #d14 } // Literal.String.Backtick
  .sc { color: #d14 } // Literal.String.Char
  .sd { color: #d14 } // Literal.String.Doc
  .s2 { color: #d14 } // Literal.String.Double
  .se { color: #d14 } // Literal.String.Escape
  .sh { color: #d14 } // Literal.String.Heredoc
  .si { color: #d14 } // Literal.String.Interpol
  .sx { color: #d14 } // Literal.String.Other
  .sr { color: #009926 } // Literal.String.Regex
  .s1 { color: #d14 } // Literal.String.Single
  .ss { color: #990073 } // Literal.String.Symbol
  .bp { color: #999999 } // Name.Builtin.Pseudo
  .vc { color: #008080 } // Name.Variable.Class
  .vg { color: #008080 } // Name.Variable.Global
  .vi { color: #008080 } // Name.Variable.Instance
  .il { color: #009999 } // Literal.Number.Integer.Long
}
"#;

    let syntax_scss_path = dest_sass_dir.join("_syntax.scss");
    fs::write(&syntax_scss_path, syntax_scss_content)
        .map_err(|e| format!("Failed to create _syntax.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_sass/_syntax.scss".into(),
        description: "Created syntax highlighting stylesheet".into(),
    });

    // Create normalize stylesheet
    let normalize_scss_content = r#"// Normalize.css styles
html {
  line-height: 1.15;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
}

main {
  display: block;
}

h1 {
  font-size: 2em;
  margin: 0.67em 0;
}

hr {
  box-sizing: content-box;
  height: 0;
  overflow: visible;
}

pre {
  font-family: monospace, monospace;
  font-size: 1em;
}

a {
  background-color: transparent;
}

abbr[title] {
  border-bottom: none;
  text-decoration: underline;
  text-decoration: underline dotted;
}

b, strong {
  font-weight: bolder;
}

code, kbd, samp {
  font-family: monospace, monospace;
  font-size: 1em;
}

small {
  font-size: 80%;
}

sub, sup {
  font-size: 75%;
  line-height: 0;
  position: relative;
  vertical-align: baseline;
}

sub {
  bottom: -0.25em;
}

sup {
  top: -0.5em;
}

img {
  border-style: none;
  max-width: 100%;
}

button, input, optgroup, select, textarea {
  font-family: inherit;
  font-size: 100%;
  line-height: 1.15;
  margin: 0;
}

button, input {
  overflow: visible;
}

button, select {
  text-transform: none;
}

button, [type="button"], [type="reset"], [type="submit"] {
  -webkit-appearance: button;
}

button::-moz-focus-inner, [type="button"]::-moz-focus-inner, [type="reset"]::-moz-focus-inner, [type="submit"]::-moz-focus-inner {
  border-style: none;
  padding: 0;
}

button:-moz-focusring, [type="button"]:-moz-focusring, [type="reset"]:-moz-focusring, [type="submit"]:-moz-focusring {
  outline: 1px dotted ButtonText;
}

fieldset {
  padding: 0.35em 0.75em 0.625em;
}

legend {
  box-sizing: border-box;
  color: inherit;
  display: table;
  max-width: 100%;
  padding: 0;
  white-space: normal;
}

progress {
  vertical-align: baseline;
}

textarea {
  overflow: auto;
}

[type="checkbox"], [type="radio"] {
  box-sizing: border-box;
  padding: 0;
}

[type="number"]::-webkit-inner-spin-button, [type="number"]::-webkit-outer-spin-button {
  height: auto;
}

[type="search"] {
  -webkit-appearance: textfield;
  outline-offset: -2px;
}

[type="search"]::-webkit-search-decoration {
  -webkit-appearance: none;
}

::-webkit-file-upload-button {
  -webkit-appearance: button;
  font: inherit;
}

details {
  display: block;
}

summary {
  display: list-item;
}

template {
  display: none;
}

[hidden] {
  display: none;
}
"#;

    let normalize_scss_path = dest_sass_dir.join("_normalize.scss");
    fs::write(&normalize_scss_path, normalize_scss_content)
        .map_err(|e| format!("Failed to create _normalize.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_sass/_normalize.scss".into(),
        description: "Created normalize stylesheet".into(),
    });

    // Create custom stylesheet for user modifications
    let custom_scss_content = r#"// Custom styles
// Add your custom styles here, they will override the default styles

// Example:
// body {
//   font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
// }
"#;

    let custom_scss_path = dest_sass_dir.join("_custom.scss");
    fs::write(&custom_scss_path, custom_scss_content)
        .map_err(|e| format!("Failed to create _custom.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_sass/_custom.scss".into(),
        description: "Created custom stylesheet".into(),
    });

    Ok(())
}

fn is_stylesheet_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "css" || ext_str == "scss" || ext_str == "sass" || ext_str == "less"
    } else {
        false
    }
} 