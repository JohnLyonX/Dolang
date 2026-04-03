# Serve Auth Review

本轮 review 以 [SERVE_AUTH_PLAN.md](/Users/liangzhanbo/CodeStudio/dolang/SERVE_AUTH_PLAN.md) 为基线，对当前已标记完成的 auth 能力做了一轮 research + code review。

review 方式：

- 对照计划中的已完成项检查实现是否真正生效
- 功能性与安全性并行审查
- 以当前实现、spec、可执行测试为主，reference/guide 作为补充

## 高优先级

### 1. 授权配置的非法值会静默退化成“更宽松”的公开访问

风险：

- `[[server.auth.authorization.rules]].require` 当前只有在值严格等于 `"authenticated"` 时才会生效，任何拼写错误都会被当成 `public`
- `[server.auth.authorization].default` 当前也只有值严格等于 `"authenticated"` 才会启用默认保护，其他任意字符串都会退化成 `public`
- `validate_runtime_auth_config(...)` 没有校验这两个字段

这意味着一类纯配置 typo 会直接把原本应受保护的路由暴露出去，属于安全边界静默放宽。

证据：

- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L113)
- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L122)
- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L134)
- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L179)
- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L186)

建议：

- 启动期强校验 `authorization.default`
- 启动期强校验每条 rule 的 `require`
- 非法值应直接启动失败，而不是回退到 `public`

### 2. refresh token 轮换不是原子消费，并发刷新存在双花窗口

风险：

- `jwt.refresh_pair(...)` 当前流程是先 `get` 校验 refresh token，再单独 `revoke`，最后再签发新 pair
- store trait 只有 `get` / `revoke`，没有原子“consume once”能力
- 两个并发刷新请求可以在 revoke 之前同时读到有效 token，随后都成功签发新的 token pair

这会破坏“旧 refresh token 只能使用一次”的安全承诺。

证据：

- [auth_jwt.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/stdlib_native/auth_jwt.rs#L82)
- [auth_jwt.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/stdlib_native/auth_jwt.rs#L94)
- [auth_jwt.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/stdlib_native/auth_jwt.rs#L143)
- [auth_jwt.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/stdlib_native/auth_jwt.rs#L151)
- [refresh_store.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/refresh_store.rs#L7)

建议：

- 在 `RefreshTokenStore` 上增加原子 `consume` / `revoke_if_active` 语义
- memory/sqlite/postgres 三个 backend 都要做 compare-and-set 风格实现
- 增加并发 refresh 集成测试，验证只有一个请求能成功

## 中优先级

### 3. 失效或过期的 session cookie 在 401/403 路径下不会被清除

风险：

- `apply_session_auth(...)` 在发现 session 过期时会设置 `pending_clear_cookie = true`
- 但 auth 失败响应在 Axum 层直接返回，绕过了 `append_auth_response_headers(...)`
- 结果是服务端会删除 store 中的 session，但客户端仍持续保留旧 cookie

这不是直接越权，但会导致客户端持续携带无效 cookie，增加排障成本，也与“runtime 统一管理 cookie 清除”的设计目标不一致。

证据：

- [http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/http.rs#L195)
- [http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/http.rs#L206)
- [axum_backend.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/backends/axum_backend.rs#L258)
- [axum_backend.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/backends/axum_backend.rs#L301)
- [axum_backend.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/backends/axum_backend.rs#L313)

建议：

- 401/403 分支也要附加 auth response headers
- 至少在 expired/invalid session 命中时回写 clear-cookie

### 4. session/refresh store 配置错误现在要么静默降级到 memory，要么直接 panic

风险：

- 未知 `driver` 会直接走 `_ => Memory(...)`，配置 typo 不会报错
- sqlite/postgres 初始化大量使用 `expect(...)` / `panic!(...)`
- `load_context_and_program(...)` 是在 `set_project_config(...)` 完成 store 初始化之后才调用 `validate_runtime_auth_config(...)`

这会带来两类问题：

- 用户以为自己启用了持久化 backend，实际上 silently 退回 memory
- 缺失 URL / 环境变量 / schema 初始化失败时，不是受控 startup error，而是 panic 终止

对功能性和安全性都有影响，尤其是 refresh token/session persistence 的语义会被无声破坏。

证据：

- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L416)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L429)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L447)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L457)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L462)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L488)
- [loader.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/loader.rs#L71)
- [loader.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/loader.rs#L74)

建议：

- 启动期校验 `driver` 允许值
- 把 store 初始化改成 `Result` 返回，而不是 `expect/panic`
- 将 store config 校验纳入统一 startup config error 通道

## 低优先级

### 5. `default_scheme` 目前仍然只有校验，没有实际运行时语义

现状：

- 配置模型和文档都暴露了 `default_scheme`
- 当前实现只在启动期校验它的取值是否合法，以及 bearer 是否依赖 jwt enabled
- 请求链路、stdlib、principal 解析顺序都没有消费这个字段

这不会直接导致越权，但会让配置项呈现“看起来支持、实际上不生效”的状态，容易误导项目配置。

证据：

- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L92)
- [config.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/auth/config.rs#L145)
- [http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/http.rs#L128)
- [http.md](/Users/liangzhanbo/CodeStudio/dolang/docs/reference/http.md#L253)

建议：

- 要么给 `default_scheme` 明确补上运行时语义
- 要么在文档中降级为保留字段，避免用户误判

## 结论

本轮 review 下来，主路径能力基本可用，但仍有几处“已完成项语义未完全收口”的问题：

- 最高风险在授权配置的静默放宽，以及 refresh token 并发双花窗口
- 中风险集中在 startup config 错误处理和 auth 失败时 cookie 清理不一致
- 低风险主要是配置项语义未完全兑现

如果按修复优先级排序，建议：

1. 先修授权配置非法值的启动期校验
2. 再修 refresh token 的原子消费语义
3. 然后收口 store 初始化错误与 clear-cookie on 401/403
4. 最后决定 `default_scheme` 是补实现还是降级文档承诺
