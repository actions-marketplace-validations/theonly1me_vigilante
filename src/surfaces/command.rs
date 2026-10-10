use std::collections::BTreeMap;

const CODE_RUNNERS: &[&str] = &[
    "bash",
    "bun",
    "bunx",
    "bundle",
    "cargo",
    "curl",
    "deno",
    "docker",
    "dotnet",
    "env",
    "eval",
    "exec",
    "find",
    "gem",
    "gh",
    "git",
    "go",
    "java",
    "kubectl",
    "make",
    "node",
    "node-gyp",
    "npm",
    "npx",
    "perl",
    "php",
    "pip",
    "pip3",
    "pnpm",
    "prebuild-install",
    "python",
    "python3",
    "rails",
    "rake",
    "ruby",
    "scp",
    "sh",
    "ssh",
    "sudo",
    "ts-node",
    "tsx",
    "uv",
    "uvx",
    "wget",
    "xargs",
    "yarn",
    "zsh",
];
const PACKAGE_MANAGERS: &[&str] = &["npm", "pnpm", "yarn"];
const SHELL_OPERATORS: &[char] = &['|', ';', '&', '$', '`', '<', '>', '(', ')', '\n'];
const MAXIMUM_SCRIPT_DEPTH: usize = 4;

pub fn runs_code(program: &str) -> bool {
    let name = program.rsplit('/').next().unwrap_or(program);
    CODE_RUNNERS.contains(&name)
}

pub fn is_package_manager(program: &str) -> bool {
    PACKAGE_MANAGERS.contains(&program)
}

pub type PackageScripts<'a> = BTreeMap<&'a str, &'a str>;

pub struct LocalCommand<'a> {
    pub command: &'a str,
    pub package_scripts: Option<&'a PackageScripts<'a>>,
}

pub fn is_plain(local: LocalCommand) -> bool {
    let mut command = local.command;
    for _ in 0..=MAXIMUM_SCRIPT_DEPTH {
        if command.contains(SHELL_OPERATORS) {
            return false;
        }
        let words: Vec<&str> = command.split_whitespace().collect();
        match words.as_slice() {
            [manager, "run" | "run-script", script, ..] if is_package_manager(manager) => {
                let Some(scripts) = local.package_scripts else {
                    return true;
                };
                let Some(next) = scripts.get(script) else {
                    return false;
                };
                command = next;
            }
            [manager, "audit", ..] if is_package_manager(manager) => return true,
            [program, ..] => return !program.contains('/') && !runs_code(program),
            [] => return true,
        }
    }
    false
}
