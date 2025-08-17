use crate::Recipe;

use std::fmt::Write;

pub struct MarkdownBuilder<'a> {
    recipe: &'a Recipe,
    url: Option<&'a str>,
    ingredient_section_name: &'a str,
    default_recipe_name: &'a str,
    default_section_name: &'a str,
    result: String,
}

impl<'a> MarkdownBuilder<'a> {
    const PROPERTY_MARKER: &'static str = "---";

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

    pub fn with_url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_ingredient_section(mut self, name: &'a str) -> Self {
        self.ingredient_section_name = name;
        self
    }

    pub fn with_default_name(mut self, name: &'a str) -> Self {
        self.default_recipe_name = name;
        self
    }

    pub fn with_default_section(mut self, name: &'a str) -> Self {
        self.default_section_name = name;
        self
    }

    pub fn convert(mut self) -> String {
        self.put_url_author();
        self.put_name();
        self.put_description();
        self.put_ingredients();
        self.put_sections();
        self.result
    }

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

    fn put_name(&mut self) {
        let name = self
            .recipe
            .name
            .as_deref()
            .unwrap_or(self.default_recipe_name);
        writeln!(self.result, "# {name}").unwrap();
    }

    fn put_description(&mut self) {
        if let Some(description) = &self.recipe.description {
            writeln!(self.result).unwrap();
            writeln!(self.result, "{description}").unwrap();
        }
    }

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

    fn put_sections(&mut self) {
        for section in &self.recipe.how_to_sections {
            let section_name = section.name.as_deref().unwrap_or(self.default_section_name);
            writeln!(self.result).unwrap();
            writeln!(self.result, "## {section_name}").unwrap();
            if !section.steps.is_empty() {
                writeln!(self.result).unwrap();
            }
            for step in &section.steps {
                writeln!(self.result, "{step}").unwrap();
            }
        }
    }
}
