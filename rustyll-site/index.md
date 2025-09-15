---
layout: default
title: Welcome to Rustyll
---

# Welcome to Rustyll

Rustyll is a blazing-fast, Jekyll-compatible static site generator written in Rust. Experience the power and safety of Rust with the familiarity of Jekyll.

## Features

- ⚡ **Lightning Fast**: Build times measured in milliseconds, not minutes
- 🔄 **Jekyll Compatible**: Easy migration from existing Jekyll sites
- 📝 **Powerful Markdown**: Advanced markdown rendering with extensions
- 💧 **Liquid Templates**: Full Liquid templating support
- 🚀 **Incremental Builds**: Only rebuild what changed
- 🔧 **Extensible**: Plugin system for custom functionality

## Recent Posts

{% for post in site.posts limit:5 %}
- [{{ post.title }}]({{ post.url | relative_url }}) - {{ post.date | date }}
{% endfor %}

## Getting Started

```bash
# Install Rustyll
cargo install rustyll

# Create a new site
rustyll new my-site
cd my-site

# Build your site
rustyll build

# Serve locally
rustyll serve
```

## Demonstration

This demo site showcases Rustyll's capabilities:

### Syntax Highlighting

{% highlight rust %}
fn main() {
    println!("Hello from Rustyll!");

    let site = Site::new("_config.yml")?;
    site.build()?;
}
{% endhighlight %}

### Collections Support

Browse our [projects](/projects) and [documentation](/docs) to see collections in action.

## Performance

Rustyll builds this demo site in under 100ms, making it one of the fastest static site generators available.

---

Ready to get started? Check out our [documentation](/docs) or dive into the [source code](https://github.com/better-web-initiative/rustyll).