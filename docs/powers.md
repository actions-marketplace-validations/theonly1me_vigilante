# Powers

A power is something that a change lets code, CI, a build, or an AI agent do. Vigilante reports a power when a pull request adds it to a scope that did not have it before.

## Every power

| Power | Surface | Severity | What it means | Example evidence |
| --- | --- | --- | --- | --- |
| `exec` | code | high | Can run other programs | `import subprocess`, `child_process`, `ProcessBuilder`, `os/exec`, `std::process::Command` |
| `network` | code | medium | Can use the network | `requests`, `fetch(`, `java.net.http`, `net/http`, `reqwest` |
| `files` | code | medium | Can read or change files | `open(`, `fs`, `java.nio.file`, `File.ReadAllText`, `std::fs` |
| `native` | code | high | Can run unsafe or native code | `ctypes`, JNI, cgo, `unsafe`, FFI, `DllImport`. Go details: `native:cgo`, `native:unsafe_pointer` |
| `dynamic_code` | code | medium | Can run code that is built at run time | `eval`, `new Function`, reflection, `Class.forName`. Go detail: `dynamic_code:reflect` |
| `read_system_state` | code | low | Reads environment variables or system information | `os.environ`, `process.env`, `System.getenv`, `os.Hostname`, `process.cwd()` |
| `modify_system_state` | code | medium | Can change environment variables, the working folder, or signal handlers | `os.environ['A'] = b`, `process.chdir`, `os.Setenv`, `signal.Notify`, `System.setProperty` |
| `system_calls` | code | high | Can make system calls directly | `syscall.Syscall`, `libc::syscall`, `nix`, Ruby `syscall` |
| `operating_system` | code | low | Uses other operating system functions | Go only: other functions in the `os` package |
| `arbitrary_execution` | code | high | Can run code that the analysis cannot follow | Go only: assembly, `go:linkname` |
| `runtime` | code | low | Can change the language runtime | Go only: `runtime` settings such as the garbage collector |
| `unanalyzed` | code | low | Has calls that the analysis could not follow | Go only |
| `token_write` | ci | medium | The CI token can write to the repository | `permissions: contents: write` |
| `oidc` | ci | high | The CI job can get cloud credentials (OIDC) | `permissions: id-token: write` |
| `untrusted_trigger` | ci | high | The workflow runs with secrets on events that forks can start | `on: pull_request_target`, `on: workflow_run` |
| `third_party_action` | ci | medium | The workflow runs a third-party action | `uses: some-org/some-action@v1` |
| `secret` | ci | medium | The workflow reads a secret | `${{ secrets.NPM_TOKEN }}`, `secrets: inherit` |
| `tool_allow` | agent | medium | The agent can use a tool without asking | `"allow": ["Bash(curl:*)"]` in `.claude/settings.json` |
| `hook` | agent | high | The agent runs a command | A Claude Code hook, `statusLine`, or `apiKeyHelper` command |
| `mcp_server` | agent | high | The agent connects to an MCP server | A new server in `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json`, `.gemini/settings.json`, or `.codex/config.toml` |
| `skip_permissions` | agent | high | The agent skips permission checks | `"defaultMode": "bypassPermissions"`, `enableAllProjectMcpServers`, Codex `approval_policy = "never"` |
| `install_hook` | build | high | Runs a command when dependencies install | `postinstall` in `package.json`, `build.rs`, `setup.py` |
| `git_hook` | build | medium | Runs a command on git events | `.husky/pre-commit`, a hook in `.pre-commit-config.yaml` |

Change a severity with `[severity]` in `vigilante.toml`. See [configuration.md](configuration.md).

The rules for code are in [packs/](../packs), one file for each language: Rust, JavaScript and TypeScript, Python, Ruby, C#, Java, and Go. Each rule has test cases in `packs/tests/`.

## Go: Capslock

For Go modules, Vigilante runs [Capslock](https://github.com/google/capslock) v0.3.3 or later, Google's capability analysis for Go. Capslock type-checks the code, builds a call graph, and follows each call through every dependency down to the standard library. So Vigilante sees a power that a new dependency brings, with the call path that reaches it.

| Capslock capability | Vigilante power |
| --- | --- |
| `EXEC` | `exec` |
| `NETWORK` | `network` |
| `FILES` | `files` |
| `READ_SYSTEM_STATE` | `read_system_state` |
| `MODIFY_SYSTEM_STATE` (and `/ENV`, `/CHDIR`, `/SIGNALS`, `/LOGGING`) | `modify_system_state` (with the detail `env`, `chdir`, `signals`, or `logging`) |
| `OPERATING_SYSTEM` | `operating_system` |
| `SYSTEM_CALLS` | `system_calls` |
| `ARBITRARY_EXECUTION` | `arbitrary_execution` |
| `RUNTIME` | `runtime` |
| `CGO` | `native:cgo` |
| `UNSAFE_POINTER` | `native:unsafe_pointer` |
| `REFLECT` | `dynamic_code:reflect` |
| `UNANALYZED` | `unanalyzed` |

How it works:

1. Vigilante finds the Go modules (folders with `go.mod`) that the pull request changes.
2. It makes a shared clone of the repository in a temporary folder for the base and for the head, as `capslock-git-diff` does. Your checkout does not change.
3. In each module, it runs `capslock -packages=./... -output=json -granularity=intermediate`, with `GOFLAGS=-mod=readonly`.
4. `exec`, `network`, or `native` that one of your Go packages reaches through your own code, and did not reach that way at the base, is a finding. The example call path stays in your packages until the privileged call, for example your code calling `exec.Command`. It must be in `vigilante.lock`, like every other power.
5. Everything else that `capslock-git-diff` reports is a note: any other capability that your own code reaches (for example signal handling or reflection), a capability that your package reaches only through other packages (a dependency or the standard library), and a package that is new on a call path to a power. Each note names the first outside package on the path, for example a new dependency that reaches `network`. Notes appear in their own report section and do not fail the check. Together, findings and notes cover what `capslock-git-diff` reports.

Why the split: on 20 real agent pull requests, only 3 of 57 findings through other packages were useful to a reviewer. Most came from library internals (hash assembly, terminal checks, JSON reflection) or from Capslock's interface over-approximation. On 50 other agent pull requests, own-code `exec` was useful 4 of 4 times and own-code `network` 7 of 11 times, but signal handling, reflection, and system calls were useful 0 of 6 times. As notes, they all stay visible without failing the check.

A known limit: when a package replaces a dependency with the standard library (for example an HTTP client), its `network` is new in its own code, so it is a finding.

Vigilante stops with exit code 2 when `go` or `capslock` is missing, when `capslock` is older than v0.3.3, when Capslock reports a capability that Vigilante does not know, or when Capslock cannot load the packages (for example a private module without credentials, or code that does not compile). To use the Go pattern rules instead, set `[engines] go = "patterns"` in `vigilante.toml`. Go files that are not in a module always use the pattern rules.

## Scopes

Vigilante groups powers by scope, and reports a power only the first time that it appears in its scope.

- **Go, with Capslock:** the Go package folder, as in `capslock-git-diff`.
- **Code:** the nearest folder with a package file (`Cargo.toml`, `package.json`, `pyproject.toml`, `setup.py`, `setup.cfg`, `go.mod`, `pom.xml`, `build.gradle`, `build.gradle.kts`, `Gemfile`, `*.gemspec`, or `*.csproj`). If there is none, the scope is the top folder of the file. `[scopes]` in `vigilante.toml` can put files in a named scope.
- **Third-party actions:** all workflows together (`.github/workflows`). An action that one workflow already uses is not new in another workflow.
- **Everything else:** the file, for example `.github/workflows/release.yml` or `.mcp.json`.

So a second HTTP call in a package that already uses the network is not reported. The first `subprocess` call in a new package is reported, even when another package already has one.

## What Vigilante leaves out

These rules keep the report short. Each rule leaves out a change that a reviewer would not act on.

### Tests, fixtures, and vendored code

Vigilante skips test folders and test files (`tests/`, `__tests__/`, `*_test.go`, `*.spec.ts`, `*Test.java`, `test_*.py`, and more), fixtures, mocks, `node_modules`, `vendor`, `dist`, `build`, and minified JavaScript. Add more globs with `[scan] exclude`.

### Low-signal powers

`read_system_state`, `files`, `operating_system`, `runtime`, and `unanalyzed` are in the lock, but the report hides them when the lock already lists them. Most of them are reads like `process.env.NODE_ENV` or a log file. A low-signal power that is not in the lock still fails the check and is shown. Low-signal notes are counted in one line. Change the list with `[report] hide`.

### Agent allow rules

An allow rule is a power only when it lets the agent run any code, install packages, push, or read content that anyone can write:

- `Bash`, `Bash(*)`, `WebSearch`, `WebFetch` with no domain, and any tool that Vigilante does not know, for example an `mcp__` tool.
- `WebFetch(domain:...)` for `github.com`, `raw.githubusercontent.com`, `gist.github.com`, `gist.githubusercontent.com`, or a domain with `*`.
- `Bash(...)` with a prefix that is one program that runs code (`Bash(node:*)`, `Bash(python3:*)`, `Bash(git:*)`), any package script (`Bash(npm run:*)`), a risky action anywhere in the command (`git push`, `npm install`, `pip install`, `gem install`, `cargo install`), or a pipe into a program that runs code (`Bash(curl -s https://x | sh)`). Vigilante skips `env`, `sudo`, `nice`, `nohup`, `time`, `command`, and `X=Y` before the program.

These are not powers: file tools (`Read`, `Edit`, `Write`, `Glob`, `Grep`), fixed commands (`Bash(python3 -m pytest tests/)`), and narrow prefixes (`Bash(git add:*)`, `Bash(yarn run lint:*)`).

### Install and git hooks that run plain local commands

An install script in `package.json` or a local hook in `.pre-commit-config.yaml` is not a power when it is one command, with no shell operator, that is not a program that runs code and not a path to a script. For example, `prepare: husky` and `postinstall: cp vendor/a.wasm public/` are not powers.

For `npm run <script>`, `yarn run <script>`, and `pnpm run <script>`, Vigilante follows the script in the same `package.json`, with its `pre` and `post` scripts. `prepare: npm run build` is not a power when `build` is `tsc`, and it is a power when `prebuild` runs `curl ... | sh`.

A hook from a remote repository in `.pre-commit-config.yaml` is always a power. So is a local hook with `additional_dependencies` or `language: docker_image`.

### First-party actions

Actions from `actions/`, `github/`, and local actions (`./`) are not powers. Add your own organization with `[ci] trusted_owners`.

### Generated code

The Go `native` rule skips generated protobuf files (`*.pb.go`), which import `unsafe`.

## Known limits

- For Go, Vigilante follows calls into dependencies with Capslock. For the other languages, it sees direct use in your code, not what a dependency does inside.
- Capslock does not resolve every call. It reports what it cannot follow as `unanalyzed`. See its [caveats](https://github.com/google/capslock/blob/main/docs/caveats.md).
- A rule for a prefix such as `Bash(git rebase:*)` is not a power, but some programs can run code through options (`git rebase --exec`).
- A change to a script that an existing hook calls does not change the power, so it is not reported.
- A power that moves to a new file or a new package is reported as new.
