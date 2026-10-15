# 002: Pattern rules and the rules that cut noise

## Problem

Each language has its own calls and imports for each power. A rule set in code would be hard to read and hard to extend. Also, many real changes add a power that no reviewer would act on, such as an allow rule for a test command.

## Decision

1. Each language has one ast-grep rule file in `packs/`. `metadata.capability` names the power. Teams can add their own rules with `[rules] paths`.
2. Each rule has test cases in `packs/tests/`. The cases include traps: the same text in a string, in a comment, and in a similar name.
3. The engine applies the ast-grep `files` and `ignores` keys, so a rule can skip generated files such as `*.pb.go`.
4. These rules leave out changes that a reviewer would not act on:
   - An allow rule for an agent is a power only if the agent can then run any code, install packages, push, or read open content.
   - An install script or a local git hook is not a power if it is one plain local command.
   - A third-party action has one scope for all workflows, so an action that one workflow uses is not new in another.
   - `[ci] trusted_owners` makes the actions of an organization first party.
   - Vigilante does not scan tests, fixtures, or vendored code.
   - In Rust, `unsafe impl Send`, `unsafe impl Sync`, and `#[wasm_bindgen]` imports are not `native`.

## Consequences

- A rule finds direct use only. The engines in [004](004-call-graphs-for-packages.md) follow calls into packages.
- Some programs can run code through options, such as `git rebase --exec`. A narrow allow rule for such a program is not a power.

## Verification

`cargo test --locked --test integration packs` checks each rule against each case. If you remove a rule, its `invalid` cases fail.
