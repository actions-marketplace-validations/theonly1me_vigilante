# Powers

A power is an action that a change lets code, CI, a build, or an AI agent do. Vigilante reports a power when a pull request adds it to a scope that did not have it before. It also reports a [risk](risks.md) when a call gets more dangerous input, also in a scope that has the power.

## Every power

| Power | Surface | Severity | What it means | Example evidence |
| --- | --- | --- | --- | --- |
| `exec` | code | high | Can run other programs | `import subprocess`, `child_process`, `ProcessBuilder`, `os/exec`, `std::process::Command` |
| `network` | code | medium | Can use the network | `requests`, `fetch(`, `java.net.http`, `net/http`, `reqwest` |
| `files` | code | medium | Can read or change files | `open(`, `fs`, `java.nio.file`, `File.ReadAllText`, `std::fs` |
| `native` | code | high | Can run unsafe or native code | `ctypes`, JNI, cgo, `unsafe`, FFI, `DllImport`. Go details: `native:cgo`, `native:unsafe_pointer` |
| `dynamic_code` | code | medium | Can run code that the program makes at run time | `eval`, `new Function`, reflection, `Class.forName`. Go detail: `dynamic_code:reflect` |
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
| `secret` | ci | medium | The workflow reads a secret | `${{ secrets.NPM_TOKEN }}`, `${{ secrets['NPM_TOKEN'] }}`, `secrets: inherit`. Detail `all` for `toJSON(secrets)` or a computed name |
| `remote_script` | ci | high | CI runs a script that it downloads | `curl https://example.com/install.sh \| sh`, `bash <(curl ...)`. The detail is the host |
| `tool_allow` | agent | medium | The agent can use a tool without asking | `"allow": ["Bash(curl:*)"]` in `.claude/settings.json`, `allowed-tools: Bash` in a skill or a command |
| `hook` | agent | high | The agent runs a command | A Claude Code hook in settings or in a skill, `statusLine`, or `apiKeyHelper` command |
| `mcp_server` | agent | high | The agent connects to an MCP server | A new server in `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json`, `.gemini/settings.json`, or `.codex/config.toml` |
| `skip_permissions` | agent | high | The agent skips permission checks | `"defaultMode": "bypassPermissions"`, `enableAllProjectMcpServers`, Codex `approval_policy = "never"` |
| `install_hook` | build | high | Runs a command when dependencies install | `postinstall` in `package.json`, `build.rs`, `setup.py` |
| `git_hook` | build | medium | Runs a command on git events | `.husky/pre-commit`, a hook in `.pre-commit-config.yaml` |

To change a severity, use `[severity]` in `vigilante.toml`. See [configuration.md](configuration.md).

The rules for code are in [packs/](../packs). Each language has one file: Rust, JavaScript and TypeScript, Python, Ruby, C#, Java, and Go. Each rule has test cases in `packs/tests/`.

## Calls through packages

For JavaScript, TypeScript, Python, Java, Rust, C#, and Ruby, Vigilante makes its own call graph. It follows calls from your code into the locked packages, and each result has a call path. The rule for findings and notes is the same as for Go:

- `exec`, `network`, and `native` that your own code reaches are findings.
- All other results are notes.

See [dependencies.md](dependencies.md).

## Risks

A risk is one use of a power, with the class of its input. Vigilante reports each changed risk, also in a scope that already has the power. See [risks.md](risks.md).

## Go: Capslock

For Go modules, Vigilante runs [Capslock](https://github.com/google/capslock) v0.3.3 or later. Capslock is the capability analysis of Google for Go. It type-checks the code, makes a call graph, and follows each call through each dependency to the standard library. So Vigilante sees a power that a new dependency brings, with the call path to that power.

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

1. Vigilante finds the Go modules (folders with `go.mod`) that the pull request changes. It skips a module that has no `.go` file, for example a Hugo site module. A `.go` file in a nested module belongs to the nested module.
2. It makes a shared clone of the repository in a temporary folder for the base and for the head, as `capslock-git-diff` does. Your checkout does not change.
3. In each module, it runs `capslock -packages=./... -output=json -granularity=intermediate`, with `GOFLAGS=-mod=readonly`.
4. A finding is `exec`, `network`, or `native` that a Go package reaches through its own code. At the base, the package did not reach that power in this way. The example call path stays in your packages until the privileged call. An example is your code that calls `exec.Command`. A finding must be in `vigilante.lock`, as each other power must be.
5. All other results of `capslock-git-diff` are notes. A note can be one of these:
   - another capability that your own code reaches, for example signal handling or reflection;
   - a capability that your package reaches only through other packages, such as a dependency or the standard library;
   - a package that is new on a call path to a power.
6. Each note names the first outside package on the path, for example a new dependency that reaches `network`. Notes have their own section in the report. They make the check fail only when a `deny` rule matches. The findings and the notes together cover all the results of `capslock-git-diff`.

Why notes: a finding through other packages almost always comes from library internals, such as hash assembly, terminal checks, and JSON reflection. Signal handling, reflection, and system calls in your own code seldom need a review. As notes, these results stay visible, and they do not make the check fail.

A known limit: a package can replace a dependency with the standard library, for example an HTTP client. Then its `network` power is new in its own code, so it is a finding.

Vigilante stops with exit code 2 in these conditions:

- `go` or `capslock` is missing.
- `capslock` is older than v0.3.3.
- Capslock reports a capability that Vigilante does not know.
- Capslock cannot load the packages, for example a private module without credentials, or code that does not compile.

To use the Go pattern rules, set `[engines] go = "patterns"` in `vigilante.toml`. Go files that are not in a module always use the pattern rules.

## Scopes

Vigilante puts powers into scopes. It reports a power only the first time that the power appears in its scope.

- **Go, with Capslock:** the Go package folder, as in `capslock-git-diff`.
- **Code:** the nearest folder with a package file (`Cargo.toml`, `package.json`, `pyproject.toml`, `setup.py`, `setup.cfg`, `go.mod`, `pom.xml`, `build.gradle`, `build.gradle.kts`, `Gemfile`, `*.gemspec`, or `*.csproj`). If there is no such folder, the scope is the top folder of the file. `[scopes]` in `vigilante.toml` can put files into a scope with a name.
- **Third-party actions and downloaded scripts:** all workflows and composite actions together (`.github/workflows`). An action that one workflow uses is not new in another workflow.
- **All other files:** the file, for example `.github/workflows/release.yml` or `.mcp.json`.

So Vigilante does not report a second HTTP call in a package that already uses the network. It reports the first `subprocess` call in a new package, also when another package already has one.

## What Vigilante leaves out

These rules keep the report short. Each rule leaves out a change that a reviewer would not act on.

### Test code and vendored code

Tests, fixtures, and mocks run in CI and in agent sessions, so Vigilante reads them as test code. Test code includes these files:

- test folders and test files, for example `tests/`, `__tests__/`, `*.Tests/`, `*_test.go`, `*.spec.ts`, `*Test.java`, and `test_*.py`;
- fixtures and mocks;
- benchmark folders, which are `bench/`, `benchmark/`, `benchmarks/`, and `perf/`.

A folder name must match the whole name. `perf/` is test code, but `perfect/` and `src/perf.js` are not.

The pattern rules find the powers in test code. The report lists them in the section "Test code", apart from the findings. A diff lists each power that is new in its file. Test code does not go into the lock, and it blocks only when a `deny` rule matches. To give test code the same lock and approval as other code, set `[policy] tests = "lock"`. The call graph engines do not follow calls from test code.

Vigilante skips `node_modules`, `vendor`, `dist`, `build`, and minified JavaScript. To skip more files, add globs to `[scan] exclude`.

### Low-signal powers

The lock records `read_system_state`, `files`, `operating_system`, `runtime`, and `unanalyzed`. The report hides them when the lock already has them. Most of them are reads, such as `process.env.NODE_ENV` or a log file. If a low-signal power is not in the lock, the report shows it, and the check fails. The report shows the low-signal notes as one count. To change the list, use `[report] hide`.

### Agent allow rules

An allow rule is a power only when it lets the agent run any code, install packages, push, or read content that anyone can write:

- `Bash`, `Bash(*)`, `WebSearch`, `WebFetch` with no domain, and each tool that Vigilante does not know, for example an `mcp__` tool.
- `WebFetch(domain:...)` for `github.com`, `raw.githubusercontent.com`, `gist.github.com`, `gist.githubusercontent.com`, or a domain with `*`.
- `Bash(...)` with one of these:
  - a prefix that is one program that runs code, such as `Bash(node:*)`, `Bash(python3:*)`, or `Bash(git:*)`;
  - a prefix for any package script, such as `Bash(npm run:*)`;
  - a risky action in any part of the command, such as `git push`, `npm install`, `pip install`, `gem install`, or `cargo install`;
  - a pipe into a program that runs code, such as `Bash(curl -s https://x | sh)`;
  - an open option that runs code or sends data, such as `Bash(node -e *)`, `Bash(sh -c *)`, `Bash(python3 -c *)`, or `Bash(curl -X POST *)`.

Before it reads the program name, Vigilante skips `env`, `sudo`, `nice`, `nohup`, `time`, `command`, and `X=Y`.

These are not powers: file tools (`Read`, `Edit`, `Write`, `Glob`, `Grep`), fixed commands such as `Bash(python3 -m pytest tests/)`, and narrow prefixes such as `Bash(git add:*)` and `Bash(yarn run lint:*)`.

The deny rules for the same tool are part of the power. For example, `allow: ["Bash"]` with `deny: ["Bash(rm:*)"]` gives `tool_allow:Bash, except Bash(rm:*)`. So a change to the deny rules of a tool changes the power, and the report shows it.

Vigilante also reads the frontmatter of skills (`.claude/skills/**/SKILL.md`) and commands (`.claude/commands/**/*.md`). The rules above apply to `allowed-tools`, and `hooks` are `hook` powers.

### Install and git hooks that run plain local commands

An install script in `package.json` or a local hook in `.pre-commit-config.yaml` is not a power when all these conditions are true:

- It is one command, with no shell operator.
- The command is not a program that runs code.
- The command is not a path to a script.
- In `package.json`, Vigilante knows the program. The known programs are `cp`, `mv`, `rm`, `mkdir`, `echo`, `touch`, `chmod`, `ln`, `true`, `cat`, `printf`, `exit`, `husky`, `tsc`, `rimraf`, `lefthook`, `simple-git-hooks`, and `is-ci`.

For example, `prepare: husky` and `postinstall: cp vendor/a.wasm public/` are not powers. `postinstall: company-bootstrap` is a power, because Vigilante cannot read what the program does.

For `npm run <script>`, `yarn run <script>`, and `pnpm run <script>`, Vigilante follows the script in the same `package.json`, with its `pre` and `post` scripts. So `prepare: npm run build` is not a power when `build` is `tsc`. It is a power when `prebuild` runs `curl ... | sh`.

A hook from a remote repository in `.pre-commit-config.yaml` is always a power. A local hook with `additional_dependencies` or `language: docker_image` is also a power.

### First-party actions

Actions from `actions/`, `github/`, and local actions (`./`) are not powers. To add your own organization, use `[ci] trusted_owners`.

### Generated code and marker types

The Go `native` rule skips generated protobuf files (`*.pb.go`), which import `unsafe`.

The Rust `native` rule skips `unsafe impl Send` and `unsafe impl Sync`. These lines only mark a type as safe to share between threads. The rule also skips `extern` blocks with `#[wasm_bindgen]`. These blocks import JavaScript functions in WebAssembly, and they do not call native code.

## Known limits

- Some calls resolve by name and import, not by type. See [dependencies.md](dependencies.md#known-limits).
- Capslock does not resolve each call. It reports the calls that it cannot follow as `unanalyzed`. See its [caveats](https://github.com/google/capslock/blob/main/docs/caveats.md).
- A rule for a prefix such as `Bash(git rebase:*)` is not a power. But some programs can run code through options, such as `git rebase --exec`.
- Vigilante does not report a change to a script that an existing hook runs, because the power stays the same.
- If a power moves to a new file or a new package, Vigilante reports it as new.
