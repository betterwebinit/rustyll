# Layouts Directory

This directory contains layout templates migrated from Jekyll.

## Layout Usage

Layouts in Rustyll:
- Define the HTML structure of pages
- Can be specified in front matter using `layout: template_name`
- Can inherit from other layouts

Example usage in a content file:
```yaml
---
layout: default
title: My Page
---

Content goes here...
```

## Changes from Jekyll

The layout system in Rustyll is compatible with Jekyll, but some specific features
or extensions might need adjustments.
