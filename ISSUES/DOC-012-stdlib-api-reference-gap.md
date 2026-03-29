# DOC-012: 旧 `stdlib.md` 是目前最完整的 API 参考，但处于迁移期

**优先级：** P2
**影响章节：** `docs/guide/stdlib.md`（旧）、`docs/guide/13-stdlib-overview.md`（新）

## 问题描述

旧版 `docs/guide/stdlib.md` 目前仍是项目中**最完整的内置方法 + 标准库 API 参考**，包含：

- `Int` / `Float` 的 `.to_str()` 等方法
- `String` 的完整方法列表（新版第 8 章未完全覆盖）
- `List` 的 `flatten()`、`sort_desc()` 等第 8 章没有的方法
- 所有 `std.*` 模块的函数签名与参数说明

但这个文件处于"迁移期旧页面"状态，不在主线阅读路径上。新版 `13-stdlib-overview.md` 只是一个"总览"，没有完整的 API 参考功能，用户无处查到完整 API。

**具体缺口：**

| 内容 | 旧 stdlib.md | 新 13 章 |
|------|-------------|---------|
| Int/Float 方法 | 有 | 无 |
| String 完整方法 | 有（约 20 个）| 有（约 13 个）|
| List.flatten() | 有 | 无 |
| List.sort_desc() | 有 | 无 |
| std.time 完整函数列表 | 有 | 只有 1-2 个示例 |
| std.uuid 完整函数列表 | 有 | 只有 is_valid |
| std.http 完整参数说明 | 有 | 只有 3 行示例 |

## 期望改进

**方案 A（推荐）：** 新建 `docs/reference/stdlib-api.md`，整合旧 `stdlib.md` 的完整内容，作为独立的 API 参考文档，与 guide 章节并列。

**方案 B：** 将旧 `stdlib.md` 正式纳入主线，在 `13-stdlib-overview.md` 末尾加"完整 API 参考"链接指向它。

无论哪种方案，都需要：
1. 核实旧文档中每个函数的当前可用性（运行时实现是否存在）
2. 对已移除或改名的函数打上标注
3. 稳定性等级（stable/preview/experimental）标注
