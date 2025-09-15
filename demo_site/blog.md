---
layout: page
title: Blog
permalink: /blog/
---

# Blog

Welcome to the Rustyll blog! Here you'll find articles about static site generation, Rust development, and web performance.

## Recent Posts

<ul>
{% for post in site.posts limit:5 %}
<li>
  <strong><a href="{{ post.url }}">{{ post.title }}</a></strong> - {{ post.date | date_to_string }}<br>
  <em>{{ post.content | strip_html | truncatewords: 20 }}</em>
</li>
{% endfor %}
</ul>

## Categories

Browse posts by category:

- [Announcements](/category/announcements/)
- [Tutorials](/category/tutorials/)
- [Performance](/category/performance/)

---

*Subscribe to our RSS feed to stay updated with the latest posts.*