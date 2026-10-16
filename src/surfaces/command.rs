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
const PLAIN_PROGRAMS: &[&str] = &[
    "cat",
    "chmod",
    "cp",
    "echo",
    "exit",
    "husky",
    "is-ci",
    "lefthook",
    "ln",
    "mkdir",
    "mv",
    "printf",
    "rimraf",
    "rm",
    "simple-git-hooks",
    "touch",
    "true",
    "tsc",
];
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
    pub unknown_programs_run_code: bool,
}

pub fn is_plain(local: LocalCommand) -> bool {
    let mut pending = vec![(local.command, 0)];
    while let Some((command, depth)) = pending.pop() {
        if depth > MAXIMUM_SCRIPT_DEPTH || command.contains(SHELL_OPERATORS) {
            return false;
        }
        let words: Vec<&str> = command.split_whitespace().collect();
        match words.as_slice() {
            [manager, "run" | "run-script", script, ..] if is_package_manager(manager) => {
                let Some(scripts) = local.package_scripts else {
                    continue;
                };
                let Some(body) = scripts.get(script) else {
                    return false;
                };
                let hooks = [format!("pre{script}"), format!("post{script}")];
                let hook_bodies = hooks.iter().filter_map(|hook| scripts.get(hook.as_str()));
                for next in std::iter::once(body).chain(hook_bodies) {
                    pending.push((*next, depth + 1));
                }
            }
            [manager, "audit", ..] if is_package_manager(manager) => {}
            [program, ..] if program.contains('/') || runs_code(program) => return false,
            [program, ..]
                if local.unknown_programs_run_code && !PLAIN_PROGRAMS.contains(program) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}
