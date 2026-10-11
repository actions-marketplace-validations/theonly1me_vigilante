# 003: Go with Capslock

## Problem

Capslock follows Go calls through each dependency to the standard library, and `capslock-git-diff` compares two commits. A second Go engine in Vigilante would repeat that work and give different results.

## Decision

1. Vigilante runs Capslock v0.3.3 or later on each changed Go module, at the base and at the head. It uses a shared clone in a temporary folder, so the checkout of the user does not change.
2. Each Capslock capability maps to one Vigilante power. An unknown capability stops the run with exit code 2, so a new Capslock version cannot drop a power.
3. A finding is `exec`, `network`, or `native` that a Go package reaches through its own code. All other results are notes, as in `capslock-git-diff`.
4. If `go` or `capslock` is missing, or Capslock cannot load the code, Vigilante stops with exit code 2. `[engines] go = "patterns"` turns the engine off.
5. The GitHub Action installs Go and Capslock when the repository has `go.mod`.

## Consequences

- Most powers that come through library internals, such as reflection or terminal checks, are notes. They stay visible, and they do not make the check fail.
- A package that replaces a dependency with the standard library gets a new finding in its own code.
- Go code must compile, and private modules need credentials in the job.

## Verification

The Go tests compare the findings and notes of Vigilante with the output of `capslock-git-diff` on fixture modules.
