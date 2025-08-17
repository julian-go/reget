#[cfg(feature = "markdown")]
fn main() {
    let html = std::fs::read_to_string("examples/recipe.html").unwrap();
    let recipe = reget::parse_recipe(&html).unwrap();
    let md = recipe
        .to_markdown()
        .with_url("https://example.org/recipe")
        .with_ingredient_section("Zutaten")
        .with_default_section("Zubereitung")
        .convert();
    println!("{md}");
}

#[cfg(not(feature = "markdown"))]
fn main() {
    eprintln!(
        "This example requires the 'markdown' feature. Try `cargo run --features markdown --example markdown`"
    );
}
