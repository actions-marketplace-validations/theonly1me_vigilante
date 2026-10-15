# Working on Vigilante

Vigilante is a Rust CLI and a GitHub Action. It reads a pull request and reports the new powers that the change gives to code, CI, builds, and AI agents. A committed `vigilante.lock` file records the approved powers.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before you change code or docs. Before you change a folder, read the `AGENTS.md` file in that folder if it has one.

## Map

| Path | Contents |
| --- | --- |
| `src/main.rs`, `src/lib.rs` | The CLI and the library: `scan`, `review`, and the report formats. |
| `src/surfaces/` | CI workflows, agent config, and install hooks. |
| `src/code/`, `packs/` | The pattern rules for each language. |
| `src/deep/` | The shared call graph engine. Each language folder in `src/` gives a parser, a resolver, and a typed resolver. |
| `src/go/` | The Go engine, which runs Capslock. |
| `src/registry/`, `src/lockfiles/` | Package downloads, checksums, and lockfile readers. |
| `src/container/`, `src/process/` | Docker builds with no network, and child processes with a time limit. |
| `tests/integration/` | End-to-end tests on temporary git repositories with fake registries. |
| `action.yml`, `action/` | The GitHub Action. |
| `npm/`, `packaging/`, `pyproject.toml` | The release packages. |
| `docs/` | The guides, the [architecture](docs/architecture.md), and the [design records](docs/design/README.md). |

## Rules that a reviewer enforces

1. By default, Vigilante does not run code from the repository or from its packages. Only `[engines] rust = "typed"` runs build code, and only in a container with no network.
2. Type data never removes a finding. A finding comes from the typed result or from the name-based result.
3. In a diff, Vigilante reads `vigilante.toml` from the merge base. A pull request cannot change its own policy.
4. If a tool is missing, fails, or reaches its time limit, Vigilante resolves calls by name and lists the scope under "Not analyzed".
5. Input that Vigilante cannot read stops the run with exit code 2. The check fails closed.
6. Each download gets a checksum check. Archive extraction rejects a path outside the target folder.
7. `vigilante lock` writes the same bytes for the same input.

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
scripts/check-docs.sh
```

`cargo test --locked -- --include-ignored` also runs the tests that need Node.js 22, Docker, network access, or Go with Capslock v0.3.3. CI runs all of them. Run the focused tests while you work, and run all the checks before you hand off.

## Code

- Keep each file under 200 lines. This rule includes tests.
- Write no comments. Use names, types, and tests to show the intent.
- Use full words in names.
- If a function has two or more arguments, give it one options struct.
- Write the test first. Then change the code that the test guards, and make sure that the test fails.
- Do not use em dashes or en dashes.

## Documentation

- Write all Markdown in ASD-STE100 Simplified Technical English at 90% strictness. Use the `write-plain-english` skill. `scripts/check-docs.sh` checks each file. `CHANGELOG.md` and `CODE_OF_CONDUCT.md` do not need this check.
- Record each product decision or architecture decision in `docs/design/`. When the flow or a trust boundary changes, update [docs/architecture.md](docs/architecture.md).
- Keep personal details, machine paths, and work notes out of the repository.

## Git and pull requests

- Write each commit message and each pull request title as one conventional-commit subject, such as `fix(deep): stop a type check at its time limit`. Add no body and no trailers. Release Please reads these subjects to make releases.
- Sign each commit.
- Use the [pull request template](.github/pull_request_template.md).
- Do not force push to `main`.

<shadowclone-guidance>
## Your agent build

Follow repository requirements. Repository build choices override personal global choices. Load each selected workflow skill at the moment listed for it.

- when a change adds a skip, block, fallback, default, or retry: choose-by-consequence
- when designing or reshaping a module or subsystem: design-deep-modules
- when behavior is wrong or slow and the cause is not known: diagnose-before-editing
- before a change that has many steps or open design choices: plan-with-review-page
- when a decision depends on facts outside the repository: research-primary-sources
- while a merge, rebase, or cherry-pick has conflicts: resolve-conflicts-by-intent
- when fixing a bug, a regression, or a confirmed review finding: scope-confirmed-changes
- when a pull request needs a review: shadowclone-review
- when a pull request needs a guided review with notes: guided-pr-review
- when taking a change or a pull request to ready for review: shadowclone-work
- when adding, changing, or proving a test: tests-that-catch-bugs
- before you say that work is done or ready for review: verify-and-review
- when a pull request has review comments or bot findings: verify-review-findings
- before you write any text that a person reads: write-plain-english
</shadowclone-guidance>
