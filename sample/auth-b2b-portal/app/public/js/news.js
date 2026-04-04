const listEl = document.getElementById("news-list");
const statusEl = document.getElementById("news-status");

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function renderStatus(text) {
  statusEl.textContent = text;
}

function articleCard(article) {
  const publishedAt = article.published_at ? new Date(article.published_at * 1000).toISOString().slice(0, 10) : "Draft";
  return `
    <article class="news-card">
      <div class="news-meta">
        <span>${escapeHtml(article.category_name)}</span>
        <span>${escapeHtml(publishedAt)}</span>
      </div>
      <h2><a href="/news/${encodeURIComponent(article.slug)}">${escapeHtml(article.title)}</a></h2>
      <p class="news-summary">${escapeHtml(article.summary)}</p>
    </article>
  `;
}

async function loadNews() {
  renderStatus("Loading published news...");
  const response = await fetch("/api/news");
  const articles = await response.json();

  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }

  listEl.innerHTML = articles.map(articleCard).join("");
  renderStatus(`${articles.length} published article(s) loaded.`);
}

loadNews().catch((error) => {
  renderStatus(error.message);
});
