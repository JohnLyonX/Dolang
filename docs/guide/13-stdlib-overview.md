# 13. 标准库总览

Dolang 当前的标准库分成两类：

- native 模块：由 Rust 实现并注册到 runtime
- 纯 `.dol` 模块：放在 `stdlib/` 中

如果你想快速判断“某个能力应该去哪里找”，这一章先给你一个总地图。

## 当前可见的主要模块

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

## 按用途理解标准库

### 字符串与文本

- `std.str`
- `std.str.check`
- `std.str.fmt`

```dol
$mod std.str;
$mod std.str.fmt;

$>> str.trim("  hello  ");
$>> fmt.pad_left("42", 5, "0");
```

### 数学与数值

- `std.math`
- `std.math.stats`
- `std.math.trig`

```dol
$mod std.math;
$mod std.math.stats;

$>> math.sqrt(16.0);
$>> stats.mean([1, 2, 3, 4, 5]);
```

### JSON 与数据处理

- `std.json`
- `std.path`

```dol
$mod std.json;
$mod std.path;

$ obj = json.parse("{\"name\":\"dolang\"}");
$>> json.get(obj, "name");
$>> path.basename("/usr/local/bin");
```

### 文件与环境变量

- `std.fs`
- `std.env`

```dol
$mod std.env;
$mod std.fs;

$>> env.get_or("APP_ENV", "dev");
$>> fs.exists("README.md");
```

### 时间、唯一 ID 与服务调用

- `std.time`
- `std.uuid`
- `std.http`

```dol
$mod std.time;
$mod std.uuid;
$mod std.http;

$>> time.format(0, "%Y-%m-%d");
$>> uuid.is_valid("550e8400-e29b-41d4-a716-446655440000");
```

### 基础工具

- `std.core.iter`
- `std.core.check`

```dol
$mod std.core.iter;
$mod std.core.check;

$>> iter.range(0, 5);
$>> check.type_of(42);
```

## 更细一点的模块地图

如果你已经学到这里，可以把常见模块先这样记：

- 文本处理：`std.str`、`std.str.check`、`std.str.fmt`
- 数学计算：`std.math`、`std.math.stats`、`std.math.trig`
- JSON 处理：`std.json`
- 路径处理：`std.path`
- 文件和目录：`std.fs`
- 环境变量：`std.env`
- 时间处理：`std.time`
- 唯一 ID：`std.uuid`
- HTTP 客户端：`std.http`
- 通用工具：`std.core.iter`、`std.core.check`

例如 `std.path` 里除了 `basename`，还可以继续留意：

- `join`
- `dirname`
- `ext`

例如 `std.json` 里除了 `parse` / `stringify`，还可以继续留意：

- `get`
- `has`
- `keys`
- `values`
- `set`
- `delete`
- `pretty`
- `merge`

## 一个最小例子

```dol
$mod std.math;
$mod std.str;

$>> math.sqrt(16.0);
$>> str.trim("  hello  ");
```

## 什么时候用标准库

建议把标准库理解成三层用途：

- 基础字符串、数学、JSON、文件、环境变量、时间、UUID、HTTP
- 一些以 `.dol` 编写的组合工具
- 未来继续扩展的公共模块入口

## 第一次学时怎么用

建议第一次先掌握这几个模块：

1. `std.str`
2. `std.math`
3. `std.json`
4. `std.fs`
5. `std.env`

然后再补：

- `std.time`
- `std.uuid`
- `std.http`
- `std.core.iter`
- `std.core.check`
- `std.str.fmt`
- `std.path`

## 一个更完整的标准库组合例子

```dol
$mod std.fs;
$mod std.json;
$mod std.path;
$mod std.time;
$mod std.uuid;

$ raw = fs.read_text("tests/fixtures/stdlib/hello.txt");
$>> raw;
$>> path.basename("/usr/local/bin");
$>> time.year(0);
$>> uuid.is_valid("550e8400-e29b-41d4-a716-446655440000");
$>> json.stringify({"ok": true});
```

## 迁移期参考

- [stdlib.md](stdlib.md)
- `docs/stdlib/README.md`

## 下一章

继续看 [14-io-env-config.md](14-io-env-config.md)。
