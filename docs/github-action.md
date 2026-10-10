# Run Vigilante in CI

## GitHub Actions

Add `.github/workflows/vigilante.yml`:

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
      - uses: theonly1me/vigilante@v0.1.0
```

### What the action does

1. If the repository has a `go.mod` file, it installs the latest stable Go (`actions/setup-go`, no cache) and Capslock v0.3.3 (`go install`, which checks module checksums).
2. It downloads the Vigilante release archive for the runner.
3. It verifies the build attestation of the archive with `gh attestation verify`. The attestation must come from `.github/workflows/release.yml` in `theonly1me/vigilante`, from the tag of the version, on a GitHub-hosted runner. If the check fails, the job stops before Vigilante runs.
4. It restores the package cache (`actions/cache`). The key is the runner OS and the hash of the lockfiles and project files, so a new lock gets a new cache.
5. On a `pull_request` event, it runs `vigilante diff --base <base commit> --head <head commit>`. On any other event, it runs `vigilante check`.
6. It writes the report to the job log and to the job summary.
7. The job fails with exit code 1 when a power is not in the lock, the lock is stale, or a `deny` rule matches. It fails with exit code 2 on an error.

### Inputs

| Input | Default | Description |
| --- | --- | --- |
| `version` | The version of the action | The Vigilante version to download |

### Requirements

- `fetch-depth: 0` on `actions/checkout`. `vigilante diff` needs the merge base of the pull request. With a shallow checkout, the action stops with exit code 2 and tells you to set `fetch-depth: 0`.
- A Linux or macOS runner. The action stops with exit code 2 on Windows.
- `gh` on the runner. GitHub-hosted runners have it.
- Network access to the public package registries (npm, PyPI, Maven Central, Google Maven, crates.io, nuget.org, rubygems.org, and GitHub). Vigilante downloads the locked packages to follow calls into them. See [dependencies.md](dependencies.md#network-use-and-the-cache).
- For Go: code that compiles, because Capslock type-checks it. Private Go modules need `GOPRIVATE` and git credentials in the job, as for `go build`. Without them, Vigilante stops with exit code 2.

### Permissions and forks

The job needs only `contents: read` and no secrets. It works on pull requests from forks with the `pull_request` event. Do not use `pull_request_target` for Vigilante. It does not need secrets or write access.

### Pin the action

`@v0.1.0` is a tag. For the strongest guarantee, pin the action to the full commit SHA of the tag:

```yaml
      - uses: theonly1me/vigilante@<commit SHA of v0.1.0>
```

The action downloads the Vigilante version that is in its own `action.yml`, so a pinned action always runs the same binary.

## Protect the check

On a `pull_request` event, GitHub runs the workflow file from the pull request. So a pull request can remove the Vigilante step from its own workflow. Do these three steps so that a change like that needs a review:

1. Give these files to your security team in `CODEOWNERS`. `CODEOWNERS` is usually at `.github/CODEOWNERS`, so the `.github/` line covers it.

   ```text
   vigilante.lock      @your-org/security
   vigilante.toml      @your-org/security
   .vigilante/         @your-org/security
   .github/            @your-org/security
   ```

2. In the branch protection rule or ruleset for `main`, turn on "Require review from Code Owners".
3. In the same rule, make the `vigilante` check required.

Vigilante also helps here. In a pull request, it reads `vigilante.toml` from the merge base, so a pull request cannot exclude its own files or turn off the lock. A change to a workflow shows up in the report when it adds a power.

## Other CI systems

Vigilante needs the full git history and the two commits of the change. Install it with one of the commands in the [README](../README.md#install), then run:

```sh
vigilante diff --base "$BASE_COMMIT" --head "$HEAD_COMMIT"
```

`BASE_COMMIT` is the commit of the target branch, and `HEAD_COMMIT` is the commit of the change. Vigilante finds their merge base. For a repository with Go modules, also install Go and `go install github.com/google/capslock/cmd/capslock@v0.3.3`. To keep package downloads between runs, cache the folder in `$XDG_CACHE_HOME/vigilante` (default `~/.cache/vigilante` on Linux). The exit codes are the same as in GitHub Actions. To check the whole repository against the lock, for example on the main branch, run `vigilante check`.

## On your machine

Before you open a pull request, run this command:

```sh
vigilante diff --base origin/main
```

If you add a power on purpose, run `vigilante lock` and commit `vigilante.lock` with the change.
