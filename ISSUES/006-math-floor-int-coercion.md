# Issue 006：math.floor/ceil/round 不接受 Int 参数

**类型**: Bug
**优先级**: P0
**状态**: Closed ✅

## 问题描述

`math.floor(5)` 报错，必须写成 `math.floor(5.0)` 或 `math.floor(5 * 1.0)` 才能工作：

```
error: math.floor: expected Float at argument 0, got Int
```

## 根本原因

`crates/dolang-runtime/src/stdlib_native/math.rs` 的 `float_arg` helper：

```rust
DolangValue::Float(f) => Ok(*f),
other => Err(Error::Interpreter(format!("expected Float, got {}", other.type_name())))
```

只接受 Float，拒绝 Int。

## 修复方案

在 `float_arg` 中增加 `Int → f64` 隐式提升：

```rust
DolangValue::Int(n) => Ok(*n as f64),
DolangValue::Float(f) => Ok(*f),
other => Err(...)
```

## 已修复

- `crates/dolang-runtime/src/stdlib_native/math.rs`：`float_arg` 增加 Int coercion
