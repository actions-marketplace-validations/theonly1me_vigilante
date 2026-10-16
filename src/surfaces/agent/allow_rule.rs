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
const COMMAND_WRAPPERS: &[&str] = &["command", "env", "nice", "nohup", "sudo", "time"];
const CODE_FLAGS: &[(&[&str], &[&str])] = &[
    (&["bash", "dash", "python", "python3", "sh", "zsh"], &["-c"]),
    (&["bun", "node"], &["-e", "--eval", "-p", "--print"]),
    (&["deno"], &["eval"]),
    (&["perl", "ruby"], &["-e"]),
    (&["php"], &["-r"]),
    (
        &["curl"],
        &[
            "-X",
            "--request",
            "-d",
            "--data",
            "-F",
            "--form",
            "-T",
            "--upload-file",
        ],
    ),
    (&["wget"], &["--post-data", "--post-file", "--method"]),
];
const PREFIX_FLAGS: &[&str] = &["--data"];

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
        match without_wrappers(&segment.words) {
            [] => is_open,
            [program] => (is_open || segment.is_piped) && runs_code(program),
            [program, rest @ ..] => {
                rest.iter()
                    .any(|word| RISKY_COMMANDS.contains(&(program, word)))
                    || (segment.is_piped && runs_code(program))
                    || (is_open && is_package_manager(program) && rest == ["run"])
                    || (is_open
                        && rest
                            .iter()
                            .any(|word| is_code_flag(CodeFlag { program, word })))
            }
        }
    })
}

fn without_wrappers<'words, 'a>(words: &'words [&'a str]) -> &'words [&'a str] {
    let start = words
        .iter()
        .position(|word| !COMMAND_WRAPPERS.contains(word) && !word.contains('='))
        .unwrap_or(words.len());
    &words[start..]
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

struct CodeFlag<'a> {
    program: &'a str,
    word: &'a str,
}

fn is_code_flag(flag: CodeFlag) -> bool {
    let program = flag.program.rsplit('/').next().unwrap_or(flag.program);
    let flag_name = flag
        .word
        .split_once('=')
        .map_or(flag.word, |(name, _)| name);
    CODE_FLAGS.iter().any(|(programs, flags)| {
        programs.contains(&program)
            && (flags.contains(&flag_name)
                || PREFIX_FLAGS
                    .iter()
                    .any(|prefix| flags.contains(prefix) && flag_name.starts_with(prefix)))
    })
}
