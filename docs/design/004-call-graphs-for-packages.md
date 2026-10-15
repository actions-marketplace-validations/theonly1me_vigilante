# 004: Call graphs for packages in other languages

## Problem

For JavaScript, TypeScript, Python, Java, Rust, C#, and Ruby, no mature tool follows calls into packages and maps them to powers. Without this, a new dependency can bring a power that the report does not show.

## Decision

1. Vigilante has one shared engine in `src/deep/`. Each language gives a parser, an import resolver, and the sinks from its rule pack.
2. Vigilante reads the lockfile of the scope. For Maven POMs and NuGet `PackageReference` items, it resolves the versions itself, because a build runs code from the pull request.
3. It downloads each package that the code imports from its public registry, and checks the checksum from the lockfile or the registry. It extracts archives with path checks and size limits. It never runs install scripts or package code.
4. It keeps a cache of the packages. The cache key has the address and the checksum.
5. It reads Java packages as bytecode and C# packages as IL, and it reads other packages as source.
6. The finding rule is the same as for Go. A power that a scope reaches only through another scope of the same repository is a note, because that scope shows its own finding.
7. Each package or code file that Vigilante cannot read gives a line under "Not analyzed". The check does not fail for these lines.

## Consequences

- Vigilante now uses the network. It downloads only from public registries and from `[registries] allow`, and it never sends credentials.
- A call by name can reach each method with that name. [005](005-typed-call-resolution.md) removes most of these false paths.
- yarn 2 lockfiles have a checksum for the yarn zip file, so Vigilante cannot check those downloads.

## Verification

The tests run each language against fake registries. A tampered package stops the run with exit code 2, and an archive entry with `../` fails.
