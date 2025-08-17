# reget
[![Crates.io](https://img.shields.io/crates/v/reget)](https://crates.io/crates/reget)
![License](https://img.shields.io/badge/license-MIT-blue) 

A simple library for extracting a [recipe](src/model.rs) from HTML documents using structured data (JSON-LD) embedded within.

With the optional `markdown` feature, recipes can be converted to a markdown string.

This library assumes the document follows the [schema.org recipe specification](https://schema.org/Recipe).

## Installation

```bash
cargo add reget
```

## Usage

```rust
use reget::parse_recipe;

fn main() {
    let html = "<html>...</html>"; // HTML with JSON-LD recipe data
    let recipe = parse_recipe(html).unwrap();
}
```

## Markdown support

The optional markdown feature can be used to convert recipes directly to markdown.

```rust
let md = recipe
    .to_markdown()
    .with_url("https://example.org/recipe")
    .convert();
```

Or try the [examples](examples/).