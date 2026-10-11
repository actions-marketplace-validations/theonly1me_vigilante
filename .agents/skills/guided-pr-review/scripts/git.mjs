import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { ancestor, patchRows } from "./check.mjs";

function fileKey(options) {
  return options.pr + ":" + options.path;
}

function git(options) {
  const result = spawnSync(
    "git",
    ["--literal-pathspecs", "-C", options.checkout, ...options.arguments],
    {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error("Git evidence failed: " + result.stderr.trim().slice(0, 400));
  return result.stdout;
}

export function source(options) {
  const result = spawnSync(
    "git",
    ["-C", options.checkout, "show", options.sha + ":" + options.path],
    { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.status !== 0) {
    const exists = spawnSync("git", [
      "-C",
      options.checkout,
      "cat-file",
      "-e",
      options.sha + ":" + options.path,
    ]);
    if (exists.status === 1 || exists.status === 128) return null;
    throw new Error("A committed source file could not be read.");
  }
  return result.stdout.includes("\0") ? null : result.stdout;
}

function filesAt(options) {
  const names = git({
    checkout: options.checkout,
    arguments: ["diff", "--name-status", "-z", "--find-renames", options.base, options.head],
  }).split("\0");
  const files = [];
  while (names.length && names[0]) {
    const status = names.shift();
    const oldPath = names.shift();
    const path = /^[RC]/.test(status) ? names.shift() : oldPath;
    const patch = git({
      checkout: options.checkout,
      arguments: [
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--find-renames",
        "--unified=5",
        options.base,
        options.head,
        "--",
        ...new Set([oldPath, path]),
      ],
    });
    const rows = patchRows(patch).map((row) => ({ ...row, owner: options.pr }));
    const isMarkdown = /\.(md|mdx|markdown)$/i.test(path);
    files.push({
      id: fileKey({ pr: options.pr, path }),
      pr: options.pr,
      path,
      diffAnchor: "diff-" + createHash("sha256").update(path).digest("hex"),
      oldPath,
      status,
      rows,
      patch: rows.length ? null : patch,
      additions: rows.filter((row) => row.kind === "add").length,
      deletions: rows.filter((row) => row.kind === "remove").length,
      baseMarkdown: isMarkdown ? source({ ...options, sha: options.base, path: oldPath }) : null,
      headMarkdown: isMarkdown ? source({ ...options, sha: options.head, path }) : null,
    });
  }
  return files;
}

function addedOwners(options) {
  const text = git({
    checkout: options.checkout,
    arguments: ["blame", "--line-porcelain", options.head, "--", options.path],
  });
  const owners = new Map();
  let line = 0;
  let commit = "";
  for (const value of text.split("\n")) {
    const header = /^([a-f0-9]{40}) \d+ (\d+)/.exec(value);
    if (header) {
      commit = header[1];
      line = Number(header[2]);
    }
    if (value.startsWith("\t"))
      owners.set(line, options.pulls.find((pull) => pull.commits.has(commit))?.id ?? "context");
  }
  return owners;
}

export async function collectReview(input) {
  const pulls = [];
  for (const pull of input.pulls) {
    for (const sha of [pull.baseSha, pull.headSha])
      git({ checkout: pull.checkout, arguments: ["cat-file", "-e", sha + "^{commit}"] });
    const mergeBaseSha = git({
      checkout: pull.checkout,
      arguments: ["merge-base", pull.baseSha, pull.headSha],
    }).trim();
    const files = filesAt({
      checkout: pull.checkout,
      base: mergeBaseSha,
      head: pull.headSha,
      pr: pull.id,
    });
    const { checkout, reviewFile, ...publicPull } = pull;
    pulls.push({
      ...publicPull,
      mergeBaseSha,
      files,
      findings: [],
      skippedPaths: [],
    });
  }
  let combined = null;
  const endpoint =
    input.pulls.length > 1 &&
    input.pulls.find((candidate) =>
      input.pulls.every(
        (pull) =>
          pull.repository === candidate.repository &&
          ancestor({ checkout: candidate.checkout, base: pull.headSha, head: candidate.headSha }),
      ),
    );
  if (endpoint) {
    const baseSha = git({
      checkout: endpoint.checkout,
      arguments: ["merge-base", "--octopus", ...pulls.map((pull) => pull.mergeBaseSha)],
    }).trim();
    const ordered = [...pulls].sort((first, second) =>
      ancestor({ checkout: endpoint.checkout, base: first.headSha, head: second.headSha }) ? -1 : 1,
    );
    const ownership = ordered.map((pull) => ({
      ...pull,
      commits: new Set(
        git({
          checkout: endpoint.checkout,
          arguments: ["rev-list", pull.headSha, "^" + pull.mergeBaseSha],
        })
          .trim()
          .split("\n"),
      ),
    }));
    const files = filesAt({
      checkout: endpoint.checkout,
      base: baseSha,
      head: endpoint.headSha,
      pr: "stack",
    });
    for (const file of files) {
      const owners = file.rows.some((row) => row.newLine !== null)
        ? addedOwners({
            checkout: endpoint.checkout,
            head: endpoint.headSha,
            path: file.path,
            pulls: ownership,
          })
        : new Map();
      for (const row of file.rows) {
        if (row.kind === "add") row.owner = owners.get(row.newLine) ?? "context";
        else if (row.kind === "remove")
          row.owner =
            ordered
              .filter((pull) =>
                pull.files.some(
                  (entry) =>
                    entry.oldPath === file.oldPath &&
                    entry.rows.some(
                      (removed) => removed.kind === "remove" && removed.text === row.text,
                    ),
                ),
              )
              .map((pull) => pull.id)
              .join(",") || "stack";
        else row.owner = "context";
      }
    }
    combined = { baseSha, headSha: endpoint.headSha, files };
  }
  return { pulls, combined };
}
