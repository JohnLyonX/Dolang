# EPIC-02 HTTP 增强

**类型**: Epic
**状态**: Open
**子 Issues**: HTTP-001 · HTTP-002 · HTTP-003

---

## 概述

Dolang HTTP 服务端当前已具备：5 种 HTTP 方法定义、路径参数、查询参数自动注入、
请求头读取（`$HDR`）、响应体多种类型（JSON / HTML / String）、状态码控制（`$RES`）、
模块挂载（`$HTTP.link`）、静态文件服务（`$STATIC`）。

核心功能完整，但仍缺少：**响应头写入**、**CORS 支持**，以及 **HTTP 行为的规范化测试覆盖**。

---

## 子 Issues

| Issue | 功能 | 优先级 | 状态 |
|-------|------|--------|------|
| [HTTP-001](HTTP-001-set-hdr.md) | `$SET_HDR` — 响应头设置 | P1 | Open |
| [HTTP-002](HTTP-002-cors.md) | CORS 支持 | P2 | Open（语法待定）|
| [HTTP-003](HTTP-003-spec-tests.md) | HTTP 专项 spec 测试 | P1 | Open |

---

## 背景与动机

- **`$SET_HDR`**：目前只能读取请求头（`$HDR`），无法写响应头。缓存控制、自定义 Header、Content-Disposition 等均依赖此能力。
- **CORS**：跨域支持是前后端分离架构的必要条件，目前完全缺失。
- **测试**：HTTP 核心功能（路径参数、查询参数、$RES、$SET_HDR）目前无任何 spec 测试用例，存在回归风险。

---

## 评论

<!-- 由 @用户 填写，记录决策、优先级调整、阻塞事项 -->
