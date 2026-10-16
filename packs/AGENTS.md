# Rule packs

Each file in `packs/` holds the ast-grep rules for one language. A rule finds a privileged call or import in a file. `metadata.capability` names the power. The deep engines also use these matches as the sinks of the call graph.

## Add or change a rule

1. Write the test cases first in `packs/tests/<language>.yml`, with the same `id` as the rule.
2. Put code that must match under `invalid`. Put code that must not match under `valid`. This is the ast-grep test format.
3. Add traps to `valid`: the same text in a string, in a comment, and in a name that only looks similar.
4. Write the rule in `packs/<language>.yml`. Use `files` and `ignores` to skip generated files.
5. Run `cargo test --locked --test integration packs`. Each rule must match each `invalid` case and no `valid` case.
6. Remove the new rule, and make sure that its `invalid` case fails. Then put the rule back.

## Rules

- Use one capability for each rule.
- Do not add a rule for `files` or `read_system_state` that matches common code. The report hides these powers, but the lock records them.
- If a call is a power only through a package, put the package in an import rule. The engine then treats calls through that import as sinks.
- If some members of such a package do not have the power, list them in `metadata.inert`. For example, `subprocess.list2cmdline` only formats text.
