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
- Never use `unwrap()` or `expect()` in production paths — use proper `Result` propagation
- All errors must return structured `DolangError`, never `panic!` in runtime
- Cross-platform: no `std::os::unix` or `libc` without `#[cfg(unix)]` guard

## Rust Conventions
- Use `#[derive(Debug, Clone, PartialEq)]` on all AST/Value types
- Prefer `match` over `if let` chains for exhaustive handling
- Use `Box<T>` for recursive AST nodes
- Errors via `thiserror` or custom error enums, not `String`
- No `clone()` in hot paths (eval loop) without justification

## Git Workflow
- Active branch: `dev`
- Commit format: `feat(lexer): add += operator token`
- Merge to `main` only when feature is stable and tested

## Tools to Favor
- `Edit`, `Write` for code changes
- `Bash` for `cargo build` / `cargo test` / `cargo clippy`
- `Grep`, `Glob` for locating token/AST/eval code
- `cargo clippy` before every commit
