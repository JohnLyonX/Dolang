# DOC-006: `$main() {}` 入口点没有专门解释

**优先级：** P1
**影响章节：** `docs/guide/16-http-organization.md`、`docs/guide/12-projects-and-package.md`

## 问题描述

`$main() {}` 是 serve 模式的程序入口，用于放置全局初始化逻辑（如全局 `@CORS` 声明）。但在所有 guide 章节中，它只在示例代码里隐式出现，从未得到正面解释：

- 什么时候必须写 `$main()`？什么时候不用？
- `$main()` 里可以写什么？限制是什么？
- 如果不写 `$main()`，`dolang serve` 如何找到入口？
- `$main()` 和 `package.toml` 的 `entry` 字段是什么关系？
- `@CORS("*")` 写在 `$main()` 上是全局 CORS，写在别处有什么不同？

旧版 `web.md` 有一句简单说明，但新版 15、16 章均未覆盖。

## 期望改进

在 `12-projects-and-package.md` 或 `16-http-organization.md` 中新增 `$main()` 专题段落：

1. `$main()` 的作用：serve 模式的顶层入口，执行顺序在所有路由注册之前
2. 什么情况下需要写（有全局配置时），什么情况下可以省略
3. 全局 `@CORS` 必须写在 `$main()` 上的原因
4. 完整示例：
   ```dol
   @CORS("*")
   $main() {
       $>> "[INFO] server starting";
   }

   $GET("/health") health() -> String {
       $# "ok";
   }
   ```
