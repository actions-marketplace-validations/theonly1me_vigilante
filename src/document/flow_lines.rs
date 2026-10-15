const TOKEN_STARTS: [char; 6] = [':', '-', '?', ',', '[', '{'];
const BLOCK_SCALAR_HEADER: [char; 12] =
    ['+', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Default)]
struct Scanner {
    flow_depth: usize,
    flow_indent: usize,
    quote: Option<char>,
    block_scalar_parent: Option<usize>,
}

pub fn indent_flow_lines(text: &str) -> Option<String> {
    let mut scanner = Scanner::default();
    let mut changed = false;
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| {
            let fixed = scanner.line(line);
            changed |= fixed != line;
            fixed
        })
        .collect();
    changed.then(|| lines.join("\n"))
}

impl Scanner {
    fn line(&mut self, line: &str) -> String {
        let content = line.trim_start_matches(' ');
        let indent = line.len() - content.len();
        let is_blank = content.trim().is_empty();
        if let Some(parent) = self.block_scalar_parent {
            if is_blank || indent > parent {
                return line.to_string();
            }
            self.block_scalar_parent = None;
        }
        let output = if self.flow_depth > 0 && !is_blank && indent <= self.flow_indent {
            format!("{}{content}", " ".repeat(self.flow_indent + 1))
        } else {
            line.to_string()
        };
        self.scan(ScannedLine { content, indent });
        output
    }

    fn scan(&mut self, line: ScannedLine) {
        let mut previous: Option<char> = None;
        let mut characters = line.content.chars().peekable();
        while let Some(character) = characters.next() {
            if let Some(quote) = self.quote {
                let is_escape = match quote {
                    '"' => character == '\\',
                    _ => character == quote && characters.peek() == Some(&quote),
                };
                if is_escape {
                    characters.next();
                } else if character == quote {
                    self.quote = None;
                    previous = Some(character);
                }
                continue;
            }
            let at_token_start = previous.is_none_or(|before| TOKEN_STARTS.contains(&before));
            match character {
                '#' if previous.is_none() => return,
                ' ' | '\t' if characters.peek() == Some(&'#') => return,
                ' ' | '\t' => continue,
                '"' | '\'' if at_token_start => self.quote = Some(character),
                '[' | '{' if at_token_start || self.flow_depth > 0 => {
                    if self.flow_depth == 0 {
                        self.flow_indent = line.indent;
                    }
                    self.flow_depth += 1;
                }
                ']' | '}' if self.flow_depth > 0 => self.flow_depth -= 1,
                '|' | '>' if self.flow_depth == 0 && at_token_start => {
                    let rest: String = characters.clone().collect();
                    let header = rest.trim_start_matches(BLOCK_SCALAR_HEADER);
                    if header.trim().is_empty() || header.trim_start().starts_with('#') {
                        self.block_scalar_parent = Some(line.indent);
                        return;
                    }
                }
                _ => {}
            }
            previous = Some(character);
        }
    }
}

struct ScannedLine<'a> {
    content: &'a str,
    indent: usize,
}
