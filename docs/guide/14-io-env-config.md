# 14. I/O、环境变量与配置

这一章把和宿主环境直接交互的能力收在一起：环境变量、项目配置、文件系统。

## 环境变量

### 语言表面：`$<<ENV(...)`

```dol
$ home = $<<ENV("HOME");
```

如果环境变量不存在，这类调用会失败。

### 标准库：`std.env`

```dol
$mod std.env;
$>> env.get_or("APP_ENV", "dev");
```

当你需要兜底值时，优先选 `env.get_or(...)`。

## 项目配置：`$<<CONFIG(...)`

```dol
$ app = $<<CONFIG("APP_NAME");
```

当前实现里，它有两个重要边界：

- 依赖项目配置上下文
- 只在 `serve` 模式下可用

也就是说，在 `run` 模式里把它当成通用配置读取入口会直接报错。

### 推荐的跨模式写法

如果你希望脚本和服务都能运行，先用环境变量作为兜底主路径：

```dol
$mod std.env;

$ app = env.get_or("APP_NAME", "My App");
$>> app;
```

如果某段代码只在服务项目里运行，再使用 `$<<CONFIG(...)`。

## 文件读取：`$<<FILE(...)`

当前语言层面可写的形式是：

```dol
$ text = $<<FILE("notes.txt");
$ lines = $<<FILE("notes.txt", "LINES");
```

口径统一如下：

- `$<<FILE(path)`：读取全文，结果按文本处理
- `$<<FILE(path, "LINES")`：按行读取
- 第二个参数当前主线只写 `"LINES"`，不继续扩展旧文档里的其他模式字符串

## `std.fs` 和 `$<<FILE(...)` 怎么选

如果你只是想在一行里读一次文件，`$<<FILE(...)` 足够直观。

如果你需要：

- 更明确的 API
- 写入、追加、删除
- `exists`、`is_dir`、`list` 这类能力

优先用 `std.fs`：

```dol
$mod std.fs;

$ content = fs.read_text("tests/fixtures/stdlib/hello.txt");
$ lines = fs.read_lines("tests/fixtures/stdlib/lines.txt");
```

可以把两者这样区分：

- `$<<FILE(...)`：语言层面的简写入口
- `std.fs.read_text/read_lines(...)`：脚本和项目里的主推荐写法

## 文件写入

```dol
$mod std.fs;

fs.write_text("/tmp/dolang-demo.txt", "hello");
$>> fs.read_text("/tmp/dolang-demo.txt");
fs.delete("/tmp/dolang-demo.txt");
```

## 常见坑

- 环境变量不存在会抛错，除非你用 `get_or`
- `$<<CONFIG(...)` 不是跨模式通用入口
- 文件路径错误会导致运行时错误
- 有副作用的能力最好配合 `$try / $catch`

## 下一章

继续看 [15-http-basics.md](15-http-basics.md)。
