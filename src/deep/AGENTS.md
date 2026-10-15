# The call graph engine

`src/deep/` follows calls from the code of a repository into its locked packages. It finds the shortest path to each privileged call, and sorts each path into a finding or a note.

## Parts

- `program/`: loads the files, parses them, links calls to definitions, and runs the typed resolver.
- `graph.rs` and `search.rs`: the call graph and the shortest path to each sink.
- `classify.rs`: the rule for findings and notes.

Each language in `src/<language>/` gives a `Language` (the grammar and the index of a file) and a `Resolver` (imports and package files). A language can also give a typed resolver, which maps a call site to its target definitions.

## Rules

1. A finding is `exec`, `network`, or `native` that a package reaches through its own code. Everything else is a note.
2. Typed results can remove a false path from the notes. They never remove a finding: findings come from the typed and the name-based results together.
3. A typed resolver that is missing, fails, or reaches `type_check_seconds` gives a "Not analyzed" line, and the engine resolves calls by name.
4. The engine reads package files. It never runs an install script or package code.
