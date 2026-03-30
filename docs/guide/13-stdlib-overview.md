# 13. 标准库总览

这一章只回答一个问题：**某个能力应该去哪里找。**

完整 API 查表不在这里，请直接看 [../reference/stdlib-api.md](../reference/stdlib-api.md)。

## 标准库分两类

- native 模块：由 Rust 实现并注册到 runtime
- 纯 `.dol` 模块：放在 `stdlib/` 中

## 当前常见模块

native 模块：

- `std.env`
- `std.fs`
- `std.str`
- `std.math`
- `std.json`
- `std.time`
- `std.uuid`
- `std.http`

纯 `.dol` 模块：

- `std.core.iter`
- `std.core.check`
- `std.str.check`
- `std.str.fmt`
- `std.math.stats`
- `std.math.trig`
- `std.path`

## 先判断是“值方法”还是“模块函数”

### 值方法

这些直接挂在值上：

- `"a,b".split(",")`
- `[1, 2, 3].len()`
- `user.keys()`

适合当前值本身的直接处理。

### 模块函数

这些来自 `std.*`：

```dol
$mod std.str;
$mod std.json;

$>> str.trim("  hello  ");
$ obj = json.set({"name": "Tom"}, "age", 25);
```

适合：

- 工具函数
- JSON 通用处理
- 宿主能力
- 不方便做成值方法的能力

## 按用途找

### 文本与字符串

- 值方法：`trim`、`split`、`contains`
- 模块：`std.str`、`std.str.check`、`std.str.fmt`

### 集合与 JSON

- 值方法：`len`、`keys`、`values`、`flatten`、`sort_desc`
- 模块：`std.json`

### 文件、环境与配置

- 模块：`std.fs`、`std.env`
- 章节入口：[14-io-env-config.md](14-io-env-config.md)

### HTTP

- 服务端语法：第 15、16 章
- 客户端模块：`std.http`

## 查表去哪里

- 完整 stdlib API：[../reference/stdlib-api.md](../reference/stdlib-api.md)
- 旧迁移页：[stdlib.md](stdlib.md)

## 下一章

继续看 [14-io-env-config.md](14-io-env-config.md)。
