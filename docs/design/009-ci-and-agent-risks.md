# 009: CI and agent risks

## Problem

The CI and agent readers looked for a power only. They did not see these changes:

- A `run:` step that puts the title of a pull request into a shell script. The workflow had the same powers, but an author of a pull request could run code in it.
- A workflow on `pull_request_target` that checks out the head of the pull request, and then runs its code with secrets.
- An action pin that changes from a commit SHA to `@main`.
- `secrets['NAME']`, `toJSON(secrets)`, and composite actions that run `curl ... | sh`.
- `allowed-tools` and `hooks` in the frontmatter of a skill, `Bash(node -e *)`, and a deny rule that a change removes.
- An install script that runs a program that Vigilante does not know.

## Decision

1. The surface readers give risks, in the same model as code risks ([008](008-risks.md)). A CI risk has the scope of its file. A pin risk has the scope `.github/workflows`, the same as the action.
2. Two CI risks block. The first is text from an untrusted source in a script. The second is a checkout of the head of the pull request in a workflow that runs on `pull_request_target`, `workflow_run`, or `issue_comment`. `vigilante lock` approves them.
3. A pin risk does not block. A diff shows it when the pin changes from a commit SHA to a branch or a tag. Its identity has no tag, so a tag update is not a change.
4. A download that a shell or an interpreter runs is the power `remote_script`, with the host as the detail. A download that a program reads as data is not.
5. A deny rule for the same tool is part of the detail of an allow rule. So a change to the deny rules changes the power.
6. In `package.json`, an install script that runs a program that Vigilante does not know is a power. Pre-commit hooks keep the old rule, because they run commands of the repository.

## Consequences

- A workflow that has untrusted text in a script today fails `vigilante check` until the lock approves it. The fix is an `env:` variable.
- An install script such as `prepare: vite build` is now a power.
- The frontmatter reader reads only `allowed-tools` and `hooks`. Other keys can have text that is not valid YAML.

## Verification

The tests in `tests/integration/ci_risks.rs`, `ci_scripts.rs`, and `surface_powers.rs` cover each risk, each power, and the inputs that are not risks.
