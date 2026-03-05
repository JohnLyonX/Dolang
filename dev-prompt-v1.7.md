# Development Context

Mode: Active development — Dolang Interpreter (Rust)
Focus: Interpreter implementation, language features, runtime correctness

## Behavior

- Write code first, explain after
- Prefer working solutions over perfect solutions
- Always run `cargo build` after changes to verify compilation
- Run `cargo test` after logic changes
- Keep commits atomic — one feature/fix per commit
- Default branch: `dev` — never commit directly to `main`

## Priorities

1. Get it working — compiles and runs correctly
2. Get it right — correct language semantics
3. Get it clean — idiomatic Rust, readable code

## Interpreter-Specific Rules

- Lexer changes must be followed by parser review
- Parser changes must be followed by AST review
- AST changes must be followed by interpreter eval/exec review
- `value.rs` 枚举变更必须同步检查：`PartialEq` 手动实现、`Display` 实现、`type_name()`、`builtins/mod.rs` 的 `dispatch` / `dispatch_mut` 分支
- 新增内置方法的标准路径：`builtins/<type>.rs` → `builtins/mod.rs` 分发注册 → 若为可变方法同步更新 `is_mutating()`
- 可变方法（`push`/`pop`/`reverse`/`remove`）走 `dispatch_mut` + `call_mut`，不可变方法走 `dispatch` + `call`，禁止混用
- Never use `unwrap()` or `expect()` in production paths — use proper `Result` propagation
- All errors must return structured `DolangError`, never `panic!` in runtime
- Cross-platform: no `std::os::unix` or `libc` without `#[cfg(unix)]` guard

## Rust Conventions

- AST 节点使用 `#[derive(Debug, Clone, PartialEq)]`
- `DolangValue` 不 derive `PartialEq` — 必须手动实现（`f64` NaN 陷阱 + `Function` 不可比较）
- 实现 `std::fmt::Display` trait 而非自定义 `display()` 方法
- 函数参数优先用 `&[T]` 切片而非 `Vec<T>` 转移所有权（尤其是 `builtins` 的 `args` 参数）
- Prefer `match` over `if let` chains for exhaustive handling
- Use `Box<T>` for recursive AST nodes
- Errors via `thiserror` or custom error enums, not `String`
- No `clone()` in hot paths (eval loop) without justification — `builtins/call_mut` 中的 clone 需注释说明原因
- `builtins/` 下文件命名避免与 std 冲突：用 `str_methods.rs` / `bool_methods.rs` 而非 `string.rs` / `bool.rs`
- 过渡期兼容方法命名用 `parse_legacy` / `to_legacy`，不用 `from_raw` / `to_raw`（后者在 Rust 中暗示 unsafe 裸指针）

## Module Responsibilities (v1.7)

```
eval.rs        — 表达式求值调度，不包含任何内置方法实现
exec.rs        — 语句执行
env.rs         — 环境与变量存储（HashMap<String, DolangValue>）
value.rs       — DolangValue 枚举 + Display + PartialEq + type_name
builtins/
  mod.rs       — dispatch / dispatch_mut / is_mutating 分发入口
  list.rs      — List 的 call + call_mut
  map.rs       — Map 的 call + call_mut
  number.rs    — Int / Float 的 call（无 call_mut）
  str_methods.rs  — String 的 call（无 call_mut）
  bool_methods.rs — Bool 的 call（无 call_mut）
```

## Git Workflow

- Active branch: `dev`
- Commit format: `feat(lexer): add += operator token`
- Merge to `main` only when feature is stable and tested

## Tools to Favor

- `Edit`, `Write` for code changes
- `Bash` for `cargo build` / `cargo test` / `cargo clippy`
- `Grep`, `Glob` for locating token/AST/eval code
- `cargo clippy` before every commit
