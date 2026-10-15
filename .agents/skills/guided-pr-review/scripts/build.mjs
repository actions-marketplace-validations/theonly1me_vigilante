import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { collectReview, source } from "./git.mjs";
import { checkInput, checkCoverage } from "./check.mjs";
import { renderReview } from "./render.mjs";

try {
  const [inputPath, outputPath] = process.argv.slice(2);
  if (!inputPath || !outputPath) throw new Error("Usage: build.mjs <input.json> <review.html>");
  const input = JSON.parse(await readFile(inputPath, "utf8"));
  checkInput(input);
  const { pulls, combined } = await collectReview(input);
  for (const pull of input.pulls) {
    const review = pull.reviewFile ? JSON.parse(await readFile(pull.reviewFile, "utf8")) : null;
    if (
      review &&
      (review.pull?.headSha !== pull.headSha ||
        review.pull?.baseSha !== pull.baseSha ||
        review.pull?.repository !== pull.repository)
    )
      throw new Error("The local findings do not match the pinned PR.");
    const result = pulls.find((entry) => entry.id === pull.id);
    result.findings = (review?.findings ?? []).map((finding, index) => ({
      ...finding,
      id: pull.id + "-finding-" + index,
      pr: pull.id,
    }));
    result.skippedPaths = review?.skippedPaths ?? [];
  }
  const evidence = input.evidence.map((item) => {
    const pull = input.pulls.find((pull) => pull.id === item.pr);
    const sha =
      item.side === "BASE"
        ? pulls.find((entry) => entry.id === pull.id).mergeBaseSha
        : pull.headSha;
    const text = source({ checkout: pull.checkout, sha, path: item.path })
      ?.split("\n")
      .slice(item.start - 1, item.end)
      .join("\n");
    if (text === undefined || text !== item.quote)
      throw new Error("Source evidence does not match: " + item.path + ":" + item.start);
    return {
      ...item,
      sha,
      text,
      url:
        pull.url.replace(
          /\/pull\/\d+$/,
          "/blob/" + sha + "/" + item.path.split("/").map(encodeURIComponent).join("/"),
        ) +
        "#L" +
        item.start,
    };
  });
  const groups = input.groups.map((group) => {
    const paths = [
      ...group.steps.flatMap((step) =>
        (
          step.files ?? (step.evidence ?? []).map((id) => evidence.find((item) => item.id === id))
        ).map((file) => file.path),
      ),
      ...group.files.map((file) => file.path),
    ];
    return {
      ...group,
      files: [...group.files].sort(
        (first, second) => paths.indexOf(first.path) - paths.indexOf(second.path),
      ),
    };
  });
  const id = createHash("sha256")
    .update(
      JSON.stringify({
        pins: pulls.map((pull) => [pull.repository, pull.number, pull.baseSha, pull.headSha]),
        title: input.title,
        groups: input.groups,
        evidence,
      }),
    )
    .digest("hex");
  const review = {
    version: 2,
    id,
    title: input.title,
    summary: input.summary,
    status: input.status ?? "complete",
    pulls,
    groups,
    fileOrder: [
      ...new Set([
        ...groups.flatMap((group) => group.files.map((file) => file.pr + ":" + file.path)),
        ...pulls.flatMap((pull) => pull.files.map((file) => file.id)),
      ]),
    ],
    evidence,
    combined,
    annotations: { notes: [], decisions: {}, reviewed: [], revision: 0 },
  };
  const coverage = checkCoverage(review);
  await writeFile(outputPath, await renderReview(review), { flag: "wx", mode: 0o600 });
  console.log(
    "build: " +
      review.pulls.length +
      " PRs, " +
      review.groups.length +
      " reading steps, " +
      coverage.covered +
      "/" +
      coverage.total +
      " changed files, " +
      review.pulls.reduce((count, pull) => count + pull.findings.length, 0) +
      " local findings",
  );
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
