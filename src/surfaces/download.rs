const DOWNLOADERS: &[&str] = &["curl", "wget"];
const INTERPRETERS: &[&str] = &[
    "bash", "dash", "node", "perl", "pwsh", "python", "python3", "ruby", "sh", "zsh",
];
const SUBSTITUTIONS: &[&str] = &["<(", "$("];
const WRAPPERS: &[&str] = &["env", "sudo"];
const SOURCE_COMMANDS: &[&str] = &[".", "eval", "source"];
const UNKNOWN_HOST: &str = "unknown";

pub fn downloaded_script_host(line: &str) -> Option<String> {
    let commands = line.replace("&&", ";").replace("||", ";");
    let piped = commands.split(';').find_map(|command| {
        let stages: Vec<&str> = command.split('|').collect();
        stages.windows(2).find_map(|pair| {
            let [download, run] = pair else {
                return None;
            };
            let downloads =
                program_of(download).is_some_and(|program| DOWNLOADERS.contains(&program));
            (downloads && runs_standard_input(run)).then_some(*download)
        })
    });
    piped.or_else(|| substituted_download(line)).map(host_of)
}

fn substituted_download(line: &str) -> Option<&str> {
    SUBSTITUTIONS.iter().find_map(|opening| {
        DOWNLOADERS.iter().find_map(|downloader| {
            let start = line.find(&format!("{opening}{downloader}"))?;
            let before: Vec<&str> = line[..start]
                .split_whitespace()
                .map(|word| word.trim_matches(['"', '\'']))
                .filter(|word| !word.is_empty())
                .collect();
            let runs_it = match before.as_slice() {
                [.., program, "-c"] => INTERPRETERS.contains(&base_name(program)),
                [.., program] => {
                    SOURCE_COMMANDS.contains(program) || INTERPRETERS.contains(&base_name(program))
                }
                [] => false,
            };
            runs_it.then(|| &line[start..])
        })
    })
}

fn runs_standard_input(stage: &str) -> bool {
    let mut words = stage
        .split_whitespace()
        .skip_while(|word| WRAPPERS.contains(word) || word.contains('=') || word.starts_with('-'));
    let Some(program) = words.next() else {
        return false;
    };
    if !INTERPRETERS.contains(&base_name(program)) {
        return false;
    }
    for word in words {
        match word {
            "-" | "--" => return true,
            _ if word.starts_with('-') => {}
            _ => return false,
        }
    }
    true
}

fn program_of(stage: &str) -> Option<&str> {
    stage
        .split_whitespace()
        .find(|word| !WRAPPERS.contains(word) && !word.contains('=') && !word.starts_with('-'))
        .map(base_name)
}

fn base_name(program: &str) -> &str {
    program.rsplit('/').next().unwrap_or(program)
}

fn host_of(text: &str) -> String {
    let Some((_, address)) = text.split_once("://") else {
        return UNKNOWN_HOST.to_string();
    };
    let end = address
        .find(|character: char| "/:?\"')".contains(character) || character.is_whitespace())
        .unwrap_or(address.len());
    match &address[..end] {
        "" => UNKNOWN_HOST.to_string(),
        host => host.to_string(),
    }
}
