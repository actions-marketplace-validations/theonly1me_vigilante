use super::syntax::NAMESPACE_SEPARATOR;

pub fn constant_path(constant: &str) -> String {
    let parts: Vec<String> = constant
        .split(NAMESPACE_SEPARATOR)
        .filter(|part| !part.is_empty())
        .map(underscore)
        .collect();
    format!("{}.rb", parts.join("/"))
}

pub fn underscore(word: &str) -> String {
    let characters: Vec<char> = word.chars().collect();
    let mut result = String::with_capacity(word.len() + 4);
    for (position, character) in characters.iter().enumerate() {
        if character.is_uppercase() && position > 0 {
            let previous = characters[position - 1];
            let next_is_lower = characters
                .get(position + 1)
                .is_some_and(|next| next.is_lowercase());
            if previous.is_lowercase()
                || previous.is_ascii_digit()
                || (previous.is_uppercase() && next_is_lower)
            {
                result.push('_');
            }
        }
        result.extend(character.to_lowercase());
    }
    result
}

pub fn gem_hints(first: &str) -> Vec<String> {
    let mut hints = vec![
        first.to_string(),
        first.replace('_', "-"),
        first.replace('_', ""),
    ];
    hints.dedup();
    hints
}
