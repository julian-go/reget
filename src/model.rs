/// A single ingredient used in a recipe
pub type Ingredient = String;

/// A single how-to-step of a recipe
pub type HowToStep = String;

/// A collection of how-to-steps with an optional title
#[derive(Default, Debug, PartialEq)]
pub struct HowToSection {
    pub name: Option<String>,
    pub steps: Vec<HowToStep>,
}

/// A recipe
#[derive(Default, Debug, PartialEq)]
pub struct Recipe {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub how_to_sections: Vec<HowToSection>,
}
