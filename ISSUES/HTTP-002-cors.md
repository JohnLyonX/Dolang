# HTTP-002 CORS 支持

**Epic**: [EPIC-02](EPIC-02-http-enhancement.md)
**优先级**: P2
**状态**: Open（**语法方案待定，见评论区**）

---

## 问题概述

前后端分离架构下，浏览器会对跨域请求发起 CORS 预检（OPTIONS 请求）。
目前 Dolang 不处理 CORS，导致前端直接调用 Dolang Web 服务时被浏览器拦截。

需要在语言层面提供 CORS 配置能力，让开发者能简洁地声明跨域策略。

---

## 语法方案（待定）

### 方案 A：`$main()` 内函数调用形式（简洁）

```dolang
$main() {
    $CORS("*");        // 允许所有来源
    $serve(8080);
}
```

带细粒度配置：
```dolang
$main() {
    $CORS({
        "origins": ["https://example.com", "https://app.example.com"],
        "methods": ["GET", "POST", "PUT", "DELETE"],
        "headers": ["Content-Type", "Authorization"],
        "max_age": 3600,
    });
    $serve(8080);
}
```

### 方案 B：顶层 `$CORS { }` 声明块（显式，与 `$HTTP { }` 风格一致）

```dolang
$CORS {
    origins: ["https://example.com", "*"],
    methods: ["GET", "POST"],
    headers: ["Content-Type", "Authorization"],
    max_age: 3600,
}

$GET("/api") get() { $# {"ok": true}; }
```

---

## TODO 清单

> **TODO 留空，待语法方案在评论区确认后填写。**

预计涉及文件（无论哪种方案）：
- `crates/dolang-frontend/src/token/token.rs` — 新增 CORS 相关 token
- `crates/dolang-frontend/src/lexer/lexer.rs` — 词法识别
- `crates/dolang-frontend/src/ast/ast.rs` — AST 节点
- `crates/dolang-frontend/src/parser/stmt/` — 解析逻辑
- `crates/dolang-runtime/src/runtime/context.rs` — 存储 CORS 配置
- `crates/dolang-cli/src/server.rs` — Axum 层添加 CORS 中间件（`tower-http` 的 `CorsLayer`）

---

## 验收标准

- [ ] `cargo build` 编译通过
- [ ] `cargo test` 全部通过
- [ ] CORS `*` 配置下，OPTIONS 预检请求返回正确的 `Access-Control-Allow-*` 响应头
- [ ] 指定来源白名单时，不在白名单的来源不带 CORS 头（浏览器将拒绝）
- [ ] 不配置 `$CORS` 时，服务行为与当前一致（无 CORS 头，不破坏现有用法）
- [ ] CORS 配置只能在 `$main()` 中声明一次（重复声明报错或后设覆盖，需明确语义）

---

## 评论

<!-- 由 @用户 填写 -->
<!-- 请在此确认采用方案 A（$CORS("*") 调用形式）还是方案 B（$CORS { } 声明块） -->
<!-- 以及是否需要支持 credentials、max_age 等高级配置 -->
