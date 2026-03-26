# HTTP-001 $SET_HDR 响应头设置

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P1
**状态**: Open

---

## 问题概述

Dolang 目前支持读取请求头（`$HDR("Header-Name")`），但无法向响应写入自定义 Header。
以下常见场景均被阻塞：
- `Cache-Control` / `ETag` — 缓存控制
- `Content-Disposition` — 文件下载文件名
- `X-Request-Id` — 请求链路追踪
- `Authorization` 相关响应头 — 认证流程

本 Issue 新增 `$SET_HDR(name, value)` 语句，允许在 HTTP handler 中写入响应头。

---

## 语法

```dolang
$GET("/api/data") get_data() {
    $SET_HDR("Cache-Control", "max-age=3600");
    $SET_HDR("X-Request-Id", uuid.v4());
    $# {"data": 42};
}
```

---

## TODO 清单

**前端（词法 + 语法 + AST）**
- [ ] `crates/dolang-frontend/src/token/token.rs`：新增 `SetHdr` 变体
- [ ] `crates/dolang-frontend/src/lexer/lexer.rs`：在 `read_dollar()` 中匹配 `$SET_HDR` → `Type::SetHdr`
- [ ] `crates/dolang-frontend/src/ast/ast.rs`：新增 `SetHdrStmt { name: Box<Expr>, value: Box<Expr> }` 和 `Stmt::SetHdr(SetHdrStmt)` 变体
- [ ] `crates/dolang-frontend/src/parser/stmt/http.rs`：实现 `parse_set_hdr()` — 解析 `$SET_HDR(expr, expr);`
- [ ] `crates/dolang-frontend/src/parser/stmt/mod.rs`：在分发函数中处理 `Type::SetHdr`

**运行时（执行 + 上下文）**
- [ ] `crates/dolang-runtime/src/runtime/context.rs`：`RuntimeContext` 新增 `response_headers: IndexMap<String, String>` 字段（初始为空 Map）
- [ ] `crates/dolang-runtime/src/interpreter/exec/http.rs`：新增 `handle_set_hdr()` — 求值两个表达式并写入 `context.response_headers`
- [ ] `crates/dolang-runtime/src/interpreter/exec/mod.rs`：分发 `Stmt::SetHdr(stmt)` → `handle_set_hdr(stmt, state, context, w)`
- [ ] `crates/dolang-runtime/src/runtime/http.rs`：`HandlerOutput` 新增 `response_headers: IndexMap<String, String>`；`execute_http_route` 执行后将 `context.response_headers` 写入 output；每次 handler 执行前清空 context 中的 response_headers

**CLI / HTTP 服务器层**
- [ ] `crates/dolang-cli/src/server.rs`：Axum handler 中将 `HandlerOutput.response_headers` 逐条追加到 `axum::response::Response`

**测试**
- [ ] 新建 `tests/spec/valid/http/set_hdr.dol` 覆盖基本设置行为（通过实际 HTTP 请求验证）

**文档**
- [ ] `docs/reference/http.md`：新增 `$SET_HDR` 章节（语法、参数说明、示例、注意事项）
- [ ] `docs/CHANGELOG.md`：更新 Unreleased/Added

---

## 验收标准

- [ ] `cargo build` 编译通过
- [ ] `cargo test` 全部通过
- [ ] `$SET_HDR("X-Foo", "bar")` 在 HTTP 响应头中可读到 `X-Foo: bar`
- [ ] 同一 handler 中多次调用 `$SET_HDR` 可设置多个不同响应头
- [ ] 同名 Header 后设覆盖先设（Map 语义）
- [ ] `$SET_HDR` 在非 HTTP handler 上下文中调用时抛出运行时错误（而非静默忽略）
- [ ] name / value 为表达式，支持变量和 f-string

---

## 评论

<!-- 由 @用户 填写，记录决策、阻塞、进展 -->
