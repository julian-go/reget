use reget::parse_recipe;
use std::fs;

fn main() {
    let html = fs::read_to_string("examples/recipe.html").unwrap();
    println!("{:#?}", parse_recipe(&html).unwrap());
}
