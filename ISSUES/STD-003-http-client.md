# STD-003 std.http HTTP 客户端

**Epic**: [EPIC-01](EPIC-01-stdlib-enhancement.md)
**优先级**: P2
**状态**: Open

---

## 问题概述

Dolang 目前只能**接收** HTTP 请求（作为 Web 服务器），但无法**发出** HTTP 请求。
在微服务架构和第三方 API 集成场景中，服务间调用是核心需求：
- 调用其他微服务接口
- 集成第三方 API（天气、支付、短信等）
- 服务编排（聚合多个下游响应）

本 Issue 实现 `std.http` 模块，提供同步 HTTP 客户端能力。

---

## TODO 清单

**依赖准备**
- [ ] 在 `crates/dolang-runtime/Cargo.toml` 添加 `reqwest = { version = "0.12", features = ["blocking", "json"] }`

**核心实现**
- [ ] 新建 `crates/dolang-runtime/src/stdlib_native/http_client.rs`，实现：
  - `http.get(url: String)` → Map
  - `http.get(url: String, headers: Map)` → Map（带请求头，重载）
  - `http.post(url: String, body: Map)` → Map
  - `http.post(url: String, body: Map, headers: Map)` → Map
  - `http.put(url: String, body: Map)` → Map
  - `http.delete(url: String)` → Map
  - `http.request(method: String, url: String, body: Map, headers: Map)` → Map（通用）

  返回值结构（统一 Map）：
  ```
  {
    "status": Int,          // HTTP 状态码，如 200
    "body": String,         // 响应体文本
    "headers": Map          // 响应头 Map<String, String>
  }
  ```

  网络错误、超时等抛出 Dolang 运行时错误，可被 `$try/$catch` 捕获。

**注册**
- [ ] 在 `crates/dolang-runtime/src/stdlib_native/mod.rs` 中：
  - `mod http_client;`
  - `register_http_client` 注册到 `"std.http"` 模块
  - 注意：模块名 `std.http` 与 HTTP 服务端语法不冲突（服务端是关键字，客户端是模块）

**测试**
- [ ] 新建 `tests/spec/valid/stdlib/http_client.dol`（使用 `httpbin.org` 或本地 mock）
  - 基本结构测试：返回值包含 `status`、`body`、`headers` 三个键
  - `http.get()` 返回 status 为 Int
  - 错误 URL 时 `$try/$catch` 可捕获错误
- [ ] 考虑网络不可用时的测试隔离方案（可跳过网络测试或用条件编译）

**文档**
- [ ] 更新 `docs/CHANGELOG.md` Unreleased/Added 区域

---

## 验收标准

- [ ] `cargo build` 编译通过
- [ ] `cargo test` 全部通过（网络相关测试可标记为 `#[ignore]` 待 CI 环境确定）
- [ ] `http.get("https://httpbin.org/get")` 返回 Map，`res["status"]` 为 `200`
- [ ] `http.post("https://httpbin.org/post", {"key": "val"})` 返回 Map，`res["status"]` 为 `200`
- [ ] 请求无效 URL 时，`$try { http.get("not-a-url") } $catch e { ... }` 可捕获错误
- [ ] `http.get` 和 `http.post` 可在 HTTP handler 内部调用

---

## 注意事项

- 使用 `reqwest::blocking` 避免引入 async 运行时，与 Dolang 同步解释器模型一致
- 超时建议默认 30 秒，后续可通过可选参数配置
- 模块名 `std.http` 与服务端 `$GET`/`$POST` 等关键字无冲突，因为模块通过 `$mod std.http;` 导入后以 `http.xxx()` 调用

---

## 评论

<!-- 由 @用户 填写，记录决策、阻塞、进展 -->
