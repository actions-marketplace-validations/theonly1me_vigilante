# Configuration

Vigilante works with no configuration. To change what it reports, add `vigilante.toml` to the root of the repository. Every key is optional. Vigilante stops with exit code 2 when the file has a key that it does not know.

## Example

```toml
[scan]
exclude = ["examples/**"]

[scopes]
payments = "services/payments/**"

[policy]
mode = "lock"
deny = [
  { scope = "web/**", capability = "exec" },
  { capability = "skip_permissions" },
]

[report]
hide = ["read_system_state", "files", "operating_system", "runtime", "unanalyzed"]

[severity]
network = "high"
third_party_action = "low"

[rules]
paths = [".vigilante/rules"]

[ci]
trusted_owners = ["your-org"]

[engines]
go = "capslock"
javascript = "deep"
ruby = "patterns"

[registries]
allow = ["https://npm.example.com/"]
```

## Keys

| Key | Default | Description |
| --- | --- | --- |
| `scan.exclude` | `[]` | More globs of files to skip. Vigilante always skips tests (including test projects such as `App.Tests/`), fixtures, mocks, `node_modules`, `vendor`, `dist`, `build`, and minified JavaScript. |
| `scopes.<name>` | none | Puts the code files that match the glob in one scope with this name. Without it, the scope of a code file is its nearest package root. |
| `policy.mode` | `"lock"` | `"lock"` fails when a power is not in `vigilante.lock`. `"report"` only reports. |
| `policy.deny` | `[]` | Powers that always fail the check, even when they are in the lock. `scope` is a glob that matches the scope path, and its default is every scope. `capability` is a power name from [powers.md](powers.md). |
| `report.hide` | `["read_system_state", "files", "operating_system", "runtime", "unanalyzed"]` | Powers that the report hides when they are in the lock. They are still in `vigilante.lock`. A hidden power that is not in the lock is shown. |
| `severity.<power>` | See [powers.md](powers.md) | `"low"`, `"medium"`, or `"high"` for a power. |
| `rules.paths` | `[]` | Folders with your own code rules. See [Your own rules](#your-own-rules). |
| `ci.trusted_owners` | `[]` | GitHub owners whose actions count as first party, like `actions/` and `github/`. The match ignores case. |
| `engines.go` | `"capslock"` | `"capslock"` follows Go calls into dependencies with Capslock and needs `go` and `capslock`. `"patterns"` uses the direct-use rules in `packs/go.yml`. See [powers.md](powers.md#go-capslock). |
| `engines.javascript` | `"typed"` | `"typed"` resolves calls with the TypeScript checker, then by name, and needs Node.js. `"deep"` resolves calls by name and import. `"patterns"` uses only the direct-use rules in `packs/`. See [dependencies.md](dependencies.md#calls-resolved-by-type). |
| `engines.python`, `engines.java`, `engines.rust`, `engines.csharp`, `engines.ruby` | `"deep"` | `"deep"` follows calls into the locked packages of that language. `"patterns"` uses only the direct-use rules in `packs/`. See [dependencies.md](dependencies.md). |
| `registries.allow` | `[]` | More registries that Vigilante can download packages from. Use it only for registries that need no credentials. See [dependencies.md](dependencies.md#registries-that-are-not-public). |

Globs use `/` between folders. `*` matches inside one folder, and `**` matches any number of folders. `services/**` matches the scopes `services/billing` and `services/billing/api`, but not the scope `services` itself.

## Which files each command reads

| Command | `vigilante.toml` | `vigilante.lock` |
| --- | --- | --- |
| `vigilante diff --base <base> --head <head>` | From the merge base of base and head | From head |
| `vigilante check` | From the working directory, or from `--commit` | From the same place |
| `vigilante lock` | From the working directory | Writes it in the working directory |

In a pull request, the configuration comes from the merge base. So a pull request cannot exclude its own files, change to report mode, or remove a `deny` rule.

## vigilante.lock

`vigilante lock` writes the lock. It lists every power in each scope, sorted, so two runs on the same files give the same file.

```toml
version = 1

[code."services/billing"]
powers = [
    "exec",
    "network",
]

[ci.".github/workflows"]
powers = ["third_party_action:acme-tools/publish-action"]

[ci.".github/workflows/release.yml"]
powers = [
    "token_write:contents",
    "secret:PYPI_TOKEN",
]

[agent.".mcp.json"]
powers = ["mcp_server:fs-tools: npx -y fs-tools-mcp"]
```

Each table is a surface (`code`, `ci`, `agent`, or `build`) and a scope. Each power is a name, and for most powers a detail after the colon, for example the secret name. Do not edit the file by hand. Run `vigilante lock` again.

`vigilante check` also fails when the lock has a power that the repository no longer has. Run `vigilante lock` to remove it.

## Your own rules

Vigilante finds code powers with [ast-grep](https://ast-grep.github.io/) rules. You can add rules for your own wrappers, for example an internal HTTP client. Put the rule files in a folder, and list the folder in `rules.paths`.

```yaml
id: acme-http-client
language: python
message: Uses the internal HTTP client
metadata:
  capability: network
rule:
  kind: import_from_statement
  has:
    field: module_name
    regex: ^acme\.http(\.|$)
```

- `metadata.capability` is the power that the rule finds. It must be a code power from [powers.md](powers.md), for example `exec`, `network`, `files`, or `modify_system_state`.
- `language` is one of `rust`, `tsx`, `python`, `ruby`, `csharp`, `java`, or `go`. Vigilante reads every JavaScript and TypeScript file with the `tsx` grammar, so use `tsx` for both.
- `files` and `ignores` limit the rule to some paths, as in ast-grep.
- A file can have more than one rule. Put `---` between them. Each `id` must be unique, also across the built-in rules.

The built-in rules are in [packs/](../packs). You can test a rule with `ast-grep scan` and `ast-grep test` before you add it.
