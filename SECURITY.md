# Security

Report a security problem in private. On the **Security** tab of this repository, select **Report a vulnerability**. Do not put the details in a public issue.

## What to report

- A change that adds a power, but Vigilante does not report the power or the check passes.
- Code from a checked repository or package that Vigilante runs outside the limits in [docs/dependencies.md](docs/dependencies.md#calls-resolved-by-type).
- A package download that Vigilante uses without a checksum check.
- An archive entry that Vigilante writes outside its target folder.
- A release file with an attestation that does not match the release workflow of this repository.

Include a small reproduction with your own sample files, and the version of Vigilante. Before version 1.0, security fixes go into the newest release only. There is no bounty, and there is no fixed time for a reply.

## Check a release

Each release file has a build attestation from [.github/workflows/release.yml](.github/workflows/release.yml). To check an archive, run this command:

```sh
gh attestation verify vigilante-x86_64-unknown-linux-musl.tar.gz --repo theonly1me/vigilante
```

The npm packages have provenance, and the PyPI files come from trusted publishing. Both name the same workflow.
