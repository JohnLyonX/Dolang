# DOC-002: 通配符导入 `$mod math.*` 的行为在两处描述矛盾

**优先级：** P0
**影响章节：** `docs/guide/11-modules.md`、旧版 `docs/guide/modules.md`

## 问题描述

`$mod path.*` 通配符导入的语义在两个文档中说法不同：

**新版 11-modules.md（guide README 中的说法）：**
> 通配符导入会将子模块注册为命名空间，仍然通过 `submodule.fn()` 的方式访问。

**旧版 modules.md：**
> 通配符导入会将模块内的所有公有函数"铺平"到当前命名空间，可以直接调用 `fn()` 而不需要前缀。

这两种语义完全相反。用户写了 `$mod math.*` 之后，不知道该用 `math.sqrt(x)` 还是直接 `sqrt(x)`。

## 期望改进

1. 确认当前运行时的实际行为（查看 `exec/modules.rs`）。
2. 在 `11-modules.md` 中用可运行示例明确说明通配符导入后的访问方式。
3. 旧版 `modules.md` 中的矛盾说法打上弃用标注或删除。

## 验证方法

```dol
$mod std.math.*;
$>> sqrt(4);       // 是否报错？
$>> math.sqrt(4);  // 是否可用？
```

在 `dolang run` 下实际运行，以运行时行为为准更新文档。
