# 008: Risks

## Problem

Vigilante reported a power only the first time that it appeared in a scope. A package that already ran a fixed command could add a shell command with web request input, and the check passed. The power did not change, but the risk did.

## Decision

1. A risk is one call to a sink in the code of the repository, with the class of its input. The sink has the power `exec`, `network`, `dynamic_code`, `native`, or `system_calls`.
2. The input classes are `constant`, `dynamic`, `shell`, and `untrusted`.
   - `untrusted`: the input comes from a web request, a serverless event, or the URL of a browser page. The input can also come from such a source through the parameters of other functions of the repository.
   - `shell`: the call runs a shell, and the command is not constant.
   - `dynamic`: the input changes at run time.
   - `constant`: the input is a literal.
3. The identity of a risk is `power:function:sink:class`, for example `exec:app.export:subprocess.run:untrusted`. The identity has no line number. So a risk that only moves does not change.
4. A diff shows each risk that the head has and the merge base does not have. It also shows a risk when the scope already has the power. Constant risks are not shown.
5. Two classes block: `untrusted` input to `exec`, `dynamic_code`, or `network`, and `shell` with `exec`. `vigilante lock` writes the identity of each blocking risk to the `risks` list of its scope. An approved risk does not block. CODEOWNERS on `vigilante.lock` then decides who approves it.
6. Vigilante treats command line arguments and standard input as local input, because the person who starts the program can already run programs.

## Consequences

- Risks come from the call graph engine. Go uses Capslock, which has no argument facts, so Go has no risks.
- The engine finds input by text, in the arguments of the call and in local assignments up to three steps back. It does not follow data through fields, return values, or package code.
- A change to the name of a function changes the identity of its risks. `vigilante lock` then writes the new identity.
- `vigilante check` fails when the lock approves a risk that the repository does not have now.

## Verification

The tests in `tests/integration/risks.rs`, `risk_classes.rs`, and `risk_reports.rs` cover each class, the approval, a stale approval, and the reports.
