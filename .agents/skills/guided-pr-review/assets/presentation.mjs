import { escapeHtml, fileMarkup } from "../scripts/format.mjs";

export const reviewStyle = String.raw`:root { color-scheme: dark; --background: #090a10; --panel: #141620; --border: #303246; --text: #e5e4ed; --muted: #9c9bae; --accent: #b6a2ee; }
* { box-sizing: border-box; }
[hidden] { display: none !important; }
body { margin: 0; background: var(--background); color: var(--text); font: 15px/1.6 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
button, input, textarea, select { font: inherit; color: inherit; }
button { cursor: pointer; border: 1px solid var(--border); border-radius: 6px; background: var(--panel); padding: 7px 12px; }
button:hover, button.selected { border-color: var(--accent); color: var(--accent); }
button:disabled { cursor: default; opacity: .45; }
a, .source-link { color: var(--accent); }
input, textarea, select { background: var(--background); border: 1px solid var(--border); border-radius: 5px; padding: 8px; }
textarea { resize: vertical; }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
header { position: sticky; top: 0; z-index: 20; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 12px 24px; background: #10121b; border-bottom: 1px solid var(--border); }
.brand { white-space: nowrap; font-size: 14px; }
.brand span { color: var(--accent); }
.header-actions, .pr-tabs, .view-tabs, .file-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
.header-actions { font-size: 13px; }
#save-review { background: var(--accent); color: var(--background); }
#save-status { max-width: 260px; color: var(--muted); font-size: 12px; }
.document { display: grid; grid-template-columns: 236px minmax(0, 1fr); max-width: 1440px; margin: auto; }
.route { position: sticky; top: 64px; height: calc(100vh - 64px); overflow: auto; padding: 24px 14px; border-right: 1px solid var(--border); }
.route h2 { margin: 0 10px 16px; color: var(--muted); font-size: 12px; letter-spacing: .08em; text-transform: uppercase; }
.route-step { display: flex; gap: 12px; width: 100%; margin: 4px 0; padding: 12px 10px; text-align: left; border-color: transparent; background: transparent; }
.route-step.selected { border-left: 2px solid var(--accent); background: var(--panel); }
.route-step small { display: block; margin-top: 4px; color: var(--muted); font-size: 11px; }
.route-footer { margin: 28px 10px; color: var(--muted); font-size: 12px; }
main { min-width: 0; padding: 24px 40px 64px; }
.overview { margin-bottom: 28px; }
.overview h2 { margin: 0; font-size: 17px; }
.overview > p { margin: 8px 0 16px; color: var(--muted); font-size: 14px; }
.pr-tabs { margin: 16px 0; font-size: 12px; }
.view-tabs { margin: 12px 0; }
.view-tabs button { padding: 5px 12px; font-size: 12px; }
.pr-details { color: var(--muted); font-size: 12px; }
summary { cursor: pointer; }
.pr-details p { padding-left: 18px; }
.step-meta { display: flex; align-items: center; justify-content: space-between; color: var(--muted); font-size: 12px; }
h1 { margin: 14px 0 18px; font-size: 27px; line-height: 1.25; }
h3 { margin: 0 0 6px; font-size: 16px; }
p { margin: 10px 0; }
.why { padding-left: 14px; border-left: 2px solid var(--accent); }
.before-after { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin: 22px 0; }
.before-after > div { padding: 16px; border: 1px solid var(--border); border-radius: 8px; background: var(--panel); }
.before-after span { color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: .08em; }
.before-after p { margin-bottom: 0; font-size: 14px; }
.flow { display: flex; flex-wrap: wrap; gap: 10px; margin: 18px 0; }
.flow > div { flex: 1; min-width: 130px; padding: 12px; border: 1px solid #554875; border-radius: 7px; font-size: 13px; }
.flow span { color: var(--accent); }
.flow-proof, .sources { margin: 8px 0 18px; color: var(--muted); font-size: 12px; }
.source-link { padding: 0; border: 0; background: none; font-size: 12px; text-decoration: underline; }
.passage { margin: 20px 0; }
.step-files { margin: 24px 0; }
.file-card { margin: 16px 0; border: 1px solid var(--border); border-radius: 8px; overflow: hidden; }
.file-card > summary { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 12px 14px; background: var(--panel); font: 12px/1.5 ui-monospace, monospace; overflow-wrap: anywhere; }
.file-stat { flex-shrink: 0; color: var(--muted); font-size: 11px; }
.file-stat b { color: #8fc9a5; }
.file-toolbar { padding: 10px 14px; border-top: 1px solid var(--border); font-size: 12px; }
.file-toolbar button { padding: 3px 10px; font-size: 12px; }
.file-toolbar a { margin-left: auto; }
.file-content { max-height: 540px; overflow: auto; background: #0b1011; }
.source-card { margin: 16px 0; border: 1px solid var(--border); border-radius: 8px; overflow: hidden; }
.source-card figcaption { padding: 10px 14px; background: var(--panel); color: var(--muted); font-size: 12px; overflow-wrap: anywhere; }
.source-card figcaption a { float: right; margin-left: 12px; }
.source-card .code-lines { overflow: auto; background: #0b1011; }
.source-card .code-line { grid-template-columns: 40px 0 0 minmax(max-content, 1fr) auto; }
.source-card .code-line > span.line-number { visibility: hidden; padding: 0; }
.code-line { display: grid; grid-template-columns: 34px 34px 18px minmax(max-content, 1fr) auto; min-width: max-content; padding-right: 12px; font: 12px/1.8 ui-monospace, SFMono-Regular, monospace; }
.code-line.add { background: #10251e; }
.code-line.remove { background: #2b171c; }
.code-line.chosen { background: #332a4b; }
.line-number { padding: 0 6px; text-align: right; color: #76818a; border: 0; border-radius: 0; background: none; user-select: none; font: inherit; }
.line-number:hover { color: var(--accent); }
.sign, .owner { user-select: none; color: var(--muted); }
.owner { padding-left: 18px; font-size: 10px; }
.code-line code { white-space: pre; color: #c6d8d3; }
.hunk { padding: 6px 14px; background: #181827; color: var(--muted); font: 11px ui-monospace, monospace; }
.metadata, .markdown { margin: 0; padding: 16px; }
.markdown pre, pre { overflow: auto; font-size: 12px; }
code { font-family: ui-monospace, SFMono-Regular, monospace; }
.dry-run { margin: 16px 0; padding: 14px 16px; border: 1px solid var(--border); border-radius: 7px; }
.dry-run summary span { float: right; color: var(--muted); font-size: 11px; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
th, td { padding: 10px; text-align: left; vertical-align: top; border-bottom: 1px solid var(--border); }
th { color: var(--muted); font-size: 12px; }
.step-navigation { display: flex; justify-content: space-between; margin-top: 28px; }
.step-navigation button { max-width: 48%; text-align: left; }
.reading-step { scroll-margin-top: 80px; }
.note, .finding { margin: 12px 16px; padding: 14px; border-left: 3px solid var(--accent); border-radius: 6px; background: #211b31; font-size: 13px; }
.finding { background: #171d29; border-color: #879fc4; }
.note-meta { display: flex; gap: 10px; align-items: center; color: var(--muted); font-size: 11px; }
.note blockquote, .finding blockquote { margin: 8px 0; padding-left: 10px; color: var(--muted); border-left: 1px solid var(--border); white-space: pre-wrap; }
.note-actions { display: flex; gap: 8px; margin-top: 8px; }
.note-actions button { padding: 2px 8px; font-size: 11px; }
.finding.dismissed { opacity: .55; }
.finding.included { border-color: var(--accent); }
#notes-panel { position: fixed; z-index: 40; top: 64px; right: 0; bottom: 0; width: min(460px, 100vw); overflow: auto; padding: 24px 16px; background: #10121b; border-left: 1px solid var(--border); box-shadow: -12px 0 40px #0006; }
#notes-panel h2 { margin: 18px 14px 8px; font-size: 18px; }
.human-notes > p { margin: 12px 14px; color: var(--muted); font-size: 13px; }
#notes-panel[data-view="findings"] .human-notes, #notes-panel[data-view="notes"] .agent-findings { display: none; }
#close-notes { float: right; }
#composer { position: fixed; z-index: 60; right: 24px; top: 96px; width: min(380px, calc(100vw - 32px)); padding: 18px; border: 1px solid var(--accent); border-radius: 10px; background: var(--panel); box-shadow: 0 12px 60px #0008; }
.composer-title { display: flex; justify-content: space-between; align-items: center; }
#composer blockquote { max-height: 100px; overflow: auto; margin: 12px 0; padding-left: 10px; border-left: 2px solid var(--border); color: var(--muted); white-space: pre-wrap; font-size: 12px; }
#target-label { display: flex; gap: 8px; align-items: center; font-size: 12px; }
#composer textarea { width: 100%; min-height: 100px; margin: 12px 0; }
#selection-comment { position: fixed; z-index: 70; background: var(--accent); color: var(--background); padding: 4px 12px; box-shadow: 0 4px 16px #0008; }
dialog { position: fixed; width: min(820px, 90vw); max-height: 85vh; overflow: auto; padding: 24px; color: var(--text); background: var(--panel); border: 1px solid var(--border); border-radius: 8px; }
dialog::backdrop { background: #0008; }
dialog h2 { font-size: 17px; }
#close-source { float: right; }
#source-pin { color: var(--muted); font-size: 12px; }
#source-code { white-space: pre; }
#file-search { width: 100%; margin: 8px 0; }
.prototype { color: #e4c486; }
.muted { color: var(--muted); }
@media (max-width: 760px) {
  header { padding: 10px 14px; flex-wrap: wrap; }
  .document { display: block; }
  .route { position: static; display: flex; height: auto; padding: 10px; overflow: auto; border-bottom: 1px solid var(--border); }
  .route h2, .route-footer { display: none; }
  .route-step { min-width: 180px; }
  main { padding: 20px 16px 40px; }
  .before-after { grid-template-columns: 1fr; }
  .file-card > summary { align-items: flex-start; flex-direction: column; }
  .header-actions { gap: 6px; }
  #composer { right: 16px; top: 128px; }
}
`;

export function sourceExcerptMarkup(options) {
  const { item, pulls } = options;
  const pull = pulls.find((pull) => pull.id === item.pr);
  const file = {
    id: "evidence:" + item.id,
    pr: item.pr,
    rows: item.text.split("\n").map((text, index) => ({
      kind: "context",
      side: item.side,
      text,
      oldLine: item.start + index,
      newLine: item.start + index,
      owner: item.pr,
    })),
  };
  return `<figure class="source-card"><figcaption>#${pull.number} · ${escapeHtml(item.path)}:${item.start} to ${item.end} · ${item.side}<a href="${escapeHtml(item.url)}" target="_blank" rel="noreferrer">Open pinned source</a></figcaption>${fileMarkup({ file, pulls, sha: { BASE: item.sha, HEAD: item.sha } })}</figure>`;
}

export function findingMarkup(finding) {
  const state = review.annotations.decisions[finding.id];
  const pull = review.pulls.find((pull) => pull.id === finding.pr);
  const sourceUrl =
    pull.url.replace(
      /\/pull\/\d+$/,
      "/blob/" + pull.headSha + "/" + finding.path.split("/").map(encodeURIComponent).join("/"),
    ) +
    "#L" +
    finding.line;
  const proof = finding.evidence
    .map(
      (item) =>
        "<p><code>" +
        escapeHtml(item.location) +
        "</code></p><blockquote>" +
        escapeHtml(item.quote) +
        "</blockquote>",
    )
    .join("");
  return `<aside class="finding ${escapeHtml(state ?? "")}" data-finding="${escapeHtml(finding.id)}"><div class="note-meta"><strong>Agent</strong><span>#${escapeHtml(pull.number)} · <a href="${escapeHtml(sourceUrl)}" target="_blank" rel="noreferrer">${escapeHtml(finding.path)}:${escapeHtml(finding.line)}</a></span><span>${state === "included" ? "in your review" : state === "dismissed" ? "dismissed" : "proposal"}</span></div><p>${escapeHtml(finding.explanation)}</p><details><summary>Inspect the reasoning</summary><p><strong>Trigger:</strong> ${escapeHtml(finding.failureScenario)}</p>${proof}<p><strong>Countercheck:</strong> ${escapeHtml(finding.refutation)}</p><p><strong>Suggested direction:</strong> ${escapeHtml(finding.suggestion ?? "Inspect the affected contract before choosing a fix.")}</p></details><div class="note-actions"><button data-include-finding="${escapeHtml(finding.id)}" type="button">${state === "included" ? "Remove from review" : "Add to my review"}</button><button data-dismiss-finding="${escapeHtml(finding.id)}" type="button">${state === "dismissed" ? "Restore" : "Dismiss"}</button></div></aside>`;
}

export function setSourceLink(options) {
  options.element.removeAttribute("href");
  options.element.textContent = "Source link unavailable.";
  if (options.url.startsWith("https://")) {
    options.element.href = options.url;
    options.element.textContent = "Open pinned source";
  }
}
