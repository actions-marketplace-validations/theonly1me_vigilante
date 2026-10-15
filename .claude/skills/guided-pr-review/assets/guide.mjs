const review = JSON.parse(document.getElementById("review-data").textContent);
const initialAnnotations = structuredClone(review.annotations);
let dirty = false;
let showFindings = false;
let currentStep = 0;
let currentPr = review.combined ? "stack" : review.pulls[0].id;
let mode = "guide";

function renderFileList() {
  const search = document.getElementById("file-search").value.toLowerCase();
  const files = review.fileOrder
    .map(getFile)
    .filter((file) => currentPr === "stack" || currentPr === file.pr || review.pulls.length === 1)
    .filter((file) => file.path.toLowerCase().includes(search));
  document.getElementById("file-list").innerHTML = files
    .map((file) => {
      const pull = review.pulls.find((pull) => pull.id === file.pr);
      return fileCard({ file, open: true, prLabel: "#" + pull.number, url: pull.url });
    })
    .join("");
  document
    .querySelectorAll("#file-list .file-card")
    .forEach((card) => loadCard({ card, deferNotes: true }));
  renderNotes();
}

function refreshGuide() {
  document.getElementById("guide").hidden = mode !== "guide";
  document.getElementById("files").hidden = mode !== "files";
  document
    .querySelectorAll("[data-mode]")
    .forEach((button) => button.classList.toggle("selected", button.dataset.mode === mode));
  document
    .querySelectorAll("[data-pr]")
    .forEach((button) => button.classList.toggle("selected", button.dataset.pr === currentPr));
  document
    .querySelectorAll(".route-step")
    .forEach((button) =>
      button.classList.toggle("selected", Number(button.dataset.step) === currentStep),
    );
  document.querySelectorAll(".reading-step").forEach((element, index) => {
    element.hidden = index !== currentStep;
    element.querySelectorAll(".file-card").forEach((card) => {
      const file = getFile(card.dataset.file);
      card.hidden = file.pr !== currentPr;
      if (!card.hidden && !element.hidden && card.open && card.dataset.loaded !== "true")
        loadCard({ card, deferNotes: true });
    });
  });
  const previous = document.getElementById("previous");
  const next = document.getElementById("next");
  previous.disabled = currentStep === 0;
  next.disabled = currentStep === review.groups.length - 1;
  previous.textContent = currentStep ? "← " + review.groups[currentStep - 1].title : "First step";
  next.textContent =
    currentStep < review.groups.length - 1
      ? review.groups[currentStep + 1].title + " →"
      : "Last step";
  document.querySelectorAll("[data-reviewed]").forEach((input) => {
    input.checked = review.annotations.reviewed.includes(input.dataset.reviewed);
  });
  if (mode === "files") renderFileList();
  renderNotes();
}

function selectStep(index) {
  currentStep = Math.max(0, Math.min(review.groups.length - 1, index));
  mode = "guide";
  document.getElementById("selection-comment").hidden = true;
  refreshGuide();
  const step = document.getElementById("step-" + review.groups[currentStep].id);
  step.scrollIntoView({ block: "start" });
}

function showSource(id) {
  const item = review.evidence.find((item) => item.id === id);
  const number = review.pulls.find((pull) => pull.id === item.pr).number;
  document.getElementById("source-title").textContent =
    item.path + ":" + item.start + (item.end === item.start ? "" : " to " + item.end);
  document.getElementById("source-pin").textContent =
    `PR #${number} · ${item.side} · ${item.sha.slice(0, 12)}`;
  const file = getFile("evidence:" + id);
  document.getElementById("source-code").innerHTML = fileMarkup({
    file,
    pulls: review.pulls,
    sha: file.sha,
  });
  setSourceLink({
    element: document.getElementById("source-link"),
    url: item.url,
  });
  document.getElementById("source-dialog").showModal();
}

function pageAction(event) {
  const button = event.target.closest("button");
  if (!button) return;
  if (button.dataset.step) selectStep(Number(button.dataset.step));
  if (button.dataset.pr) {
    currentPr = button.dataset.pr;
    refreshGuide();
  }
  if (button.dataset.mode) {
    mode = button.dataset.mode;
    refreshGuide();
  }
  if (button.dataset.view)
    loadCard({ card: button.closest(".file-card"), view: button.dataset.view });
  if (button.dataset.evidence) showSource(button.dataset.evidence);
  notesAction(event);
}

try {
  const cached = JSON.parse(localStorage.getItem("guided-review-v2:" + review.id) ?? "null");
  if (
    cached &&
    cached.revision > review.annotations.revision &&
    Array.isArray(cached.notes) &&
    cached.notes.every(
      (note) =>
        review.pulls.some((pull) => pull.id === note.pr) &&
        typeof note.body === "string" &&
        typeof note.quote === "string" &&
        Array.isArray(note.ranges),
    )
  ) {
    review.annotations = cached;
    dirty = true;
    document.getElementById("save-status").textContent = "Recovered unsaved changes";
  }
} catch {
  document.getElementById("save-status").textContent =
    "Browser recovery is unavailable. Use Save review before closing.";
}
document.addEventListener("click", pageAction);
document.addEventListener("pointerdown", startLineSelection);
document.addEventListener("pointerup", (event) => {
  finishLineSelection(event);
  captureSelection(event);
});
document.addEventListener(
  "toggle",
  (event) => {
    if (
      event.target.matches(".file-card") &&
      event.target.open &&
      !event.target.hidden &&
      !event.target.closest(".reading-step")?.hidden &&
      event.target.dataset.loaded !== "true"
    )
      loadCard({ card: event.target });
  },
  true,
);
document.addEventListener("change", (event) => {
  if (!event.target.dataset.reviewed) return;
  const id = event.target.dataset.reviewed;
  review.annotations.reviewed = event.target.checked
    ? [...new Set([...review.annotations.reviewed, id])]
    : review.annotations.reviewed.filter((value) => value !== id);
  changed();
});
document.getElementById("toggle-findings").onclick = () => showReviewPanel("findings");
document.getElementById("toggle-notes").onclick = () => showReviewPanel("notes");
document.getElementById("close-notes").onclick = () => {
  document.getElementById("notes-panel").hidden = true;
};
document.getElementById("cancel-note").onclick = () => {
  document.getElementById("composer").hidden = true;
};
document.getElementById("selection-comment").onclick = () => openComposer();
document.getElementById("note-form").onsubmit = saveNote;
document.getElementById("save-review").onclick = saveReview;
document.getElementById("close-source").onclick = () => {
  document.getElementById("source-dialog").close();
};
document.getElementById("source-dialog").onclose = () => {
  document.body.append(
    document.getElementById("composer"),
    document.getElementById("selection-comment"),
  );
};
document.getElementById("previous").onclick = () => selectStep(currentStep - 1);
document.getElementById("next").onclick = () => selectStep(currentStep + 1);
document.getElementById("file-search").oninput = renderFileList;
window.addEventListener("beforeunload", (event) => {
  if (
    dirty ||
    (!document.getElementById("composer").hidden &&
      document.getElementById("note-body").value.trim())
  ) {
    event.preventDefault();
    event.returnValue = "";
  }
});
refreshGuide();
