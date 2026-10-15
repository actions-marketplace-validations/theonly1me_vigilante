# 005: Typed call resolution

## Problem

The engine in [004](004-call-graphs-for-packages.md) resolves a call by its name and imports. A call through an interface, a trait, or an object of unknown type goes to each method with that name. This adds false paths to the notes and misses calls that only types can follow.

## Decision

1. A language can give a typed resolver. It maps a call site to its target definitions. The linker uses it before the name-based rules. A call through a function value becomes an `unanalyzed` note.
2. A finding comes from the typed result or from the name-based result. So type data, a failed build, or an `as any` cannot remove a finding.
3. The resolvers:
   - JavaScript and TypeScript: the TypeScript 6.0.3 checker, on the host, with Node.js.
   - Python: Pyright through scip-python 0.6.6, on the host, without the Python of the machine.
   - Rust: rust-analyzer in Docker, only with `[engines] rust = "typed"`.
   - Java and C#: the class hierarchy from source, class files, and assemblies. No build runs.
4. Vigilante downloads each tool with a pinned checksum, and pins the Docker image by digest.
5. The Rust build runs in one container. It downloads crates with network access. Then Vigilante disconnects the network, and the container runs `rust-analyzer scip`. The source is read-only, and only the output folder is writable.
6. Each check has a time limit, `[engines] type_check_seconds`, with a default of 300. If a tool is missing, fails, or reaches the limit, Vigilante stops it, resolves calls by name, and shows a line under "Not analyzed".

## Consequences

- The typed engines need Node.js, and the Rust build needs Docker. GitHub-hosted Linux runners have both.
- A large TypeScript project can reach the time limit. Then the report is the same as with the name-based engine.
- Calls in package code use the name-based rules.
- Java and C# do not look up an inherited method in the base class.

## Verification

- A test shows that a typed call does not reach a package method with the same name.
- A test shows that a call through a callback is an `unanalyzed` note.
- A container test shows that the build step has no network.
- Tests with a slow fake `node` show that each check stops at the limit and falls back to names.
