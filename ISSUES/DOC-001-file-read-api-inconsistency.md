# DOC-001: `$<<FILE()` API 在两处文档描述不一致

**优先级：** P0
**影响章节：** `docs/guide/14-io-env-config.md`、`docs/reference/syntax.md`、旧版 `docs/guide/examples.md`

## 问题描述

`$<<FILE()` 是语言层面的文件读取语法，但其参数形式和返回值在不同文档中的说法相互矛盾：

**说法 A（reference/syntax.md 的暗示）：**
```dol
$ content = $<<FILE("data.txt");
// 返回 File 对象
```

**说法 B（旧版 examples.md 的用法）：**
```dol
$ lines = $<<FILE("data.txt", "lines");   // 带第二参数，返回 List
$ text  = $<<FILE("data.txt", "text");    // 带第二参数，返回 String
```

用户不清楚：
- 单参数和双参数是否都合法？
- 不带第二参数时返回什么？
- `"lines"` 和 `"text"` 是固定的模式字符串还是其他含义？

## 期望改进

1. 在 `14-io-env-config.md` 中明确说明 `$<<FILE()` 的完整 API：
   - 参数形式（一个还是两个？）
   - 每种形式的返回值类型
   - 和 `std.fs.read_text` / `std.fs.read_lines` 的关系与选用建议

2. 删除或标注旧版 `examples.md` 中与当前实现不一致的用法。

## 验证方法

查看 `crates/dolang-frontend/src/parser/expr/primary.rs` 中 `$<<FILE` token 的解析逻辑，以及 `crates/dolang-runtime/src/interpreter/eval/` 中对应的求值实现，确认实际支持的参数形式。
