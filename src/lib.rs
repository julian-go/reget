//! # Recipe Extraction from HTML documents
//!
//! `reget` provides a [single function](parse_recipe) to extract a [recipe](Recipe) from HTML documents
//! using structured data (JSON-LD) embedded within.
//!
//! This library assumes the document follows the [schema.org recipe specification](https://schema.org/Recipe).
//!
//! ## Example
//!
//! ```
//! use reget::parse_recipe;
//!
//! let html = r#"
//! <!DOCTYPE html>
//! <html lang="en">
//! <script type="application/ld+json">
//! {
//!   "@type": "Recipe",
//!   "name": "Delicious Cookies",
//!   "author": "Lorem Ipsum",
//!   "recipeIngredient": ["2 cups flour", "1 cup sugar"],
//!   "recipeInstructions": "Mix ingredients and bake."
//! }
//! </script>
//! </html>
//! "#;
//!
//! let recipe = parse_recipe(html).unwrap();
//! ```

mod constants;
#[cfg(feature = "markdown")]
mod markdown;
mod model;

use constants::LdFields;
#[cfg(feature = "markdown")]
pub use markdown::MarkdownBuilder;
pub use model::{HowToSection, HowToStep, Ingredient, Recipe};

use scraper::{Html, Selector};
use serde_json::{Map, Value};

const JSON_LD_SELECTOR: &str = r#"script[type="application/ld+json"]"#;
const RECIPE_TYPE: &str = "Recipe";
const HOW_TO_SECTION_TYPE: &str = "HowToSection";

/// Parses the [recipe](Recipe) from the given HTML document. Will return None if no
/// linked data is found in the document.
///
/// This function will only extract the first recipe it finds and only if it follows
/// [schema.org recipe specification](https://schema.org/Recipe).
///
/// For an example see [here](crate).
pub fn parse_recipe(html: &str) -> Option<Recipe> {
    let json = extract_recipe_json(html)?;
    Some(extract_recipe(&json))
}

fn extract_recipe(json: &Map<String, Value>) -> Recipe {
    Recipe {
        name: json
            .get(LdFields::NAME)
            .and_then(Value::as_str)
            .map(String::from),
        author: json.get(LdFields::AUTHOR).and_then(extract_author),
        description: json
            .get(LdFields::DESCRIPTION)
            .and_then(Value::as_str)
            .map(String::from),
        ingredients: json
            .get(LdFields::RECIPE_INGREDIENT)
            .map(extract_ingredients)
            .unwrap_or_default(),
        how_to_sections: json
            .get(LdFields::RECIPE_INSTRUCTIONS)
            .map(extract_instructions)
            .unwrap_or_default(),
    }
}

/// Looks for `type="application/ld+json"` in the provided html with `"@type": Recipe`.
fn extract_recipe_json(html: &str) -> Option<Map<String, Value>> {
    let sel = Selector::parse(JSON_LD_SELECTOR).unwrap();
    let document = Html::parse_document(html);

    for e in document.select(&sel) {
        let s = e.text().collect::<String>();

        let value = match serde_json::from_str::<Value>(&s) {
            Ok(val) => val,
            Err(_) => continue, // parsing json failed
        };

        match find_recipe_in_value(value) {
            Some(val) => return Some(val),
            None => continue, // this is not the recipe
        };
    }
    None
}

/// Tries to recursively find a recipe by looking for the tag `"@type": Recipe`.
fn find_recipe_in_value(value: Value) -> Option<Map<String, Value>> {
    match value {
        Value::Object(obj) => {
            if is_recipe_type(&obj) {
                return Some(obj);
            }
            for (_, v) in obj {
                if let Some(recipe) = find_recipe_in_value(v) {
                    return Some(recipe);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(recipe) = find_recipe_in_value(item) {
                    return Some(recipe);
                }
            }
        }
        _ => {}
    }
    None
}

/// Verifies that the obj contains the tag `"@type": Recipe`.
fn is_recipe_type(obj: &serde_json::Map<String, Value>) -> bool {
    match obj.get(LdFields::TYPE) {
        Some(Value::String(s)) => s == RECIPE_TYPE,
        Some(Value::Array(arr)) => arr
            .iter()
            .any(|t| matches!(t, Value::String(type_str) if type_str == RECIPE_TYPE)),
        _ => false,
    }
}

/// Extracts the author
///
/// It deals with:
///     - "author": "first last",
///     - "author": { "name": "first last" },
///     - "author": [ "first last", "first last" ]
///     - "author": [ { "name": "first last" }, { "name": "first last" } ]
///
/// For arrays of authors it returns them as a comma separated string
fn extract_author(value: &serde_json::Value) -> Option<String> {
    match value {
        // If the field is just a string, return the string
        Value::String(str) => Some(str.clone()),
        // If the field has a name field, return its value
        Value::Object(obj) => match obj.get(LdFields::NAME) {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        },
        // If it is an array, return them as a comma seperated list
        Value::Array(arr) => {
            let strings = arr
                .iter()
                .filter_map(extract_author)
                .collect::<Vec<String>>();
            if strings.is_empty() {
                None
            } else {
                Some(strings.join(", "))
            }
        }
        _ => None,
    }
}

/// Extracts the ingredients
///
/// It deals with:
///     - "recipeIngredient": "ingredient",
///     - "recipeIngredient": [ "ingredient1", "ingredient2" ]
fn extract_ingredients(value: &serde_json::Value) -> Vec<Ingredient> {
    match value {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|ingredient| match ingredient {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        Value::String(s) => vec![s.to_string()],
        _ => vec![],
    }
}

/// Extracts the instructions
///
/// It deals with:
///     - "recipeInstructions": "step text"
///     - "recipeInstructions": [ "step1", "step2" ]
///     - "recipeInstructions": [ { "text": "step1" }, { "text": "step2" } ]
///     - "recipeInstructions": [ { "@type": "HowToSection", "name": "...", "itemListElement": [...] }, ... ]
///
/// For HowToSection objects, each section contains an array of steps.
/// For arrays of steps or plain text, returns a single section with all steps concatenated.
fn extract_instructions(value: &serde_json::Value) -> Vec<HowToSection> {
    match value {
        // Array of sections or steps
        Value::Array(arr) => {
            // If any item is a HowToSection, treat as sections
            if arr.iter().any(is_how_to_section_obj) {
                arr.iter().filter_map(extract_section).collect()
            } else {
                vec![HowToSection {
                    name: None,
                    steps: arr.iter().flat_map(extract_step).collect(),
                }]
            }
        }
        // Single section object
        Value::Object(_) if is_how_to_section_obj(value) => {
            extract_section(value).into_iter().collect()
        }
        // Single step or text
        _ => vec![HowToSection {
            name: None,
            steps: extract_step(value),
        }],
    }
}

/// Extracts a [HowToSection]
fn extract_section(value: &serde_json::Value) -> Option<HowToSection> {
    if let Value::Object(obj) = value {
        let name = obj
            .get(LdFields::NAME)
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let steps_val = obj.get(LdFields::ITEM_LIST_ELEMENT).unwrap_or(value);
        Some(HowToSection {
            name,
            steps: extract_step(steps_val),
        })
    } else {
        None
    }
}

/// Extracts a [HowToStep]
fn extract_step(value: &serde_json::Value) -> Vec<HowToStep> {
    match value {
        Value::Array(arr) => arr.iter().flat_map(extract_step).collect(),
        Value::String(text) => vec![text.trim().to_string()],
        Value::Object(obj) => obj
            .get(LdFields::TEXT)
            .and_then(Value::as_str)
            .map(|text| vec![text.trim().to_string()])
            .unwrap_or_default(),
        _ => vec![],
    }
}

/// Determines if the value is a [HowToSection] object.
fn is_how_to_section_obj(value: &serde_json::Value) -> bool {
    matches!(
        value,
        Value::Object(obj) if obj.get(LdFields::TYPE) == Some(&Value::String(HOW_TO_SECTION_TYPE.into()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    mod html_extraction {
        use super::*;

        #[test]
        fn basic_1() {
            let html = include_str!("../tests/fixtures/basic_1.html");
            let recipe = parse_recipe(html).unwrap();
            assert_eq!(
                recipe,
                Recipe {
                    name: Some("recipe_name".into()),
                    author: Some("author_name".into()),
                    description: Some("description".into()),
                    ingredients: vec!["ingredient_1".into(), "ingredient_2".into()],
                    how_to_sections: vec![HowToSection {
                        name: None,
                        steps: vec!["instruction_1".into()],
                    }]
                }
            )
        }

        #[test]
        fn basic_2() {
            let html = include_str!("../tests/fixtures/basic_2.html");
            let recipe = parse_recipe(html).unwrap();
            assert_eq!(
                recipe,
                Recipe {
                    name: Some("recipe_name".into()),
                    author: Some("author_name".into()),
                    description: Some("description".into()),
                    ingredients: vec!["ingredient_1".into(), "ingredient_2".into()],
                    how_to_sections: vec![HowToSection {
                        name: None,
                        steps: vec!["instruction_1".into(), "instruction_2".into()],
                    }]
                }
            )
        }

        #[test]
        fn basic_3() {
            let html = include_str!("../tests/fixtures/basic_3.html");
            let recipe = parse_recipe(html).unwrap();
            assert_eq!(
                recipe,
                Recipe {
                    name: Some("recipe_name".into()),
                    author: Some("author_name".into()),
                    description: Some("description".into()),
                    ingredients: vec!["ingredient_1".into(), "ingredient_2".into()],
                    how_to_sections: vec![HowToSection {
                        name: None,
                        steps: vec!["instruction_1".into(), "instruction_2".into()],
                    }]
                }
            )
        }

        #[test]
        fn basic_4() {
            let html = include_str!("../tests/fixtures/basic_4.html");
            let recipe = parse_recipe(html).unwrap();
            assert_eq!(
                recipe,
                Recipe {
                    name: Some("recipe_name".into()),
                    author: Some("author_name".into()),
                    description: Some("description".into()),
                    ingredients: vec!["ingredient_1".into(), "ingredient_2".into()],
                    how_to_sections: vec![
                        HowToSection {
                            name: None,
                            steps: vec!["instruction_1".into(), "instruction_2".into()],
                        },
                        HowToSection {
                            name: Some("section_2".into()),
                            steps: vec!["instruction_3".into(), "instruction_4".into()],
                        }
                    ]
                }
            )
        }

        #[test]
        fn basic_5() {
            let html = include_str!("../tests/fixtures/basic_5.html");
            let recipe = parse_recipe(html).unwrap();
            assert_eq!(
                recipe,
                Recipe {
                    name: Some("recipe_name".into()),
                    author: Some("author_name".into()),
                    description: Some("description".into()),
                    ingredients: vec!["ingredient_1".into(), "ingredient_2".into()],
                    how_to_sections: vec![
                        HowToSection {
                            name: Some("section_1".into()),
                            steps: vec!["instruction_1".into(), "instruction_2".into()],
                        },
                        HowToSection {
                            name: Some("section_2".into()),
                            steps: vec!["instruction_3".into(), "instruction_4".into()],
                        }
                    ]
                }
            )
        }
    }

    mod name {
        use super::*;

        #[test]
        fn extract_simple_name() {
            let json = json!({"name": "Recipe Name"});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.name, Some("Recipe Name".to_string()));
        }

        #[test]
        fn extract_missing_name() {
            let json = json!({"description": "Recipe Name"});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.name, None);
        }

        #[test]
        fn extract_non_string_name() {
            let json = json!({"name": 123});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.name, None);
        }
    }

    mod author {
        use super::*;

        #[test]
        fn extract_string_author() {
            let value = Value::String("John Doe".to_string());
            let result = extract_author(&value);
            assert_eq!(result, Some("John Doe".to_string()));
        }

        #[test]
        fn extract_object_author() {
            let value = json!({"name": "Jane Smith"});
            let result = extract_author(&value);
            assert_eq!(result, Some("Jane Smith".to_string()));
        }

        #[test]
        fn extract_object_author_no_name() {
            let value = json!({"email": "test@example.com"});
            let result = extract_author(&value);
            assert_eq!(result, None);
        }

        #[test]
        fn extract_array_authors() {
            let value = json!(["John Doe", "Jane Smith"]);
            let result = extract_author(&value);
            assert_eq!(result, Some("John Doe, Jane Smith".to_string()));
        }

        #[test]
        fn extract_array_object_authors() {
            let value = json!([
                {"name": "John Doe"},
                {"name": "Jane Smith"}
            ]);
            let result = extract_author(&value);
            assert_eq!(result, Some("John Doe, Jane Smith".to_string()));
        }

        #[test]
        fn extract_mixed_array_authors() {
            let value = json!([
                "John Doe",
                {"name": "Jane Smith"},
                {"email": "invalid@example.com"}
            ]);
            let result = extract_author(&value);
            assert_eq!(result, Some("John Doe, Jane Smith".to_string()));
        }

        #[test]
        fn extract_empty_array_authors() {
            let value = json!([]);
            let result = extract_author(&value);
            assert_eq!(result, None);
        }

        #[test]
        fn extract_invalid_type() {
            let value = Value::Number(123.into());
            let result = extract_author(&value);
            assert_eq!(result, None);
        }
    }

    mod description {
        use super::*;

        #[test]
        fn extract_simple_description() {
            let json = json!({"description": "A description"});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.description, Some("A description".to_string()));
        }

        #[test]
        fn extract_missing_description() {
            let json = json!({"name": "Cake"});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.description, None);
        }

        #[test]
        fn extract_non_string_description() {
            let json = json!({"description": 456});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.description, None);
        }

        #[test]
        fn extract_empty_description() {
            let json = json!({"description": ""});
            let recipe = extract_recipe(json.as_object().unwrap());
            assert_eq!(recipe.description, Some("".to_string()));
        }
    }

    mod ingredients {
        use super::*;

        #[test]
        fn extract_string_ingredient() {
            let value = Value::String("1 cup flour".to_string());
            let result = extract_ingredients(&value);
            assert_eq!(result, vec!["1 cup flour"]);
        }

        #[test]
        fn extract_array_ingredients() {
            let value = json!(["1 cup flour", "2 eggs", "1 cup milk"]);
            let result = extract_ingredients(&value);
            assert_eq!(result, vec!["1 cup flour", "2 eggs", "1 cup milk"]);
        }

        #[test]
        fn extract_mixed_array_ingredients() {
            let value = json!(["1 cup flour", 123, "2 eggs"]);
            let result = extract_ingredients(&value);
            assert_eq!(result, vec!["1 cup flour", "2 eggs"]);
        }

        #[test]
        fn extract_empty_array_ingredients() {
            let value = json!([]);
            let result = extract_ingredients(&value);
            assert_eq!(result, Vec::<String>::new());
        }

        #[test]
        fn extract_invalid_type_ingredients() {
            let value = Value::Number(123.into());
            let result = extract_ingredients(&value);
            assert_eq!(result, Vec::<String>::new());
        }

        #[test]
        fn extract_object_ingredients() {
            let value = json!({"ingredient": "flour"});
            let result = extract_ingredients(&value);
            assert_eq!(result, Vec::<String>::new());
        }

        #[test]
        fn extract_array_with_objects() {
            let value = json!([
                "1 cup flour",
                {"name": "eggs"},
                "2 tbsp sugar"
            ]);
            let result = extract_ingredients(&value);
            assert_eq!(result, vec!["1 cup flour", "2 tbsp sugar"]);
        }
    }

    mod instructions {
        use super::*;

        mod strings {
            use super::*;

            #[test]
            fn extract_multiline() {
                let instruction = r#"Step 1\nStep 2\nStep 3"#;
                let value = Value::String(instruction.into());
                let result = extract_instructions(&value);
                assert_eq!(result.len(), 1);
                assert_eq!(result[0].name, None);
                assert_eq!(result[0].steps, vec![instruction]);
            }

            #[test]
            fn extract_invalid_type() {
                let result = extract_step(&Value::Number(1.into()));
                assert_eq!(result, Vec::<String>::new());
            }

            #[test]
            fn extract_array() {
                let value = json!(["Step 1", "Step 2"]);
                let result = extract_instructions(&value);
                assert_eq!(result.len(), 1);
                assert_eq!(result[0].name, None);
                assert_eq!(result[0].steps, vec!["Step 1", "Step 2"]);
            }
        }

        mod how_to_steps {
            use super::*;

            #[test]
            fn extract_single() {
                let value = json!({
                    "text": "Do this",
                });
                let result = extract_instructions(&value);
                assert_eq!(result.len(), 1);
                assert_eq!(result[0].name, None);
                assert_eq!(result[0].steps, vec!["Do this"]);
            }

            #[test]
            fn extract_array() {
                let value = json!([{
                    "text": "Do this",
                },{
                    "text": "Do that"
                }]);
                let result = extract_instructions(&value);
                assert_eq!(result.len(), 1);
                assert_eq!(result[0].name, None);
                assert_eq!(result[0].steps, vec!["Do this", "Do that"]);
            }
        }

        mod how_to_sections {
            use super::*;

            #[test]
            fn extract_single() {
                let value = json!({
                    "@type": "HowToSection",
                    "name": "Test Section",
                    "itemListElement": ["Step 1", "Step 2"]
                });
                let result = extract_section(&value);
                assert!(result.is_some());
                assert!(is_how_to_section_obj(&value));
                let section = result.unwrap();
                assert_eq!(section.name, Some("Test Section".to_string()));
                assert_eq!(section.steps, vec!["Step 1", "Step 2"]);
            }

            #[test]
            fn extract_single_invalid() {
                let value = json!({
                    "other": "stuff",
                });
                let result = extract_step(&value);
                assert_eq!(result, Vec::<String>::new());
            }

            #[test]
            fn extract_single_no_name() {
                let value = json!({
                    "@type": "HowToSection",
                    "itemListElement": ["Step 1"]
                });
                let result = extract_section(&value);
                assert!(is_how_to_section_obj(&value));
                assert!(result.is_some());
                let section = result.unwrap();
                assert_eq!(section.name, None);
                assert_eq!(section.steps, vec!["Step 1"]);
            }

            #[test]
            fn extract_array() {
                let value = json!([{
                    "@type": "HowToSection",
                    "name": "Preparation",
                    "itemListElement": ["Preparation Step 1", "Preparation Step 2"]
                },{
                    "@type": "HowToSection",
                    "name": "Cooking",
                    "itemListElement": ["Cooking Step 1", "Cooking Step 2"]
                }]);
                let result = extract_instructions(&value);
                assert_eq!(result.len(), 2);
                assert_eq!(result[0].name, Some("Preparation".to_string()));
                assert_eq!(
                    result[0].steps,
                    vec!["Preparation Step 1", "Preparation Step 2"]
                );
                assert_eq!(result[1].name, Some("Cooking".to_string()));
                assert_eq!(result[1].steps, vec!["Cooking Step 1", "Cooking Step 2"]);
            }

            #[test]
            fn extract_invalid() {
                let result = extract_section(&Value::String("not an object".to_string()));
                assert!(result.is_none());
            }

            #[test]
            fn valid_section() {
                let value = json!({
                    "@type": "HowToSection"
                });
                assert!(is_how_to_section_obj(&value));
            }

            #[test]
            fn invalid_section() {
                let value = json!({
                    "@type": "something"
                });
                assert!(!is_how_to_section_obj(&value));
                let value = json!({
                    "@type": "Recipe"
                });
                assert!(!is_how_to_section_obj(&value));
                assert!(!is_how_to_section_obj(&Value::Object(Map::new())));
                assert!(!is_how_to_section_obj(&Value::String("test".to_string())));
            }
        }
    }
}
