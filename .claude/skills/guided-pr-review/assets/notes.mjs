let selectedContext = null;
let editingNote = null;
let dragStart = null;

function showReviewPanel(view) {
  const panel = document.getElementById("notes-panel");
  panel.dataset.view = view;
  panel.hidden = false;
  panel.scrollTop = 0;
  showFindings = view === "findings";
  document.getElementById("toggle-findings").classList.toggle("selected", showFindings);
  renderNotes();
}

function changed() {
  review.annotations.revision += 1;
  dirty = true;
  document.getElementById("save-status").textContent = "Unsaved changes";
  try {
    localStorage.setItem("guided-review-v2:" + review.id, JSON.stringify(review.annotations));
  } catch {
    document.getElementById("save-status").textContent =
      "Unsaved changes. This browser cannot keep a recovery copy.";
  }
}

function renderNotes() {
  document.querySelectorAll(".inline-notes").forEach((element) => {
    element.innerHTML = review.annotations.notes
      .filter((note) => !note.dismissed && note.anchor === element.dataset.notesAt)
      .map(noteMarkup)
      .join("");
  });
  document.querySelectorAll(".group-findings").forEach((element) => {
    const group = review.groups.find((group) => group.id === element.dataset.findingsFor);
    const remaining = [];
    for (const finding of review.pulls.flatMap((pull) => pull.findings)) {
      if (!group.files.some((file) => file.pr === finding.pr && file.path === finding.path))
        continue;
      const rows = [...element.closest(".reading-step").querySelectorAll(".code-line")];
      const row = rows.find(
        (row) =>
          row.dataset.file === finding.pr + ":" + finding.path &&
          row.dataset.side === "HEAD" &&
          Number(row.dataset.line) === finding.line,
      );
      if (showFindings && row && row.closest(".file-card")?.hidden !== true)
        row.nextElementSibling.insertAdjacentHTML("beforeend", findingMarkup(finding));
      else remaining.push(finding);
    }
    element.hidden = !showFindings;
    element.innerHTML = remaining.map(findingMarkup).join("");
  });
  document.getElementById("note-count").textContent = review.annotations.notes.filter(
    (note) => !note.dismissed,
  ).length;
  document.getElementById("notes-list").innerHTML = review.annotations.notes.length
    ? review.annotations.notes.map(noteMarkup).join("")
    : '<p class="muted">Select code or an explanation, then choose Comment.</p>';
  document.getElementById("findings-heading").hidden = !showFindings;
  document.getElementById("findings-list").innerHTML = showFindings
    ? review.pulls
        .flatMap((pull) => pull.findings)
        .map(findingMarkup)
        .join("")
    : "";
}

function captureSelection(event) {
  if (!document.getElementById("composer").hidden) return;
  if (event.target.closest("#composer, #notes-panel, .note, .finding, button, textarea, select"))
    return;
  const selection = window.getSelection();
  if (!selection?.rangeCount || !selection.toString().trim()) return;
  const range = selection.getRangeAt(0);
  const start = selection.anchorNode.parentElement;
  const end = selection.focusNode.parentElement;
  const card = start.closest(".file-card, .source-card, #source-dialog");
  const rows =
    card && end.closest(".file-card, .source-card, #source-dialog") === card
      ? [...card.querySelectorAll(".code-line")].filter((row) =>
          range.intersectsNode(row.querySelector("code")),
        )
      : [];
  if (rows.length) selectedContext = codeContext(rows);
  else {
    const passage = start.closest(".passage");
    if (!passage || end.closest(".passage") !== passage) return;
    selectedContext = {
      anchor: passage.dataset.anchor,
      quote: selection.toString(),
      ranges: [],
      prs: passage.dataset.prs?.split(",") ?? review.pulls.map((pull) => pull.id),
    };
  }
  if (!selectedContext) return;
  const button = document.getElementById("selection-comment");
  const dialog = start.closest("dialog");
  (dialog ?? document.body).append(button);
  button.hidden = false;
  const bounds = range.getBoundingClientRect();
  button.style.left = Math.max(12, Math.min(bounds.left, innerWidth - 110)) + "px";
  button.style.top = Math.max(70, bounds.top - 40) + "px";
}

function openComposer(note) {
  if (!document.getElementById("composer").hidden) return;
  editingNote = note?.id ?? null;
  if (note)
    selectedContext = {
      anchor: note.anchor,
      quote: note.quote,
      ranges: note.ranges,
      prs: [note.pr],
    };
  if (!selectedContext) return;
  const composer = document.getElementById("composer");
  (document.getElementById("source-dialog").open
    ? document.getElementById("source-dialog")
    : document.body
  ).append(composer);
  document.getElementById("selection-comment").hidden = true;
  document.getElementById("selected-quote").textContent = selectedContext.quote;
  document.getElementById("target-pr").innerHTML =
    (selectedContext.prs.length > 1 ? '<option value="">Choose a PR</option>' : "") +
    selectedContext.prs
      .map(
        (id) =>
          '<option value="' +
          id +
          '">#' +
          review.pulls.find((pull) => pull.id === id).number +
          "</option>",
      )
      .join("");
  document.getElementById("target-label").hidden = selectedContext.prs.length === 1;
  document.getElementById("note-body").value = note?.body ?? "";
  composer.hidden = false;
  document.getElementById("note-body").focus();
}

function notesAction(event) {
  const button = event.target.closest("button");
  if (!button) return;
  const note = review.annotations.notes.find(
    (note) => note.id === (button.dataset.editNote ?? button.dataset.dismissNote),
  );
  if (button.dataset.editNote && note) openComposer(note);
  if (button.dataset.dismissNote && note) {
    note.dismissed = !note.dismissed;
    changed();
    renderNotes();
  }
  const finding = button.dataset.includeFinding ?? button.dataset.dismissFinding;
  if (finding) {
    const state = review.annotations.decisions[finding];
    review.annotations.decisions[finding] = button.dataset.includeFinding
      ? state === "included"
        ? null
        : "included"
      : state === "dismissed"
        ? null
        : "dismissed";
    changed();
    renderNotes();
  }
}

function startLineSelection(event) {
  if (!document.getElementById("composer").hidden) return;
  if (!event.target.closest(".line-number")) return;
  event.preventDefault();
  dragStart = event.target.closest(".code-line");
}

function finishLineSelection(event) {
  if (!dragStart) return;
  const end = event.target.closest(".code-line");
  if (
    end &&
    dragStart.closest(".file-card") === end.closest(".file-card") &&
    dragStart.dataset.file === end.dataset.file
  ) {
    const parent = dragStart.closest(".code-lines");
    const rows = [...parent.querySelectorAll(".code-line")];
    const indices = [rows.indexOf(dragStart), rows.indexOf(end)].sort(
      (first, second) => first - second,
    );
    selectedContext = codeContext(rows.slice(indices[0], indices[1] + 1));
    const selectedRows = rows.slice(indices[0], indices[1] + 1);
    rows.forEach((row) => row.classList.toggle("chosen", selectedRows.includes(row)));
    openComposer();
  }
  dragStart = null;
}
