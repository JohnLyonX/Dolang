# EPIC-02 HTTP 增强

**类型**: Epic
**状态**: In Design
**子 Issues**: HTTP-001 · HTTP-002 · HTTP-003

---

## 概述

Dolang HTTP 服务端当前已具备：5 种 HTTP 方法定义、路径参数、查询参数自动注入、
请求头读取（`$HDR`）、响应体多种类型（JSON / HTML / String）、状态码控制（`$RES`）、
模块挂载（`$HTTP.link`）、静态文件服务（`$STATIC`）。

核心功能完整，但仍缺少：**响应头写入**、**CORS 支持**，以及 **HTTP 行为的规范化测试覆盖**。

本 Epic 同时引入 **`@` 注解体系**，作为 HTTP 层声明式配置的统一语法风格。

---

## 注解体系（新增）

EPIC-02 的核心语言设计决策：引入 `@` 注解符号，用于在路由和块上声明静态配置。

| 注解 | 作用域 | 说明 |
|------|--------|------|
| `@CORS(...)` | 全局 / 块级 / 路由级 | 声明跨域策略（HTTP-002） |
| `@SET_HDR({ "Header-Name": "value", ... })` | 块级 / 路由级 | 声明固定响应头（HTTP-001） |

**优先级规则**：
- `@CORS`：路由级 > 块级 > 全局，**完全替换**（不合并字段）
- `@SET_HDR`：同名 Header 路由级覆盖块级，不同名叠加

**不支持全局级的注解**：`@SET_HDR` 仅支持块级和路由级，响应头策略在 `$main()` 入口级声明无实际意义。

**Token 扩展**：
```
@ 开头分支，longest-match-first：
  @CORS     → AtCors
  @SET_HDR  → AtSetHdr
  @         → At（保留扩展空间）
```

---

## 子 Issues

| Issue | 功能 | 优先级 | 状态 |
|-------|------|--------|------|
| [HTTP-001](HTTP-001-set-hdr.md) | `@SET_HDR` — 响应头注解（路由级 / 块级） | P1 | In Design |
| [HTTP-002](HTTP-002-cors.md) | `@CORS` — CORS 跨域注解（全局 / 块级 / 路由级） | P2 | In Design |
| [HTTP-003](HTTP-003-spec-tests.md) | HTTP 专项集成测试 | P1 | Open |

---

## 实现顺序

```
HTTP-001 (@SET_HDR)
  └─ 引入 @ Token 体系 + AST 注解字段
  └─ 验证块级/路由级注解解析流程

HTTP-002 (@CORS)
  └─ 复用 HTTP-001 引入的 @ Token
  └─ 新增全局级注解（$main() 前）
  └─ tower-http CorsLayer 集成

HTTP-003 (集成测试)
  └─ 依赖 HTTP-001、HTTP-002 完成
  └─ 覆盖所有注解的优先级、覆盖、叠加行为
```

> **建议先实现 HTTP-001**：它引入 `@` Token 体系和 AST 注解字段，HTTP-002 可直接复用，避免重复改动 lexer/parser。

---

## 背景与动机

- **`@SET_HDR`**：目前只能读取请求头（`$HDR`），无法写响应头。缓存控制、自定义 Header、Content-Disposition 等均依赖此能力。原方案 `$SET_HDR(name, value)` 作为 handler 内语句已废弃，改为注解形式以统一语法风格、降低用户认知负担。
- **`@CORS`**：跨域支持是前后端分离架构的必要条件，目前完全缺失。注解形式天然适合 CORS 这类静态配置声明。
- **集成测试**：HTTP 核心功能（路径参数、查询参数、`$RES`、`@SET_HDR`、`@CORS`）目前无任何测试用例，存在回归风险。响应头和 CORS 行为验证必须通过真实 HTTP 请求完成（Rust 集成测试，非 .dol spec 文件）。

---

## 错误码规划

| 范围 | 错误码 | 归属 Issue |
|------|--------|-----------|
| `@CORS` 解析期 | `DOL-P003` · `DOL-P004` · `DOL-P005` | HTTP-002 |
| `@SET_HDR` 解析期 | `DOL-P006` · `DOL-P007` | HTTP-001 |
| CORS 启动期验证 | `DOL-C002` | HTTP-002 |
| 响应头启动期验证 | `DOL-C003` | HTTP-001 |

---

## 评论

<!-- 由 @用户 填写，记录决策、优先级调整、阻塞事项 -->

语法方案已确认：采用 `@` 注解体系，废弃原 `$SET_HDR(name, value)` 语句形式。
实现顺序：HTTP-001 → HTTP-002 → HTTP-003。
