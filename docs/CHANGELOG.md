# Changelog

本文件记录 Dolang 的用户可见变化。

记录规则：

- 只记录用户可见变化
- 每次发布都按固定栏目归档
- breaking change、弃用、移除必须写清兼容影响

## Unreleased

### Added

- 建立 `docs/spec/` 正式语言规范目录
- 增加 Phase 9 的版本、兼容性、弃用规则文档
- 增加 `std.str`、`std.math`、`std.json` 原生模块，可通过 `$mod std.*;` 使用常见字符串、数学与 JSON 能力
- 新增 `$Type` 自定义类型声明，支持描述 JSON 数据形状（字段名、类型、可选标记 `?`）
- 函数与 HTTP handler 返回类型注解支持 `JSON<TypeName>` 形式（如 `-> JSON<User>`）
- 增加 `std.time` 原生模块，提供 Unix 时间戳、UTC 格式化/解析、日期偏移与日期字段读取能力
- 增加 `std.uuid` 原生模块，提供 UUID v4 生成与格式校验能力
- 增加 `std.http` 原生模块，提供同步 HTTP 客户端能力并统一返回 `{ status, body, headers }` 结构
- `serve` 模式新增 `@SET_HDR(...)` 与 `@CORS(...)` 配置能力，可按全局 / HTTP 块 / 路由控制响应头与 CORS 策略

### Changed

- 规范化语言行为文档与 `tests/spec` 的映射关系
- 将 stdlib native module 注册从 `intrinsics.rs` 拆分到 `src/runtime/stdlib/`，为后续标准库扩展提供结构化入口

### Deprecated

- None

### Removed

- None

### Fixed

- 对齐 spec 样例与当前 parser/runtime 真实行为
- HTTP handler 在真实 `serve` 请求链路中的路径参数、query 参数与 `$HDR(...)` 请求头读取行为已补齐并覆盖集成测试
- `RuntimeMode::Test` 下 HTTP handler 注册阶段现在会执行返回类型校验；同时普通 `$fn` 与 HTTP handler 共用同一套返回类型验证逻辑
- `$RES(status, body)` HTTP 状态码修复：之前 status 参数被忽略，响应始终为 HTTP 200；
  现已正确返回指定状态码（如 404、201、500 等）
- HTTP handler 中未捕获的 `$throw` 在客户端侧稳定返回 HTTP 500 错误响应

### Changed（P0）

- `$throw` 未捕获时终端行为：HTTP handler 中未被 `$try/$catch` 捕获的 `$throw`
  现在会在终端打印 `[ERROR] uncaught throw: <value>`，便于开发调试；客户端仍收到 HTTP 500
- HTTP 服务器内部架构：引入 `HttpBackend` trait，Axum 实现封装至 `AxumBackend`，
  `server.rs` 降为纯协调层（对用户无感知，不影响任何 .dol 语法）

## Release Template

后续版本请按如下模板追加：

```md
## x.y.z - YYYY-MM-DD

### Added

- ...

### Changed

- ...

### Deprecated

- 当前行为：
- 替代行为：
- 起始版本：
- 计划移除版本：

### Removed

- ...

### Fixed

- ...
```
