# 001: Powers, scopes, and the lock

## Problem

AI agents open many pull requests, and reviewers read them quickly. One small change can give new powers to code, CI, a build, or an agent. Examples are a new `subprocess` call, a CI token that can push, and a new MCP server. Each existing tool covers one language or one type of file. No tool shows all these changes in one report with one approval record.

## Decision

1. Vigilante gives one model for all powers. A power has a surface (`code`, `ci`, `agent`, or `build`), a scope, a name, and evidence with a file and a line.
2. A scope is the unit for "new". For code, it is the nearest package root. For a workflow or a config file, it is the file. Vigilante reports a power only the first time that it appears in a scope.
3. `vigilante.lock` records the approved powers in sorted order. The same input gives the same bytes. Teams give the lock to a security team in `CODEOWNERS`, so GitHub asks for that review. Vigilante does not have its own approval system.
4. In a diff, Vigilante reads `vigilante.toml` from the merge base and the lock from the head. A pull request cannot change the rules that check it, but it can add an approved power to the lock.
5. Vigilante uses no AI to find, rank, or approve powers. A pull request author controls the text and the code, and a model that reads them can receive instructions. A parser cannot.
6. Exit code 0 is a pass, 1 is a policy failure, and 2 is an error. Input that Vigilante cannot read fails the check.

## Consequences

- The report is short, because a second use of a power in the same scope is silent.
- A power that moves to a new file or a new package shows as new.
- The lock can list powers that the report hides, such as `files` and `read_system_state`.

## Verification

- `vigilante lock` gives byte-identical files on two runs.
- A pull request that adds a power that is not in the lock exits with code 1. After `vigilante lock` and a commit, it exits with code 0 and still lists the power.
- A change on `main` after the merge base does not show in the report.
