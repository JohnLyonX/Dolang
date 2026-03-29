# DOC-004: Map 写入/赋值语法在所有文档中均未说明

**优先级：** P1
**影响章节：** `docs/guide/08-collections-and-methods.md`

## 问题描述

第 8 章详细讲了 Map 的读取操作（`map["key"]`、`.keys()`、`.values()` 等），但完全没有说明如何向 Map 写入或修改值。

用户看完第 8 章后不知道：
- `map["key"] = value` 是否合法？
- 是否有 `map.set("key", value)` 这样的方法？
- 如何初始化后再追加字段？
- `std.json.set()` 和语言层 Map 赋值有什么区别？

旧版 `stdlib.md` 里有 `std.json.set(obj, key, value)` 的说明，但那是标准库函数，不是语言原生语法。

## 期望改进

在 `08-collections-and-methods.md` 的 Map 部分补充：

1. 语言层 Map 写入语法（若支持 `map["key"] = value`，给出示例）
2. 若不支持语言层直接赋值，说明推荐的替代方案（`std.json.set`）
3. 区分"创建时初始化"与"运行时修改"两种场景的写法

示例（期望文档覆盖）：
```dol
$ user = { "name": "Alice", "age": 30 };

// 方式 A（若支持）：
user["age"] = 31;

// 方式 B（标准库）：
$mod std.json;
$ updated = json.set(user, "age", 31);
```
