# Run Vigilante in CI

## GitHub Actions

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

### What the action does

1. If the repository has a `go.mod` file, the action installs the latest stable Go (`actions/setup-go`, with no cache). It also installs Capslock v0.3.3 with `go install`, which checks the module checksums.
2. It downloads the Vigilante release archive for the runner.
3. It verifies the build attestation of the archive with `gh attestation verify`. The attestation must come from `.github/workflows/release.yml` in `theonly1me/vigilante`, on the `main` branch, at the commit of the version tag, on a GitHub-hosted runner. If the check fails, the job stops before Vigilante runs.
4. It restores the package cache (`actions/cache`). The key is the runner OS and the hash of the lockfiles and project files. So a new lock gets a new cache.
5. On a `pull_request` event, it runs `vigilante diff --base <base commit> --head <head commit>`. On all other events, it runs `vigilante check`.
6. It writes the report to the job log and to the job summary. If `comment` is `true`, it also writes the report as one comment on the pull request. It updates the same comment on each run.
7. The job fails with exit code 1 if a power is not in the lock, the lock is not current, or a `deny` rule matches. It fails with exit code 2 if an error occurs.

### Inputs

| Input | Default | Description |
| --- | --- | --- |
| `version` | The version of the action | The Vigilante version to download |
| `comment` | `false` | If `true`, write the report as a comment on the pull request. A pull request with no new powers and no earlier comment gets no comment. |

### Requirements

- `fetch-depth: 0` on `actions/checkout`. `vigilante diff` needs the merge base of the pull request. With a shallow checkout, the action stops with exit code 2 and tells you to set `fetch-depth: 0`.
- A Linux or macOS runner. On Windows, the action stops with exit code 2.
- `gh` on the runner. GitHub-hosted runners have it.
- Network access to the public package registries: npm, PyPI, Maven Central, Google Maven, crates.io, nuget.org, rubygems.org, and GitHub. Vigilante downloads the locked packages to follow calls into them. See [dependencies.md](dependencies.md#network-use-and-the-cache).
- Node.js for the typed engines of JavaScript, TypeScript, and Python. Docker for `[engines] rust = "typed"`. GitHub-hosted Linux runners have both. Without them, Vigilante resolves calls by name and lists the scope under "Not analyzed".
- For Go: code that compiles, because Capslock type-checks it. Private Go modules need `GOPRIVATE` and git credentials in the job, as `go build` does. Without them, Vigilante stops with exit code 2.

### Permissions and forks

The job needs only `contents: read`, and it needs no secrets. It works on pull requests from forks with the `pull_request` event. Do not use `pull_request_target` for Vigilante. Vigilante does not need secrets or write access.

The `comment` input needs `pull-requests: write`. Only the comment step gets the token. On a pull request from a fork, GitHub gives a token that cannot write. The step then writes a warning, and the job result does not change.

### Pin the action

<!-- x-release-please-start-version -->
`@v0.3.2` is a tag. For the strongest guarantee, pin the action to the full commit SHA of the tag:

```yaml
      - uses: theonly1me/vigilante@<commit SHA of v0.3.2>
```
<!-- x-release-please-end -->

The action downloads the Vigilante version in its own `action.yml`. So a pinned action always runs the same binary.

## Protect the check

On a `pull_request` event, GitHub runs the workflow file from the pull request. So a pull request can remove the Vigilante step from its own workflow. To make a change like that need a review, do these three steps:

1. In `CODEOWNERS`, make your security team the owner of these files. `CODEOWNERS` is usually at `.github/CODEOWNERS`, so the `.github/` line covers it.

   ```text
   vigilante.lock      @your-org/security
   vigilante.toml      @your-org/security
   .vigilante/         @your-org/security
   .github/            @your-org/security
   ```

2. In the branch protection rule or ruleset for `main`, turn on "Require review from Code Owners".
3. In the same rule, make the `vigilante` check required.

To check these rules, run `vigilante doctor` with `gh` signed in. It reads `CODEOWNERS` from the working directory and the rules of the default branch from GitHub. It checks the rule folders in `[rules] paths` of `vigilante.toml`, and each file that is in a protected folder now. It exits with code 1 when a rule is missing. It also exits with code 1 when it cannot read a protected folder that exists. If your required check has another name, add `--check <name>`. Classic branch protection needs a token with admin access. Rulesets do not.

Vigilante also helps here. In a pull request, it reads `vigilante.toml` from the merge base. So a pull request cannot exclude its own files or turn off the lock. If a change to a workflow adds a power, the report shows it.

## Other CI systems

Vigilante needs the full git history and the two commits of the change. To install it, use a command from the [README](../README.md#install). Then run this command:

```sh
vigilante diff --base "$BASE_COMMIT" --head "$HEAD_COMMIT"
```

`BASE_COMMIT` is the commit of the target branch. `HEAD_COMMIT` is the commit of the change. Vigilante finds their merge base.

- For a repository with Go modules, also install Go, and run `go install github.com/google/capslock/cmd/capslock@v0.3.3`.
- To keep the package downloads between runs, cache the folder `$XDG_CACHE_HOME/vigilante`. On Linux, the default is `~/.cache/vigilante`.
- The exit codes are the same as in GitHub Actions.
- To check the full repository against the lock, for example on the main branch, run `vigilante check`.

## On your machine

Before you open a pull request, run this command:

```sh
vigilante diff --base origin/main
```

If you add a power on purpose, run `vigilante lock`. Then commit `vigilante.lock` with the change.
