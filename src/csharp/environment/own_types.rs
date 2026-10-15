use std::collections::HashMap;

use crate::Error;
use crate::tree::Tree;

const TYPE_KEYWORDS: &[&str] = &["class", "struct", "interface", "record", "enum"];
const SOURCE_SUFFIX: &str = ".cs";

pub fn own_types(tree: &dyn Tree) -> Result<HashMap<String, String>, Error> {
    let paths: Vec<&str> = tree
        .paths()
        .iter()
        .map(String::as_str)
        .filter(|path| path.ends_with(SOURCE_SUFFIX))
        .collect();
    let mut types = HashMap::new();
    for file in tree.read(&paths)? {
        let words: Vec<&str> = file
            .text
            .split(|character: char| {
                !(character.is_alphanumeric() || character == '_' || character == '.')
            })
            .filter(|word| !word.is_empty())
            .collect();
        let mut namespace = "";
        for pair in words.windows(2) {
            match pair[0] {
                "namespace" => namespace = pair[1],
                keyword if TYPE_KEYWORDS.contains(&keyword) => {
                    let name = pair[1];
                    let key = if namespace.is_empty() {
                        name.to_string()
                    } else {
                        format!("{namespace}.{name}")
                    };
                    types.entry(key).or_insert_with(|| file.path.clone());
                }
                _ => {}
            }
        }
    }
    Ok(types)
}
