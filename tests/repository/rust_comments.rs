pub fn comment_lines(text: &str) -> Vec<usize> {
    let characters: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut line = 1;
    let mut index = 0;
    while index < characters.len() {
        let rest = &characters[index..];
        let next = index
            + match rest {
                ['/', '/', ..] => {
                    lines.push(line);
                    rest.iter()
                        .take_while(|&&character| character != '\n')
                        .count()
                }
                ['/', '*', ..] => {
                    lines.push(line);
                    block_length(rest)
                }
                ['r', '"' | '#', ..] if starts_raw_string(&characters, index) => {
                    raw_string_length(rest)
                }
                ['"', ..] => string_length(rest),
                ['\'', '\\', ..] => {
                    2 + rest[2..]
                        .iter()
                        .take_while(|&&character| character != '\'')
                        .count()
                        + 1
                }
                ['\'', _, '\'', ..] => 3,
                _ => 1,
            };
        let end = next.min(characters.len());
        line += characters[index..end]
            .iter()
            .filter(|&&character| character == '\n')
            .count();
        index = end;
    }
    lines
}

fn starts_raw_string(characters: &[char], index: usize) -> bool {
    let after_identifier = index > 0 && {
        let previous = characters[index - 1];
        previous.is_alphanumeric() || previous == '_'
    };
    let hashes = characters[index + 1..]
        .iter()
        .take_while(|&&character| character == '#')
        .count();
    !after_identifier && characters.get(index + 1 + hashes) == Some(&'"')
}

fn raw_string_length(rest: &[char]) -> usize {
    let hashes = rest[1..]
        .iter()
        .take_while(|&&character| character == '#')
        .count();
    let body = 2 + hashes;
    let mut position = body;
    while position < rest.len() {
        let closes = rest[position] == '"'
            && rest[position + 1..]
                .iter()
                .take(hashes)
                .filter(|&&character| character == '#')
                .count()
                == hashes;
        if closes {
            return position + 1 + hashes;
        }
        position += 1;
    }
    rest.len()
}

fn string_length(rest: &[char]) -> usize {
    let mut position = 1;
    while position < rest.len() {
        match rest[position] {
            '\\' => position += 2,
            '"' => return position + 1,
            _ => position += 1,
        }
    }
    rest.len()
}

fn block_length(rest: &[char]) -> usize {
    rest.windows(2)
        .position(|pair| pair == ['*', '/'])
        .map_or(rest.len(), |position| position + 2)
}
