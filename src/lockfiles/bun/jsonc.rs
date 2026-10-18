pub fn plain_json(text: &str) -> String {
    without_trailing_commas(&without_comments(text))
}

fn without_comments(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    let mut in_string = false;
    while let Some(character) = characters.next() {
        if in_string {
            output.push(character);
            match character {
                '\\' => output.extend(characters.next()),
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match (character, characters.peek()) {
            ('/', Some('/')) => {
                if characters.by_ref().any(|skipped| skipped == '\n') {
                    output.push('\n');
                }
            }
            ('/', Some('*')) => {
                characters.next();
                let mut previous = ' ';
                for skipped in characters.by_ref() {
                    if skipped == '\n' {
                        output.push('\n');
                    }
                    if previous == '*' && skipped == '/' {
                        break;
                    }
                    previous = skipped;
                }
                output.push(' ');
            }
            _ => {
                in_string = character == '"';
                output.push(character);
            }
        }
    }
    output
}

fn without_trailing_commas(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut in_string = false;
    let mut is_escaped = false;
    for (index, character) in text.char_indices() {
        if in_string {
            match character {
                _ if is_escaped => is_escaped = false,
                '\\' => is_escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else if character == ',' && text[index + 1..].trim_start().starts_with(['}', ']']) {
            continue;
        } else {
            in_string = character == '"';
        }
        output.push(character);
    }
    output
}
