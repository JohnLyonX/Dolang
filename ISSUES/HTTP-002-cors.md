# HTTP-002 CORS 支持

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P2
**状态**: Done

---

## 问题概述

前后端分离架构下，浏览器会对跨域请求发起 CORS 预检（OPTIONS 请求）。
目前 Dolang 不处理 CORS，导致前端直接调用 Dolang Web 服务时被浏览器拦截。

需要在语言层面提供 CORS 配置能力，让开发者能简洁地声明跨域策略。

---

## 语法方案（已确认：`@CORS` 注解方式）

采用新增 `@` 注解符号，支持**全局、块级、路由级**三个层次声明，优先级为：**路由级 > 块级 > 全局**。

### 全局 CORS（`$main()` 前）

```dolang
@CORS("*")
$main() {
    $serve(8080);
}
```

细粒度全局配置：

```dolang
@CORS({
    origins: ["https://example.com", "https://app.example.com"],
    methods: ["GET", "POST", "PUT", "DELETE"],
    headers: ["Content-Type", "Authorization"],
    max_age: 3600,
    credentials: false,
})
$main() {
    $serve(8080);
}
```

### 块级 CORS（`$HTTP {}` 前）

```dolang
@CORS("*")
$main() {
    $serve(8080);
}

// 块级覆盖全局
@CORS({ origins: ["https://admin.example.com"], methods: ["GET", "POST"] })
$HTTP("/admin") {
    $GET("/dashboard") dashboard() { $# {"ok": true}; }
}
```

### 路由级 CORS（单个路由前）

```dolang
@CORS("*")
$main() { $serve(8080); }

$HTTP("/api") {
    // 此路由覆盖全局，只允许特定来源
    @CORS({ origins: ["https://trusted.com"] })
    $GET("/data") getData() { $# {"ok": true}; }

    // 此路由继承全局 @CORS("*")
    $GET("/public") publicInfo() { $# {"ok": true}; }
}
```

### 优先级规则（完全替换，不合并）

```
路由级 @CORS  ──存在──▶  使用路由级配置
     │ 不存在
     ▼
块级 @CORS   ──存在──▶  使用块级配置（在 $HTTP 执行时写入每个子路由）
     │ 不存在
     ▼
全局 @CORS   ──存在──▶  使用全局配置（Axum 层 fallback）
     │ 不存在
     ▼
无配置       ──────────▶  无 CORS 头（与当前行为一致，向下兼容）
```

> **完全替换语义**：细粒度配置不继承上层字段，必须完整声明所需字段。

---

## TODO 清单

### 1. Frontend — Token

- [x] `crates/dolang-frontend/src/token/token.rs`
  - 新增 `At` token（`@`）
  - 新增 `AtCors` token（`@CORS`）

### 2. Frontend — Lexer

- [x] `crates/dolang-frontend/src/lexer/lexer.rs`
  - 在 `next_token()` 中增加 `@` 开头分支，longest-match-first：
    - `@CORS` → `AtCors`
    - `@` → `At`（保留扩展空间）

### 3. Frontend — AST

- [x] `crates/dolang-frontend/src/ast/ast.rs`
  - 新增 `CorsConfig` 结构体：
    ```rust
    pub struct CorsConfig {
        pub allow_all: bool,
        pub origins: Vec<String>,
        pub methods: Vec<String>,
        pub headers: Vec<String>,
        pub max_age: Option<u64>,
        pub credentials: bool,
    }
    ```
  - `HttpFnStmt` 新增字段：`pub cors: Option<CorsConfig>`
  - `HttpBlockStmt` 新增字段：`pub cors: Option<CorsConfig>`
  - `Program` 新增字段：`pub global_cors: Option<CorsConfig>`

### 4. Frontend — Parser

- [x] `crates/dolang-frontend/src/parser/stmt/http.rs`
  - `parse_http_fn()` 前识别 `@CORS(...)` 注解，写入 `HttpFnStmt.cors`
  - `parse_http_block()` 前识别 `@CORS(...)` 注解，写入 `HttpBlockStmt.cors`
- [x] `crates/dolang-frontend/src/parser/mod.rs`（或 `parse_stmts()`）
  - `$main()` 前识别 `@CORS(...)` 注解，写入 `Program.global_cors`
- [x] 新增错误码（见错误机制章节）

### 5. Frontend — Diagnostic Codes

- [x] `crates/dolang-frontend/src/diagnostics/codes.rs`
  - 新增：
    ```rust
    pub const PARSE_CORS_INVALID_POSITION: &str = "DOL-P003";
    pub const PARSE_CORS_DUPLICATE:        &str = "DOL-P004";
    pub const PARSE_CORS_INVALID_SYNTAX:   &str = "DOL-P005";
    pub const CONFIG_CORS_INVALID:         &str = "DOL-C002";
    ```

### 6. Runtime — Context

- [x] `crates/dolang-runtime/src/runtime/context.rs`
  - 新增字段：`pub global_cors: Option<CorsConfig>`
  - 新增方法：`set_global_cors()` / `global_cors()`

### 7. Runtime — HttpRoute

- [x] `crates/dolang-runtime/src/interpreter/mod.rs`
  - `HttpRoute` 新增字段：`pub cors: Option<CorsConfig>`

### 8. Runtime — Interpreter

- [x] `crates/dolang-runtime/src/interpreter/exec/http.rs`
  - `handle_http_fn()`：将 `stmt.cors` 写入 `HttpRoute.cors`
  - `handle_http_block()`：将块级 `cors` 写入每个子 `HttpRoute.cors`（若子路由未自定义）

### 9. Backend — Axum

- [x] `crates/dolang-cli/src/backends/axum_backend.rs`
  - 实现 `resolve_cors(route_cors, global_cors) -> Option<CorsConfig>` 优先级合并函数
  - 为每个路由构建 `tower_http::cors::CorsLayer`
  - OPTIONS 预检交由 `CorsLayer` 自动处理
- [x] `crates/dolang-cli/Cargo.toml`
  - 确认 `tower-http` 开启 `cors` feature：
    ```toml
    tower-http = { version = "...", features = ["cors", "fs"] }
    ```

### 10. Runtime — Startup 验证

- [x] `crates/dolang-cli/src/server.rs`（或 `axum_backend.rs`）
  - `$serve()` 启动前调用 `validate_cors_config()`（见错误机制）

---

## 错误机制与容灾

### 错误码体系

| 错误码 | 阶段 | 触发场景 |
|--------|------|---------|
| `DOL-P003` | 解析期 | `@CORS` 出现在非法位置（普通函数、变量声明等） |
| `DOL-P004` | 解析期 | 同一节点重复声明 `@CORS` |
| `DOL-P005` | 解析期 | `@CORS` 参数不是字符串 `"*"` 也不是合法对象字面量 |
| `DOL-C002` | 启动期 | CORS 配置语义验证失败（值不合法） |

---

### 解析期错误（Parser — DOL-P）

所有解析期错误均包含文件名、行号、列号，程序**不启动**。

#### DOL-P003：`@CORS` 位置非法

`@CORS` 只允许出现在以下位置：
- `$main()` 之前（全局）
- `$HTTP {}` 块之前（块级）
- `$GET` / `$POST` / `$PUT` / `$DEL` / `$PATCH` 路由之前（路由级）

其他位置均报错：

```
error[DOL-P003]: @CORS annotation is not allowed here
  --> main.dol:5:1
  = note: @CORS can only annotate $main(), $HTTP blocks, or HTTP route functions ($GET, $POST, ...)
```

#### DOL-P004：重复 `@CORS`

同一节点只允许声明一次 `@CORS`：

```
error[DOL-P004]: duplicate @CORS annotation
  --> main.dol:3:1
  = note: @CORS already declared for this $HTTP block at line 2
  = note: remove the duplicate annotation
```

#### DOL-P005：`@CORS` 参数语法错误

```
error[DOL-P005]: invalid @CORS argument
  --> main.dol:1:7
  = note: expected "*" (allow all) or a config object { origins: [...], ... }
  = note: found: 42
```

---

### 启动期验证（Server Startup — DOL-C002）

在 `$serve()` 调用（即 Axum 服务器绑定端口之前）执行 `validate_cors_config()` 对所有 CORS 配置进行语义校验。

**验证规则及行为：**

| 场景 | 严重度 | 行为 |
|------|--------|------|
| `credentials: true` + `origins: ["*"]` | **Error** | 启动失败（浏览器强制禁止此组合） |
| `max_age` 为负数 | **Error** | 启动失败 |
| `origins` 列表为空 | **Warning** | 继续启动，等价于无 CORS 配置，打印警告 |
| `origins` 中包含非法 URL 格式 | **Warning** | 跳过该条目，继续启动，打印警告 |
| `methods` 包含未知 HTTP 方法 | **Warning** | 跳过该条目，继续启动，打印警告 |
| `headers` 包含非法 Header 名称 | **Warning** | 跳过该条目，继续启动，打印警告 |

**Error 示例输出：**

```
error[DOL-C002]: invalid CORS configuration
  = note: credentials: true cannot be combined with origins: ["*"]
  = note: browsers require explicit origins when credentials are enabled
  = note: fix: replace "*" with specific allowed origins
```

**Warning 示例输出：**

```
warning[DOL-C002]: invalid origin URL skipped: "not-a-url"
  = note: in @CORS at main.dol:2:1
  = note: only URLs with http:// or https:// scheme are allowed
```

---

### 容灾机制（Fault Tolerance）

#### 1. 无 `@CORS` → 向下兼容

无任何 `@CORS` 声明时，服务正常启动，**不添加任何 CORS 头**。现有程序行为完全不变。

#### 2. 优先级降级（Fallback Chain）

当某层级的 `@CORS` 配置因警告被清空（如 `origins` 全部无效），降级到上一层：

```
路由级配置无效 → 尝试块级 → 尝试全局 → 无 CORS 头
```

降级时打印：

```
warning[DOL-C002]: route-level @CORS config is empty after validation, falling back to block-level config
```

#### 3. OPTIONS 预检错误隔离

CORS 预检（OPTIONS 请求）由 `tower-http` 的 `CorsLayer` 处理，与业务路由处理器**完全隔离**。预检失败不影响非跨域请求的正常处理。

#### 4. 启动期 panic 修复（现有问题）

现有 `server.rs` 使用 `.unwrap()` 绑定端口，端口占用时 panic。CORS 启动验证需在此之前完成，同时建议将端口绑定改为：

```rust
let listener = tokio::net::TcpListener::bind(&addr).await
    .map_err(|e| Error::Interpreter(format!("failed to bind {}:{} — {}", host, port, e)))?;
```

> 此修复超出本 issue 范围，可作为独立 Bug 处理，但 CORS 验证本身不依赖它。

#### 5. HTTP handler 内部错误（现有机制）

HTTP handler 执行期间的任何 `Flow::Err` 或 `Flow::Throw` 已由现有机制返回 HTTP 500：

```json
{ "error": "<error message>" }
```

CORS 头由中间件在 handler 执行前注入，handler 出错不影响 CORS 响应头的正确性。

---

## 验收标准

### 正确性

- [x] `cargo build` 编译通过
- [x] `cargo test` 全部通过
- [x] `@CORS("*")` 全局配置下，OPTIONS 预检请求返回正确的 `Access-Control-Allow-*` 响应头
- [x] 指定来源白名单时，不在白名单的来源不带 CORS 头（浏览器将拒绝）
- [x] 不声明 `@CORS` 时，服务行为与当前一致（无 CORS 头，不破坏现有用法）
- [x] 路由级 `@CORS` 优先于块级，块级优先于全局（完全替换，不合并）
- [x] `credentials: true` 配合指定 `origins` 时，`Access-Control-Allow-Credentials: true` 正确返回

### 错误处理

- [x] `@CORS` 声明在非法位置时，报 `DOL-P003` 并附带行列信息，程序不启动
- [x] 同一节点重复声明 `@CORS` 时，报 `DOL-P004`，程序不启动
- [x] `@CORS` 参数语法错误时，报 `DOL-P005`，程序不启动
- [x] `credentials: true` + `origins: ["*"]` 时，报 `DOL-C002` Error，服务不启动
- [x] `max_age` 为负数时，报 `DOL-C002` Error，服务不启动
- [x] `origins` 含非法 URL 时，报 `DOL-C002` Warning 并跳过，服务正常启动
- [x] `origins` 列表经校验后为空时，降级到上层配置并打印 Warning

### 兼容性

- [x] 所有已有测试用例（无 `@CORS` 的程序）仍通过，行为不变
- [x] `$HTTP.link()` 模块链接的路由正确继承块级 CORS 配置

---

## 评论

CORS 支持 路由级、块级、全局三层，优先级：路由级 > 块级 > 全局（完全替换语义）。

新增注解关键符号 `@`，语法示例：

```dolang
@CORS("*")
$main() { $serve(8080); }

@CORS({
    origins: ["https://example.com", "*"],
    methods: ["GET", "POST"],
    headers: ["Content-Type", "Authorization"],
    max_age: 3600,
    credentials: false,
})
$main() { $serve(8080); }
```

错误机制与容灾方案已在上方章节中全面设计。
