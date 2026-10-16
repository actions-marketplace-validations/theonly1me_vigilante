# Benchmark

The corpus in `tests/corpus/` has pairs of changes for each language and each surface. Each pair has a base, an unsafe change, and a safe change. The test `tests/integration/corpus.rs` runs a diff from the base to each change, and checks two rules:

- An unsafe change blocks. If it does not block, the case gives the reason, for example a known limit.
- A safe change passes, and the report shows nothing.

So a change to Vigilante that misses an unsafe change, or that blocks a safe change, fails the test.

## Results

These results come from a debug build on macOS (arm64). The slowest review includes the time to make the test repository.

| Group | Pairs | Unsafe changes caught | Unsafe changes that block | Safe changes that block | Safe changes shown | Slowest review |
| --- | --- | --- | --- | --- | --- | --- |
| agent | 4 | 4 | 4 | 0 | 0 | 328 ms |
| build | 3 | 3 | 3 | 0 | 0 | 303 ms |
| ci | 5 | 5 | 4 | 0 | 0 | 340 ms |
| csharp | 2 | 2 | 2 | 0 | 0 | 347 ms |
| go | 5 | 4 | 4 | 0 | 0 | 3279 ms |
| java | 2 | 2 | 2 | 0 | 0 | 329 ms |
| javascript | 5 | 5 | 5 | 0 | 0 | 379 ms |
| python | 7 | 6 | 6 | 0 | 0 | 347 ms |
| ruby | 2 | 2 | 2 | 0 | 0 | 328 ms |
| rust | 2 | 2 | 1 | 0 | 0 | 375 ms |
| **All** | **37** | **35** | **33** | **0** | **0** | |

Two unsafe changes are caught but do not block, by design. A command that becomes dynamic and an action pin that becomes a branch are shown as risks. See [risks.md](risks.md).

Two unsafe changes are not caught. These are known limits:

- Python: request input that an object field carries to a command. The engine does not follow data through fields.
- Go: request input that reaches a command in a package that already runs commands. Capslock gives no facts about arguments, so Go has no risks.

## Go and capslock-git-diff

For the 5 Go pairs, the table compares Vigilante with `capslock-git-diff` v0.3.3. `capslock-git-diff` exits with code 1 when a package gets a new capability.

| Pair | Vigilante, unsafe | `capslock-git-diff`, unsafe | Vigilante, safe | `capslock-git-diff`, safe |
| --- | --- | --- | --- | --- |
| First exec in a package | blocks | reports | passes | passes |
| First network call in a package | blocks | reports | passes | passes |
| An import alias for `os/exec` | blocks | reports | passes | passes |
| An unsafe pointer conversion | blocks | reports | passes | passes |
| Input that reaches a command in a package that already runs commands | passes | passes | passes | passes |

The two tools agree on each pair. For the other languages and for CI, agent, and build files, Capslock has no results.

## Run the corpus

1. To run the corpus for all groups except Go, run this command:

   ```sh
   cargo test --locked --test integration corpus -- --nocapture
   ```

2. To also run the Go pairs, put `go` and `capslock` on `PATH`, and add `--include-ignored`.

The test prints the results table.

## Add a pair

Add a `[[case]]` to `tests/corpus/code.toml`, `surfaces.toml`, or `go.toml`:

| Key | Value |
| --- | --- |
| `name` | The group, a colon, and the change, for example `python: eval of request input`. |
| `group` | The language or the surface. |
| `base`, `unsafe`, `safe` | The files of each commit, as a table from the path to the text. `unsafe` and `safe` change the files of `base`. |
| `unsafe_result` | `"blocks"` (the default), `"shows"`, or `"passes"`. |
| `reason` | Why the unsafe change does not block. It is necessary when `unsafe_result` is not `"blocks"`. |
