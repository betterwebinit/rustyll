# Collections and Data Handling

Jekyll provides two primary ways to organize structured content: **Collections** (for document sets like posts, authors, projects, etc.) and **Data Files** (for arbitrary data like site config, navigation menus, etc.). Both are exposed to templates via the Liquid context, and can shape the site's structure and content.

## Collections

Collections allow grouping related content into a set, with a shared configuration. **Posts** are actually a special built-in collection with extra date-based handling. Custom collections let you create similar concepts, such as staff profiles, recipes, projects, etc.

### Configuring Collections

Collections are defined in `_config.yml`, under a `collections` key:

```yaml
collections:
  # This creates a collection called "staff"
  staff:
    # Whether to generate pages from the collection's documents
    output: true
    # Custom permalink pattern for the collection's documents
    permalink: /team/:name/
  # Another collection without output (data-only)
  recipes:
    output: false
```

The collection name becomes the directory name (prefixed with underscore), so for the "staff" collection, documents would go in `_staff/`.

### Collection Documents

Collection documents are files in the collection directory (e.g., `_staff/jane-doe.md`). Like posts/pages, they can have YAML front matter:

```markdown
---
name: Jane Doe
position: Developer
email: jane@example.com
---
Jane has been with the company since 2015...
```

Jekyll processes and converts these documents like pages. If `output: true` for that collection, each document gets its own output page (using the collection's permalink pattern).

### Accessing Collections in Templates

Collections are available through `site.collections` (a list of all collections) and through `site.<collection_name>` (a list of all documents in that collection):

```liquid
{% raw %}
<!-- List all staff members -->
{% for member in site.staff %}
  <h2>{{ member.name }} - {{ member.position }}</h2>
  <p>{{ member.content }}</p>
{% endfor %}
{% endraw %}
```

Collections have metadata like:

* `collection.label` - The collection name ('staff')
* `collection.docs` - Array of documents in this collection
* `collection.files` - Array of static files in this collection dir
* `collection.relative_directory` - Directory name ('_staff')
* `collection.directory` - Full directory path

Each collection document has:

* `doc.id` - A unique identifier (used for permalinks)
* `doc.url` - URL of the generated page (if output: true)
* `doc.path` - Path to the document source file
* `doc.relative_path` - Path relative to site source
* `doc.collection` - Name of the collection it belongs to
* `doc.date` - Parsed date if in filename (like posts) or from front matter
* All other front matter variables are accessible directly
* `doc.content` - Rendered content of the document
* `doc.output` - The complete rendered page (if output: true)

### Collection Configuration Options

* **output** (boolean): Whether to generate individual pages from the documents (default: false)
* **permalink** (string): Pattern for output files. Variables include `:collection`, `:path`, `:name`, and any front matter fields.
* **sort_by** (string): Field to sort by (default: none)
* **order** (array): List of front matter field values to order by (for manual ordering)

### Directory Structure for Collections

Collections live in folders prefixed with underscore, matching the collection name. For example:

```
_staff/
  jane-doe.md
  john-smith.md
_recipes/
  pancakes.md
  bread.md
```

### The Posts Collection

Posts are a special collection that gets additional features:

* File naming convention (`YYYY-MM-DD-title.md`) for date parsing
* Automatic category and tags handling
* Chronological sorting
* Special permalink formats (date-based)
* Access to `page.previous` and `page.next` for navigation
* Available at `site.posts` (equivalent to `site.collections.posts.docs`)

### Default Collection Behavior

* **Posts**: By default, Jekyll enables the "posts" collection with `output: true` even if not explicitly configured, with permalink: `/:categories/:year/:month/:day/:title:output_ext`
* **Drafts**: Not a formal collection, but `_drafts` directory holds unpublished posts (rendered only with `--drafts` flag)

### Implementation Considerations

When implementing collections:

* Keep the date-based front matter and filename-derived date features for posts.
* Allow configurable `output` and `permalink` per collection.
* Make collection documents available via `site.collections` and direct `site.<collection_name>`.
* Parse front matter and convert collection documents if `output: true`.
* Support sorting documents by date, or custom sort_by field.

## Data Files

Data files provide a way to store structured data for the site. These are useful for things like navigation menus, localization strings, product data, etc. Jekyll supports YAML, JSON, CSV, and TSV formats.

### Directory Structure

Data files live in the `_data` directory:

```
_data/
  navigation.yml
  team.yml
  products.json
  locales/
    en.yml
    fr.yml
```

### Data File Formats

* **YAML** (.yml, .yaml): Most common. Multi-document YAML files are not supported.
* **JSON** (.json): For JSON-formatted data.
* **CSV** (.csv): Comma-separated values with a header row.
* **TSV** (.tsv): Tab-separated values with a header row.

### Accessing Data in Templates

Data files are available via `site.data.<filename>` (without extension). Subdirectories create nested objects:

```liquid
<!-- Using data from _data/navigation.yml -->
<nav>
  {% for item in site.data.navigation %}
    <a href="{{ item.url }}">{{ item.title }}</a>
  {% endfor %}
</nav>

<!-- Using nested data from _data/locales/en.yml -->
<p>{{ site.data.locales.en.greeting }}</p>
```

### Example Data File

A navigation menu in YAML (`_data/navigation.yml`):

```yaml
- title: Home
  url: /

- title: About
  url: /about/

- title: Blog
  url: /blog/
```

### Processing of Data Files

* YAML and JSON files are parsed into nested objects/arrays
* CSV/TSV files are converted to arrays of hashes where column headers become keys
* Files in subdirectories are accessible via nested paths

## Combined Examples: Using Collections with Data

These systems often work together. For example, a site might use:

1. A "projects" collection for detailed project pages
2. A data file for project categories or featured projects

```liquid
<!-- Display projects by category -->
{% for category in site.data.project_categories %}
  <h2>{{ category.name }}</h2>
  {% assign projects = site.projects | where: "category", category.slug %}
  {% for project in projects %}
    <h3><a href="{{ project.url }}">{{ project.title }}</a></h3>
    <p>{{ project.excerpt }}</p>
  {% endfor %}
{% endfor %}
```

## Metadata Files

Jekyll also supports `.jekyll-metadata` for incremental builds. This isn't normally accessed directly in templates but is used by Jekyll to track file modification times. Your SSG might implement a similar tracking system for incremental builds.

## Implementing Collections and Data in Rust

### Collections Implementation

1. Parse the `collections` configuration from `_config.yml`
2. For each collection:
   - Read all files in the corresponding `_<collection-name>` directory
   - Parse front matter and content
   - Apply the appropriate converter (e.g., Markdown to HTML)
   - Generate output files if `output: true`
   - Make collection documents available in the template context

### Data Implementation

1. Scan the `_data` directory for files
2. Parse each file according to its format (YAML, JSON, CSV, TSV)
3. Build a nested structure matching the directory hierarchy
4. Make data available in the template context as `site.data.<filename>`

### Code Example (Pseudocode)

Here's a simplified approach to implementing collections:

```rust
// Define types for collections
struct Collection {
    label: String,
    output: bool,
    permalink: Option<String>,
    documents: Vec<Document>,
    // Other metadata
}

struct Document {
    content: String,
    front_matter: HashMap<String, Value>,
    collection: String,
    url: Option<String>,
    date: Option<DateTime>,
    // Other metadata
}

// Read collections from config and filesystem
fn process_collections(config: &Config, site_source: &Path) -> Vec<Collection> {
    let mut collections = Vec::new();
    
    // Always include posts collection
    let mut posts = Collection {
        label: "posts".to_string(),
        output: true,
        permalink: Some("/:categories/:year/:month/:day/:title:output_ext".to_string()),
        documents: Vec::new(),
    };
    
    // Read posts
    let posts_dir = site_source.join("_posts");
    if posts_dir.exists() {
        for entry in fs::read_dir(posts_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |ext| ext == "md" || ext == "html") {
                let (front_matter, content) = parse_file(&path);
                
                // Parse date from filename (YYYY-MM-DD-title.md)
                let date = parse_date_from_filename(&path);
                
                let doc = Document {
                    content,
                    front_matter,
                    collection: "posts".to_string(),
                    date,
                    // Generate URL based on permalink pattern
                    url: Some(generate_url("posts", &front_matter, &permalink_pattern)),
                };
                
                posts.documents.push(doc);
            }
        }
    }
    
    collections.push(posts);
    
    // Read custom collections from config
    if let Some(collection_configs) = config.get("collections") {
        for (label, settings) in collection_configs.as_mapping().unwrap() {
            let output = settings.get("output").map_or(false, |v| v.as_bool().unwrap_or(false));
            let permalink = settings.get("permalink").and_then(|v| v.as_str()).map(String::from);
            
            let mut collection = Collection {
                label: label.as_str().unwrap().to_string(),
                output,
                permalink,
                documents: Vec::new(),
            };
            
            // Read documents for this collection
            let collection_dir = site_source.join(format!("_{}", label.as_str().unwrap()));
            if collection_dir.exists() {
                for entry in fs::read_dir(collection_dir).unwrap() {
                    let path = entry.unwrap().path();
                    if path.extension().map_or(false, |ext| ext == "md" || ext == "html") {
                        let (front_matter, content) = parse_file(&path);
                        
                        let doc = Document {
                            content,
                            front_matter,
                            collection: label.as_str().unwrap().to_string(),
                            // Generate URL based on permalink pattern if output is true
                            url: if output {
                                Some(generate_url(&label.as_str().unwrap(), &front_matter, &permalink))
                            } else {
                                None
                            },
                            date: front_matter.get("date").and_then(|d| parse_date(d)),
                        };
                        
                        collection.documents.push(doc);
                    }
                }
            }
            
            collections.push(collection);
        }
    }
    
    collections
}
```

For data files:

```rust
// Read data files
fn process_data_files(site_source: &Path) -> HashMap<String, Value> {
    let mut data = HashMap::new();
    let data_dir = site_source.join("_data");
    
    if data_dir.exists() {
        process_data_directory(&data_dir, &mut data);
    }
    
    data
}

fn process_data_directory(dir: &Path, data: &mut HashMap<String, Value>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        
        if path.is_dir() {
            // Handle subdirectory - create nested HashMap
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            let mut subdir_data = HashMap::new();
            process_data_directory(&path, &mut subdir_data);
            data.insert(dir_name.to_string(), Value::Object(subdir_data));
        } else {
            // Handle file based on extension
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let extension = path.extension().unwrap().to_str().unwrap();
            
            let file_data = match extension {
                "yml" | "yaml" => {
                    let content = fs::read_to_string(&path).unwrap();
                    serde_yaml::from_str(&content).unwrap()
                },
                "json" => {
                    let content = fs::read_to_string(&path).unwrap();
                    serde_json::from_str(&content).unwrap()
                },
                "csv" => {
                    let file = File::open(&path).unwrap();
                    let mut reader = csv::Reader::from_reader(file);
                    let records: Result<Vec<_>, _> = reader.deserialize().collect();
                    Value::Array(records.unwrap())
                },
                "tsv" => {
                    let file = File::open(&path).unwrap();
                    let mut reader = csv::ReaderBuilder::new()
                        .delimiter(b'\t')
                        .from_reader(file);
                    let records: Result<Vec<_>, _> = reader.deserialize().collect();
                    Value::Array(records.unwrap())
                },
                _ => continue, // Skip unknown formats
            };
            
            data.insert(file_stem.to_string(), file_data);
        }
    }
}
```

## Design Considerations

When implementing collections and data in your Rust SSG, consider:

1. **Efficiency**: Collections can be large. Implement efficient reading and processing.
2. **Extensibility**: Allow plugins to add or modify collections and data.
3. **Compatibility**: Ensure Liquid templates can access collections and data the same way as in Jekyll.
4. **Flexibility**: Support all Jekyll's collection configuration options.
5. **Special handling for posts**: Maintain Jekyll's special behavior for posts.

By supporting both Collections and Data Files with their full capabilities, your Rust SSG will enable rich content modeling identical to Jekyll's. This allows users to migrate existing Jekyll sites without modification of their content structure. 