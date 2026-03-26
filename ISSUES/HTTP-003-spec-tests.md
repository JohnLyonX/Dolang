# HTTP-003 HTTP 专项 Spec 测试补全

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P1
**状态**: Open

---

## 问题概述

Dolang HTTP 服务端核心功能（路径参数、查询参数、`$RES` 状态码、`$SET_HDR`）均无专项 spec 测试覆盖，
存在以下风险：
- 后续改动可能悄无声息地破坏已有行为
- 新功能开发时无基准用例参照
- 外部贡献者难以验证行为是否符合预期

HTTP 相关测试与纯语言 spec 测试不同，需要启动真实 HTTP 服务器并发请求，
需要评估是否在 `tests/` 中建立独立的 `http_integration/` 测试分组。

---

## TODO 清单

**测试目录规划**
- [ ] 确认 HTTP 集成测试放置位置：
  - 方案 A：`tests/spec/valid/http/` — 与 stdlib spec 测试风格一致（测试 .dol 文件）
  - 方案 B：`tests/http_integration/` — 独立 Rust 集成测试，启动服务器 + 发 HTTP 请求断言
  - 建议：方案 B 更贴近真实行为验证，但成本更高；方案 A 可验证解析和运行逻辑，但无法验证真实响应头

**测试用例（.dol 或 Rust 集成测试）**

- [ ] **路径参数**：`$GET("/users/:id")` 中 `id` 正确注入为变量
- [ ] **多段路径参数**：`$GET("/org/:org/repo/:repo")` 中两个参数均正确注入
- [ ] **查询参数**：`?page=1&size=10` 中 `page`、`size` 正确注入为变量
- [ ] **`$RES` 状态码**：`$RES(404, "not found")` 真实响应 HTTP 404
- [ ] **`$RES` 201**：`$RES(201, {"id": 1})` 真实响应 HTTP 201
- [ ] **`$SET_HDR`**：响应头 `X-Custom: value` 在响应中可读（依赖 HTTP-001 完成）
- [ ] **`$HDR` 读取**：请求头 `X-Test: hello` 经 `$HDR("X-Test")` 可读到 `"hello"`
- [ ] **body 注入**：POST 请求体 `{"name":"Alice"}` 在 handler 中 `body["name"]` 为 `"Alice"`
- [ ] **错误处理**：handler 内 `$throw "oops"` 未捕获时响应 HTTP 500
- [ ] **模块挂载**：`$HTTP("/v1").link("routers/api")` 路由正确前缀

**文档**
- [ ] `docs/CHANGELOG.md`：更新测试覆盖说明

---

## 验收标准

- [ ] `cargo test` 全部通过（含新增 HTTP 集成测试）
- [ ] 路径参数、查询参数注入有测试覆盖
- [ ] `$RES(404, ...)` 等非 200 状态码有测试覆盖
- [ ] `$SET_HDR` 响应头有测试覆盖（依赖 HTTP-001 完成后补充）
- [ ] 测试可在 CI 环境中稳定运行（无随机端口冲突）

---

## 评论

<!-- 由 @用户 填写 -->
<!-- 请确认 HTTP 集成测试的组织方案（方案 A：.dol spec 文件 vs 方案 B：Rust 集成测试） -->
<!-- 以及是否有端口分配策略偏好（固定端口 vs 随机端口 + 端口发现） -->
