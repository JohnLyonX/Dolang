# Issue 005：mutable method 被屏蔽（push/pop/reverse/remove）

**类型**: Bug
**优先级**: P0
**状态**: Closed ✅

## 问题描述

`push`、`pop`、`reverse`、`remove` 等需要修改接收者的方法，在 `eval_method_call` 中被
提前拦截并返回错误，导致完全无法使用：

```
error: mutable method 'push' not yet supported
```

## 根本原因

`crates/dolang-runtime/src/interpreter/eval/calls.rs`：

```rust
if super::super::builtins::is_method_mutating(&call.method) {
    return Err(runtime_error(..., "mutable method '...' not yet supported"));
}
```

`dispatch_mut` 和 `list::call_mut` 已实现，但从未被调用到。

## 修复方案

在 `crates/dolang-runtime/src/interpreter/exec/variables.rs` 的 `handle_expr_stmt` 中：

1. 识别 `Expr::MethodCall` 且方法为 mutating
2. 提取接收者变量名（必须是 `Expr::VarLookup`）
3. 用 `state.env.get_mut()` 取可变引用，调用 `dispatch_mut`
4. 修改自动写回（引用语义）

## 已修复

- `crates/dolang-runtime/src/interpreter/exec/variables.rs`：新增 `handle_mut_method_call`

## 已知限制

mutable 方法仅支持作为**语句**使用（`result.push(x);`），
不支持作为**表达式**（`$ x = list.pop();`）。
