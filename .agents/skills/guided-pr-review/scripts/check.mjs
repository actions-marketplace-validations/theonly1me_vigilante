import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";

function requireText(value, label) {
  if (typeof value !== "string" || !value.trim()) throw new Error(label + " must contain text.");
}

function unique(items, label) {
  const values = new Set();
  for (const item of items) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(item.id) || values.has(item.id))
      throw new Error("Invalid or duplicate " + label + " id.");
    values.add(item.id);
  }
  return values;
}

export function checkInput(input) {
  requireText(input.title, "Review title");
  if (
    !Array.isArray(input.pulls) ||
    !input.pulls.length ||
    !Array.isArray(input.groups) ||
    !input.groups.length ||
    !Array.isArray(input.evidence)
  )
    throw new Error("The guide needs PRs, reading groups, and evidence.");
  const pullIds = unique(input.pulls, "PR");
  const evidenceIds = unique(input.evidence, "evidence");
  unique(input.groups, "group");
  for (const pull of input.pulls) {
    if (![pull.baseSha, pull.headSha].every((sha) => /^[a-f0-9]{40}$/.test(sha)))
      throw new Error("Pin each PR's full base and head SHA.");
    if (
      !/^[\w.-]+\/[\w.-]+$/.test(pull.repository) ||
      !Number.isInteger(pull.number) ||
      pull.number < 1
    )
      throw new Error("Each PR needs its repository and number.");
    const url = new URL(pull.url);
    if (
      url.protocol !== "https:" ||
      url.pathname !== "/" + pull.repository + "/pull/" + pull.number
    )
      throw new Error("The PR URL does not match its scope.");
  }
  for (const evidence of input.evidence) {
    if (
      !pullIds.has(evidence.pr) ||
      !["BASE", "HEAD"].includes(evidence.side) ||
      !Number.isInteger(evidence.start) ||
      !Number.isInteger(evidence.end) ||
      evidence.start < 1 ||
      evidence.end < evidence.start ||
      evidence.end - evidence.start > 100
    )
      throw new Error("Evidence needs a PR, side, and bounded source range.");
    if (
      typeof evidence.path !== "string" ||
      evidence.path.startsWith("/") ||
      evidence.path.split("/").some((part) => part === ".." || part === ".") ||
      evidence.path.includes("\0")
    )
      throw new Error("Evidence paths must stay inside the repository.");
    requireText(evidence.quote, "Evidence quote");
  }
  for (const group of input.groups) {
    requireText(group.title, "Reading step");
    requireText(group.summary, "Step explanation");
    if (!Array.isArray(group.files) || !Array.isArray(group.steps))
      throw new Error("Each reading step needs files and passages.");
    for (const file of group.files)
      if (!pullIds.has(file.pr)) throw new Error("A file refers to another PR.");
    for (const file of group.steps.flatMap((step) => step.files ?? []))
      if (!group.files.some((entry) => entry.pr === file.pr && entry.path === file.path))
        throw new Error("A passage file must belong to its reading group.");
    const references = [
      ...(group.evidence ?? []),
      ...group.steps.flatMap((step) => [...(step.evidence ?? []), ...(step.definitions ?? [])]),
      ...(group.diagram?.nodes ?? []).flatMap((node) => node.evidence ?? []),
      ...(group.diagram?.edges ?? []).flatMap((edge) => edge.evidence ?? []),
      ...(group.dryRuns ?? []).flatMap((run) => run.steps.flatMap((step) => step.evidence ?? [])),
    ];
    if (references.some((id) => !evidenceIds.has(id)))
      throw new Error("A passage or visual refers to missing evidence.");
    for (const node of group.diagram?.nodes ?? [])
      if (!node.evidence?.length) throw new Error("Every diagram node needs code evidence.");
    for (const edge of group.diagram?.edges ?? [])
      if (!edge.evidence?.length) throw new Error("Every diagram transition needs code evidence.");
  }
}

export function checkCoverage(review) {
  const files = new Set(review.pulls.flatMap((pull) => pull.files.map((file) => file.id)));
  const covered = new Set(
    review.groups.flatMap((group) => group.files.map((file) => file.pr + ":" + file.path)),
  );
  for (const key of covered)
    if (!files.has(key)) throw new Error("A reading step refers to an unchanged file: " + key);
  const missing = [...files].filter((key) => !covered.has(key));
  if (missing.length && review.status !== "prototype")
    throw new Error(
      "The reading route omits " + missing.length + " changed files:\n" + missing.join("\n"),
    );
  return { total: files.size, covered: files.size - missing.length, missing };
}

export function readReviewData(html) {
  const match = /<script type="application\/json" id="review-data">([\s\S]*?)<\/script>/.exec(html);
  if (!match) throw new Error("The HTML has no embedded review data.");
  const review = JSON.parse(match[1]);
  if (review.version !== 2 || !/^[a-f0-9]{64}$/.test(review.id))
    throw new Error("The review format or document identity is invalid.");
  const ids = new Set(review.pulls.map((pull) => pull.id));
  if (!Array.isArray(review.annotations?.notes))
    throw new Error("The review annotations are invalid.");
  for (const note of review.annotations.notes) {
    if (
      !ids.has(note.pr) ||
      typeof note.id !== "string" ||
      typeof note.body !== "string" ||
      typeof note.quote !== "string" ||
      typeof note.anchor !== "string" ||
      !Array.isArray(note.ranges)
    )
      throw new Error("An annotation has invalid text or source references.");
  }
  checkCoverage(review);
  return review;
}

if (process.argv[1] && import.meta.url === new URL(process.argv[1], "file:").href) {
  try {
    const html = await readFile(process.argv[2], "utf8");
    const review = readReviewData(html);
    if (/<script[^>]+src=|<link[^>]+href=https?:/i.test(html))
      throw new Error("The review contains remote runtime assets.");
    const coverage = checkCoverage(review);
    console.log(
      "check: " +
        review.pulls.length +
        " PRs, " +
        coverage.covered +
        "/" +
        coverage.total +
        " files in the route, " +
        review.annotations.notes.length +
        " human notes",
    );
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

export function ancestor(options) {
  const result = spawnSync("git", [
    "-C",
    options.checkout,
    "merge-base",
    "--is-ancestor",
    options.base,
    options.head,
  ]);
  if (result.status !== 0 && result.status !== 1)
    throw new Error("Git ancestry could not be established.");
  return result.status === 0;
}

export function patchRows(patch) {
  const rows = [];
  let oldLine = 0;
  let newLine = 0;
  let active = false;
  for (const line of patch.split("\n")) {
    const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
    if (hunk) {
      oldLine = Number(hunk[1]);
      newLine = Number(hunk[2]);
      active = true;
      rows.push({ kind: "hunk", text: line, oldLine: null, newLine: null });
    } else if (active && /^[ +\-]/.test(line)) {
      const kind = line[0] === "+" ? "add" : line[0] === "-" ? "remove" : "context";
      rows.push({
        kind,
        text: line.slice(1),
        oldLine: kind === "add" ? null : oldLine++,
        newLine: kind === "remove" ? null : newLine++,
      });
    }
  }
  return rows;
}
