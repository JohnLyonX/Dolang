const state = {
  currentTab: "products",
  listData: [],
  selectedItem: null,
  formMode: "create",
  loading: false,
  errorMessage: "",
  successMessage: "",
  categories: [],
  orderDetail: null
};

const toolbarEl = document.getElementById("toolbar");
const contentEl = document.getElementById("content");
const modalEl = document.getElementById("modal");
const messageEl = document.getElementById("message");

const tabs = [
  { key: "products", title: "Products", subtitle: "Manage catalog records and pricing." },
  { key: "users", title: "Users", subtitle: "Maintain customer records and contact data." },
  { key: "orders", title: "Orders", subtitle: "Inspect order activity without edit controls." }
];

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

async function apiGet(path) {
  const response = await fetch(path);
  const payload = await response.json();
  if (!response.ok || payload.error) {
    throw new Error(payload.error || `Request failed: ${response.status}`);
  }
  return payload;
}

async function apiSend(path, method, payload) {
  const response = await fetch(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload)
  });
  const result = await response.json();
  if (!response.ok || result.error) {
    throw new Error(result.error || `Request failed: ${response.status}`);
  }
  return result;
}

function setMessage(kind, text) {
  state.successMessage = kind === "success" ? text : "";
  state.errorMessage = kind === "error" ? text : "";
  renderMessage();
}

function renderMessage() {
  const text = state.errorMessage || state.successMessage;
  if (!text) {
    messageEl.innerHTML = "";
    return;
  }

  const kind = state.errorMessage ? "error" : "success";
  messageEl.innerHTML = `<div class="message-banner ${kind}">${escapeHtml(text)}</div>`;
}

function currentTabMeta() {
  return tabs.find((tab) => tab.key === state.currentTab);
}

function renderToolbar() {
  const active = currentTabMeta();
  toolbarEl.innerHTML = `
    <div class="hero">
      <h1>Dolang Admin</h1>
      <p>${escapeHtml(active.subtitle)}</p>
    </div>
    <nav class="nav">
      ${tabs.map((tab) => `
        <button class="${tab.key === state.currentTab ? "active" : ""}" data-tab="${tab.key}">
          ${escapeHtml(tab.title)}
        </button>
      `).join("")}
    </nav>
  `;

  toolbarEl.querySelectorAll("[data-tab]").forEach((button) => {
    button.addEventListener("click", async () => {
      if (state.currentTab === button.dataset.tab) {
        return;
      }
      state.currentTab = button.dataset.tab;
      state.selectedItem = null;
      state.formMode = "create";
      state.orderDetail = null;
      setMessage("success", "");
      await loadCurrentTab();
    });
  });
}

function tableHeadForCurrentTab() {
  if (state.currentTab === "users") {
    return "<tr><th>ID</th><th>Name</th><th>Email</th><th>Created</th><th>Actions</th></tr>";
  }
  if (state.currentTab === "orders") {
    return "<tr><th>ID</th><th>User</th><th>Status</th><th>Created</th><th>Actions</th></tr>";
  }
  return "<tr><th>ID</th><th>Name</th><th>Price</th><th>Category</th><th>Actions</th></tr>";
}

function rowsForCurrentTab() {
  if (state.currentTab === "users") {
    return state.listData.map((item) => `
      <tr class="${state.selectedItem && state.selectedItem.id === item.id ? "selected" : ""}">
        <td>${escapeHtml(item.id)}</td>
        <td>${escapeHtml(item.name)}</td>
        <td>${escapeHtml(item.email)}</td>
        <td>${escapeHtml(item.created_at)}</td>
        <td>
          <div class="table-actions">
            <button class="ghost" data-edit="${item.id}">Edit</button>
            <button class="danger" data-delete="${item.id}">Delete</button>
          </div>
        </td>
      </tr>
    `).join("");
  }

  if (state.currentTab === "orders") {
    return state.listData.map((item) => `
      <tr class="${state.selectedItem && state.selectedItem.id === item.id ? "selected" : ""}">
        <td>${escapeHtml(item.id)}</td>
        <td>${escapeHtml(item.user_id)}</td>
        <td>${escapeHtml(item.status)}</td>
        <td>${escapeHtml(item.created_at)}</td>
        <td>
          <div class="table-actions">
            <button class="ghost" data-view="${item.id}">View</button>
          </div>
        </td>
      </tr>
    `).join("");
  }

  return state.listData.map((item) => `
    <tr class="${state.selectedItem && state.selectedItem.id === item.id ? "selected" : ""}">
      <td>${escapeHtml(item.id)}</td>
      <td>${escapeHtml(item.name)}</td>
      <td>${escapeHtml(item.price)}</td>
      <td>${escapeHtml(categoryName(item.category_id))}</td>
      <td>
        <div class="table-actions">
          <button class="ghost" data-edit="${item.id}">Edit</button>
          <button class="danger" data-delete="${item.id}">Delete</button>
        </div>
      </td>
    </tr>
  `).join("");
}

function categoryName(categoryId) {
  const match = state.categories.find((item) => item.id === categoryId);
  return match ? match.name : `#${categoryId}`;
}

function renderFormCard() {
  if (state.currentTab === "orders") {
    return renderOrdersDetail();
  }

  const item = state.selectedItem || {};
  const title = state.formMode === "edit" ? "Edit Record" : "Create Record";

  if (state.currentTab === "users") {
    return `
      <section class="detail-card">
        <form class="form-card" id="editor-form">
          <div class="form-head">
            <div>
              <h3>${title}</h3>
              <p class="subtle">Users are created and updated through the JSON API.</p>
            </div>
            ${state.formMode === "edit" ? '<button type="button" class="ghost" id="reset-form">New</button>' : ""}
          </div>
          <div class="field">
            <label for="user-name">Name</label>
            <input id="user-name" name="name" value="${escapeHtml(item.name || "")}" required>
          </div>
          <div class="field">
            <label for="user-email">Email</label>
            <input id="user-email" name="email" type="email" value="${escapeHtml(item.email || "")}" required>
          </div>
          <div class="form-actions">
            <button class="action" type="submit">${state.formMode === "edit" ? "Save User" : "Create User"}</button>
          </div>
        </form>
      </section>
    `;
  }

  return `
    <section class="detail-card">
      <form class="form-card" id="editor-form">
        <div class="form-head">
          <div>
            <h3>${title}</h3>
            <p class="subtle">Products use the same write path as the admin APIs.</p>
          </div>
          ${state.formMode === "edit" ? '<button type="button" class="ghost" id="reset-form">New</button>' : ""}
        </div>
        <div class="field">
          <label for="product-name">Name</label>
          <input id="product-name" name="name" value="${escapeHtml(item.name || "")}" required>
        </div>
        <div class="field">
          <label for="product-price">Price</label>
          <input id="product-price" name="price" type="number" step="0.01" value="${escapeHtml(item.price || "")}" required>
        </div>
        <div class="field">
          <label for="product-category">Category</label>
          <select id="product-category" name="category_id" required>
            ${state.categories.map((category) => `
              <option value="${category.id}" ${Number(item.category_id) === Number(category.id) ? "selected" : ""}>
                ${escapeHtml(category.name)}
              </option>
            `).join("")}
          </select>
        </div>
        <div class="form-actions">
          <button class="action" type="submit">${state.formMode === "edit" ? "Save Product" : "Create Product"}</button>
        </div>
      </form>
    </section>
  `;
}

function renderOrdersDetail() {
  if (!state.orderDetail) {
    return `
      <section class="detail-card">
        <h3>Order Detail</h3>
        <p class="empty">Select an order row to inspect the user and line items.</p>
      </section>
    `;
  }

  const detail = state.orderDetail;
  const items = detail.items || [];

  return `
    <section class="detail-card">
      <h3>Order Detail</h3>
      <div class="detail-grid">
        <div class="detail-block">
          <h4>Order</h4>
          <dl>
            <dt>ID</dt><dd>${escapeHtml(detail.order.id)}</dd>
            <dt>Status</dt><dd>${escapeHtml(detail.order.status)}</dd>
            <dt>User ID</dt><dd>${escapeHtml(detail.order.user_id)}</dd>
            <dt>Created</dt><dd>${escapeHtml(detail.order.created_at)}</dd>
          </dl>
        </div>
        <div class="detail-block">
          <h4>User</h4>
          <dl>
            <dt>Name</dt><dd>${escapeHtml(detail.user.name)}</dd>
            <dt>Email</dt><dd>${escapeHtml(detail.user.email)}</dd>
            <dt>Created</dt><dd>${escapeHtml(detail.user.created_at)}</dd>
          </dl>
        </div>
        <div class="detail-block">
          <h4>Items</h4>
          ${items.length === 0 ? '<p class="empty">No line items.</p>' : `
            <div class="table-wrap">
              <table>
                <thead>
                  <tr><th>ID</th><th>Product</th><th>Qty</th><th>Price</th></tr>
                </thead>
                <tbody>
                  ${items.map((item) => `
                    <tr>
                      <td>${escapeHtml(item.id)}</td>
                      <td>${escapeHtml(item.product_name)}</td>
                      <td>${escapeHtml(item.quantity)}</td>
                      <td>${escapeHtml(item.price)}</td>
                    </tr>
                  `).join("")}
                </tbody>
              </table>
            </div>
          `}
        </div>
      </div>
    </section>
  `;
}

function renderContent() {
  const active = currentTabMeta();
  contentEl.innerHTML = `
    <div class="shell">
      <section class="panel">
        <div class="panel-head">
          <div>
            <h2>${escapeHtml(active.title)}</h2>
            <p>${escapeHtml(active.subtitle)}</p>
          </div>
          ${state.currentTab === "orders"
            ? '<button class="ghost" id="refresh-list">Refresh</button>'
            : `<button class="action" id="new-item">New ${escapeHtml(active.title.slice(0, -1))}</button>`}
        </div>
        <div class="table-wrap">
          <table>
            <thead>${tableHeadForCurrentTab()}</thead>
            <tbody>${rowsForCurrentTab() || `<tr><td colspan="5" class="empty">No records.</td></tr>`}</tbody>
          </table>
        </div>
      </section>
      ${renderFormCard()}
    </div>
  `;

  modalEl.innerHTML = "";
  bindContentActions();
}

function bindContentActions() {
  const newButton = document.getElementById("new-item");
  if (newButton) {
    newButton.addEventListener("click", () => {
      state.selectedItem = null;
      state.formMode = "create";
      renderContent();
    });
  }

  const refreshButton = document.getElementById("refresh-list");
  if (refreshButton) {
    refreshButton.addEventListener("click", async () => {
      await loadCurrentTab();
    });
  }

  const resetButton = document.getElementById("reset-form");
  if (resetButton) {
    resetButton.addEventListener("click", () => {
      state.selectedItem = null;
      state.formMode = "create";
      renderContent();
    });
  }

  const form = document.getElementById("editor-form");
  if (form) {
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      await handleSubmit(new FormData(form));
    });
  }

  contentEl.querySelectorAll("[data-edit]").forEach((button) => {
    button.addEventListener("click", () => {
      const id = Number(button.dataset.edit);
      const match = state.listData.find((item) => Number(item.id) === id);
      state.selectedItem = match || null;
      state.formMode = "edit";
      renderContent();
    });
  });

  contentEl.querySelectorAll("[data-delete]").forEach((button) => {
    button.addEventListener("click", async () => {
      const id = Number(button.dataset.delete);
      await handleDelete(id);
    });
  });

  contentEl.querySelectorAll("[data-view]").forEach((button) => {
    button.addEventListener("click", async () => {
      const id = Number(button.dataset.view);
      state.selectedItem = state.listData.find((item) => Number(item.id) === id) || null;
      state.orderDetail = await apiGet(`/api/orders/${id}/detail`);
      renderContent();
    });
  });
}

async function handleSubmit(formData) {
  try {
    if (state.currentTab === "users") {
      const payload = {
        name: formData.get("name"),
        email: formData.get("email")
      };

      if (state.formMode === "edit" && state.selectedItem) {
        await apiSend(`/api/users/${state.selectedItem.id}`, "PUT", payload);
        setMessage("success", "User updated.");
      } else {
        await apiSend("/api/users", "POST", payload);
        setMessage("success", "User created.");
      }
    } else {
      const payload = {
        name: formData.get("name"),
        price: Number(formData.get("price")),
        category_id: Number(formData.get("category_id"))
      };

      if (state.formMode === "edit" && state.selectedItem) {
        await apiSend(`/api/products/${state.selectedItem.id}`, "PUT", payload);
        setMessage("success", "Product updated.");
      } else {
        await apiSend("/api/products", "POST", payload);
        setMessage("success", "Product created.");
      }
    }

    state.selectedItem = null;
    state.formMode = "create";
    await loadCurrentTab();
  } catch (error) {
    setMessage("error", error.message);
  }
}

async function handleDelete(id) {
  try {
    if (state.currentTab === "users") {
      await apiSend(`/api/users/${id}`, "DELETE", {});
      setMessage("success", `User ${id} deleted.`);
    } else {
      await apiSend(`/api/products/${id}`, "DELETE", {});
      setMessage("success", `Product ${id} deleted.`);
    }

    if (state.selectedItem && Number(state.selectedItem.id) === id) {
      state.selectedItem = null;
      state.formMode = "create";
    }

    await loadCurrentTab();
  } catch (error) {
    setMessage("error", error.message);
  }
}

async function loadCurrentTab() {
  try {
    state.loading = true;
    renderToolbar();
    contentEl.innerHTML = '<section class="panel"><p class="empty">Loading...</p></section>';

    if (state.currentTab === "products") {
      const [products, categories] = await Promise.all([
        apiGet("/api/products"),
        apiGet("/api/categories")
      ]);
      state.listData = products;
      state.categories = categories;
      state.orderDetail = null;
    } else if (state.currentTab === "users") {
      state.listData = await apiGet("/api/users");
      state.categories = [];
      state.orderDetail = null;
    } else {
      state.listData = await apiGet("/api/orders");
      state.categories = [];
      state.orderDetail = null;
    }

    renderToolbar();
    renderContent();
  } catch (error) {
    setMessage("error", error.message);
    contentEl.innerHTML = `<section class="panel"><p class="empty">${escapeHtml(error.message)}</p></section>`;
  } finally {
    state.loading = false;
  }
}

loadCurrentTab();
