# Serve Auth Development Plan

本文档汇总当前 `serve auth platform` 的完整开发计划与执行状态。

目标：

- 计划完整
- 每一步可验证
- 清晰区分已完成与未完成项

范围：

- `package.toml` 下的 `server.auth` 配置
- `serve` 模式请求级认证与授权
- `std.auth.*` 标准库
- session store: `memory` / `sqlite` / `postgres`

## 总体状态

当前阶段结论：

- 主认证链路已可用
- session / jwt / route auth 已接通主路径
- refresh token 已有标准库闭环
- 文档已初步同步
- 仍有一批增强项未完成

## 执行清单

### Phase 1: 配置模型

目标：

- 在 `package.toml` 中声明 auth/session/jwt/authorization

任务：

- [x] 增加 `[server.auth]`
- [x] 增加 `[server.auth.session]`
- [x] 增加 `[server.auth.session.store]`
- [x] 增加 `[server.auth.jwt]`
- [x] 增加 `[server.auth.authorization]`
- [x] 支持 `memory` / `sqlite` / `postgres` store 配置

主要落点：

- `crates/dolang-runtime/src/module/manifest.rs`
- `crates/dolang-runtime/src/config.rs`

验证：

- [x] `cargo test -p dolang-runtime parse_project_config -- --nocapture`
- [x] `cargo test -p dolang-runtime auth -- --nocapture`

### Phase 2: Runtime Auth Core

目标：

- 在请求进入 handler 前解析认证态
- 注入统一 principal

任务：

- [x] 增加 `RuntimeAuthConfig`
- [x] 增加 `RequestAuthContext`
- [x] 增加统一 `Principal`
- [x] 增加 `SessionRecord`
- [x] 按 `identity_sources` 解析 `cookie` / `bearer`
- [x] 将 principal 注入请求级上下文

主要落点：

- `crates/dolang-runtime/src/runtime/auth/`
- `crates/dolang-runtime/src/runtime/context.rs`
- `crates/dolang-runtime/src/runtime/http.rs`

验证：

- [x] `cargo test -p dolang-runtime auth -- --nocapture`
- [x] `cargo test --test integration_suite live_http_bearer_helpers_are_visible_in_request_context -- --nocapture`

### Phase 3: Session Store

目标：

- 统一 session store 抽象
- 支持内存、SQLite、Postgres

任务：

- [x] 定义 `SessionStore` trait
- [x] 实现 `MemorySessionStore`
- [x] 实现 `SqliteSessionStore`
- [x] 实现 `PostgresSessionStore`
- [x] 启动时初始化 schema

主要落点：

- `crates/dolang-runtime/src/runtime/auth/session_store.rs`
- `crates/dolang-runtime/src/runtime/auth/session_memory.rs`
- `crates/dolang-runtime/src/runtime/auth/session_sqlite.rs`
- `crates/dolang-runtime/src/runtime/auth/session_postgres.rs`

验证：

- [x] `cargo test -p dolang-runtime memory_store -- --nocapture`
- [x] `cargo test -p dolang-runtime sqlite_store_round_trips_session_record -- --nocapture`
- [x] `cargo test -p dolang-runtime postgres_store_round_trips_session_record_when_url_present -- --nocapture`

说明：

- Postgres live test 依赖本地可用数据库环境

### Phase 4: JWT 与 Password 原语

目标：

- 提供 JWT 签发/校验
- 提供密码哈希/校验

任务：

- [x] access token `sign` / `verify`
- [x] refresh token `sign_refresh` / `verify_refresh`
- [x] token pair `issue_pair` / `refresh_pair`
- [x] password `hash` / `verify`

主要落点：

- `crates/dolang-runtime/src/runtime/auth/jwt.rs`
- `crates/dolang-runtime/src/runtime/auth/password.rs`
- `crates/dolang-runtime/src/stdlib_native/auth_jwt.rs`
- `crates/dolang-runtime/src/stdlib_native/auth_password.rs`

验证：

- [x] `cargo test -p dolang-runtime auth -- --nocapture`
- [x] `cargo test --test integration_suite auth_stdlib_fixtures_cover_core_flows -- --nocapture`

### Phase 5: Session 标准库

目标：

- 让 Dolang handler 可以操作当前 session

任务：

- [x] `session.current()`
- [x] `session.exists()`
- [x] `session.id()`
- [x] `session.create(...)`
- [x] `session.destroy()`
- [x] `session.get(key)`
- [x] `session.set(key, value)`
- [x] `session.delete(key)`
- [x] `session.rotate()`

主要落点：

- `crates/dolang-runtime/src/stdlib_native/auth_session.rs`

验证：

- [x] `cargo test --test integration_suite auth_stdlib_fixtures_cover_core_flows -- --nocapture`
- [x] `cargo test --test integration_suite live_http_login_sets_cookie_and_allows_protected_route -- --nocapture`
- [x] `cargo test --test integration_suite live_http_login_and_logout_emit_configured_cookie_headers -- --nocapture`

### Phase 6: Guard 标准库

目标：

- 提供显式授权辅助 API

任务：

- [x] `guard.principal()`
- [x] `guard.authenticated()`
- [x] `guard.has_role(...)`
- [x] `guard.has_permission(...)`
- [x] `guard.require_role(...)`
- [x] `guard.require_permission(...)`
- [x] `guard.has_any_role(...)`
- [x] `guard.has_all_roles(...)`
- [x] `guard.has_any_permission(...)`
- [x] `guard.has_all_permissions(...)`
- [x] `guard.require_any_role(...)`
- [x] `guard.require_all_roles(...)`
- [x] `guard.require_any_permission(...)`
- [x] `guard.require_all_permissions(...)`

主要落点：

- `crates/dolang-runtime/src/stdlib_native/auth_guard.rs`

验证：

- [x] `cargo test --test integration_suite auth_stdlib_fixtures_cover_core_flows -- --nocapture`
- [x] `cargo test --test integration_suite auth_guard_require_role_fixture_fails -- --nocapture`

### Phase 7: HTTP 请求链路接入

目标：

- `serve` 模式下自动完成认证与 cookie 回写

任务：

- [x] 在 handler 前执行 auth pipeline
- [x] session cookie 读取
- [x] bearer token 读取
- [x] 认证失败返回 `401`
- [x] 授权失败返回 `403`
- [x] session 登录后 `Set-Cookie`
- [x] session 登出后清 cookie
- [x] session idle timeout 刷新
- [x] cookie header 按配置输出 `Path` / `Max-Age` / `SameSite` / `HttpOnly` / `Secure`

主要落点：

- `crates/dolang-runtime/src/runtime/http.rs`
- `crates/dolang-cli/src/backends/axum_backend.rs`

验证：

- [x] `cargo test --test integration_suite live_http_session_protected_route_returns_401_without_cookie -- --nocapture`
- [x] `cargo test --test integration_suite live_http_role_guard_returns_403_for_authenticated_but_unauthorized_user -- --nocapture`
- [x] `cargo test --test integration_suite live_http_login_and_logout_emit_configured_cookie_headers -- --nocapture`

### Phase 8: 路由授权规则

目标：

- 将配置中的授权规则接入 runtime

任务：

- [x] `require = "authenticated"`
- [x] `roles_any`
- [x] `roles_all`
- [x] `permissions_any`
- [x] `permissions_all`
- [x] `default = "authenticated"` 默认保护策略

主要落点：

- `crates/dolang-runtime/src/runtime/auth/config.rs`
- `crates/dolang-runtime/src/runtime/http.rs`

验证：

- [x] `cargo test --test integration_suite live_http_authorization_default_authenticated_requires_login -- --nocapture`
- [x] `cargo test --test integration_suite live_http_permission_guard_returns_403_for_missing_required_permission -- --nocapture`

### Phase 9: Refresh Token HTTP 闭环

目标：

- 降低项目层实现 access/refresh 刷新流程的样板代码

任务：

- [x] 增加 `jwt.issue_pair(payload)`
- [x] 增加 `jwt.refresh_pair(refresh_token)`
- [x] live HTTP 验证 `/login -> /refresh -> protected route`

主要落点：

- `crates/dolang-runtime/src/stdlib_native/auth_jwt.rs`
- `tests/integration_suite.rs`

验证：

- [x] `cargo test --test integration_suite live_http_refresh_pair_issues_new_access_token_for_protected_route -- --nocapture`

### Phase 10: 文档同步

目标：

- 对齐当前实现与用户可见文档

任务：

- [x] 更新 security model
- [x] 更新 HTTP reference
- [x] 更新 stdlib API reference
- [x] 更新 changelog

主要落点：

- `docs/spec/security-model.md`
- `docs/reference/http.md`
- `docs/reference/stdlib-api.md`
- `docs/CHANGELOG.md`

验证：

- [x] 文档已落盘
- [x] 文档内容与当前实现一致到主路径

## 未完成项

以下任务尚未完成，建议作为下一阶段 backlog：

### P0

- [x] 统一 `401/403` JSON 错误响应体
  - 目标：不只返回状态码，还返回稳定错误结构
  - 已验证：
    - live integration 断言 `status`、`code`、`message`

- [x] refresh token 轮换失效策略
  - 目标：旧 refresh token 刷新后失效，避免长期重复使用
  - 已验证：
    - 第一次 `/refresh` 成功
    - 第二次用旧 refresh token 调用 `/refresh` 返回 `401`

### P1

- [x] refresh token 持久化与撤销模型
  - 目标：为登出全部设备、主动失效、一次性刷新打基础
  - 已验证：
    - 引入 refresh token store
    - 增加 revoke / blacklist 集成测试

- [x] JWT 算法扩展
  - 目标：不只支持 `HS256`
  - 已验证：
    - 配置指定算法
    - 签发与校验 round trip

- [x] 更完整的授权策略抽象
  - 目标：支持更复杂策略组合
  - 已验证：
    - 多规则优先级
    - 更细粒度 claims-based guard

### P2

- [x] cookie/session 安全策略进一步收紧
  - 例如：
    - [x] session cookie 启动期安全校验：`cookie_same_site` 只允许 `lax|strict|none`
    - [x] session cookie 启动期安全校验：`SameSite=None` 必须同时启用 `Secure`
    - [x] CSRF 配套能力：session 自动生成 `csrf_token`，cookie 写请求默认校验 `X-CSRF-Token`
    - [x] 更严格的 session fixation 防护：`rotation = "always"` 下重新登录会立即失效旧 session cookie
    - [x] login 时的 rotate 策略细化：`rotation` 允许值收敛为 `off|on_login|always`

- [x] 更完整的 auth 示例工程
  - 目标：提供可直接运行的登录、刷新、登出、管理员路由样例

## 推荐执行顺序

如果继续开发，建议按这个顺序推进：

1. `401/403` 标准错误响应体
2. refresh token 轮换失效
3. refresh token 持久化与撤销
4. JWT 算法扩展
5. 示例工程与文档补强

## 当前基线验收命令

建议在继续开发前先确认当前基线：

```bash
cargo test -p dolang-runtime auth -- --nocapture
cargo test --test integration_suite auth_ -- --nocapture
cargo test --test integration_suite -- --nocapture
```

当前状态：

- [x] 基线可通过上述命令验证
