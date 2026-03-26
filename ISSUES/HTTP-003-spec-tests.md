# HTTP-003 HTTP 专项 Spec 测试补全

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P1
**状态**: Done

---

## 问题概述

Dolang HTTP 服务端核心功能（路径参数、查询参数、`$RES` 状态码、`@SET_HDR`、`@CORS`）均无专项 spec 测试覆盖，
存在以下风险：
- 后续改动可能悄无声息地破坏已有行为
- 新功能开发时无基准用例参照
- 外部贡献者难以验证行为是否符合预期

HTTP 相关测试与纯语言 spec 测试不同，需要启动真实 HTTP 服务器并发请求，
需要评估是否在 `tests/` 中建立独立的 `http_integration/` 测试分组。

---

## TODO 清单

### 测试目录规划

- [x] 确认 HTTP 集成测试放置位置：
  - 方案 A：`tests/spec/valid/http/` — 与 stdlib spec 测试风格一致（测试 .dol 文件）
  - 方案 B：`tests/http_integration/` — 独立 Rust 集成测试，启动服务器 + 发 HTTP 请求断言
  - 建议：方案 B 更贴近真实行为验证，但成本更高；方案 A 可验证解析和运行逻辑，但无法验证真实响应头
  - **注**：`@CORS` 和 `@SET_HDR` 响应头测试必须使用方案 B（需要检查真实 HTTP 响应头）

---

### 测试用例

#### 基础路由

- [x] **路径参数**：`$GET("/users/:id")` 中 `id` 正确注入为变量
- [x] **多段路径参数**：`$GET("/org/:org/repo/:repo")` 中两个参数均正确注入
- [x] **查询参数**：`?page=1&size=10` 中 `page`、`size` 正确注入为变量
- [x] **body 注入**：POST 请求体 `{"name":"Alice"}` 在 handler 中 `body["name"]` 为 `"Alice"`
- [x] **`$HDR` 读取**：请求头 `X-Test: hello` 经 `$HDR("X-Test")` 可读到 `"hello"`
- [x] **`$RES` 状态码**：`$RES(404, "not found")` 真实响应 HTTP 404
- [x] **`$RES` 201**：`$RES(201, {"id": 1})` 真实响应 HTTP 201
- [x] **错误处理**：handler 内 `$throw "oops"` 未捕获时响应 HTTP 500
- [x] **模块挂载**：`$HTTP("/v1").link("routers/api")` 路由正确前缀

#### `@SET_HDR` 响应头（依赖 HTTP-001 完成）

- [x] **路由级单条**：`@SET_HDR({ "X-Custom": "value" })` 在响应头中可读到 `X-Custom: value`
- [x] **路由级多条**：多条 `@SET_HDR` 均出现在响应头中（叠加）
- [x] **块级继承**：块级 `@SET_HDR` 应用到块内所有路由
- [x] **块级 + 路由级同名覆盖**：路由级 `@SET_HDR({ "Cache-Control": "max-age=3600" })` 覆盖块级 `@SET_HDR({ "Cache-Control": "no-cache" })`，响应头为 `max-age=3600`
- [x] **块级 + 路由级不同名叠加**：两者 Header 均出现在响应中
- [x] **无 `@SET_HDR`**：响应头中无意外注入的自定义 Header（向下兼容）

#### `@CORS` 跨域（依赖 HTTP-002 完成）

- [x] **全局 `@CORS("*")`**：OPTIONS 预检请求返回 `Access-Control-Allow-Origin: *`
- [x] **全局细粒度配置**：指定 `origins` 白名单时，白名单内来源返回 CORS 头，白名单外来源不返回
- [x] **路由级覆盖全局**：路由级 `@CORS` 配置覆盖全局，其他路由仍用全局配置
- [x] **块级覆盖全局**：块级 `@CORS` 配置覆盖全局，块外路由仍用全局配置
- [x] **`credentials: true`**：响应头包含 `Access-Control-Allow-Credentials: true`
- [x] **无 `@CORS`**：无 `Access-Control-Allow-*` 响应头（向下兼容）

#### `@CORS` + `@SET_HDR` 组合

- [x] **同时声明**：`@CORS` 与 `@SET_HDR` 同时存在时，两者响应头均正确返回，互不干扰
- [x] **顺序无关**：`@CORS` 在 `@SET_HDR` 前后声明，行为一致

---

### 文档

- [x] `docs/CHANGELOG.md`：更新测试覆盖说明

---

## 验收标准

- [x] `cargo test` 全部通过（含新增 HTTP 集成测试）
- [x] 路径参数、查询参数注入有测试覆盖
- [x] `$RES(404, ...)` 等非 200 状态码有测试覆盖
- [x] `@SET_HDR` 路由级和块级响应头均有测试覆盖（依赖 HTTP-001 完成后补充）
- [x] `@CORS` 全局、块级、路由级优先级均有测试覆盖（依赖 HTTP-002 完成后补充）
- [x] `@CORS` + `@SET_HDR` 组合场景有测试覆盖
- [x] 测试可在 CI 环境中稳定运行（无随机端口冲突）

---

## 评论

<!-- 由 @用户 填写 -->
<!-- 请确认 HTTP 集成测试的组织方案（方案 A：.dol spec 文件 vs 方案 B：Rust 集成测试） -->
<!-- 以及是否有端口分配策略偏好（固定端口 vs 随机端口 + 端口发现） -->

`@SET_HDR` 和 `@CORS` 响应头验证必须通过真实 HTTP 请求检查响应头，方案 A（.dol spec 文件）无法覆盖，需采用方案 B（Rust 集成测试）。
