# Issue 010：json — 增强操作（has / set / delete / pretty）

**类型**: Feature
**所属 Epic**: E01
**优先级**: P1
**状态**: Closed ✅

## 背景

`std.json` 目前只有 `parse`、`stringify`、`get` 三个函数，
不能做键存在检查、写入、删除、美化输出等基本操作。

## 需要新增的 native 函数

| 函数签名 | 描述 |
|---------|------|
| `json.has(obj, key) -> Bool` | 检查对象是否包含某键 |
| `json.set(obj, key, value) -> Map` | 设置键值，返回新对象（不可变语义）或修改原对象 |
| `json.delete(obj, key) -> Map` | 删除键，返回新对象 |
| `json.keys(obj) -> List` | 返回对象所有键 |
| `json.values(obj) -> List` | 返回对象所有值 |
| `json.pretty(value) -> String` | 美化 JSON 输出（带缩进） |
| `json.merge(a, b) -> Map` | 合并两个 JSON 对象（b 覆盖 a） |

## 设计决策

**`json.set` 的语义**：建议返回新对象（不可变语义），与函数式风格一致；
或修改原 Map，与 `push` 的可变语义一致。需要和语言整体设计保持一致。

## 涉及文件

- `crates/dolang-runtime/src/stdlib_native/json.rs`

## 验收标准

```dolang
$mod std.json;

$ obj = json.parse("{\"name\": \"dolang\", \"version\": 1}");
$>> json.has(obj, "name");      // true
$>> json.has(obj, "missing");   // false
$>> json.keys(obj);             // ["name", "version"]
$ obj2 = json.set(obj, "version", 2);
$>> json.get(obj2, "version");  // 2
$>> json.pretty(obj2);
```
