# 006: Release and distribution

## Problem

Users run Vigilante from npm, PyPI, Homebrew, crates.io, GitHub Releases, and a GitHub Action. A release by hand to six channels is slow, and a mistake can publish different binaries in different channels.

## Decision

1. One build makes all release files: static binaries for 4 targets, archives, 5 npm packages, wheels, and a Homebrew formula. No package has an install script.
2. Release Please reads the conventional commits on `main` and keeps a release pull request open. It sets the version in each file that has one.
3. When a maintainer merges the release pull request, Release Please makes the tag and the GitHub release. The release workflow then builds, attests, and uploads each file.
4. The publish jobs use the `release` environment, which only `main` can use. Each job skips a version that its registry already has.
5. PyPI uses trusted publishing from the first release. crates.io and npm cannot make a new package with trusted publishing, so the first release uses tokens. After that, they use trusted publishing, and the tokens go away.
6. The Action verifies the attestation of each archive. The attestation must come from `release.yml` on `main`, at the commit of the version tag.
7. The release workflow starts CI on the release pull request with `workflow_dispatch`, because a pull request that `GITHUB_TOKEN` makes does not start workflows.

## Consequences

- A release needs one human step: the merge of the release pull request.
- The version is in several files. `packaging/check-versions.sh` fails if they are different.
- Windows has no binary.

## Verification

- `build.yml` runs on each pull request that changes the packaging, and installs the npm package and the wheel on Linux and macOS.
- `action/test.sh` checks the flags of the attestation check.
