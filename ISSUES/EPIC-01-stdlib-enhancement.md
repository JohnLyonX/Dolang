# EPIC-01 标准库扩展

**类型**: Epic
**状态**: Open
**子 Issues**: STD-001 · STD-002 · STD-003

---

## 概述

Dolang 当前已有 5 个原生标准库模块：`std.fs`、`std.env`、`std.json`、`std.str`、`std.math`，
覆盖文件 I/O、环境变量、JSON 处理、字符串操作、数学运算，共 60+ 函数。

本 Epic 目标：扩展 3 个高频 Web 开发所需模块，使 Dolang 具备完整的后端业务开发能力。

---

## 子 Issues

| Issue | 模块 | 优先级 | 状态 |
|-------|------|--------|------|
| [STD-001](STD-001-time-module.md) | `std.time` — 日期时间 | P1 | Open |
| [STD-002](STD-002-uuid-module.md) | `std.uuid` — UUID 生成 | P2 | Open |
| [STD-003](STD-003-http-client.md) | `std.http` — HTTP 客户端 | P2 | Open |

---

## 背景与动机

- **`std.time`**：任何 Web 服务都需要时间戳、日期格式化、过期计算。目前 Dolang 无任何时间 API。
- **`std.uuid`**：生成唯一 ID 是后端接口的基础需求（日志追踪、数据库主键、请求 ID）。
- **`std.http`**：微服务架构下，服务间调用和第三方 API 集成不可缺少；目前 Dolang 只能接收 HTTP 请求，无法发出。

---

## 未来预留

以下模块在本 Epic 中**不实现**，作为 P3 规划预留：
- `std.crypto` — MD5 / SHA256 / HMAC 哈希
- `std.base64` — Base64 编码/解码
- `stdlib/http/` — 纯 Dolang HTTP 辅助库（路由分组、响应构造工具）

---

## 评论

<!-- 由 @用户 填写，记录决策、优先级调整、阻塞事项 -->
