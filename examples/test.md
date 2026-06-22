---
title: Glypho Test Document
author: Glypho Project
---

# Glypho Markdown Converter

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()

Glypho is a fast, lightweight tool for converting Markdown documents to styled HTML. It supports syntax highlighting, mathematical expressions, and responsive layouts.

- Write content in standard Markdown
- Preview rendered HTML instantly
- Export to a single self-contained file

## Task List

- [x] Implement markdown parsing
- [x] Add syntax highlighting
- [ ] Support custom themes
- [x] Add math rendering with KaTeX

---

## Sample Code

### Rust Build Script

```rust
use std::fs;
use std::path::Path;
use handlebars::Handlebars;
use std::collections::BTreeMap;

fn main() -> eyre::Result<()> {
    let mut hb = Handlebars::new();
    let css = fs::read_to_string("./src/assets/style.css")?;
    let js = fs::read_to_string("./src/assets/prism.js")?;
    let index = fs::read_to_string("./src/assets/index.html")?;

    hb.register_template_string("index.html", index)?;

    let mut data = BTreeMap::new();
    data.insert("css".to_string(), css);
    data.insert("js".to_string(), js);

    let rendered = hb.render("index.html", &data)?;
    let dest = Path::new("./src/").join("template.html");
    fs::write(&dest, rendered)?;

    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
```

### Shell Commands

```sh
glypho build document.md --output index.html
glypho serve --port 8080 --watch
glypho export report.md --theme dark
```

---

## Features

| Feature | Library | Status |
|---------|---------|--------|
| Markdown parsing | pulldown-cmark | Implemented |
| Syntax highlighting | Prism.js | Implemented |
| Math rendering | KaTeX | Implemented |
| Table of contents | Built-in | Planned |
| PDF export | None yet | Planned |

---

## Blockquote

> Good tools make complex tasks feel simple. The goal of Glypho is to remove friction from publishing technical documentation while maintaining full compatibility with standard Markdown.

---

## Inline Math

The Pythagorean theorem states that $a^2 + b^2 = c^2$ for right triangles.

A more complex identity:

$$
e^{i\pi} + 1 = 0
$$

---

## Links and References

- [CommonMark Spec](https://commonmark.org)
- [GitHub Flavored Markdown](https://github.github.com/gfm/)
- [KaTeX Documentation](https://katex.org/docs/supported.html)

---

## License

MIT License — Free for personal and commercial use.
