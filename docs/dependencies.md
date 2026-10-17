# Dependencies and call paths

Vigilante follows calls from your code into the packages that your code uses. It does this for Go, JavaScript and TypeScript, Python, Java, Rust, C#, and Ruby. Each result has a call path, for example `services/billing/billing/export.py:5 via export.export → subprocess.run`.

By default, Vigilante does not run code from the repository or from a package. It does not install packages, run install scripts, or run a build. The only exception is `[engines] rust = "typed"`. See [Calls resolved by type](#calls-resolved-by-type).

## Findings and notes

The rule is the same for each language:

1. A finding is `exec`, `network`, or `native` that your own code reaches. The call path stays in the scope until the privileged call. A finding must be in `vigilante.lock`.
2. All other results are notes:
   - another power that your own code reaches, for example `dynamic_code` or `modify_system_state`;
   - a power that your code reaches only through a package, or through another scope of the repository (that scope shows its own finding);
   - a package that is new on a call path to a power.
3. Notes are in the section "New packages on call paths". They do not go into the lock. A note makes the check fail only when a `deny` rule matches its power. For the powers in `report.hide`, such as `files`, the Markdown report shows only a count. The JSON report has all the notes.

A new dependency that reaches the network is a note that names the dependency. A finding appears only the first time that its power appears in its scope.

## How it works

1. Vigilante reads the lockfile of the package or project that has the changed code.
2. It downloads each locked package that your code imports, from the public registry of that package.
3. It compares each package with the checksum in the lockfile or from the registry. If they are different, it stops with exit code 2.
4. It reads your code and the package code, and makes a call graph of functions, imports, and calls.
5. The rules in `packs/` mark the privileged calls, for example `child_process.exec`, `subprocess.run`, `ProcessBuilder`, `std::process::Command`, `Process.Start`, and `system`.
6. It finds the shortest call path from each function in the changed scopes to each privileged call.

For Go, Vigilante runs Capslock. See [powers.md](powers.md#go-capslock).

## Lockfiles, registries, and checksums

| Language | What Vigilante reads | Where packages come from | Checksum |
| --- | --- | --- | --- |
| JavaScript and TypeScript | `package-lock.json`, `npm-shrinkwrap.json`, `pnpm-lock.yaml`, `yarn.lock` | registry.npmjs.org, registry.yarnpkg.com, GitHub | `integrity` in the lockfile |
| Python | `uv.lock`, `poetry.lock`, `Pipfile.lock`, `requirements.txt` with exact versions | PyPI, GitHub | hashes in the lockfile, else the PyPI JSON API |
| Java | `pom.xml` (Vigilante's own resolver), `gradle.lockfile` | Maven Central, Google Maven | `.sha1` file next to each jar |
| Rust | `Cargo.lock` | crates.io, GitHub | `checksum` in `Cargo.lock` |
| C# | `packages.lock.json`, or `PackageReference` (Vigilante's own resolver) | nuget.org | `contentHash` in the lockfile, else the nuget.org catalog |
| Ruby | `Gemfile.lock` | rubygems.org, GitHub | `CHECKSUMS` in `Gemfile.lock`, else the rubygems.org API |

- Java: Vigilante resolves a POM as Maven does. It reads parents, properties, `dependencyManagement`, BOM imports, scopes, `optional`, and exclusions, and it uses the nearest version. It reads compiled classes (bytecode).
- C#: without `packages.lock.json`, Vigilante resolves `PackageReference` items. It reads central versions (`Directory.Packages.props`) and properties. It uses the lowest version of each range and the nearest version. It reads compiled assemblies (IL).
- Gradle without `gradle.lockfile`: Vigilante reads only the code in the repository and shows a line. It does not run Gradle.
- Code that no lockfile covers: Vigilante reads only the code in the repository and shows a line.
- Vigilante reads path dependencies and workspace dependencies from the repository.
- yarn 2 and later: the checksum in `yarn.lock` is for the zip file of yarn. So Vigilante cannot compare the download with it.

## Network use and the cache

Vigilante downloads packages only from the public registries above and from the registries in `[registries] allow`. It never sends credentials.

Vigilante keeps the downloads in a cache folder, so a second run reads them from the disk:

- `$XDG_CACHE_HOME/vigilante` if `XDG_CACHE_HOME` has a value;
- if not, `~/.cache/vigilante` on Linux and `~/Library/Caches/vigilante` on macOS.

The cache key of a package has its address and its checksum. A large project can use about 1 GB. One download can be at most 256 MiB. For a larger package, the report shows a line.

## Registries that are not public

Vigilante skips a registry that is not in the list above, and it shows a line in the report. If a registry needs no credentials, add it to `vigilante.toml`:

```toml
[registries]
allow = ["https://npm.example.com/"]
```

Vigilante reads `vigilante.toml` from the merge base. So a pull request cannot add a registry for its own review.

## Lines in "Not analyzed"

The report lists the code and the packages that Vigilante did not read. Some reasons are:

- no lockfile covers the code;
- a package is not in the repository and not in the lockfile;
- a registry is not public;
- a git source is not on GitHub;
- a package has no exact version or no checksum;
- a package is larger than 256 MiB;
- a type check did not complete.

The Markdown report shows 10 lines and a count. The JSON report has all of them.

## Calls resolved by type

Vigilante resolves calls by type with two tools. It downloads each tool from npm, compares it with a pinned checksum, and runs it with Node.js. These tools do not run code from the repository or from its packages.

- **JavaScript and TypeScript:** the TypeScript 6.0.3 type checker resolves the calls in your code. It reads package files only as far as your code imports them. Calls in package code use the name-based rules.
- **Python:** Pyright, through `@sourcegraph/scip-python` 0.6.6. Vigilante copies the code and the locked packages that it read into temporary folders. Then it runs the tool without the Python of the machine, so the packages on the machine do not change the result. Pyright resolves the calls in your code, also the calls into packages. Calls in package code use the name-based rules.
- **Rust, when `[engines] rust = "typed"`:** rust-analyzer in a Docker container (`rust:1.99-slim`, pinned by digest). Vigilante copies the workspace into the container. First, with network access, the container installs rust-analyzer and runs `cargo fetch`, which does not run project code. Then Vigilante disconnects the network, and the container runs `rust-analyzer scip`. This step runs `build.rs` and proc macros. The source is read-only, and only the folder for the index is writable.

Java and C# need no build. Vigilante reads the base class and the interfaces of each class. It reads them from source, and from the class files and assemblies of the packages. A call on a base class or an interface also goes to the methods that override it in the loaded classes.

These rules apply to the typed results:

1. A call that the checker resolves goes only to the function that its types name. It does not go to each function with that name.
2. A call through a function value, such as a callback, is an `unanalyzed` note, as in Capslock.
3. A call that the checker cannot resolve uses the name-based rules.
4. A finding can come from the typed result or from the name-based result. So type data never removes a finding.

Vigilante resolves calls by name and lists the scope under "Not analyzed" in these conditions:

- Node.js or Docker is not available.
- A download or a build fails.
- A check reaches `[engines] type_check_seconds`. The default is 300 seconds.

## Choose an engine

To set the engine of a language, use `vigilante.toml`:

```toml
[engines]
javascript = "deep"
ruby = "patterns"
```

- `"typed"` resolves calls by type, then by name. It is the default for `javascript` and `python`.
- `"deep"` resolves calls by name and import. It is the default for `java`, `rust`, `csharp`, and `ruby`.
- `"patterns"` uses only the direct-use rules in `packs/`.

`rust = "typed"` is not the default, because it runs build code. It needs Docker on a Linux runner.

## Known limits

- Without type data, a call on an object of unknown type goes to each function with that name in the packages that the file imports. A call through an interface or a trait does the same. This can add packages to the notes, as the interface calls of Capslock do.
- Vigilante does not follow calls that the code makes at run time: reflection, `eval`, dynamic imports, and Ruby metaprogramming such as `send` and `define_method`.
- Rust: Vigilante does not read calls inside macro arguments.
- Java and C#: Vigilante does not look up an inherited method in the base class.
- Vigilante does not read Kotlin, F#, or Visual Basic code.
- Ruby: constants resolve by the Zeitwerk file rule (`Acme::Tools::Shell` is in `acme/tools/shell.rb`).
- Vigilante skips private registries and feeds that need credentials.
