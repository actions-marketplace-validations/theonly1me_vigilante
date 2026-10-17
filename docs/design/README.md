# Design records

These records give the decisions behind the current behavior, and their tradeoffs. If a new decision replaces an old one, update the old record. For the current behavior, read the [architecture](../architecture.md) and the [guides](../README.md).

To start a record, copy the [template](template.md). Keep only the sections that a reader needs.

| Record | Decision |
| --- | --- |
| [001: Powers, scopes, and the lock](001-powers-and-the-lock.md) | One model for powers on all surfaces, a sorted lock, and the config from the merge base |
| [002: Pattern rules](002-pattern-rules.md) | One ast-grep rule file for each language, with test cases and rules that cut noise |
| [003: Go with Capslock](003-go-with-capslock.md) | Run Capslock, and block only own-code `exec`, `network`, and `native` |
| [004: Call graphs for packages](004-call-graphs-for-packages.md) | One shared engine that reads locked packages with checksums and runs no package code |
| [005: Typed call resolution](005-typed-call-resolution.md) | Type checkers and a contained Rust build, with the union rule and a time limit |
| [006: Release and distribution](006-release-and-distribution.md) | One build for six channels, Release Please, attestations, and trusted publishing |
| [007: Repository guardrails](007-repository-guardrails.md) | CI checks, the repository test, the ruleset, and the agent files |
| [008: Risks](008-risks.md) | Report each changed call with untrusted input or a shell, also when the scope already has the power |
| [009: CI and agent risks](009-ci-and-agent-risks.md) | Script injection, checkout of the pull request head, action pins, downloaded scripts, skills, and deny rules |
| [010: Approval guarantees](010-approval-guarantees.md) | The result first, `vigilante doctor`, and an opt-in pull request comment |
