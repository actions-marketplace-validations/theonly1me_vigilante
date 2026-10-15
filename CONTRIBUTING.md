# Contributing

## Set up

1. Install Rust stable with `rustfmt` and `clippy`.
2. Install Node.js 22 or later. The typed engines for JavaScript, TypeScript, and Python use it, and `scripts/check-docs.sh` uses it.
3. To run all tests, also install Docker, Go, and Capslock v0.3.3 (`go install github.com/google/capslock/cmd/capslock@v0.3.3`).
4. Run `cargo build`.

## Checks

Run these commands before you open a pull request:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked -- --include-ignored
scripts/check-docs.sh
```

The ignored tests need Node.js, Docker, network access, or Go with Capslock. Each one names its tool in its `#[ignore]` mark. CI runs all of them on Linux. On macOS, CI skips the Docker tests.

CI also runs these checks:

- actionlint and zizmor on the workflows and on `action.yml`.
- `cargo deny` on the dependencies: advisories, licenses, and sources. [deny.toml](deny.toml) holds the policy.
- `action/test.sh` on the Action scripts.
- The `repository` test, which checks the rules below on each tracked file.

## Make a change

1. For a new product decision or architecture decision, write a record in [docs/design/](docs/design/README.md) first.
2. Write the test first. Test the result that a user sees, such as a report line, an exit code, or the lock file.
3. Make the change.
4. Prove the test. Change the code that the test guards, and make sure that the test fails for the reason that you expect. Then restore the code.
5. Update each page in [docs/](docs/README.md) that your change affects.
6. Run the checks.

[AGENTS.md](AGENTS.md) has the map of the code and the rules that a reviewer enforces.

## Code rules

The `repository` test enforces the first three rules.

- Keep each code file under 200 lines. This rule includes tests.
- Write no comments. Use names, types, and tests to show the intent.
- Do not use em dashes or en dashes in any file.
- Use full words in names.
- If a function has two or more arguments, give it one options struct.
- Fix each lint. Do not turn a lint off.

## Documentation

Write all Markdown in ASD-STE100 Simplified Technical English at 90% strictness:

- Write one idea in each sentence.
- Keep an instruction to 20 words or fewer, and a description to 25 words or fewer.
- Use the active voice, common words, and the same term for the same thing.
- Put a condition before its instruction.

`scripts/check-docs.sh` reports long sentences, dashes, the passive voice, phrasal verbs, and vague words. It fails on each error and each warning. `CHANGELOG.md` and `CODE_OF_CONDUCT.md` do not need this check.

Keep personal details, machine paths, and work notes out of the repository. The `repository` test checks for home folders and private terms.

## Pull requests

- Use the [pull request template](.github/pull_request_template.md).
- Write the title as one conventional-commit subject, such as `fix(deep): stop a type check at its time limit`. Write each commit message in the same form, with no body.
- Use `feat` for a new feature, `fix` for a bug fix, and `!` for a change that breaks users. Release Please uses these types to choose the next version.
- Sign each commit.
- CI must pass. A pull request to `main` needs the checks `test`, `action`, `workflows`, `dependencies`, and `docs`.

## Release

Releases are automatic. A maintainer only merges the release pull request.

1. On each push to `main`, Release Please updates a release pull request. The pull request sets the next version in `Cargo.toml`, `Cargo.lock`, `npm/vigilante/package.json`, `action.yml`, and the docs, and it updates `CHANGELOG.md`.
2. The release workflow starts CI on the release pull request.
3. When a maintainer merges the release pull request, Release Please makes the tag and the GitHub release.
4. The release workflow builds the binaries for 4 targets, the npm packages, the wheels, and the Homebrew formula. It attests each file and adds it to the GitHub release.
5. Then it publishes to crates.io, npm, PyPI, and the Homebrew tap. Each step skips a version that the registry already has, so you can run a failed job again.

The publish jobs use the `release` environment. Only `main` can use it.

### Set up the registries one time

crates.io and npm cannot make a new package through trusted publishing. So the first release uses tokens.

1. On crates.io, make an API token with the `publish-new` and `publish-update` scopes for `vigilante`. Run `gh secret set CARGO_REGISTRY_TOKEN --env release`.
2. On npm, make a granular access token that can publish to the `@theonly1me` scope. Run `gh secret set NPM_TOKEN --env release`.
3. On PyPI, add a pending trusted publisher for `vigilante-scan`, with the workflow `release.yml` and the environment `release`.
4. Add a deploy key with write access to the `homebrew-tap` repository. Run `gh secret set HOMEBREW_TAP_DEPLOY_KEY --env release` with the private key.

After the first release, use trusted publishing:

1. On crates.io, add a trusted publisher for `vigilante` with the workflow `release.yml` and the environment `release`.
2. For each of the 5 npm packages, run `npm trust github <package> --file release.yml --repo theonly1me/vigilante --env release --allow-publish`.
3. Delete the `CARGO_REGISTRY_TOKEN` and `NPM_TOKEN` secrets. The workflow then uses trusted publishing.
