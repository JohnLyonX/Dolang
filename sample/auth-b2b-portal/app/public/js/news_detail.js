const articleEl = document.getElementById("news-article");
const statusEl = document.getElementById("news-status");
const slug = decodeURIComponent(window.location.pathname.split("/").filter(Boolean).pop() || "");

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
    month: "long",
    day: "numeric"
  });
}

async function loadArticle() {
  statusEl.textContent = "Loading article...";
  const response = await fetch(`/api/news/${encodeURIComponent(slug)}`);
  const article = await response.json();

  if (!response.ok) {
    throw new Error(article.message || `Request failed: ${response.status}`);
  }

  articleEl.innerHTML = `
    <div class="news-meta">
      <span>${escapeHtml(article.category_name)}</span>
      <span>${escapeHtml(formatDate(article.published_at))}</span>
      <span>${escapeHtml(article.slug)}</span>
    </div>
    <h1>${escapeHtml(article.title)}</h1>
    <p class="news-standfirst">${escapeHtml(article.summary)}</p>
    <div class="news-body">${escapeHtml(article.body)}</div>
  `;
  statusEl.textContent = "Article loaded.";
}

loadArticle().catch((error) => {
  statusEl.textContent = error.message;
});
