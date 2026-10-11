import { readFile } from "node:fs/promises";
import {
  reviewStyle,
  findingMarkup,
  setSourceLink,
  sourceExcerptMarkup,
} from "../assets/presentation.mjs";
import { escapeHtml, jsonForHtml, markdown, fileCard, behaviorMarkup } from "./format.mjs";

function sourceLinks(options) {
  return [...new Set(options.ids)]
    .map((id) => {
      const item = options.review.evidence.find((item) => item.id === id);
      return (
        '<button class="source-link" data-evidence="' +
        id +
        '" type="button" title="' +
        escapeHtml(item.path) +
        '">' +
        escapeHtml(item.label ?? item.path.split("/").pop() + ":" + item.start) +
        "</button>"
      );
    })
    .join(" ");
}

function groupMarkup(options) {
  const { group, review, index } = options;
  const numbers = group.prs
    .map((id) => "#" + review.pulls.find((pull) => pull.id === id).number)
    .join(" + ");
  const placedPaths = new Set();
  const { flow, dryRuns } = behaviorMarkup({
    group,
    sources: (ids) => sourceLinks({ review, ids }),
  });
  const fileSlots = (paths) =>
    [...new Set(paths)]
      .filter((path) => !placedPaths.has(path))
      .map((path) => {
        placedPaths.add(path);
        const related = group.files.filter((item) => item.path === path);
        const combinedFile = review.combined?.files.find((file) => file.path === path);
        const combinedCard = combinedFile
          ? fileCard({
              file: combinedFile,
              open: true,
              prLabel: "Combined stack",
              url: review.pulls.find((pull) => pull.id === related.at(-1).pr).url,
            })
          : "";
        const cards = related
          .map((item) => {
            const pull = review.pulls.find((pull) => pull.id === item.pr);
            return fileCard({
              file: pull.files.find((file) => file.path === path),
              open: true,
              prLabel: "#" + pull.number,
              url: pull.url,
            });
          })
          .join("");
        return (
          '<div class="file-slot" data-path="' +
          escapeHtml(path) +
          '" data-prs="' +
          related.map((item) => item.pr).join(",") +
          '">' +
          combinedCard +
          cards +
          "</div>"
        );
      })
      .join("");
  const passages = group.steps
    .map((step, stepIndex) => {
      const files =
        step.files ??
        (step.evidence ?? [])
          .map((id) => review.evidence.find((item) => item.id === id))
          .filter((item) =>
            group.files.some((file) => file.pr === item.pr && file.path === item.path),
          );
      const definitions = (step.definitions ?? [])
        .map((id) =>
          sourceExcerptMarkup({
            item: review.evidence.find((item) => item.id === id),
            pulls: review.pulls,
          }),
        )
        .join("");
      return `<section class="passage" data-anchor="${group.id}:passage:${stepIndex}" data-prs="${group.prs.join(",")}"><h3>${escapeHtml(step.title)}</h3>${markdown(step.text)}${definitions}<div class="sources">${sourceLinks({ review, ids: step.evidence ?? [] })}</div><div class="inline-notes" data-notes-at="${group.id}:passage:${stepIndex}"></div></section>${fileSlots(files.map((file) => file.path))}`;
    })
    .join("");
  const remaining = fileSlots(group.files.map((file) => file.path));
  return `<article class="reading-step" id="step-${group.id}" data-group="${group.id}" ${index ? "hidden" : ""}><div class="step-meta"><span>Step ${index + 1} of ${review.groups.length} · ${numbers}</span><label><input type="checkbox" data-reviewed="${group.id}"> Reviewed</label></div><h1>${escapeHtml(group.title)}</h1><div class="passage" data-anchor="${group.id}:summary" data-prs="${group.prs.join(",")}">${markdown(group.summary)}<div class="inline-notes" data-notes-at="${group.id}:summary"></div></div><p class="why">${escapeHtml(group.why)}</p><div class="before-after"><div><span>Before</span><p>${escapeHtml(group.before)}</p></div><div><span>After</span><p>${escapeHtml(group.after)}</p></div></div>${flow}${passages}${remaining ? '<div class="step-files"><h3>Remaining file changes</h3>' + remaining + "</div>" : ""}<div class="group-findings" data-findings-for="${group.id}"></div>${dryRuns}</article>`;
}

export async function renderReview(review) {
  const asset = async (name) => readFile(new URL("../assets/" + name, import.meta.url), "utf8");
  const count = review.pulls.reduce((total, pull) => total + pull.files.length, 0);
  const findings = review.pulls.reduce((total, pull) => total + pull.findings.length, 0);
  const prTabs =
    review.pulls.length > 1
      ? '<div class="pr-tabs">' +
        (review.combined ? '<button data-pr="stack" type="button">Combined stack</button>' : "") +
        review.pulls
          .map(
            (pull) =>
              '<button data-pr="' +
              pull.id +
              '" type="button">#' +
              pull.number +
              " " +
              escapeHtml(pull.title) +
              "</button>",
          )
          .join("") +
        "</div>"
      : "";
  const sidebar =
    '<aside class="route"><h2>Reading route</h2>' +
    review.groups
      .map(
        (group, index) =>
          '<button class="route-step" data-step="' +
          index +
          '" type="button"><span>' +
          (index + 1) +
          "</span><span>" +
          escapeHtml(group.title) +
          "<small>" +
          group.prs
            .map((id) => "#" + review.pulls.find((pull) => pull.id === id).number)
            .join(" + ") +
          " · " +
          group.files.length +
          " file changes</small></span></button>",
      )
      .join("") +
    '<div class="route-footer">' +
    count +
    " file changes across " +
    review.pulls.length +
    " PR" +
    (review.pulls.length === 1 ? "" : "s") +
    "<p>Read the code. Make your own notes. Then compare the agent findings.</p></div></aside>";
  const ci = review.pulls
    .map((pull) => {
      const counts = new Map();
      for (const check of pull.ci ?? [])
        counts.set(
          check.conclusion ?? check.state ?? "pending",
          (counts.get(check.conclusion ?? check.state ?? "pending") ?? 0) + 1,
        );
      return (
        '<p><a href="' +
        escapeHtml(pull.url) +
        '" target="_blank" rel="noreferrer">#' +
        pull.number +
        " " +
        escapeHtml(pull.title) +
        "</a><br><code>" +
        pull.headSha.slice(0, 10) +
        "</code> · " +
        pull.files.length +
        " files · " +
        [...counts].map(([status, total]) => total + " " + status.toLowerCase()).join(", ") +
        "</p>"
      );
    })
    .join("");
  const body = `<header><strong class="brand">◇ shadowclone <span>/ guided review</span></strong><div class="header-actions"><button id="toggle-findings" type="button">Agent findings <span>${findings}</span></button><button id="toggle-notes" type="button">Your notes <span id="note-count">0</span></button><button id="save-review" type="button">Save review</button><span id="save-status" role="status">Saved HTML</span></div></header><div class="document">${sidebar}<main><div class="overview"><h2>${escapeHtml(review.title)}</h2>${markdown(review.summary)}${review.status === "prototype" ? '<p class="prototype">Prototype: one part of the reading route. The full file inventory remains available.</p>' : ""}${prTabs}<div class="view-tabs"><button data-mode="guide" type="button">Guide</button><button data-mode="files" type="button">All changes (${count})</button></div><details class="pr-details"><summary>PR details and CI</summary>${ci}</details></div><div id="guide">${review.groups.map((group, index) => groupMarkup({ group, review, index })).join("")}<nav class="step-navigation"><button id="previous" type="button">Previous</button><button id="next" type="button">Next</button></nav></div><section id="files" hidden><h1>Every changed file</h1><p>These are the actual per-PR changes, including files outside the main reading step.</p><input id="file-search" type="search" placeholder="Find a file" aria-label="Find a changed file"><div id="file-list"></div></section></main></div><aside id="notes-panel" hidden><button id="close-notes" type="button">Close</button><section class="human-notes"><h2>Your review notes</h2><p>Save this HTML, then attach it to your chat and request staging.</p><div id="notes-list"></div></section><section class="agent-findings"><h2 id="findings-heading" hidden>Agent findings</h2><div id="findings-list"></div></section></aside><aside id="composer" hidden><form id="note-form"><div class="composer-title"><strong>Your note</strong><button id="cancel-note" type="button" aria-label="Cancel note">×</button></div><blockquote id="selected-quote"></blockquote><label id="target-label">Post on <select id="target-pr"></select></label><textarea id="note-body" aria-label="Your rough note" placeholder="Write the concern or question in your words." required></textarea><button type="submit">Save note</button></form></aside><button id="selection-comment" type="button" hidden>Comment</button><dialog id="source-dialog"><button id="close-source" type="button">Close</button><h2 id="source-title"></h2><p id="source-pin"></p><pre><code id="source-code"></code></pre><a id="source-link" target="_blank" rel="noreferrer">Open pinned source</a></dialog>`;
  const format = (await readFile(new URL("./format.mjs", import.meta.url), "utf8")).replace(
    /^export /gm,
    "",
  );
  const scripts = [
    format,
    findingMarkup.toString(),
    setSourceLink.toString(),
    ...(await Promise.all(["document.mjs", "notes.mjs", "guide.mjs"].map(asset))),
  ]
    .map((script) => script.replace(/^export /gm, ""))
    .join("\n");
  return (await asset("review.html"))
    .replace("{{TITLE}}", () => escapeHtml(review.title))
    .replace("{{STYLE}}", () => reviewStyle)
    .replace("{{BODY}}", () => body)
    .replace("{{DATA}}", () => jsonForHtml(review))
    .replace(
      '<script type="module" id="review-runtime"></script>',
      () => '<script type="module" id="review-runtime">' + scripts + "</script>",
    );
}
