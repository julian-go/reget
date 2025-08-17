/// A recipe extracted from HTML using [parse_recipe](crate::parse_recipe).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// The name of the recipe.
    pub name: Option<String>,
    /// The author of the recipe.
    ///
    /// If the recipe specifies multiple authors, they will be joined into a single string
    /// (e.g. "Author One, Author Two")
    pub author: Option<String>,
    /// The description of the recipe.
    pub description: Option<String>,
    /// A list of ingredients used by the recipe.
    pub ingredients: Vec<Ingredient>,
    /// A list of [how-to-sections](HowToSection) for the recipe.
    ///
    /// If the recipe does not use [how-to-sections](HowToSection) this will contain
    /// a single section without a name.
    pub how_to_sections: Vec<HowToSection>,
}

/// A collection of [how-to-steps](HowToStep) with an optional name
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct HowToSection {
    /// The name of the section, if available.
    pub name: Option<String>,
    /// A list of steps in the section.
    pub steps: Vec<HowToStep>,
}

/// A single ingredient used in a recipe
pub type Ingredient = String;

/// A single how-to-step of a recipe
pub type HowToStep = String;

impl Recipe {
    #[cfg(feature = "markdown")]
    pub fn to_markdown(&self) -> crate::MarkdownBuilder {
        crate::MarkdownBuilder::from(self)
    }
}
