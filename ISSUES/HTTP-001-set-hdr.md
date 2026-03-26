# HTTP-001 `@SET_HDR` 响应头设置

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P1
**状态**: Done

---

## 问题概述

Dolang 目前支持读取请求头（`$HDR("Header-Name")`），但无法向响应写入自定义 Header。
以下常见场景均被阻塞：
- `Cache-Control` / `ETag` — 缓存控制
- `Content-Disposition` — 文件下载文件名
- `X-Request-Id` — 请求链路追踪
- `Authorization` 相关响应头 — 认证流程

本 Issue 新增 `@SET_HDR({ "Header-Name": "value", ... })` 注解，允许在路由级或块级声明固定响应头。

---

## 语法方案（已确认：`@SET_HDR` 注解方式）

采用与 `@CORS` 一致的注解风格，支持**路由级**和**块级**两个层次，**不设计全局级**。

### 路由级

```dolang
@SET_HDR({ "Cache-Control": "max-age=3600" })
@SET_HDR({ "X-Frame-Options": "DENY" })
$GET("/api/data") getData() {
    $# {"data": 42};
}
```

### 块级（作用于块内所有路由）

```dolang
@SET_HDR({ "X-Frame-Options": "DENY" })
@SET_HDR({ "X-Content-Type-Options": "nosniff" })
$HTTP("/api") {
    $GET("/users") getUsers() { $# []; }
    $GET("/items") getItems() { $# []; }
}
```

### 块级 + 路由级混用（路由级覆盖同名块级）

```dolang
@SET_HDR({ "Cache-Control": "no-cache" })
$HTTP("/api") {
    // 此路由 Cache-Control 被路由级覆盖为 max-age=3600
    @SET_HDR({ "Cache-Control": "max-age=3600" })
    $GET("/static") getStatic() { $# {"ok": true}; }

    // 此路由继承块级 Cache-Control: no-cache
    $GET("/dynamic") getDynamic() { $# {"ok": true}; }
}
```

### 优先级规则（同名 Header 近端覆盖，不同名叠加）

```
路由级 @SET_HDR  ──存在──▶  使用路由级值（覆盖同名块级）
      │ 不存在
      ▼
块级 @SET_HDR   ──存在──▶  使用块级值
      │ 不存在
      ▼
不设置          ──────────▶  无该响应头（与当前行为一致）
```

> **不支持全局级**（`$main()` 前）：响应头策略属于路由/块范畴，在入口级声明无实际意义。

---

## TODO 清单

### 1. Frontend — Token

- [x] `crates/dolang-frontend/src/token/token.rs`
  - 新增 `AtSetHdr` token（`@SET_HDR`）
  - 注：`At` token 已在 HTTP-002 中引入，此处复用

### 2. Frontend — Lexer

- [x] `crates/dolang-frontend/src/lexer/lexer.rs`
  - 在 `@` 开头分支中补充 `@SET_HDR` 的 longest-match-first 匹配：
    ```
    @CORS     → AtCors
    @SET_HDR  → AtSetHdr
    @         → At
    ```

### 3. Frontend — AST

- [x] `crates/dolang-frontend/src/ast/ast.rs`
  - 新增 `SetHdrEntry` 结构体（单条 Header）：
    ```rust
    pub struct SetHdrEntry {
        pub name: String,
        pub value: String,
    }
    ```
  - `HttpFnStmt` 新增字段：`pub headers: Vec<SetHdrEntry>`
  - `HttpBlockStmt` 新增字段：`pub headers: Vec<SetHdrEntry>`

### 4. Frontend — Parser

- [x] `crates/dolang-frontend/src/parser/stmt/http.rs`
  - `parse_http_fn()` 前识别零条或多条 `@SET_HDR({ "Header": "value", ... })` 注解，写入 `HttpFnStmt.headers`
  - `parse_http_block()` 前识别零条或多条 `@SET_HDR({ "Header": "value", ... })` 注解，写入 `HttpBlockStmt.headers`
  - 注：`@CORS` 与 `@SET_HDR` 可同时出现，顺序不限，解析器需同时识别两者

### 5. Frontend — Diagnostic Codes

- [x] `crates/dolang-frontend/src/diagnostics/codes.rs`
  - 新增：
    ```rust
    pub const PARSE_SET_HDR_INVALID_POSITION: &str = "DOL-P006";
    pub const PARSE_SET_HDR_INVALID_SYNTAX:   &str = "DOL-P007";
    pub const CONFIG_HDR_INVALID:             &str = "DOL-C003";
    ```

### 6. Runtime — HttpRoute

- [x] `crates/dolang-runtime/src/interpreter/mod.rs`
  - `HttpRoute` 新增字段：`pub response_headers: Vec<(String, String)>`

### 7. Runtime — Interpreter

- [x] `crates/dolang-runtime/src/interpreter/exec/http.rs`
  - `handle_http_fn()`：将 `stmt.headers` 写入 `HttpRoute.response_headers`
  - `handle_http_block()`：将块级 `headers` 合并到每个子 `HttpRoute.response_headers`（路由级同名覆盖块级，不同名叠加）

### 8. Backend — Axum

- [x] `crates/dolang-cli/src/backends/axum_backend.rs`
  - 在 `build_http_response()` 中将 `route.response_headers` 逐条追加到 `axum::response::Response`

---

## 错误机制

### 错误码

| 错误码 | 阶段 | 触发场景 |
|--------|------|---------|
| `DOL-P006` | 解析期 | `@SET_HDR` 出现在非法位置（`$main()`、普通函数、变量等） |
| `DOL-P007` | 解析期 | 参数不是合法对象字面量（如传入变量、表达式、旧双字符串形式） |
| `DOL-C003` | 启动期 | Header 名称包含非法字符（如空格、控制字符） |

#### DOL-P006：位置非法

```
error[DOL-P006]: @SET_HDR annotation is not allowed here
  --> main.dol:3:1
  = note: @SET_HDR can only annotate $HTTP blocks or HTTP route functions ($GET, $POST, ...)
  = note: global-level response headers are not supported
```

#### DOL-P007：参数语法错误

```
error[DOL-P007]: invalid @SET_HDR arguments
  --> main.dol:5:1
  = note: @SET_HDR requires a single object literal: @SET_HDR({ "Header-Name": "value", ... })
  = note: dynamic expressions are not supported in annotations
```

#### DOL-C003：Header 名称非法（启动期 Warning）

```
warning[DOL-C003]: invalid response header name skipped: "X Bad Header"
  = note: in @SET_HDR at main.dol:2:1
  = note: header names must not contain spaces or control characters (RFC 7230)
```

### 容灾机制

| 场景 | 行为 |
|------|------|
| 无任何 `@SET_HDR` 声明 | 正常启动，响应头与当前一致（向下兼容） |
| Header 名称包含非法字符 | `DOL-C003` Warning，跳过该条目，服务正常启动 |
| 块级与路由级同名冲突 | 路由级覆盖块级，无报错（预期行为） |
| 同一节点多条 `@SET_HDR` 同名 | 后声明的覆盖先声明的，打印 `DOL-C003` Warning |

---

## 验收标准

### 正确性

- [x] `cargo build` 编译通过
- [x] `cargo test` 全部通过
- [x] 路由级 `@SET_HDR({ "X-Foo": "bar" })` 在 HTTP 响应头中可读到 `X-Foo: bar`
- [x] 同一节点多条 `@SET_HDR` 可设置多个不同响应头（叠加）
- [x] 块级与路由级同名 Header 时，路由级值覆盖块级值
- [x] 块级与路由级不同名 Header 时，两者均出现在响应中

### 错误处理

- [x] `@SET_HDR` 声明在 `$main()` 前时，报 `DOL-P006` 并附带行列信息，程序不启动
- [x] `@SET_HDR` 声明在普通函数前时，报 `DOL-P006`，程序不启动
- [x] `@SET_HDR` 参数不是字符串字面量时，报 `DOL-P007`，程序不启动
- [x] Header 名称含非法字符时，报 `DOL-C003` Warning，服务正常启动

### 兼容性

- [x] 所有已有测试用例（无 `@SET_HDR` 的程序）仍通过，行为不变
- [x] `$HTTP.link()` 模块链接的路由正确继承块级 `@SET_HDR` 配置
- [x] 与 `@CORS` 同时声明时，两者互不干扰，顺序不限

---

## 评论

原方案 `$SET_HDR(name, value)` 作为 handler 内部语句已废弃。

改为 `@SET_HDR({ "Header-Name": "value", ... })` 注解形式，与 `@CORS` 风格一致，降低用户认知负担（只需学习一套 `@` 注解体系）。

**不设计全局级**：响应头策略属于路由/块范畴，在 `$main()` 入口级声明没有实际意义，避免引入无用的语言复杂度。
