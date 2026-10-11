# Report contract

Create UTF-8 JSON outside the public checkout. The builder reads only pinned Git objects and local review results.
It refuses an existing output file. It collects all per-PR diffs and verifies every quoted excerpt.

## Input

Use these fields:

```json
{
  "title": "Review the job retry",
  "summary": "Follow the result and notification through failures.",
  "pulls": [
    {
      "id": "pr-1",
      "repository": "example/project",
      "number": 1,
      "url": "<PR HTTPS URL>",
      "title": "Retry the job",
      "checkout": "<private local checkout>",
      "baseSha": "<full SHA>",
      "headSha": "<full SHA>",
      "reviewFile": "<private local shadowclone-review JSON>",
      "ci": []
    }
  ],
  "evidence": [
    {
      "id": "retry-boundary",
      "pr": "pr-1",
      "path": "src/job.ts",
      "side": "HEAD",
      "start": 12,
      "end": 14,
      "quote": "<exact committed source>"
    }
  ],
  "groups": [
    {
      "id": "job",
      "title": "Follow the retry boundary",
      "prs": ["pr-1"],
      "summary": "The callback now repeats after a failure.",
      "why": "A retry can repeat completed writes.",
      "before": "One attempt.",
      "after": "Up to three attempts.",
      "files": [{ "pr": "pr-1", "path": "src/job.ts" }],
      "evidence": ["retry-boundary"],
      "steps": [
        {
          "title": "Read the callback",
          "text": "<plain Markdown explanation>",
          "evidence": ["retry-boundary"],
          "definitions": ["retry-boundary"],
          "files": [{ "pr": "pr-1", "path": "src/job.ts" }]
        }
      ],
      "diagram": { "nodes": [], "edges": [] },
      "dryRuns": []
    }
  ]
}
```

Local reviewer JSON contains pull.repository, pull.baseSha, pull.headSha, findings, and skippedPaths.
The stock CLI produces those fields. A host-agent review records the same pins and its actual investigation output.
Findings retain the stock path, line, severity, explanation, failureScenario, evidence, suggestion, and refutation fields.
Do not copy benchmark findings into another PR. Do not rewrite their claims while building the document.

CI entries use name and conclusion. They become short status counts, not a badge wall.
Markdown explanations escape raw HTML. The actual Markdown diff and rendered base and head remain inspectable.

Order passages by behavior. Define the main contract before showing its callers, effects, and supporting changes.
steps.definitions lists evidence identifiers for signature excerpts shown beside the explanation.
steps.files orders the actual diffs after that passage. Each file must belong to the group's files list.
Without steps.files, the builder follows changed files in steps.evidence order.
The group's remaining files follow its main passages. Every file starts expanded and links to its exact PR diff.

## Complete coverage

Every changed PR/file pair must appear in groups.files. Repeating a file across related groups does not inflate totals.
The builder rejects missing files for a complete guide. Generated artifacts and tests need an explained supporting group.
If you did not examine a file, say so in that group's explanation. Listing its diff does not establish investigation.
Use status=prototype only for a clearly labeled early UX check. Never present that artifact as the complete review.

The builder derives each PR's merge base. It derives a combined head only when one supplied revision contains every requested head.
Combined added lines use Git blame and the PR commit ranges for ownership.
If removed text occurs in several PRs, its label names those possible owners. Inspect the per-PR patches before staging.
Per-PR views remain authoritative for exact reviewer-visible anchors.

## Visuals and dry runs

A diagram node has id, label, and evidence identifiers.
An edge has from, to, label, and evidence identifiers.
Every node and transition needs committed evidence. Keep labels short; put citations behind the source disclosure.

A dry run has title, kind (reasoned or executed), input, steps, and outcome.
Each step has condition, state, effect, outcome, and evidence identifiers.
An executed run also explains its command, fixture boundary, and observed output in the surrounding text.
Never promote a source trace into an executed result.

## Saved annotations

The output embeds one application/json script named review-data, with version=2 and a stable document id.
It contains the pinned facts, local findings, route, code evidence, and annotations.
Read this data block without executing the HTML's scripts when staging.

annotations contains notes, decisions, reviewed group identifiers, and a revision number.
A human note contains id, pr, anchor, quote, ranges, body, originalBody, and dismissed.
A source range contains path, side, start, end, and sha. Mixed diff selections can contain several source ranges.
Prose notes have an empty ranges array. The staging workflow resolves them to current changed lines.
decisions maps finding identifiers to included, dismissed, or null.
Preserve removed notes and dismissed findings as inactive records. They must not reappear in a staged review.

Save review requests a handle to the original HTML. It checks document identity and the prior annotation state before writing.
It updates the embedded JSON only and preserves the document's remaining text.
The original file, not browser storage or a separate JSON export, carries the portable review.
