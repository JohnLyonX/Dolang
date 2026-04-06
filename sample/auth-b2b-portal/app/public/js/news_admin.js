const identityPanel = document.getElementById("identity-panel");
const articleList = document.getElementById("article-list");
const responsePanel = document.getElementById("response-panel");
const transportInput = document.getElementById("transport");
const csrfInput = document.getElementById("csrf-token");
const accessInput = document.getElementById("access-token");
const articleIdInput = document.getElementById("article-id");
const slugInput = document.getElementById("slug");
const titleInput = document.getElementById("title");
const summaryInput = document.getElementById("summary");
const bodyInput = document.getElementById("body");
const categoryIdInput = document.getElementById("category-id");

const state = {
  transport: "cookie",
  csrfToken: "",
  accessToken: "",
  principal: null
};

function pretty(value) {
  return typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function syncIdentity() {
  transportInput.value = state.transport;
  csrfInput.value = state.csrfToken;
  accessInput.value = state.accessToken;

  identityPanel.innerHTML = `
    <h2>Identity</h2>
    <div class="article-meta">
      <span>scheme: ${escapeHtml(state.principal?.scheme || "none")}</span>
      <span>subject: ${escapeHtml(state.principal?.subject || "anonymous")}</span>
      <span>roles: ${escapeHtml((state.principal?.roles || []).join(", "))}</span>
    </div>
  `;
}

function setResponse(label, payload) {
  responsePanel.textContent = `${label}\n\n${pretty(payload)}`;
}

function updateFromMe(payload) {
  if (!payload || payload.authenticated !== true) {
    return;
  }
  state.principal = payload.principal || null;
  if (payload.session && payload.session.csrf_token) {
    state.csrfToken = payload.session.csrf_token;
  }
  syncIdentity();
}

async function requestJson(path, options = {}) {
  const transport = options.transport || state.transport;
  const headers = {};

  if (options.body != null) {
    headers["Content-Type"] = "application/json";
  }
  if (transport === "bearer" && state.accessToken) {
    headers.Authorization = `Bearer ${state.accessToken}`;
  }
  if (options.csrf === true && transport !== "bearer" && state.csrfToken) {
    headers["X-CSRF-Token"] = state.csrfToken;
  }

  const response = await fetch(path, {
    method: options.method || "GET",
    credentials: transport === "bearer" ? "omit" : "include",
    headers,
    body: options.body != null ? JSON.stringify(options.body) : undefined
  });

  const raw = await response.text();
  const payload = raw ? JSON.parse(raw) : {};
  if (!response.ok) {
    throw new Error(pretty(payload));
  }
  return payload;
}

async function bootstrapIdentity() {
  try {
    const cookieMe = await requestJson("/me", { transport: "cookie" });
    updateFromMe(cookieMe);
  } catch (error) {
    syncIdentity();
  }
}

function fillForm(article) {
  articleIdInput.value = article.article_id || "";
  slugInput.value = article.slug || "";
  titleInput.value = article.title || "";
  summaryInput.value = article.summary || "";
  bodyInput.value = article.body || "";
  categoryIdInput.value = article.category_id || "cat_company";
}

function currentPayload() {
  return {
    slug: slugInput.value,
    title: titleInput.value,
    summary: summaryInput.value,
    body: bodyInput.value,
    category_id: categoryIdInput.value
  };
}

async function loadArticles() {
  const payload = await requestJson("/api/admin/news");
  articleList.innerHTML = payload.map((article) => `
    <article class="article-card">
      <div class="article-meta">
        <span>${escapeHtml(article.status)}</span>
        <span>${escapeHtml(article.category_name)}</span>
        <span>${escapeHtml(article.slug)}</span>
      </div>
      <h3>${escapeHtml(article.title)}</h3>
      <p>${escapeHtml(article.summary)}</p>
      <div class="button-row">
        <button type="button" data-edit="${escapeHtml(article.article_id)}">Edit</button>
        <button type="button" data-publish="${escapeHtml(article.article_id)}">Publish</button>
        <button type="button" data-unpublish="${escapeHtml(article.article_id)}">Unpublish</button>
      </div>
    </article>
  `).join("");

  articleList.querySelectorAll("[data-edit]").forEach((button) => {
    button.addEventListener("click", () => {
      const article = payload.find((item) => item.article_id === button.dataset.edit);
      if (article) {
        fillForm(article);
      }
    });
  });

  articleList.querySelectorAll("[data-publish]").forEach((button) => {
    button.addEventListener("click", async () => {
      const result = await requestJson(`/api/admin/news/${button.dataset.publish}/publish`, {
        method: "POST",
        transport: state.transport,
        csrf: true
      });
      setResponse("publish", result);
      await loadArticles();
    });
  });

  articleList.querySelectorAll("[data-unpublish]").forEach((button) => {
    button.addEventListener("click", async () => {
      const result = await requestJson(`/api/admin/news/${button.dataset.unpublish}/unpublish`, {
        method: "POST",
        transport: state.transport,
        csrf: true
      });
      setResponse("unpublish", result);
      await loadArticles();
    });
  });
}

document.getElementById("editor-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const articleId = articleIdInput.value;
  const method = articleId ? "PUT" : "POST";
  const path = articleId ? `/api/admin/news/${articleId}` : "/api/admin/news";

  const payload = await requestJson(path, {
    method,
    transport: state.transport,
    csrf: true,
    body: currentPayload()
  });
  setResponse(articleId ? "update" : "create", payload);
  if (payload.article_id) {
    fillForm(payload);
  }
  await loadArticles();
});

document.getElementById("publish-btn").addEventListener("click", async () => {
  if (!articleIdInput.value) {
    setResponse("publish", "Select or create an article first.");
    return;
  }
  const payload = await requestJson(`/api/admin/news/${articleIdInput.value}/publish`, {
    method: "POST",
    transport: state.transport,
    csrf: true
  });
  setResponse("publish", payload);
  await loadArticles();
});

document.getElementById("unpublish-btn").addEventListener("click", async () => {
  if (!articleIdInput.value) {
    setResponse("unpublish", "Select or create an article first.");
    return;
  }
  const payload = await requestJson(`/api/admin/news/${articleIdInput.value}/unpublish`, {
    method: "POST",
    transport: state.transport,
    csrf: true
  });
  setResponse("unpublish", payload);
  await loadArticles();
});

document.getElementById("reset-btn").addEventListener("click", () => {
  fillForm({});
});

transportInput.addEventListener("change", () => {
  state.transport = transportInput.value;
  syncIdentity();
});

accessInput.addEventListener("input", () => {
  state.accessToken = accessInput.value.trim();
});

bootstrapIdentity()
  .then(loadArticles)
  .catch((error) => {
    setResponse("bootstrap", error.message);
  });
