export function annotatedHtml(options) {
  const pattern = /(<script type="application\/json" id="review-data">)([\s\S]*?)(<\/script>)/;
  const match = pattern.exec(options.html);
  if (!match) throw new Error("Choose the original guided review HTML.");
  const original = JSON.parse(match[2]);
  if (original.id !== options.id || original.version !== 2)
    throw new Error("This file belongs to another review.");
  if (JSON.stringify(original.annotations) !== JSON.stringify(options.expected))
    throw new Error("The file's notes changed outside this page. Reopen it before saving.");
  original.annotations = options.annotations;
  const encoded = JSON.stringify(original)
    .replace(/</g, "\\u003c")
    .replace(/\u2028/g, "\\u2028")
    .replace(/\u2029/g, "\\u2029");
  return options.html.replace(pattern, (_, start, data, end) => start + encoded + end);
}

export async function writeAnnotations(options) {
  const original = await (await options.handle.getFile()).text();
  if (options.previous !== null && original !== options.previous)
    throw new Error("The HTML changed outside this page. Your unsaved notes are preserved.");
  const updated = annotatedHtml({
    html: original,
    id: options.id,
    expected: options.expected,
    annotations: options.annotations,
  });
  const writable = await options.handle.createWritable({ mode: "exclusive" });
  try {
    if ((await (await options.handle.getFile()).text()) !== original)
      throw new Error("The HTML changed before the write. Your notes remain unsaved.");
    await writable.write(updated);
    await writable.close();
  } catch (error) {
    await writable.abort();
    throw error;
  }
  return updated;
}

let reviewHandle = null;
let previousFile = null;
let savedAnnotations = null;
let saveInProgress = false;

async function saveReview() {
  if (saveInProgress) return;
  const button = document.getElementById("save-review");
  const status = document.getElementById("save-status");
  if (typeof window.showOpenFilePicker !== "function") {
    status.textContent = "Open this HTML in Chrome or Edge to save into the original file.";
    return;
  }
  saveInProgress = true;
  button.disabled = true;
  try {
    if (!reviewHandle) {
      const [handle] = await window.showOpenFilePicker({
        multiple: false,
        types: [{ description: "Guided review HTML", accept: { "text/html": [".html"] } }],
      });
      if (!handle) throw new Error("No file selected. Your notes remain unsaved.");
      reviewHandle = handle;
    }
    const snapshot = structuredClone(review.annotations);
    const expected = savedAnnotations ?? initialAnnotations;
    status.textContent = "Saving into the original HTML…";
    previousFile = await writeAnnotations({
      handle: reviewHandle,
      previous: previousFile,
      id: review.id,
      expected,
      annotations: snapshot,
    });
    savedAnnotations = snapshot;
    dirty = JSON.stringify(review.annotations) !== JSON.stringify(snapshot);
    status.textContent = dirty
      ? "New changes are not saved yet"
      : "Saved into " + reviewHandle.name;
    document.getElementById("review-data").textContent = JSON.stringify(review);
  } catch (error) {
    if (savedAnnotations === null) reviewHandle = null;
    status.textContent =
      error.name === "AbortError" ? "Save cancelled. Your notes remain unsaved." : error.message;
  } finally {
    saveInProgress = false;
    button.disabled = false;
  }
}
function getFile(id) {
  if (id.startsWith("evidence:")) {
    const item = review.evidence.find((item) => "evidence:" + item.id === id);
    return {
      id,
      pr: item.pr,
      path: item.path,
      rows: item.text.split("\n").map((text, index) => ({
        kind: "context",
        side: item.side,
        text,
        oldLine: item.start + index,
        newLine: item.start + index,
        owner: item.pr,
      })),
      sha: { BASE: item.sha, HEAD: item.sha },
    };
  }
  return (
    (review.combined?.files ?? []).find((file) => file.id === id) ??
    review.pulls.flatMap((pull) => pull.files).find((file) => file.id === id)
  );
}
function loadCard(options) {
  const file = getFile(options.card.dataset.file);
  const pull = review.pulls.find((pull) => pull.id === file.pr);
  const sha = file.sha ?? {
    BASE: pull?.mergeBaseSha ?? review.combined.baseSha,
    HEAD: pull?.headSha ?? review.combined.headSha,
  };
  options.card.querySelector(".file-content").innerHTML = fileMarkup({
    file,
    pulls: review.pulls,
    sha,
    view: options.view,
  });
  options.card.dataset.loaded = "true";
  options.card
    .querySelectorAll("[data-view]")
    .forEach((button) =>
      button.classList.toggle("selected", button.dataset.view === (options.view ?? "diff")),
    );
  if (!options.deferNotes) renderNotes();
}
function codeContext(rows) {
  const first = rows[0];
  if (!first || rows.some((row) => row.dataset.file !== first.dataset.file)) return null;
  const file = getFile(first.dataset.file);
  const ranges = [];
  for (const row of rows) {
    const line = Number(row.dataset.line);
    const previous = ranges.at(-1);
    if (previous?.side === row.dataset.side && line === previous.end + 1) previous.end = line;
    else
      ranges.push({
        path: row.dataset.side === "BASE" ? (file.oldPath ?? file.path) : file.path,
        side: row.dataset.side,
        start: line,
        end: line,
        sha: row.dataset.sha,
      });
  }
  const owners = [
    ...new Set(
      rows
        .flatMap((row) => row.dataset.owner.split(","))
        .filter((id) => review.pulls.some((pull) => pull.id === id)),
    ),
  ];
  const prs = owners.length
    ? owners
    : file.pr === "stack"
      ? review.pulls
          .filter((pull) => pull.files.some((entry) => entry.path === file.path))
          .map((pull) => pull.id)
      : [file.pr];
  return {
    anchor: rows.at(-1).dataset.anchor,
    quote: rows.map((row) => row.querySelector("code").textContent).join("\n"),
    ranges,
    prs,
  };
}
function saveNote(event) {
  event.preventDefault();
  const body = document.getElementById("note-body").value.trim();
  const pr = document.getElementById("target-pr").value;
  if (!body || !pr) {
    document.getElementById("target-pr").focus();
    return;
  }
  const note = review.annotations.notes.find((note) => note.id === editingNote);
  if (note) note.body = body;
  else
    review.annotations.notes.push({
      id: crypto.randomUUID(),
      pr,
      anchor: selectedContext.anchor,
      quote: selectedContext.quote,
      ranges: selectedContext.ranges,
      body,
      originalBody: body,
      dismissed: false,
    });
  document.getElementById("composer").hidden = true;
  changed();
  renderNotes();
}
