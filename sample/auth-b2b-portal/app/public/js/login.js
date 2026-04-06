const loginForm = document.getElementById("login-form");
const statusEl = document.getElementById("login-status");

function renderStatus(text, isError = false) {
  statusEl.textContent = text;
  statusEl.dataset.state = isError ? "error" : "neutral";
}

async function requestJson(path, options = {}) {
  const response = await fetch(path, {
    method: options.method || "GET",
    credentials: "include",
    headers: options.body ? { "Content-Type": "application/json" } : {},
    body: options.body ? JSON.stringify(options.body) : undefined
  });
  const raw = await response.text();
  const payload = raw ? JSON.parse(raw) : {};
  if (!response.ok) {
    throw new Error(payload.error || payload.message || `Request failed: ${response.status}`);
  }
  return payload;
}

loginForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  renderStatus("Signing in...");
  try {
    await requestJson("/auth/login", {
      method: "POST",
      body: {
        username: document.getElementById("username").value,
        password: document.getElementById("password").value
      }
    });
    renderStatus("Sign-in succeeded. Redirecting to newsroom admin.");
    window.location.href = "/news-admin";
  } catch (error) {
    renderStatus(error.message, true);
  }
});
