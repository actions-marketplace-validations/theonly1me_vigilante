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

pub fn runs_code(program: &str) -> bool {
    let name = program.rsplit('/').next().unwrap_or(program);
    CODE_RUNNERS.contains(&name)
}

pub fn is_package_manager(program: &str) -> bool {
    PACKAGE_MANAGERS.contains(&program)
}
