# 007: Repository guardrails

## Problem

Vigilante checks the security of other repositories. Its own repository must show the same care: pinned tools, checked dependencies, and rules that a machine enforces.

## Decision

1. CI runs these checks on each pull request, each with a time limit:
   - format, lints, and tests on Linux and macOS;
   - the Action scripts;
   - actionlint and zizmor on the workflows;
   - `cargo deny` for advisories, licenses, and sources;
   - the plain English check on the docs.
2. The `repository` test checks each tracked file. Code files stay under 200 lines and have no comments. No file has an em dash, an en dash, a home folder of a real machine, or a private term. The test keeps private terms as SHA-256 hashes.
3. A ruleset on `main` requires a pull request and the CI checks, and blocks force pushes and branch deletion.
4. Dependabot updates actions and crates each week, and waits 7 days after each release of a dependency.
5. `AGENTS.md` files and the shadowclone workflow skills give coding agents the same rules as people.
6. The docs use ASD-STE100 Simplified Technical English at 90% strictness.

## Consequences

- `deny.toml` allows the `unsafe-libyaml` advisory, because no maintained release exists and only `ast-grep-config` uses it.
- `.github/zizmor.yml` allows the `./` form for the two reusable workflow calls in `release.yml`, because actionlint does not accept the newer `$/` form yet.

## Verification

Each rule of the `repository` test fails when a tracked file breaks it.
