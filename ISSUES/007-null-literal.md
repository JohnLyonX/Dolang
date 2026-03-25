# Issue 007：null 字面量支持

**类型**: Feature
**优先级**: P0
**状态**: Closed ✅

## 问题描述

Dolang 没有 `null` 字面量，`null` 被当作普通变量名，运行时报 undefined variable：

```
error: undefined variable 'null'
```

导致 `is_null` 等 stdlib 函数无法实现。

## 修复方案（4 步）

1. **token**：`crates/dolang-frontend/src/token/token.rs` 新增 `Type::Null`
2. **lexer**：`crates/dolang-frontend/src/lexer/lexer.rs` 识别 `"null"` → `Type::Null`
3. **AST**：`crates/dolang-frontend/src/ast/ast.rs` 新增 `NullLiteral` 和 `Expr::Null`
4. **eval**：`crates/dolang-runtime/src/interpreter/eval/mod.rs` 增加 `Expr::Null(_) => Ok(DolangValue::Null)`

## 已修复

- `crates/dolang-frontend/src/token/token.rs`
- `crates/dolang-frontend/src/lexer/lexer.rs`
- `crates/dolang-frontend/src/ast/ast.rs`
- `crates/dolang-runtime/src/interpreter/eval/mod.rs`
