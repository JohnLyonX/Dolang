# gRPC Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Dolang 落地一个契约驱动、面向 HTTP/BFF/集成网关场景的 unary `std.grpc` client。

**Architecture:** 首版仅实现 runtime-native 形态的 gRPC client，不引入语言级 DSL。实现拆为配置与模块注册、契约加载、消息映射、unary 传输执行、结构化结果与测试样例五层，按 TDD 分阶段推进。

**Tech Stack:** Rust, `dolang-runtime`, native stdlib modules, protobuf/gRPC Rust crates, descriptor/proto loading, integration tests

---

## 文件结构与责任划分

### 现有文件

- `crates/dolang-runtime/src/stdlib_native/mod.rs`
  负责注册标准库 native 模块，需要新增 `std.grpc` 注册入口。
- `crates/dolang-runtime/src/runtime/context.rs`
  当前承载 runtime 共享上下文；如果 gRPC client 需要共享缓存、descriptor registry 或连接上下文，需要评估是否扩展这里。
- `crates/dolang-runtime/src/stdlib_native/http_client.rs`
  现有出站客户端模式参考，用于复用参数校验、返回结构、模块注册风格。
- `tests/integration_suite.rs`
  当前主集成测试入口，适合新增 `std.grpc` 端到端覆盖。
- `crates/dolang-runtime/Cargo.toml`
  需要引入 protobuf / gRPC / descriptor 相关依赖。

### 新增文件

- `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`
  `std.grpc` 模块入口，对外暴露 `grpc.client({...})` 与 `client.call({...})`。
- `crates/dolang-runtime/src/stdlib_native/grpc/config.rs`
  解析 client 配置与调用配置。
- `crates/dolang-runtime/src/stdlib_native/grpc/contract.rs`
  descriptor / proto 加载、service / method 查询。
- `crates/dolang-runtime/src/stdlib_native/grpc/mapper.rs`
  Dolang `Map/JSON` 与 protobuf message 之间的双向映射。
- `crates/dolang-runtime/src/stdlib_native/grpc/transport.rs`
  unary 请求执行、metadata、timeout、状态码与错误收集。
- `crates/dolang-runtime/src/stdlib_native/grpc/types.rs`
  gRPC client 内部共享结构，如 client config、call config、result model、error model。
- `tests/fixtures/grpc/`
  gRPC 相关测试 fixture，如 sample proto、descriptor、Dolang 脚本。
- `examples/http-grpc-gateway/`
  HTTP -> gRPC 聚合示例项目。
- `docs/reference/grpc.md`
  `std.grpc` 参考文档。

## Phase 0: 技术预研与依赖定型

**目标：** 确定 Rust 侧 gRPC / protobuf 技术栈，避免在实现中途推翻。

**Files:**
- Modify: `crates/dolang-runtime/Cargo.toml`
- Create: `gRPC 设计.md`（已存在，作为需求基线）
- Create: `Dev-gRPC.md`

- [ ] 评估 descriptor 与 `.proto` 两条路径在 Rust 侧的实现方式，优先选定一套能同时支持“读取 descriptor”和“按需兼容 proto”的库组合。
- [ ] 确认 unary gRPC transport 的 Rust 依赖方案，明确是否使用 `tonic` 作为传输层、`prost` / `prost-types` 作为 message 基础。
- [ ] 明确 descriptor 加载所需 crate，并记录首版支持矩阵：支持的字段类型、暂不支持的 protobuf 特性、错误降级策略。
- [ ] 在 `crates/dolang-runtime/Cargo.toml` 中新增最小依赖集合，避免一次性引入服务端或生成代码相关的无关组件。
- [ ] 运行依赖解析验证。
Run: `cargo check -p dolang-runtime`
Expected: PASS，且新增依赖可被正确解析。

## Phase 1: 模块骨架与配置模型

**目标：** 先把 `std.grpc` 模块和 client/call 配置骨架搭起来，不碰复杂映射和传输。

**Files:**
- Modify: `crates/dolang-runtime/src/stdlib_native/mod.rs`
- Create: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`
- Create: `crates/dolang-runtime/src/stdlib_native/grpc/types.rs`
- Create: `crates/dolang-runtime/src/stdlib_native/grpc/config.rs`
- Test: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`（单元测试）

- [ ] 先为 `std.grpc` 写一组失败测试，覆盖以下最小行为：
  `context.native_module("std.grpc")` 已注册；
  `grpc.client({...})` 要求必须存在 `target`；
  `descriptor` 与 `proto` 至少提供一个；
  缺失关键字段时抛出清晰错误。
- [ ] 在 `crates/dolang-runtime/src/stdlib_native/mod.rs` 中注册 `grpc_client::register(context)`。
- [ ] 在 `grpc_client.rs` 中建立 `register()` 入口，并暴露 `client` 构造函数。
- [ ] 在 `grpc/types.rs` 中定义首版共享结构：
  `GrpcClientConfig`
  `GrpcCallConfig`
  `GrpcCallResult`
  `GrpcRemoteError`
- [ ] 在 `grpc/config.rs` 中实现从 `DolangValue::Map` 解析 client / call 配置的逻辑，并统一错误格式。
- [ ] 运行模块级测试，确认仅配置层即可通过。
Run: `cargo test -p dolang-runtime stdlib_native::tests::aggregate_registration_exposes_ -- --nocapture`
Expected: PASS，并能验证 `std.grpc` 已进入聚合注册。

## Phase 2: 契约加载层

**目标：** 落地 descriptor 主路径，并为 `.proto` 兼容入口留出明确边界。

**Files:**
- Create: `crates/dolang-runtime/src/stdlib_native/grpc/contract.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`
- Test: `crates/dolang-runtime/src/stdlib_native/grpc/contract.rs`
- Create: `tests/fixtures/grpc/proto/echo.proto`
- Create: `tests/fixtures/grpc/descriptors/echo.pb`

- [ ] 先写失败测试，覆盖：
  成功加载 descriptor；
  能查询到 `service` 与 `method`；
  method 不存在时报错；
  缺 descriptor 文件时报错。
- [ ] 在 `contract.rs` 中实现 descriptor loader，建立 service / method 索引。
- [ ] 明确 `.proto` 入口的第一阶段策略：
  首版可以只完成 API 接口和错误提示，若 `.proto` 尚未接通，则必须返回“当前版本仅稳定支持 descriptor”的清晰错误；
  不允许静默假装支持。
- [ ] 将 `grpc.client({...})` 在初始化时绑定契约上下文，避免每次 `call()` 重复加载文件。
- [ ] 运行 contract 层测试。
Run: `cargo test -p dolang-runtime grpc::contract -- --nocapture`
Expected: PASS，错误信息包含缺文件、缺 service、缺 method 的明确文案。

## Phase 3: 消息映射层

**目标：** 在 Dolang `Map/JSON` 与 protobuf message 之间建立首版可靠映射。

**Files:**
- Create: `crates/dolang-runtime/src/stdlib_native/grpc/mapper.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc/types.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc/contract.rs`
- Test: `crates/dolang-runtime/src/stdlib_native/grpc/mapper.rs`
- Create: `tests/fixtures/grpc/proto/types.proto`
- Create: `tests/fixtures/grpc/descriptors/types.pb`

- [ ] 先写失败测试，覆盖这些首版支持类型：
  `string`
  `bool`
  `int32/int64`
  `uint32/uint64`
  `float/double`
  `bytes`
  `repeated`
  `map<K,V>`
  嵌套 message
  `enum`
- [ ] 为请求侧实现字段校验：
  缺必填字段时失败；
  字段名不存在时失败；
  类型不匹配时失败；
  enum 非法值时失败。
- [ ] 为响应侧实现 protobuf -> Dolang `Map/JSON` 解码。
- [ ] 对 `optional` / presence 实现基础语义。
- [ ] 对 `oneof`、`Any`、复杂 well-known types` 明确返回“暂不支持”的运行时错误。
- [ ] 运行 mapper 层测试。
Run: `cargo test -p dolang-runtime grpc::mapper -- --nocapture`
Expected: PASS，且不支持特性返回明确错误，不做静默降级。

## Phase 4: Unary 传输层

**目标：** 打通真实 gRPC unary 调用、metadata、timeout、结构化结果。

**Files:**
- Create: `crates/dolang-runtime/src/stdlib_native/grpc/transport.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc/types.rs`
- Test: `crates/dolang-runtime/src/stdlib_native/grpc/transport.rs`

- [ ] 先写失败测试，覆盖：
  unary 成功调用；
  metadata 随请求发送；
  timeout 生效；
  远端非 OK 状态返回结构化结果；
  连接失败时返回可区分错误。
- [ ] 在 `transport.rs` 中实现 unary executor，输入为：
  已解析的 client config
  已解析的 call config
  已编码的 request message
  method descriptor
- [ ] 统一响应结构为：
  `ok`
  `status`
  `status_name`
  `body`
  `metadata`
  `error`
- [ ] 按设计区分两类错误：
  本地配置/契约/编码错误直接抛异常；
  远端非 OK 状态结构化返回。
- [ ] 确认 timeout/deadline 的覆盖规则：
  调用级配置优先于 client 默认配置。
- [ ] 运行 transport 层测试。
Run: `cargo test -p dolang-runtime grpc::transport -- --nocapture`
Expected: PASS，`NOT_FOUND`/`UNAVAILABLE`/`DEADLINE_EXCEEDED` 都能结构化返回。

## Phase 5: `std.grpc` 对外 API 落地

**目标：** 把配置、契约、映射、传输整合成最终 Dolang 可调用的 `grpc.client().call()`。

**Files:**
- Modify: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/mod.rs`
- Test: `crates/dolang-runtime/src/stdlib_native/grpc_client.rs`

- [ ] 先写失败测试，覆盖：
  `grpc.client({...})` 返回可调度对象；
  `client.call({...})` 能触发一条完整 unary 链路；
  调用级 metadata 与 client 默认 metadata 合并；
  调用级 timeout 覆盖默认 timeout。
- [ ] 在 `grpc_client.rs` 中完成完整装配：
  配置解析
  契约查询
  请求映射
  传输执行
  响应解码
  结果回传
- [ ] 对返回值做 Dolang 友好化，保持与现有 `std.http` 类似的 map 风格。
- [ ] 增补模块级错误信息测试，确保用户看到的是 `std.grpc` 语义化错误，而不是底层库原始报错。
- [ ] 运行 `std.grpc` 模块测试。
Run: `cargo test -p dolang-runtime grpc_client -- --nocapture`
Expected: PASS。

## Phase 6: 端到端集成测试

**目标：** 证明 `std.grpc` 能服务 Dolang 的核心定位，即 HTTP 网关调用 gRPC 后端。

**Files:**
- Modify: `tests/integration_suite.rs`
- Create: `tests/fixtures/grpc/serve_gateway/main.dol`
- Create: `tests/fixtures/grpc/serve_gateway/package.toml`
- Create: `tests/support/grpc.rs`（如需要）

- [ ] 先写失败测试，覆盖以下场景：
  Dolang `serve` handler 调用 gRPC unary；
  HTTP body 转 gRPC request；
  gRPC success 转 HTTP JSON；
  gRPC `NOT_FOUND` 转结构化 HTTP JSON；
  gRPC timeout 的 handler 行为可预期。
- [ ] 如果现有测试基建不足，新增最小 gRPC mock server 支撑集成测试。
- [ ] 在 `tests/integration_suite.rs` 中加入端到端验证，确保不只是单元测试通过。
- [ ] 运行集成测试子集。
Run: `cargo test --test integration_suite grpc -- --nocapture`
Expected: PASS。

## Phase 7: 文档与样例

**目标：** 给使用者一个稳定、清晰、可复制的接入路径。

**Files:**
- Create: `docs/reference/grpc.md`
- Modify: `docs/README.md`
- Modify: `README.md`
- Create: `examples/http-grpc-gateway/main.dol`
- Create: `examples/http-grpc-gateway/README.md`

- [ ] 写 `std.grpc` 参考文档，至少包含：
  client 创建参数
  call 参数
  返回结构
  错误模型
  descriptor 与 `.proto` 接入说明
  当前不支持能力列表
- [ ] 新增 HTTP -> gRPC gateway 示例，展示 Dolang 作为 API 集成层的推荐用法。
- [ ] 在 README 或 docs 导航中增加 `std.grpc` 入口。
- [ ] 手动跑样例或最小 smoke test。
Run: `cargo run -- run examples/http-grpc-gateway/main.dol`
Expected: 至少完成加载与静态检查；若示例需要外部 gRPC 服务，则补充如何运行测试后端。

## Phase 8: 收尾与发布准备

**目标：** 在合并前完成统一验证，锁定首版边界。

**Files:**
- Modify: `docs/CHANGELOG.md`（如项目当前流程需要）
- Modify: `README.md`（如有必要）

- [ ] 全量格式化与静态检查。
Run: `cargo fmt --all --check`
Expected: PASS。
- [ ] 运行 clippy。
Run: `cargo clippy --workspace --all-targets --all-features`
Expected: PASS。
- [ ] 运行测试。
Run: `cargo test --workspace`
Expected: PASS。
- [ ] 做一轮人工边界复核，确认首版没有偷偷扩展到 streaming、代码生成或语言级 DSL。
- [ ] 整理已知限制，明确写入文档。

## 阶段执行顺序建议

- [ ] 先做 Phase 0 和 Phase 1，锁定依赖与 API 骨架。
- [ ] 再做 Phase 2 和 Phase 3，优先保证“descriptor 驱动 + Map/JSON 映射”成立。
- [ ] 然后做 Phase 4 和 Phase 5，打通真实 unary client。
- [ ] 最后做 Phase 6 到 Phase 8，补齐端到端验证、文档和发布准备。

## 当前建议的首批 commit 拆分

- [ ] `chore: add grpc runtime dependencies`
- [ ] `feat: register std.grpc module skeleton`
- [ ] `feat: add grpc descriptor contract loader`
- [ ] `feat: add grpc protobuf mapper`
- [ ] `feat: add grpc unary transport executor`
- [ ] `feat: expose std.grpc client call api`
- [ ] `test: add grpc integration coverage`
- [ ] `docs: add grpc reference and gateway example`

## 自检结论

- [ ] 设计要求的核心能力都已落到阶段任务：
  `std.grpc`
  `grpc.client({...})`
  `client.call({...})`
  descriptor 主路径
  `.proto` 兼容入口
  Map/JSON 映射
  unary transport
  metadata
  timeout
  结构化错误
- [ ] 首版明确不做的内容也已在计划中锁边界：
  streaming
  代码生成
  语言级 DSL
  高级治理能力
- [ ] 各阶段都映射到了明确文件与验证命令，可按 TODO 顺序落地。
