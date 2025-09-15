# Data Directory

This directory contains data files migrated from Jekyll.

## Data Usage

Data files in Rustyll work the same way as in Jekyll:
- YAML, JSON, and CSV files can store structured data
- Data is accessible in templates through the `site.data` object

## Examples

For a file named `_data/navigation.yml` containing:
```yaml
- title: Home
  url: /
- title: About
  url: /about/
```

In a template, you'd use:
```liquid
<nav>
  <ul>
    {% for item in site.data.navigation %}
      <li><a href="{{ item.url }}">{{ item.title }}</a></li>
    {% endfor %}
  </ul>
</nav>
```

## Changes from Jekyll

Data functionality in Rustyll is compatible with Jekyll, but some advanced
features might require adjustments.
