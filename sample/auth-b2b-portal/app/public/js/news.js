const listEl = document.getElementById("news-list");
const statusEl = document.getElementById("news-status");
const leadEl = document.getElementById("lead-story");

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function formatDate(epochSeconds) {
  if (!epochSeconds) {
    return "Unscheduled";
  }
  return new Date(epochSeconds * 1000).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric"
  });
}

function renderStatus(text) {
  statusEl.textContent = text;
}

function leadStory(article) {
  if (!article) {
    leadEl.innerHTML = `
      <p class="page-label">Lead Story</p>
      <h1>No published story yet.</h1>
      <p class="story-summary">Publish the first article from the newsroom desk to populate the front page.</p>
    `;
    return;
  }

  leadEl.innerHTML = `
    <p class="page-label">Lead Story</p>
    <div class="story-meta">
      <span>${escapeHtml(article.category_name)}</span>
      <span>${escapeHtml(formatDate(article.published_at))}</span>
    </div>
    <h1>${escapeHtml(article.title)}</h1>
    <p class="story-summary">${escapeHtml(article.summary)}</p>
    <a class="story-link" href="/news/${encodeURIComponent(article.slug)}">Read full report</a>
  `;
}

function storyCard(article) {
  return `
    <article class="story-card">
      <div class="story-date">${escapeHtml(formatDate(article.published_at))}</div>
      <div>
        <div class="story-meta">
          <span>${escapeHtml(article.category_name)}</span>
          <span>${escapeHtml(article.slug)}</span>
        </div>
        <h3><a href="/news/${encodeURIComponent(article.slug)}">${escapeHtml(article.title)}</a></h3>
        <p class="story-summary">${escapeHtml(article.summary)}</p>
      </div>
    </article>
  `;
}

async function loadNews() {
  renderStatus("Loading published reports...");
  const response = await fetch("/api/news");
  const articles = await response.json();

  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }

  leadStory(articles[0] || null);
  listEl.innerHTML = articles.map(storyCard).join("");
  renderStatus(`${articles.length} published report(s) on the wire.`);
}

loadNews().catch((error) => {
  renderStatus(error.message);
});
