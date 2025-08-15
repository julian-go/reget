# reget

A simple library for extracting from HTML documents using structured data (JSON-LD) embedded within.

This library assumes the document follows the [schema.org recipe specification](https://schema.org/Recipe).

## Usage

```rust
use reget::parse_recipe;

fn main() {
    let html = "<html>...</html>"; // HTML with JSON-LD recipe data
    let recipe = parse_recipe(html).unwrap();
}
```

Or try the [example](examples/parse.rs).