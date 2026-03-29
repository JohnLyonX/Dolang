# DOC-003: `-> List` 裸类型声明已被强制拒绝，文档未反映此变更

**优先级：** P0
**影响章节：** `docs/guide/09-gradual-typing.md`、`docs/guide/15-http-basics.md`

## 问题描述

运行时已强制要求：凡是声明了 `-> List` 返回类型（不带类型参数），在函数调用或 HTTP handler 触发时会报错：

```
http handler 'getUsers' declares return type 'List' without a type parameter;
use 'List<T>' instead (e.g. 'List<User>')
```

但 `09-gradual-typing.md` 和 `15-http-basics.md` 在介绍返回类型时：
- 没有说明 `List` 裸类型不可用于返回类型声明
- 没有给出 `List<T>` 的任何示例
- 有些示例用的是 `-> JSON`，但 JSON 化的 List 用法没有说明

此外，`-> List<User>` 与 `$Type User` 的关联关系（必须先有 `$Type` 定义才能用 `List<User>`）也没有文档说明。

## 期望改进

1. 在 `09-gradual-typing.md` 的返回类型注解部分，增加：
   - `List<T>` 是正确写法，`List` 裸类型不可用于返回类型声明
   - `List<User>` 与 `$Type User` 的关联：需要先定义类型才能使用

2. 在 `15-http-basics.md` 的返回类型章节，补充 `List<T>` 的完整示例：
   ```dol
   $Type User {
       id: Int
       name: Str
   }

   $GET("/users") list_users() -> List<User> {
       $# [User { id: 1, name: "Alice" }];
   }
   ```

3. 在附录 A 语法速查表中补充返回类型的合法写法列表。
