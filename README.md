<div align="center">
  <img src="assets/vigilante-wordmark-white-bg.webp" alt="Vigilante" width="360">
  <p><b>See what a pull request lets your code, your CI, and your AI agents do, before you merge it.</b></p>
  <p>
    <a href="https://github.com/theonly1me/vigilante/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/theonly1me/vigilante/ci.yml?branch=main&amp;label=CI&amp;logo=github" alt="CI"></a>
    <a href="https://github.com/theonly1me/vigilante/releases/latest"><img src="https://img.shields.io/github/v/release/theonly1me/vigilante?label=release" alt="Latest release"></a>
    <a href="https://crates.io/crates/vigilante"><img src="https://img.shields.io/crates/v/vigilante?logo=rust" alt="crates.io"></a>
    <a href="https://www.npmjs.com/package/@theonly1me/vigilante"><img src="https://img.shields.io/npm/v/@theonly1me/vigilante?logo=npm" alt="npm"></a>
    <a href="https://pypi.org/project/vigilante-scan/"><img src="https://img.shields.io/pypi/v/vigilante-scan?logo=pypi&amp;logoColor=white" alt="PyPI"></a>
    <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0"></a>
    <a href="https://github.com/theonly1me/vigilante/blob/main/Cargo.toml"><img src="https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust" alt="Rust 1.88 or later"></a>
  </p>
</div>

Vigilante reads a pull request and lists the new powers that the pull request adds. A power is an action that a change lets code, CI, a build, or an AI agent do. Examples are the first `subprocess` call in a package, a CI token that can push, a new MCP server, and a new install script.

Your security team approves powers in a `vigilante.lock` file. If a pull request adds a power that is not in the lock, the check fails.

By default, Vigilante does not run code from the repository that it checks. It reads the files and resolves calls by type. It uses [Capslock](https://github.com/google/capslock) for Go, the TypeScript type checker for JavaScript and TypeScript, and Pyright for Python. These tools read code, and they do not run it. Vigilante also downloads the locked packages from their public registries and reads them. If you set `[engines] rust = "typed"`, Vigilante builds the Rust code in a Docker container that has no network access ([docs/dependencies.md](docs/dependencies.md#calls-resolved-by-type)).

## Why

AI agents open more pull requests than people can read line by line. A one-line change can change what each later agent run and CI job can do. Examples are a change to `.mcp.json`, `.claude/settings.json`, a workflow, or a `postinstall` script. Vigilante shows these changes at the top of the pull request. It uses the same format for each language and each type of file.

## Example

An agent opens a pull request that exports invoices. Vigilante gives this report:

> ## Vigilante: this pull request adds 6 new powers
>
> **Result: fail.** 6 powers are not in vigilante.lock. Run `vigilante lock` and commit the file.
>
> | Severity | Scope | Power | What it means | Evidence | Status |
> | --- | --- | --- | --- | --- | --- |
> | HIGH | `services/billing` (code) | `exec` | Can run other programs | `services/billing/billing/export.py:5 via export.export → subprocess.run` | **not in lock** |
> | HIGH | `.mcp.json` (agent) | `mcp_server:fs-tools: npx -y fs-tools-mcp` | The agent connects to an MCP server | `.mcp.json:3` | **not in lock** |
> | MEDIUM | `.github/workflows` (ci) | `third_party_action:acme-tools/publish-action` | The workflow runs a third-party action | `.github/workflows/release.yml:12` | **not in lock** |
> | MEDIUM | `.github/workflows/release.yml` (ci) | `token_write:contents` | The CI token can write to the repository | `.github/workflows/release.yml:6` | **not in lock** |
> | MEDIUM | `.github/workflows/release.yml` (ci) | `secret:PYPI_TOKEN` | The workflow reads a secret | `.github/workflows/release.yml:14` | **not in lock** |
> | MEDIUM | `.claude/settings.json` (agent) | `tool_allow:Bash(curl:*)` | The agent can use a tool without asking | `.claude/settings.json:3` | **not in lock** |

The same pull request also added a second network call to `services/billing`, the allow rules `Bash(pytest:*)` and `Read`, and a call to `actions/checkout`. Vigilante does not report them, for these reasons:

- The package already used the network.
- An allow rule for a test runner has a low risk.
- `Read` only reads files.
- The `actions/` actions are first party.

## What it checks

| Surface | Files | Powers |
| --- | --- | --- |
| Code | Rust, JavaScript, TypeScript, Python, Ruby, C#, Java, and Go | `exec`, `network`, `files`, `native`, `dynamic_code`, `read_system_state`, `modify_system_state`, `system_calls` |
| Code, through dependencies | Go with [Capslock](https://github.com/google/capslock). JavaScript, TypeScript, Python, Java, Rust, C#, and Ruby with the call graph of Vigilante over the locked packages. Calls resolve by type in JavaScript, TypeScript, and Python, and through the class hierarchy in Java and C#. | Each code power, with the call path. For Go, all 13 Capslock capabilities, for example `operating_system`, `runtime`, `arbitrary_execution`, and `unanalyzed`. |
| CI | `.github/workflows/*.yml`, composite actions (`action.yml`) | `token_write`, `oidc`, `untrusted_trigger`, `third_party_action`, `secret`, `remote_script` |
| AI agents | `.claude/settings.json`, Claude Code skills and commands, `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json`, `.codex/config.toml`, `.gemini/settings.json` | `tool_allow`, `hook`, `mcp_server`, `skip_permissions` |
| Build | `package.json` install scripts, `build.rs`, `setup.py`, `.husky/`, `.pre-commit-config.yaml` | `install_hook`, `git_hook` |

Vigilante reports a power the first time that it appears in a scope. After that, it reports each changed risk. Some risks block until `vigilante.lock` approves them. Examples are a call with untrusted input, and a workflow script that reads the title of a pull request. See [docs/risks.md](docs/risks.md). For code, a scope is the nearest package root. For Go, a scope is the Go package. Vigilante skips vendored code. It reports tests and fixtures as test code, apart from the findings. See [docs/powers.md](docs/powers.md) for each power and for the rules that keep the report short.

A finding is `exec`, `network`, or `native` that your package reaches through its own code. This rule is the same in each language. All other results are notes:

- another power that your own code reaches;
- a power that your package reaches only through a dependency, or through another package of the same repository;
- a new package on a call path to a power.

Notes do not make the check fail, unless a `deny` rule matches their power. For example, a new dependency that reaches the network is a note. [docs/dependencies.md](docs/dependencies.md) tells you how Vigilante reads packages.

## Install

Vigilante is one static binary for Linux and macOS, on x86_64 and arm64. Use the package manager that your team uses. The command is always `vigilante`.

| Package manager | Command |
| --- | --- |
| npm | `npm install --save-dev @theonly1me/vigilante` or `npx @theonly1me/vigilante` |
| PyPI | `pip install vigilante-scan` or `uvx --from vigilante-scan vigilante` |
| Homebrew | `brew install theonly1me/tap/vigilante` |
| Cargo | `cargo install vigilante --locked` |
| GitHub Releases | Download `vigilante-<target>.tar.gz` from the [releases page](https://github.com/theonly1me/vigilante/releases). |

Each release file has a build attestation. To make sure that an archive comes from the release workflow of this repository, run this command:

```sh
gh attestation verify vigilante-x86_64-unknown-linux-musl.tar.gz --repo theonly1me/vigilante
```

Vigilante does not support Windows. To check Go code, Vigilante needs `go` and `capslock`. To install Capslock, run `go install github.com/google/capslock/cmd/capslock@v0.3.3`. The GitHub Action installs both.

## Quick start

Run these commands in the root of your repository.

| Command | What it does |
| --- | --- |
| `vigilante scan` | Shows each power in the repository, and the code that Vigilante could not read. |
| `vigilante lock` | Writes `vigilante.lock`. Commit this file. |
| `vigilante diff --base origin/main` | Shows the powers that your branch adds, compared with its merge base. |
| `vigilante check` | Fails if `vigilante.lock` does not agree with the repository. |
| `vigilante doctor` | Checks that `CODEOWNERS` and the branch rules on GitHub protect `vigilante.lock`. It uses `gh`. |

`diff` and `check` write Markdown. For other tools, add `--format json` or `--format sarif`.

| Exit code | Meaning |
| --- | --- |
| 0 | Pass |
| 1 | A power is not in the lock, the lock is not current, or a `deny` rule matches. |
| 2 | Error, for example a file that Vigilante cannot read. |

## Run it on each pull request

Add `.github/workflows/vigilante.yml`:

<!-- x-release-please-start-version -->
```yaml
name: Vigilante
on:
  pull_request:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  vigilante:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0
          persist-credentials: false
      - uses: theonly1me/vigilante@v0.3.2
```
<!-- x-release-please-end -->

On a pull request, the action runs `vigilante diff` against the base branch. On a push, it runs `vigilante check`. The action writes the report to the job summary. The job needs no secrets, so it also works on pull requests from forks. To also write the report as a comment on the pull request, set `comment: true` and give the job the permission `pull-requests: write`.

Then make your security team the owner of the lock in `CODEOWNERS`. Also turn on "Require review from Code Owners".

```text
vigilante.lock      @your-org/security
vigilante.toml      @your-org/security
.github/            @your-org/security
```

After this change, each pull request that adds a power must change `vigilante.lock`. The security team must approve that change. Run `vigilante doctor` to make sure that GitHub enforces these rules. [docs/github-action.md](docs/github-action.md) gives the full setup. It also tells you how to protect the check and how to use Vigilante in other CI systems.

## Use it with AI agents

Tell your agents to run `vigilante diff --base origin/main` before they open a pull request. If an agent adds a power on purpose, tell it to run `vigilante lock`. The pull request then shows the new powers, and the change to the lock asks for the correct review.

## Configuration

Vigilante works with no configuration. To change what it reports, add `vigilante.toml`:

```toml
[scan]
exclude = ["examples/**"]

[policy]
deny = [{ scope = "services/**", capability = "exec" }]

[ci]
trusted_owners = ["your-org"]
```

In a pull request, Vigilante reads `vigilante.toml` from the merge base. So a pull request cannot change the rules that check it. [docs/configuration.md](docs/configuration.md) lists each key and the lock format. It also tells you how to add rules for your own wrappers.

## What Vigilante is not

- It is not a malware detector. It shows what changed, and a person decides if the change is safe.
- It does not follow calls that the code makes at run time, such as reflection, `eval`, or metaprogramming.
- It uses no AI to find or sort powers. The same input gives the same output.
- It downloads packages only from public registries and from the registries that you allow. It keeps them in a cache.

## Contribute

Read [CONTRIBUTING.md](CONTRIBUTING.md). To report a security problem, read [SECURITY.md](SECURITY.md). [docs/README.md](docs/README.md) lists all the documentation.

## License

You can use Vigilante under the [Apache License 2.0](LICENSE-APACHE) or the [MIT license](LICENSE-MIT).
