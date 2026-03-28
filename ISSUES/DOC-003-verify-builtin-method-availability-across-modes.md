# DOC-003 验证内置方法与 stdlib 在不同运行模式下的可用性

## 类型

- Documentation
- Verification

## 优先级

- P1

## 背景

补文档前，需要先确认当前代码里的内置方法和 stdlib 函数，是否真的能在以下模式下使用：

- REPL
- `dolang run x.dol`
- `dolang serve .`

本 issue 只记录验证结果，不修改 Rust 解释器实现。

## 复现步骤

```bash
cargo run -- run func_test/run/doc002_run.dol
cargo run < func_test/repl/doc002_repl.dol
cargo run < func_test/repl/doc002_repl_std_modules.dol
cd func_test/serve && cargo run -- serve .
cargo run -- run func_test/run/doc002_serve_probe.dol
```

## 实际结果

| 项目 | REPL | run | serve | 结论 |
|---|---|---|---|---|
| `File` / `Json` / `Html` 值方法 | ✅ 纯内建可用 | ✅ | ✅ | 能力存在，文档缺失 |
| `std.time` | ❌ `$mod std.time` 报 `module not found` | ✅ | ✅ | REPL 模式不可达 |
| `std.http` | ❌ `$mod std.http` 报 `module not found` | ✅ | ⚠️ 自调用探针 `502` | serve 内同进程回环结果不稳定 |
| `std.uuid` | ❌ `$mod std.uuid` 报 `module not found` | ✅ | ✅ | REPL 模式不可达 |
| `length()` 别名 | ✅ | ✅ | ✅ | 能力存在，文档缺失 |
| `String.slice(-1, 5)` | ❌ `invalid slice indices` | ❌ | ❌ | 文档与实现不一致 |
| `s.len()` vs `str.len(s)` | ⚠️ `std.str` 在 REPL 不可达 | ✅ `4 vs 2` | ✅ `4 vs 2` | 文档未区分语义 |

## 关键问题

- REPL 下 `std.str`、`std.time`、`std.http`、`std.uuid` 当前都无法通过 `$mod` 正常导入。
- `std.http` 在脚本模式可用，但 `serve` 内对自身发请求时返回 `502`，且退出时伴随 tokio runtime panic；这更像测试方式副作用，不应直接记为功能缺失。
- `String.slice()` 文档写“支持负索引”，但实际不支持。
- `s.len()` 与 `str.len(s)` 分别是 byte 长度和字符长度，文档未区分。

## 影响

- 文档如果直接宣称这些能力在所有模式下都可用，会误导用户。
- REPL 示例里使用 `std.*` 模块时，当前实现会直接失败。
- `String.slice()` 和字符串长度语义的文档偏差会导致错误预期。

## 验收标准

- [ ] 文档区分“能力已实现”和“某模式当前不可达”
- [ ] 文档补齐 `File` / `Json` / `Html` 值方法表
- [ ] 文档补齐 `std.time`、`std.http`、`std.uuid` 函数表
- [ ] 文档补入 `length()` 别名
- [ ] 文档修正 `String.slice()` 负索引描述，或实现补齐该能力
- [ ] 文档明确区分 `s.len()` 与 `str.len(s)` 的语义

## 相关文件

- `func_test/run/doc002_run.dol`
- `func_test/run/doc002_serve_probe.dol`
- `func_test/repl/doc002_repl.dol`
- `func_test/repl/doc002_repl_std_modules.dol`
- `func_test/serve/main.dol`
- `ISSUES/DOC-002-audit-missing-builtins-and-doc-gaps.md`
