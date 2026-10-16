# Release packages

One build makes the files for every channel: the archives, the npm packages, the Python wheels, and the Homebrew formula. The workflows `.github/workflows/build.yml` and `.github/workflows/release.yml` run these scripts.

## Versions

Release Please sets the version in `Cargo.toml`, `Cargo.lock`, `npm/vigilante/package.json`, and `action.yml`. `packaging/check-versions.sh` fails if two of them are different. Do not change a version by hand.

## Scripts

- `check-versions.sh`: compares the version fields with each other and with the tag.
- `registry-readme.mjs`: writes a copy of the README with absolute links to the release tag.
- `homebrew.sh`: writes `vigilante.rb` from `SHA256SUMS`.
- `smoke.sh`, `smoke-npm.sh`, `smoke-wheel.sh`: install a package and run `vigilante --version` and one failing command.
- `download-release.sh`: downloads files of a release and verifies the attestation of each file.
- `publish-npm.sh`: publishes the npm packages. With `DRY_RUN=true`, it only checks them. `build.yml` runs it that way.

## Rules

- No package has an install script. Vigilante reports install scripts as a power.
- Each release file gets a build attestation.
- A publish step skips a version that the registry already has, so a rerun is safe.
