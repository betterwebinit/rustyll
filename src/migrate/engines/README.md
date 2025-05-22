# Rustyll Engine Migrators

This directory contains migrators for various static site generators and documentation platforms that can be converted to Rustyll format.

## Directory Structure

Each engine migrator is contained in its own directory and follows a standard structure:

```
engines/
├── hugo/               # Hugo migrator
│   ├── mod.rs          # Main implementation
│   ├── config.rs       # Configuration migration 
│   ├── content.rs      # Content migration
│   ├── ...             # Other component files
│   └── README.md       # Engine-specific documentation
├── jekyll/             # Jekyll migrator
│   ├── ...
├── ...                 # Other engines
└── README.md           # This file
```

## Available Migrators

Rustyll can migrate sites from the following engines:

| Engine | Description |
|--------|-------------|
| Jekyll | Ruby-based SSG with Liquid templates |
| Hugo | Go-based SSG with fast build times |
| Zola | Rust-based SSG using Tera templates |
| Eleventy | JavaScript-based SSG with multiple template options |
| Gatsby | React-based JavaScript framework |
| Docsy | Hugo-based documentation theme |
| MDBook | Rust documentation generator |
| MkDocs | Python documentation generator |
| GitBook | Documentation platform |
| Slate | API documentation generator |
| Pelican | Python-based SSG |
| Nanoc | Ruby-based SSG |
| Middleman | Ruby-based SSG |
| Assemble | JavaScript-based SSG |
| Bridgetown | Ruby-based SSG (Jekyll successor) |
| Cobalt | Rust-based SSG |
| Fresh | Deno-based SSG |
| Harp | JavaScript-based SSG |
| Jigsaw | PHP-based SSG |
| Metalsmith | JavaScript-based SSG |
| Nikola | Python-based SSG |
| Octopress | Jekyll-based blogging platform |
| Sphinx | Python documentation generator |

## Adding New Migrators

To add a new migrator:

1. Create a new directory with the engine name
2. Implement the `EngineMigrator` trait in the module
3. Ensure detection logic is accurate and efficient
4. Add the engine to the exports in `engines.rs`
5. Add the engine to the match statements in `get_migrator()` and `detect_engine()` 