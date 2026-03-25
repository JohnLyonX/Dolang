# Issue 013：io — 格式化字符串（format）

**类型**: Feature
**所属 Epic**: E01
**优先级**: P1
**状态**: Closed — Won't Fix

## 关闭原因

`io.format(template, args)` 与 f-string `f"{var}"` 功能重叠。
Dolang 的 f-string 已在运行时完成变量插值，无需额外的格式化函数。

## 原背景

Dolang 已有原生语法覆盖了全部 I/O 需求：
- `$>>` — 输出到 stdout
- `$<<LINE` — 从 stdin 读取一行
- `$>>FILE` / `$<<FILE` — 文件读写
- `$<<ENV` — 读取环境变量
- f-string `f"{var}"` — 编译期插值

唯一缺失的是**运行时动态模板格式化**：模板字符串在运行时才确定，无法用 f-string 表达。

## 需要新增的 native 函数

### `std.io` 模块（新建 `stdlib_native/io.rs`）

| 函数签名 | 描述 |
|---------|------|
| `io.format(template, args) -> String` | 运行时格式化，`{}` 占位符按顺序替换 |

## 设计

```dolang
$ template = "Hello, {}! You have {} messages.";
$ msg = io.format(template, ["Alice", 3]);
$>> msg;
// 输出: Hello, Alice! You have 3 messages.
```

与 f-string 的区别：f-string 在编译期插值变量，`io.format` 在运行时替换占位符，
适合模板从外部读取（如配置文件、用户输入）的场景。

## 实现逻辑

1. 按顺序将 `{}` 替换为 `args` 列表中对应元素的字符串表示
2. `{}` 数量与 `args.len()` 不匹配时报错

## 涉及文件

- 新建 `crates/dolang-runtime/src/stdlib_native/io.rs`
- `crates/dolang-runtime/src/stdlib_native/mod.rs`：注册 `std.io`

## 验收标准

```dolang
$mod std.io;

$ msg = io.format("Hello, {}! Score: {}", ["Alice", 100]);
$>> msg;
// Hello, Alice! Score: 100
```
