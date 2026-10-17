# 010: Approval guarantees

## Problem

A pull request can add a power and its lock entry together, and the check passes. That is the design: the lock records the approval. But Vigilante did not show that someone must approve the lock change. It also did not check that GitHub enforces the review. Without `CODEOWNERS` and a branch rule, the author approves their own powers.

The result of a review was the last line of the report. The report was only in the job log and in the job summary.

## Decision

1. The result is the first line after the title. In a diff, the result also counts the new powers and risks that are in the lock. It tells the reviewer that a code owner of `vigilante.lock` must approve the pull request.
2. `vigilante doctor` checks the setup:
   - `CODEOWNERS` gives an owner to `vigilante.lock`, `vigilante.toml`, `.vigilante/`, `.github/workflows/`, and the `CODEOWNERS` file. The last rule that matches a path decides, as on GitHub.
   - The default branch requires a pull request, a review from code owners, and a check with `vigilante` in its name. Vigilante reads the rulesets and the classic branch protection through `gh`.
   - If `gh` is missing or fails, the check fails and names the reason.
3. The Action can write the report as one comment on the pull request. The input `comment` is off by default, because it needs `pull-requests: write`. Only the comment step gets the token. If GitHub does not let the step write, it writes a warning and the job result does not change.

## Consequences

- `vigilante doctor` reads the working directory, not the default branch, for `CODEOWNERS`.
- Classic branch protection needs a token with admin access. Rulesets do not.
- The comment step posts a new comment only when the report has something to show. It updates its earlier comment on each run, so a fixed pull request shows the pass.

## Verification

The tests in `tests/integration/report_summary.rs` and `doctor.rs`, and the scripts `action/test.sh` and `action/test-comment.sh`, cover each part.
