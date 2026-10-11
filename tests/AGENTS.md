# Tests

`tests/integration/` holds the end-to-end tests. Each test makes a temporary git repository, runs the `vigilante` binary or the library, and checks the report.

## Rules

- Test the result that a user sees: the report lines, the exit code, or the lock file.
- Use the fake registries in `tests/integration/`, such as `npm_registry.rs`, `python_index.rs`, and `file_server.rs`. A test must not use a public registry, unless it has the mark `#[ignore = "needs network"]`.
- Mark a test that needs a tool with `#[ignore = "..."]`, and name the tool: `needs node`, `needs docker`, `needs network`, or `needs go and capslock`. Start the name of a Docker test with `docker_`, because macOS CI skips those tests.
- Keep each test file under 200 lines. Put shared fixtures in a `*_fixtures.rs` file.
- Name each test for the behavior that it guards.
- Prove each new test. Change the code that it guards, and make sure that the test fails for the reason that you expect. Then restore the code.

## Fixtures

`tests/fixtures/` holds binary fixtures, such as a .NET assembly. Each binary fixture has its source and a build script next to it, and the build gives the same bytes each time.
