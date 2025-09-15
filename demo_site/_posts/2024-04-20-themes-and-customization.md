---
layout: post
title: "Themes and Customization in Rustyll"
date: 2024-04-20 16:45:00 +0000
categories: [tutorials, themes]
tags: [themes, customization, design]
author: "Design Team"
---

Discover how to customize your Rustyll site with themes, layouts, and advanced styling options.

## Built-in Themes

Rustyll comes with several built-in themes:

- **Minima**: Clean, minimal design (default)
- **Cayman**: GitHub Pages compatible theme
- **Architect**: Technical documentation theme
- **Midnight**: Dark theme for developers

## Creating Custom Layouts

Layouts in Rustyll use the same Liquid templating as Jekyll:

```liquid
<!DOCTYPE html>
<html>
<head>
  <title>{{ page.title }} - {{ site.title }}</title>
</head>
<body>
  {% raw %}{% include header.html %}{% endraw %}
  <main>
    {{ content }}
  </main>
  {% raw %}{% include footer.html %}{% endraw %}
</body>
</html>
```

## Sass Support

Rustyll has built-in Sass support for advanced styling:

```scss
$primary-color: #3498db;
$font-stack: 'Helvetica Neue', sans-serif;

.header {
  background-color: $primary-color;
  font-family: $font-stack;

  .nav-link {
    color: white;
    &:hover {
      opacity: 0.8;
    }
  }
}
```

## Asset Pipeline

The asset pipeline automatically:
- Compiles Sass to CSS
- Minifies JavaScript
- Optimizes images
- Generates source maps for development