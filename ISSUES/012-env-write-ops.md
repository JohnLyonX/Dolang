# Issue 012：env — 写操作支持（set / remove / all）

**类型**: Feature
**所属 Epic**: E01
**优先级**: P2
**状态**: Closed ✅

## 背景

`std.env` 目前只有 `get(key)` 和 `get_or(key, default)` 两个只读函数，
无法设置、删除环境变量，也无法枚举全部变量。

## 需要新增的 native 函数

| 函数签名 | 描述 |
|---------|------|
| `env.set(key, value)` | 设置环境变量（影响当前进程） |
| `env.remove(key)` | 删除环境变量 |
| `env.has(key) -> Bool` | 检查环境变量是否存在（不抛错） |
| `env.all() -> Map` | 返回所有环境变量（键值对 Map） |

## 涉及文件

- `crates/dolang-runtime/src/stdlib_native/env.rs`

## 安全考虑

`env.set` 会修改当前进程的环境变量，在 serve 模式下可能有安全风险。
可考虑在 serve 模式下禁用 `set` 和 `remove`（与 config.get 的限制逻辑对称）。

## 验收标准

```dolang
$mod std.env;

$>> env.has("HOME");         // true
$>> env.has("NONEXISTENT");  // false
env.set("MY_VAR", "hello");
$>> env.get("MY_VAR");       // "hello"
env.remove("MY_VAR");
$>> env.has("MY_VAR");       // false
```
