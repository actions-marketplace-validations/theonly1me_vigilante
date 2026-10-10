use crate::surfaces::command::{is_package_manager, runs_code};

const FILE_TOOLS: &[&str] = &[
    "Edit",
    "Glob",
    "Grep",
    "LS",
    "MultiEdit",
    "NotebookEdit",
    "NotebookRead",
    "Read",
    "Task",
    "TodoWrite",
    "Write",
];
const OPEN_CONTENT_HOSTS: &[&str] = &[
    "gist.github.com",
    "gist.githubusercontent.com",
    "github.com",
    "raw.githubusercontent.com",
];
const RISKY_COMMANDS: &[(&str, &str)] = &[
    ("bundle", "exec"),
    ("cargo", "install"),
    ("gem", "install"),
    ("git", "push"),
    ("go", "run"),
    ("npm", "exec"),
    ("npm", "i"),
    ("npm", "install"),
    ("pip", "install"),
    ("pip3", "install"),
    ("pnpm", "add"),
    ("pnpm", "dlx"),
    ("pnpm", "exec"),
    ("pnpm", "install"),
    ("rails", "new"),
    ("rails", "runner"),
    ("yarn", "add"),
    ("yarn", "dlx"),
];

pub fn grants_power(rule: &str) -> bool {
    let (tool, argument) = match rule.split_once('(') {
        Some((tool, rest)) => (tool, Some(rest.strip_suffix(')').unwrap_or(rest))),
        None => (rule, None),
    };
    match (tool, argument) {
        ("Bash", None) | ("WebSearch", _) | ("WebFetch", None) => true,
        ("Bash", Some(command)) => command_grants_power(command),
        ("WebFetch", Some(argument)) => argument
            .strip_prefix("domain:")
            .is_none_or(|host| host.contains('*') || OPEN_CONTENT_HOSTS.contains(&host)),
        (tool, _) => !FILE_TOOLS.contains(&tool),
    }
}

struct Segment<'a> {
    words: Vec<&'a str>,
    is_piped: bool,
}

fn command_grants_power(command: &str) -> bool {
    let (prefix, has_wildcard) = match command.find('*') {
        Some(index) => (command[..index].trim_end_matches([':', ' ']), true),
        None => (command, false),
    };
    let segments = segments_of(prefix);
    let last = segments.len().saturating_sub(1);
    segments.iter().enumerate().any(|(index, segment)| {
        let is_open = has_wildcard && index == last;
        match segment.words.as_slice() {
            [] => is_open,
            [program] => (is_open || segment.is_piped) && runs_code(program),
            [program, action, rest @ ..] => {
                RISKY_COMMANDS.contains(&(program, action))
                    || (segment.is_piped && runs_code(program))
                    || (is_open
                        && rest.is_empty()
                        && is_package_manager(program)
                        && *action == "run")
            }
        }
    })
}

fn segments_of(command: &str) -> Vec<Segment<'_>> {
    command
        .split("&&")
        .flat_map(|part| part.split("||"))
        .flat_map(|part| part.split(';'))
        .flat_map(|part| {
            part.split('|').enumerate().map(|(index, piece)| Segment {
                words: piece.split_whitespace().collect(),
                is_piped: index > 0,
            })
        })
        .collect()
}
