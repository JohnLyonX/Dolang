# DOC-008: 哪些运行时错误可以被 `$catch` 捕获，文档没有说明

**优先级：** P1
**影响章节：** `docs/guide/10-error-handling.md`、`docs/reference/errors.md`

## 问题描述

第 10 章只举了 `$throw` 主动抛出被 `$catch` 捕获的例子，但没有说明：

- 除零错误、未定义变量、类型不匹配等**运行时错误**是否也能被 `$catch` 捕获？
- 模块加载失败、文件读取失败是否可以被 `$catch`？
- 解析错误（语法错误）一定不能被 `$catch`，但这个边界在文档里不清晰。

用户可能会写出这样的代码并期望它能正常工作：

```dol
$try {
    $ x = undefined_var;   // 能被捕获吗？
} $catch err {
    $>> "caught: " + err;
}
```

但不知道这是否合法。

## 期望改进

在 `10-error-handling.md` 补充"可捕获错误 vs 不可捕获错误"的说明：

| 错误类型 | 示例 | 可 catch？ |
|---------|------|-----------|
| 主动抛出 | `$throw "msg"` | 是 |
| 除零 | `$ x = 1 / 0` | ? |
| 未定义变量 | `$>> undefined_var` | ? |
| 类型不匹配 | 赋值类型错误 | ? |
| 文件读取失败 | `$<<FILE("no.txt")` | ? |
| 模块加载失败 | `$mod not.exist` | ? |
| 语法错误 | 缺少分号 | 否（解析期） |

同时在 `docs/reference/errors.md` 中为每类错误标注是否可被 `$catch` 捕获。
