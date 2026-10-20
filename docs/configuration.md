# Configuration

Vigilante works with no configuration. To change what it reports, add `vigilante.toml` to the root of the repository. Each key is optional. If the file has a key that Vigilante does not know, Vigilante stops with exit code 2.

## Example

```toml
[scan]
exclude = ["examples/**"]

[scopes]
payments = "services/payments/**"

[policy]
mode = "lock"
coverage = "warn"
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
| `scan.exclude` | `[]` | More globs of files to skip. Vigilante always skips `node_modules`, `vendor`, `dist`, `build`, and minified JavaScript. It reads tests, fixtures, mocks, and benchmark folders as test code. |
| `scopes.<name>` | none | Puts the code files that match the glob into one scope with this name. Without this key, the scope of a code file is its nearest package root. |
| `policy.mode` | `"lock"` | `"lock"` fails if a power is not in `vigilante.lock`. `"report"` only shows the report. |
| `policy.deny` | `[]` | Powers that always make the check fail, also when they are in the lock or when the report shows them as notes. `scope` is a glob for the scope path, and its default is all scopes. `capability` is a power name from [powers.md](powers.md). |
| `policy.coverage` | `"warn"` | `"warn"` lists the code that Vigilante could not read under "Not analyzed". `"block"` also fails a diff that adds a new line there. In `vigilante check`, `"block"` fails on each line. |
| `policy.tests` | `"report"` | `"report"` lists the powers in test code apart from the findings, and they block only when a `deny` rule matches. `"lock"` puts them in `vigilante.lock` under the `test` surface, as other code. See [powers.md](powers.md#test-code-and-vendored-code). |
| `report.hide` | `["read_system_state", "files", "operating_system", "runtime", "unanalyzed"]` | Powers that the report hides when they are in the lock. The lock still records them. If a hidden power is not in the lock, the report shows it. |
| `severity.<power>` | See [powers.md](powers.md) | `"low"`, `"medium"`, or `"high"` for a power. |
| `rules.paths` | `[]` | Folders with your own code rules. See [Your own rules](#your-own-rules). |
| `ci.trusted_owners` | `[]` | GitHub owners whose actions are first party, as `actions/` and `github/` are. The match ignores case. |
| `engines.go` | `"capslock"` | `"capslock"` follows Go calls into dependencies with Capslock, and needs `go` and `capslock`. `"patterns"` uses the direct-use rules in `packs/go.yml`. See [powers.md](powers.md#go-capslock). |
| `engines.javascript`, `engines.python` | `"typed"` | `"typed"` resolves calls by type with the TypeScript checker or Pyright, then by name. It needs Node.js. `"deep"` resolves calls by name and import. `"patterns"` uses only the direct-use rules in `packs/`. See [dependencies.md](dependencies.md#calls-resolved-by-type). |
| `engines.rust` | `"deep"` | `"typed"` resolves calls with rust-analyzer in a Docker build with no network, then by name. `"deep"` and `"patterns"` are as above. See [dependencies.md](dependencies.md#calls-resolved-by-type). |
| `engines.java`, `engines.csharp`, `engines.ruby` | `"deep"` | `"deep"` follows calls into the locked packages of that language. `"patterns"` uses only the direct-use rules in `packs/`. See [dependencies.md](dependencies.md). |
| `engines.type_check_seconds` | `300` | The time limit for each type check or typed build, in seconds. If a check reaches it, Vigilante stops the check, resolves calls by name, and lists the scope under "Not analyzed". |
| `registries.allow` | `[]` | More registries for package downloads. Use it only for registries that need no credentials. See [dependencies.md](dependencies.md#registries-that-are-not-public). |

Globs use `/` between folders. `*` matches in one folder, and `**` matches any number of folders. `services/**` matches the scopes `services/billing` and `services/billing/api`. It does not match the scope `services`.

## Which files each command reads

| Command | `vigilante.toml` and the rule files in `rules.paths` | `vigilante.lock` |
| --- | --- | --- |
| `vigilante diff --base <base> --head <head>` | From the merge base of base and head | From head |
| `vigilante check` | From the working directory, or from `--commit` | From the same place |
| `vigilante lock` | From the working directory | Writes it in the working directory |

In a pull request, the configuration and the custom rules come from the merge base. So a pull request cannot exclude its own files, change to report mode, remove a `deny` rule, or weaken a custom rule.

## vigilante.lock

`vigilante lock` writes the lock. The lock lists each power in each scope, in sorted order. So two runs on the same files give the same file.

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

[code."services/web"]
powers = ["exec"]
risks = ["exec:web.export:subprocess.run:untrusted"]
```

Each table is a surface (`code`, `ci`, `agent`, or `build`) and a scope. Each power is a name. For most powers, a detail comes after the colon, for example the name of a secret. Do not edit the file by hand. Run `vigilante lock` again.

`risks` lists the [risks](risks.md) that you approved. `vigilante lock` writes each risk that blocks. An approved risk does not make the check fail.

`vigilante check` also fails if the lock has a power or a risk that the repository does not have now. To remove it, run `vigilante lock`.

## Your own rules

Vigilante finds code powers with [ast-grep](https://ast-grep.github.io/) rules. You can add rules for your own wrappers, for example an internal HTTP client. Put the rule files in a folder, and add the folder to `rules.paths`.

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
- `language` is `rust`, `tsx`, `python`, `ruby`, `csharp`, `java`, or `go`. Vigilante reads each JavaScript and TypeScript file with the `tsx` grammar, so use `tsx` for both.
- `files` and `ignores` limit the rule to some paths, as in ast-grep.
- `metadata.inert` lists the members of an imported module that do not have the power. Calls to these members are not sinks.
- A file can have more than one rule. Put `---` between the rules. Each `id` must be unique, also across the built-in rules.

The built-in rules are in [packs/](../packs). Before you add a rule, you can test it with `ast-grep scan` and `ast-grep test`.
