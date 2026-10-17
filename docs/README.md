# Documentation

To install Vigilante and run it, start with the [README](../README.md).

## Guides

- [Run Vigilante in CI](github-action.md): the GitHub Action, how to protect the check, and other CI systems.
- [Configuration](configuration.md): each key in `vigilante.toml`, the lock format, and your own rules.
- [Powers](powers.md): each power, the scopes, and the rules that keep the report short.
- [Risks](risks.md): the changed uses of a power that the report shows, and the ones that block.
- [Benchmark](benchmark.md): the pairs of safe and unsafe changes, and the results for each language and surface.
- [Dependencies and call paths](dependencies.md): how Vigilante reads packages, resolves calls, and uses the network.

## Understand and contribute

- [Architecture](architecture.md): the flow, the modules, and the trust boundaries.
- [Design records](design/README.md): the decisions behind the current behavior.
- [Contributing](../CONTRIBUTING.md): the setup, the checks, the rules for code and docs, and the release.
- [Security](../SECURITY.md): how to report a security problem and how to check a release.

Coding agents start with [AGENTS.md](../AGENTS.md). Some folders also have an `AGENTS.md` file.
