# DOC-015: `$<<CONFIG()` 在非 serve 模式下的行为未文档化

**优先级：** P3
**影响章节：** `docs/guide/12-projects-and-package.md`、`docs/guide/14-io-env-config.md`

## 问题描述

`$<<CONFIG("APP_NAME")` 只在 `serve` 模式下可用（读取 `package.toml [env]`）。文档提到了这个限制，但没有说明：

- 在 `dolang run` 模式下调用 `$<<CONFIG(...)` 会发生什么？（报错？返回 null？静默返回空字符串？）
- 用户如何写同时兼容 run 和 serve 模式的代码？

这个问题在用户把脚本迁移到 serve 模式时很容易踩到。

## 期望改进

在 `14-io-env-config.md` 的 `$<<CONFIG` 段落补充：

```dol
// serve 模式：正常读取
$ name = $<<CONFIG("APP_NAME");   // 返回 "My App"

// run 模式：
$ name = $<<CONFIG("APP_NAME");   // 行为是？
```

同时说明推荐的跨模式兼容写法：
```dol
// 优先读 CONFIG，回退到 ENV
$mod std.env;
$ name = $<<CONFIG("APP_NAME");
$if name == null {
    $ name = env.get_or("APP_NAME", "default");
}
```
