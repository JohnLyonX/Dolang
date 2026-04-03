# GC Plan

## Goal

为 Dolang runtime 建立一套边界清晰的内存清理策略，解决长生命周期内存容器的保留问题，同时避免把解释器主体错误地重构成一套通用 tracing GC。

## Current State

目前已经存在一层轻量化、解耦后的惰性清理调度：

- 通用调度器：`crates/dolang-runtime/src/runtime/gc.rs`
- 当前接入方：
  - `crates/dolang-runtime/src/runtime/auth/session_memory.rs`
  - `crates/dolang-runtime/src/runtime/auth/refresh_memory.rs`

这层能力的职责是：

- 维护清理节流窗口
- 判断是否到达清理时机
- 在清理执行后推进下一个窗口

这层能力不负责：

- 扫描解释器对象图
- 跟踪 `Env` / `FnEnv` / `ModuleProxy` 的引用关系
- 替代 Rust 所有权和 `Arc` 的释放机制

## Architectural Position

GC 相关能力需要分成两类：

### 1. 值得抽象的“惰性清理”

适用对象：

- memory session store
- refresh token store
- 未来任何基于 TTL / retention 的内存缓存
- 某些可过期的 registry / cache

这类对象的共性是：

- 生命周期长
- 数据项可独立过期
- 清理不必每次访问都执行
- 更适合“机会式 / 节流式”批量删除

### 2. 不应被包装成统一 GC 的解释器状态

不建议纳入统一 GC 框架的对象：

- `ProgramState.env`
- `FnEnv`
- `ModuleNamespace`
- `ModuleProxy`
- 路由模块状态
- SQL connection registry

原因：

- 它们的问题本质上不是“缺垃圾回收器”
- 而是生命周期设计、cache 边界、所有权建模、显式释放策略是否正确

这部分应继续依赖：

- Rust 所有权
- `Arc` 引用计数
- request-local / process-local 生命周期隔离
- bounded cache / eviction
- 显式 close / remove

## Recommendation

结论：

- 保留并扩展 `runtime/gc.rs`
- 但不要把解释器整体重构成统一 GC 系统

正确方向是：

1. 把“时间驱动的惰性清理”做成可复用 runtime 组件
2. 把“解释器对象何时释放”继续交给 Rust 生命周期和资源边界设计

## Current Execution Decision

当前执行范围先停在 Phase 1：

- 完成并保留 `runtime/gc.rs` 这一层
- 保持 auth memory store 作为当前仅有的接入方
- 暂不实现 `CleanupPolicy` / `PrunableStore`
- 暂不把更多 runtime 容器接入该机制

原因：

- 现阶段已经有明确收益和真实使用方
- 继续推进到 Phase 2 会开始引入“为抽象而抽象”的风险
- 在没有第三个稳定接入方之前，不值得继续扩大设计面

## Proposed Phases

### Phase 1. Stabilize `runtime/gc.rs`

目标：

- 把当前 `LazyGcWindow` 固化为 runtime 的标准节流清理工具

动作：

- 保持接口简单：
  - `new(interval_seconds)`
  - `is_due(now)`
  - `mark_ran(now)`
- 为该模块保留独立单测
- 避免把业务语义塞进调度器

完成标准：

- 调度器不感知 session / token / cache 的具体语义

当前状态：

- 已完成
- 当前实现位于 `crates/dolang-runtime/src/runtime/gc.rs`
- 当前验证集位于 `crates/dolang-runtime/src/runtime/gc_tests.rs`
- 当前接入方仅为 auth memory store，未继续抽象

### Phase 2. Extract Cleanup Policy Layer

目标：

- 让 store 只负责“删什么”
- 让调度器只负责“何时删”

建议抽象：

- `CleanupPolicy`
- 或 `PrunableStore`

候选接口方向：

```rust
trait CleanupPolicy<T> {
    fn should_retain(&self, value: &T, now: i64) -> bool;
}
```

或：

```rust
trait PrunableStore {
    fn prune(&mut self, now: i64);
}
```

选择原则：

- 优先简单
- 不引入泛型复杂度污染 auth store
- 不为了抽象而抽象

完成标准：

- session / refresh store 的业务删除条件从调度逻辑里进一步分离

### Phase 3. Reuse for Other TTL Containers

目标：

- 让 runtime 里其他“时间型容器”复用相同调度方式

适合接入的对象：

- 内存缓存
- 可过期 registry
- 未来的 serve-mode 短期状态池

不适合接入的对象：

- interpreter env
- function tables
- module namespace graph
- request execution frames

完成标准：

- 统一“节流清理”模式
- 不扩大到解释器对象管理

### Phase 4. Separate Resource Management from GC Concerns

目标：

- 明确“资源释放”与“惰性清理”是两套不同机制

重点对象：

- SQL connection registry
- module namespace cache
- serve-mode shared state

建议策略：

- SQL 连接：request-local 生命周期 + 显式 close
- module cache：loading/ready sentinel + 明确 cache scope
- 共享状态：限制作用域，不依赖 GC 兜底

完成标准：

- 不再把 registry/cache 的生命周期问题误判为“需要 GC”

## Non-Goals

以下内容不在本计划范围内：

- 实现 tracing GC
- 扫描解释器对象引用图
- 为 Dolang 值系统增加 mark-sweep / generational GC
- 替换 Rust 内存管理

## Risks

### Risk 1. Over-abstracting

如果把 `gc.rs` 设计得过大，会把简单的清理问题复杂化。

应对：

- 保持 API 小
- 只抽象节流和触发时机

### Risk 2. Conflating cleanup with ownership

如果把对象生命周期问题混进 GC 模块，会掩盖真实设计问题。

应对：

- 将 cache / registry / env / module state 的问题单独处理
- 不让 `gc.rs` 接管解释器状态

### Risk 3. Hidden hot-path cost

如果 future cleanup 接入点设计不当，可能再次把热路径变成全量扫描。

应对：

- 所有接入都必须经过节流窗口
- 保持机会式、批量式运行

## Verification

当前最小验证集：

```text
cargo test -p dolang-runtime runtime::gc_tests::lazy_gc_window_only_runs_when_due -- --nocapture
cargo test -p dolang-runtime runtime::auth::session_memory::tests -- --nocapture
cargo test -p dolang-runtime runtime::auth::refresh_memory::tests -- --nocapture
```

## Final Decision

需要继续保留和发展轻量化 GC 调度层，但只限于 runtime 中的 TTL / retention 型清理。

不建议把 Dolang 解释器主体重构成统一 GC 系统。

本轮到此为止：Phase 1 完成，Phase 2+ 暂缓，直到出现新的真实复用场景。
