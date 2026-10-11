export function escapeHtml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character],
  );
}

export function jsonForHtml(value) {
  return JSON.stringify(value)
    .replace(/</g, "\\u003c")
    .replace(/\u2028/g, "\\u2028")
    .replace(/\u2029/g, "\\u2029");
}

export function markdown(value) {
  const inline = (text) =>
    escapeHtml(text)
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/\*([^*]+)\*/g, "<em>$1</em>")
      .replace(
        /\[([^\]]+)\]\((https?:\/\/[^\s)]+|#[^\s)]+)\)/g,
        '<a href="$2" target="_blank" rel="noreferrer">$1</a>',
      );
  let fenced = false;
  let list = false;
  let table = false;
  const output = [];
  for (const line of String(value ?? "").split("\n")) {
    if (table && !line.startsWith("|")) {
      output.push("</table>");
      table = false;
    }
    if (line.startsWith("```")) {
      if (list) {
        output.push("</ul>");
        list = false;
      }
      output.push(fenced ? "</code></pre>" : "<pre><code>");
      fenced = !fenced;
    } else if (fenced) output.push(escapeHtml(line) + "\n");
    else if (line.startsWith("|")) {
      if (!table) {
        output.push("<table>");
        table = true;
      }
      if (!/^\|[\s:|-]+\|$/.test(line))
        output.push(
          "<tr>" +
            line
              .split("|")
              .slice(1, -1)
              .map((cell) => "<td>" + inline(cell.trim()) + "</td>")
              .join("") +
            "</tr>",
        );
    } else if (/^[-*] /.test(line)) {
      if (!list) {
        output.push("<ul>");
        list = true;
      }
      output.push("<li>" + inline(line.slice(2)) + "</li>");
    } else {
      if (list) {
        output.push("</ul>");
        list = false;
      }
      const heading = /^(#{1,6}) (.*)$/.exec(line);
      if (heading)
        output.push(
          "<h" + heading[1].length + ">" + inline(heading[2]) + "</h" + heading[1].length + ">",
        );
      else if (line.trim()) output.push("<p>" + inline(line) + "</p>");
    }
  }
  if (fenced) output.push("</code></pre>");
  if (table) output.push("</table>");
  if (list) output.push("</ul>");
  return output.join("");
}

export function fileMarkup(options) {
  const { file, view = "diff", pulls, sha } = options;
  if (view !== "diff") {
    const text = view === "base" ? file.baseMarkdown : file.headMarkdown;
    const prs =
      file.pr === "stack"
        ? pulls
            .filter((pull) => pull.files.some((item) => item.path === file.path))
            .map((pull) => pull.id)
        : [file.pr];
    const anchor = escapeHtml(file.id + ":" + view);
    return text === null
      ? '<p class="muted">This side has no Markdown file.</p>'
      : `<div class="markdown passage" data-anchor="${anchor}" data-prs="${escapeHtml(prs.join(","))}">${markdown(text)}<div class="inline-notes" data-notes-at="${anchor}"></div></div>`;
  }
  if (!file.rows.length)
    return (
      '<pre class="metadata">' +
      escapeHtml(file.patch || "No net change at the stack head. Inspect the per-PR diff.") +
      "</pre>"
    );
  return (
    '<div class="code-lines">' +
    file.rows
      .map((row, index) => {
        if (row.kind === "hunk") return '<div class="hunk">' + escapeHtml(row.text) + "</div>";
        const side = row.side ?? (row.kind === "remove" ? "BASE" : "HEAD");
        const line = side === "BASE" ? row.oldLine : row.newLine;
        const owners = row.owner
          .split(",")
          .map((id) => pulls.find((pull) => pull.id === id))
          .filter(Boolean);
        const owner = pulls.length > 1 ? owners.map((pull) => "#" + pull.number).join(" + ") : "";
        return `<div class="code-line ${escapeHtml(row.kind)}" data-anchor="${escapeHtml(file.id + ":" + index)}" data-file="${escapeHtml(file.id)}" data-side="${escapeHtml(side)}" data-line="${escapeHtml(line)}" data-sha="${escapeHtml(sha[side])}" data-owner="${escapeHtml(row.owner)}"><button class="line-number" type="button" aria-label="Select line ${escapeHtml(line)}">${escapeHtml(row.oldLine ?? "")}</button><span class="line-number">${escapeHtml(row.newLine ?? "")}</span><span class="sign">${row.kind === "add" ? "+" : row.kind === "remove" ? "-" : ""}</span><code>${escapeHtml(row.text)}</code><span class="owner">${escapeHtml(owner)}</span></div><div class="inline-notes" data-notes-at="${escapeHtml(file.id + ":" + index)}"></div>`;
      })
      .join("") +
    "</div>"
  );
}

export function fileCard(options) {
  const url = new URL(options.url);
  url.pathname += "/files";
  url.search = "";
  url.hash = options.file.diffAnchor;
  return `<details class="file-card" data-file="${escapeHtml(options.file.id)}" ${options.open ? "open" : ""}><summary><span>${escapeHtml(options.file.path)}</span><span class="file-stat">${escapeHtml(options.prLabel)} <b>+${escapeHtml(options.file.additions)}</b> / -${escapeHtml(options.file.deletions)}</span></summary><div class="file-toolbar"><button data-view="diff" class="selected" type="button">Diff</button>${options.file.baseMarkdown !== null || options.file.headMarkdown !== null ? '<button data-view="base" type="button">Base Markdown</button><button data-view="head" type="button">Head Markdown</button>' : ""}<a href="${escapeHtml(url.href)}" target="_blank" rel="noreferrer">Open file in PR</a></div><div class="file-content"></div></details>`;
}

export function noteMarkup(note) {
  const pull = review.pulls.find((pull) => pull.id === note.pr);
  return `<aside class="note ${note.dismissed ? "dismissed" : ""}" data-note="${escapeHtml(note.id)}"><div class="note-meta"><strong>You</strong><span>#${escapeHtml(pull.number)} · ${note.dismissed ? "removed" : "draft"}</span></div><blockquote>${escapeHtml(note.quote)}</blockquote><p>${escapeHtml(note.body)}</p><div class="note-actions"><button data-edit-note="${escapeHtml(note.id)}" type="button">Edit</button><button data-dismiss-note="${escapeHtml(note.id)}" type="button">${note.dismissed ? "Restore" : "Remove"}</button></div></aside>`;
}

export function behaviorMarkup(options) {
  const { group, sources } = options;
  const nodes = group.diagram?.nodes ?? [];
  const flow = nodes.length
    ? '<div class="flow">' +
      nodes
        .map(
          (node, nodeIndex) =>
            "<div><strong>" +
            escapeHtml(node.label) +
            "</strong>" +
            (nodeIndex < nodes.length - 1 ? '<span aria-hidden="true"> →</span>' : "") +
            "</div>",
        )
        .join("") +
      '</div><details class="flow-proof"><summary>Follow the branches and source</summary>' +
      (group.diagram.edges ?? [])
        .map(
          (edge) =>
            "<p>" +
            escapeHtml(nodes.find((node) => node.id === edge.from)?.label ?? edge.from) +
            " → " +
            escapeHtml(nodes.find((node) => node.id === edge.to)?.label ?? edge.to) +
            ": " +
            escapeHtml(edge.label) +
            " " +
            sources(edge.evidence) +
            "</p>",
        )
        .join("") +
      "</details>"
    : "";
  const dryRuns = (group.dryRuns ?? [])
    .map(
      (run) =>
        '<details class="dry-run"><summary>' +
        escapeHtml(run.title) +
        "<span>" +
        escapeHtml(run.kind ?? "reasoned") +
        "</span></summary><p>" +
        escapeHtml(run.input) +
        "</p><table><thead><tr><th>Branch</th><th>State</th><th>Effect and outcome</th></tr></thead><tbody>" +
        run.steps
          .map(
            (step) =>
              "<tr><td>" +
              escapeHtml(step.condition ?? step.when ?? step.branch) +
              "</td><td>" +
              escapeHtml(step.state) +
              "</td><td>" +
              escapeHtml((step.effect ?? step.effects ?? "") + " " + (step.outcome ?? "")) +
              " " +
              sources(step.evidence ?? []) +
              "</td></tr>",
          )
          .join("") +
        "</tbody></table><p>" +
        escapeHtml(run.outcome) +
        "</p></details>",
    )
    .join("");
  return { flow, dryRuns };
}
