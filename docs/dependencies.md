# Dependencies and call paths

Vigilante follows calls from your code into the packages that your code uses. It does this for Go, JavaScript and TypeScript, Python, Java, Rust, C#, and Ruby. Each result has a call path, for example `services/billing/billing/export.py:5 via export.export → subprocess.run`.

Vigilante never runs code from the repository or from a package. It does not install packages, run install scripts, or run a build.

## Findings and notes

The rule is the same for every language:

1. `exec`, `network`, or `native` that your own code reaches is a finding. The call path stays in your repository until the privileged call. A finding must be in `vigilante.lock`.
2. Everything else is a note:
   - any other power that your own code reaches, for example `dynamic_code` or `modify_system_state`;
   - a power that your code reaches only through a package;
   - a package that is new on a call path to a power.
3. Notes appear in the section "New packages on call paths". They do not fail the check and do not go in the lock. The Markdown report only counts the notes for the powers in `report.hide` (for example `files`), and the JSON report has all of them.

A new dependency that reaches the network is a note that names the dependency. A finding appears only the first time that its power appears in its scope.

## How it works

1. Vigilante reads the lockfile of the package or project that holds the changed code.
2. It downloads each locked package that your code imports from its public registry. It checks the package against the checksum in the lockfile or from the registry, and stops with exit code 2 when they differ.
3. It reads your code and the package code, and builds a call graph: functions, imports, and calls.
4. The rules in `packs/` mark the privileged calls, for example `child_process.exec`, `subprocess.run`, `ProcessBuilder`, `std::process::Command`, `Process.Start`, and `system`.
5. It finds the shortest call path from each function in the changed scopes to each privileged call.

For Go, Vigilante runs Google's Capslock. See [powers.md](powers.md#go-capslock).

## Lockfiles, registries, and checksums

| Language | What Vigilante reads | Where packages come from | Checksum |
| --- | --- | --- | --- |
| JavaScript and TypeScript | `package-lock.json`, `npm-shrinkwrap.json`, `pnpm-lock.yaml`, `yarn.lock` | registry.npmjs.org, registry.yarnpkg.com, GitHub | `integrity` in the lockfile |
| Python | `uv.lock`, `poetry.lock`, `Pipfile.lock`, `requirements.txt` with exact versions | PyPI, GitHub | hashes in the lockfile, else the PyPI JSON API |
| Java | `pom.xml` (Vigilante's own resolver), `gradle.lockfile` | Maven Central, Google Maven | `.sha1` file next to each jar |
| Rust | `Cargo.lock` | crates.io, GitHub | `checksum` in `Cargo.lock` |
| C# | `packages.lock.json`, or `PackageReference` (Vigilante's own resolver) | nuget.org | `contentHash` in the lockfile, else the nuget.org catalog |
| Ruby | `Gemfile.lock` | rubygems.org, GitHub | `CHECKSUMS` in `Gemfile.lock`, else the rubygems.org API |

- Java: Vigilante resolves a POM like Maven: parents, properties, `dependencyManagement`, BOM imports, scopes, `optional`, exclusions, and the nearest version. It reads compiled classes (bytecode).
- C#: without `packages.lock.json`, Vigilante resolves `PackageReference` items with central versions (`Directory.Packages.props`), properties, the lowest version of each range, and the nearest version. It reads compiled assemblies (IL).
- Gradle without `gradle.lockfile`: Vigilante reads only the code in the repository and shows a line. It does not run Gradle.
- Code that no lockfile covers: Vigilante reads only the code in the repository and shows a line.
- Path and workspace dependencies are read from the repository.
- yarn 2 and later: the checksum in `yarn.lock` covers yarn's own zip file, so Vigilante cannot check the download against it.

## Network use and the cache

Vigilante downloads packages only from the public registries above and from the registries in `[registries] allow`. It never sends credentials.

Downloads go to a cache folder, so a second run reads them from disk:

- `$XDG_CACHE_HOME/vigilante` when `XDG_CACHE_HOME` is set,
- else `~/.cache/vigilante` on Linux and `~/Library/Caches/vigilante` on macOS.

The cache key of a package includes its address and its checksum. A large project can fill about 1 GB. One download can be at most 256 MiB; a larger package is a line in the report.

## Registries that are not public

A registry that is not in the list above is skipped with a line in the report. If a registry needs no credentials, add it to `vigilante.toml`:

```toml
[registries]
allow = ["https://npm.example.com/"]
```

Vigilante reads `vigilante.toml` from the merge base, so a pull request cannot add a registry for its own review.

## Lines in "Not analyzed"

The report lists the code and packages that Vigilante did not read, for example:

- no lockfile covers the code;
- a package is in neither the repository nor the lockfile;
- a registry is not public;
- a git source is outside GitHub;
- a package has no exact version or no checksum;
- a package is larger than 256 MiB.

The Markdown report shows 10 lines and a count. The JSON report has all of them.

## Calls resolved by type

For JavaScript and TypeScript, Vigilante resolves calls with the TypeScript 6.0.3 type checker. It downloads `typescript` 6.0.3 from npm, checks it against a pinned checksum, and runs it with Node.js. The checker reads the files and runs no code from the repository or its packages.

1. A call that the checker resolves goes only to the function that the types name, not to every function with that name.
2. A call through a function value, such as a callback, is an `unanalyzed` note, as in Capslock.
3. A call that the checker cannot resolve uses the name-based rules.
4. Findings are the union of the typed and the name-based results, so type data never removes a finding.

If Node.js is not on `PATH`, or the download fails, Vigilante resolves calls by name and lists the scope under "Not analyzed".

## Choose an engine

Set a language to an engine in `vigilante.toml`:

```toml
[engines]
javascript = "deep"
ruby = "patterns"
```

- `"typed"` resolves calls by type, then by name. It is the default for `javascript`.
- `"deep"` resolves calls by name and import. It is the default for `python`, `java`, `rust`, `csharp`, and `ruby`.
- `"patterns"` uses only the direct-use rules in `packs/`.

## Known limits

- Without type data, a call on an object of unknown type, or through an interface or a trait, goes to every function with that name in the packages that the file imports. This can add packages to the notes, like Capslock's interface calls.
- Calls that are built at run time are not followed: reflection, `eval`, dynamic imports, and Ruby metaprogramming such as `send` and `define_method`.
- Rust: calls inside macro arguments are not read.
- Java and C#: inherited methods and Kotlin, F#, and Visual Basic code are not followed.
- Ruby: constants resolve by the Zeitwerk file convention (`Acme::Tools::Shell` is in `acme/tools/shell.rb`).
- Private registries and feeds that need credentials are skipped.
