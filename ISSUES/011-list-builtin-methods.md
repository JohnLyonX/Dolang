# Issue 011：list 内置方法补齐（sort / index_of / slice / flatten / unique）

**类型**: Feature
**所属 Epic**: E01
**优先级**: P1
**状态**: Closed ✅

## 背景

List 目前有 `len`、`contains`、`join`、`push`、`pop`、`reverse`，
缺少排序、查找、切片等常用操作，且这些无法通过纯 .dol 高效实现
（排序需要原地比较，slice 需要 Rust 级别的内存操作）。

## 需要新增的方法

### 不可变方法（读操作）

| 方法 | 描述 |
|------|------|
| `list.index_of(x) -> Int` | 返回第一个匹配元素的索引，不存在返回 -1 |
| `list.last_index_of(x) -> Int` | 返回最后一个匹配元素的索引 |
| `list.slice(start, end) -> List` | 返回子列表 `[start, end)`，支持负索引 |
| `list.first() -> Any` | 返回第一个元素（等价于 `list[0]`，但更安全） |
| `list.last() -> Any` | 返回最后一个元素 |
| `list.is_empty() -> Bool` | 检查是否为空 |
| `list.flatten() -> List` | 将嵌套列表展平一层 |
| `list.unique() -> List` | 去除重复元素，保持顺序 |
| `list.count(x) -> Int` | 统计某值出现次数 |

### 可变方法（写操作）

| 方法 | 描述 |
|------|------|
| `list.sort()` | 原地升序排序（元素须可比较） |
| `list.sort_desc()` | 原地降序排序 |
| `list.remove_at(index)` | 删除指定索引的元素 |
| `list.insert(index, value)` | 在指定位置插入元素 |
| `list.clear()` | 清空列表 |

## 涉及文件

- `crates/dolang-runtime/src/interpreter/builtins/list.rs`：新增上述方法
- `crates/dolang-runtime/src/interpreter/builtins/mod.rs`：`is_method_mutating` 增加新的可变方法名

## 设计注意

- `sort()` 对混合类型列表（如 Int + String）的行为需要定义（建议报错）
- `slice` 负索引语义与 Python 保持一致（-1 = 最后一个元素）
- `flatten` 只展平一层（`[[1,[2]],3]` → `[1,[2],3]`）

## 验收标准

```dolang
$ nums = [3, 1, 4, 1, 5, 9, 2, 6];
$>> nums.index_of(5);      // 4
$>> nums.slice(2, 5);      // [4, 1, 5]
$>> nums.unique();         // [3, 1, 4, 5, 9, 2, 6]
nums.sort();
$>> nums;                  // [1, 1, 2, 3, 4, 5, 6, 9]
$>> nums.is_empty();       // false
```
