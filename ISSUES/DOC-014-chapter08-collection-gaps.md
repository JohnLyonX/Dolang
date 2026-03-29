# DOC-014: 第 8 章集合方法存在多处缺口

**优先级：** P3
**影响章节：** `docs/guide/08-collections-and-methods.md`

## 问题描述

第 8 章覆盖了大部分集合操作，但有以下具体缺口：

### 1. List 缺失方法

旧版 `stdlib.md` 中有，第 8 章没有：
- `list.flatten()` — 展平嵌套 List
- `list.sort_desc()` — 降序排列
- `list.count(val)` — 虽然列出了，但没有示例
- `list.map(fn)` — 映射函数（是否支持？）
- `list.filter(fn)` — 过滤函数（是否支持？）

### 2. Map 缺失方法

- `map.len()` — 旧 stdlib.md 有，第 8 章无
- Map 的写入操作（见 DOC-004，这里不重复）

### 3. 索引越界行为未说明

```dol
$ items = [1, 2, 3];
$>> items[10];   // 返回 null？还是报错？
```

文档完全没有提这个边界行为，用户写代码时无法预测。

### 4. String 和 List 的嵌套操作未覆盖

`split` 返回 List，但 `split` 后如何进一步操作没有完整示例。

## 期望改进

1. 补充 `flatten()`、`sort_desc()` 的说明和示例
2. 补充 `map.len()` 的说明
3. 在索引访问小节补充越界行为说明
4. 若 `list.map(fn)` / `list.filter(fn)` 支持高阶函数，在本章补充；若不支持，说明推荐的替代方式（`$for` 循环）
