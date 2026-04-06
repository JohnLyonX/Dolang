const state = {
  csrfToken: "",
  accessToken: "",
  refreshToken: "",
  transport: "cookie"
};

const statusBanner = document.getElementById("status-banner");
const responseOutput = document.getElementById("response-output");
const transportInput = document.getElementById("transport");
const csrfInput = document.getElementById("csrf-token");
const accessInput = document.getElementById("access-token");
const refreshInput = document.getElementById("refresh-token");
const workspaceInput = document.getElementById("workspace-id");
const projectNameInput = document.getElementById("project-name");

function pretty(value) {
  if (typeof value === "string") {
    return value;
  }
  return JSON.stringify(value, null, 2);
}

function setBanner(kind, text) {
  statusBanner.className = `status-banner ${kind}`;
  statusBanner.textContent = text;
}

function setResponse(kind, label, payload) {
  const text = typeof payload === "string" ? payload : pretty(payload);
  setBanner(kind, label);
  responseOutput.textContent = text;
}

function syncStateView() {
  transportInput.value = state.transport;
  csrfInput.value = state.csrfToken;
  accessInput.value = state.accessToken;
  refreshInput.value = state.refreshToken;
}

function updateTokens(payload) {
  if (payload && typeof payload === "object") {
    if (payload.csrf_token) {
      state.csrfToken = payload.csrf_token;
    }
    if (payload.access_token) {
      state.accessToken = payload.access_token;
    }
    if (payload.refresh_token) {
      state.refreshToken = payload.refresh_token;
    }
    if (payload.session && payload.session.csrf_token) {
      state.csrfToken = payload.session.csrf_token;
    }
  }
  syncStateView();
}

async function requestJson(path, options = {}) {
  const method = options.method || "GET";
  const transport = options.transport || "cookie";
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
    method,
    credentials: transport === "bearer" ? "omit" : "include",
    headers,
    body: options.body != null ? JSON.stringify(options.body) : undefined
  });

  const raw = await response.text();
  let payload = raw;
  try {
    payload = raw ? JSON.parse(raw) : {};
  } catch (error) {
    payload = raw;
  }

  if (!response.ok) {
    const message = typeof payload === "string" ? payload : pretty(payload);
    throw new Error(`${response.status} ${response.statusText}\n${message}`);
  }

  return payload;
}

async function bootstrapCookieSession() {
  try {
    const payload = await requestJson("/me", { transport: "cookie" });
    if (payload && payload.authenticated === true) {
      updateTokens(payload);
      setResponse("neutral", "Recovered cookie session", payload);
    }
  } catch (error) {
    // Ignore missing, expired, or unauthenticated cookie sessions at startup.
  }
}

async function runAction(label, action) {
  try {
    const payload = await action();
    updateTokens(payload);
    setResponse("success", label, payload);
  } catch (error) {
    setResponse("error", label, error.message);
  }
}

document.getElementById("login-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const username = document.getElementById("username").value;
  const password = document.getElementById("password").value;
  await bootstrapCookieSession();

  await runAction("POST /auth/login", async () => (
    requestJson("/auth/login", {
      method: "POST",
      transport: "cookie",
      csrf: true,
      body: { username, password }
    })
  ));
});

transportInput.addEventListener("change", () => {
  state.transport = transportInput.value;
  syncStateView();
});

document.getElementById("me-cookie").addEventListener("click", async () => {
  await runAction("GET /me via cookie", async () => requestJson("/me", { transport: "cookie" }));
});

document.getElementById("me-bearer").addEventListener("click", async () => {
  await runAction("GET /me via bearer", async () => requestJson("/me", { transport: "bearer" }));
});

document.getElementById("admin-cookie").addEventListener("click", async () => {
  await runAction("GET /admin/audit via cookie", async () => (
    requestJson("/admin/audit", { transport: "cookie" })
  ));
});

document.getElementById("admin-bearer").addEventListener("click", async () => {
  await runAction("GET /admin/audit via bearer", async () => (
    requestJson("/admin/audit", { transport: "bearer" })
  ));
});

document.getElementById("refresh").addEventListener("click", async () => {
  const transport = state.transport;
  await runAction("POST /auth/refresh", async () => (
    requestJson("/auth/refresh", {
      method: "POST",
      transport,
      csrf: true,
      body: { refresh_token: state.refreshToken }
    })
  ));
});

document.getElementById("logout").addEventListener("click", async () => {
  const transport = state.transport;
  await runAction("POST /auth/logout", async () => (
    requestJson("/auth/logout", {
      method: "POST",
      transport,
      csrf: true,
      body: { refresh_token: state.refreshToken }
    })
  ));
  state.csrfToken = "";
  state.accessToken = "";
  state.refreshToken = "";
  syncStateView();
});

document.getElementById("workspace-cookie").addEventListener("click", async () => {
  await runAction("POST /workspaces/current/switch via cookie", async () => (
    requestJson("/workspaces/current/switch", {
      method: "POST",
      transport: "cookie",
      csrf: true,
      body: { workspace_id: workspaceInput.value }
    })
  ));
});

document.getElementById("workspace-bearer").addEventListener("click", async () => {
  await runAction("POST /workspaces/current/switch via bearer", async () => (
    requestJson("/workspaces/current/switch", {
      method: "POST",
      transport: "bearer",
      body: { workspace_id: workspaceInput.value }
    })
  ));
});

document.getElementById("project-cookie").addEventListener("click", async () => {
  await runAction("POST /api/projects via cookie", async () => (
    requestJson("/api/projects", {
      method: "POST",
      transport: "cookie",
      csrf: true,
      body: { name: projectNameInput.value }
    })
  ));
});

document.getElementById("project-bearer").addEventListener("click", async () => {
  await runAction("POST /api/projects via bearer", async () => (
    requestJson("/api/projects", {
      method: "POST",
      transport: "bearer",
      body: { name: projectNameInput.value }
    })
  ));
});

syncStateView();
bootstrapCookieSession();
