use crate::Recipe;

use std::fmt::Write;

/// A builder to convert the contents of a recipe a markdown string
///
/// ## Example
///
/// ```
/// let recipe = reget::Recipe::default();
/// recipe
///     .to_markdown()
///     .with_url("https://example.org/recipe")
///     .with_ingredient_section("Ingredients")
///     .with_default_section("Preparation")
///     .convert();
/// ```
///
/// <details>
/// <summary>Example Output</summary>
///
/// ```text
/// ---
/// author: ...
/// url: ...
/// ---
///
/// # Recipe Name
///
/// This is the description.
///
/// # Ingredients
///
/// - Ingredient 1
/// - Ingredient 2
///
/// # Preparation
///
/// Step 1 do xyz.
///
/// Do abc for step 2.
/// ```
/// </details>
///
pub struct MarkdownBuilder<'a> {
    /// The recipe that is being converted
    recipe: &'a Recipe,
    /// The URL where the recipe stems from,
    url: Option<&'a str>,
    /// The name being used for the ingredient section
    ingredient_section_name: &'a str,
    /// The name being used if the recipe does not have a name
    default_recipe_name: &'a str,
    /// The name being used if a how to section does not have a name
    default_section_name: &'a str,
    /// The output string being built
    result: String,
}

impl<'a> MarkdownBuilder<'a> {
    const PROPERTY_MARKER: &'static str = "---";

    /// Constructs a new MarkdownBuilder for a [recipe](Recipe)
    pub fn from(recipe: &'a Recipe) -> Self {
        MarkdownBuilder {
            recipe,
            url: None,
            ingredient_section_name: "Ingredients",
            default_recipe_name: "Recipe",
            default_section_name: "Instructions",
            result: String::new(),
        }
    }

    /// Adds an optional URL do be used when creating the markdown
    pub fn with_url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    /// Uses the name for the ingredient section, default is *Ingredients*
    pub fn with_ingredient_section(mut self, name: &'a str) -> Self {
        self.ingredient_section_name = name;
        self
    }

    /// Uses the name if the recipe does not have a name included, default
    /// is *Recipe*
    pub fn with_default_name(mut self, name: &'a str) -> Self {
        self.default_recipe_name = name;
        self
    }

    /// Uses the name for any section that does not have a name, default is
    /// *Instructions*
    pub fn with_default_section(mut self, name: &'a str) -> Self {
        self.default_section_name = name;
        self
    }

    /// Performs the conversion
    pub fn convert(mut self) -> String {
        self.put_url_author();
        self.put_name();
        self.put_description();
        self.put_ingredients();
        self.put_sections();
        self.result
    }

    /// Writes URL and author to the output string, if there are any
    fn put_url_author(&mut self) {
        if self.url.is_some() || self.recipe.author.is_some() {
            writeln!(self.result, "{}", Self::PROPERTY_MARKER).unwrap();

            if let Some(url) = self.url {
                writeln!(self.result, "url: {url}").unwrap();
            }
            if let Some(author) = &self.recipe.author {
                writeln!(self.result, "author: {author}").unwrap();
            }

            writeln!(self.result, "{}", Self::PROPERTY_MARKER).unwrap();
            writeln!(self.result).unwrap();
        }
    }

    /// Writes recipe name or the default name to the output string
    fn put_name(&mut self) {
        let name = self
            .recipe
            .name
            .as_deref()
            .unwrap_or(self.default_recipe_name);
        writeln!(self.result, "# {name}").unwrap();
    }

    /// Writes description to the output string, if there is one
    fn put_description(&mut self) {
        if let Some(description) = &self.recipe.description {
            writeln!(self.result).unwrap();
            writeln!(self.result, "{description}").unwrap();
        }
    }

    /// Writes ingredients section to the output string, if there are any
    fn put_ingredients(&mut self) {
        if !self.recipe.ingredients.is_empty() {
            writeln!(self.result).unwrap();
            writeln!(self.result, "## {}", self.ingredient_section_name).unwrap();
            writeln!(self.result).unwrap();
            for ingredient in &self.recipe.ingredients {
                writeln!(self.result, "- {ingredient}").unwrap();
            }
        }
    }

    /// Writes the different sections to the output string, if there are any
    fn put_sections(&mut self) {
        for section in &self.recipe.how_to_sections {
            let section_name = section.name.as_deref().unwrap_or(self.default_section_name);
            writeln!(self.result).unwrap();
            writeln!(self.result, "## {section_name}").unwrap();
            for step in &section.steps {
                writeln!(self.result).unwrap();
                writeln!(self.result, "{step}").unwrap();
            }
        }
    }
}
