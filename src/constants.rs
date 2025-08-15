pub struct LdFields;

/// Constants for JSON-LD fields used in the recipe schema
impl LdFields {
    pub const TYPE: &'static str = "@type";
    pub const NAME: &'static str = "name";
    pub const TEXT: &'static str = "text";
    pub const AUTHOR: &'static str = "author";
    pub const DESCRIPTION: &'static str = "description";
    pub const RECIPE_INGREDIENT: &'static str = "recipeIngredient";
    pub const RECIPE_INSTRUCTIONS: &'static str = "recipeInstructions";
    pub const ITEM_LIST_ELEMENT: &'static str = "itemListElement";
}
