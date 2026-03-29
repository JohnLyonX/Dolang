# DOC-005: `$catch err` 中 `err` 的类型与可用字段从未说明

**优先级：** P1
**影响章节：** `docs/guide/10-error-handling.md`

## 问题描述

第 10 章讲了 `$try / $catch err` 的用法，但对 `err` 变量本身的性质完全没有说明：

- `err` 是字符串还是对象？
- 能否做字符串比较（`err == "bad input"`）？
- 能否取字段（`err.message`、`err.code`）？
- 如何区分不同类型的错误（`$throw` 的字符串 vs 运行时错误）？

目前用户只能猜测，或者通过 `$>> err` 打印出来观察。

## 期望改进

在 `10-error-handling.md` 补充：

1. `err` 的实际类型（字符串 or 对象）
2. 常用操作示例：
   ```dol
   $try {
       $throw "not found";
   } $catch err {
       $>> err;             // 输出什么？
       $>> err.type();      // 类型是什么？
       $if err == "not found" {  // 字符串比较是否合法？
           $>> "资源不存在";
       }
   }
   ```
3. 运行时错误（如除零、undefined 变量）是否也能被 `$catch` 捕获，`err` 的内容是什么

## 附注

`docs/reference/errors.md` 列出了所有错误消息原文，但没有说明这些消息是否就是 `err` 的值。两个文档应该有交叉引用。
