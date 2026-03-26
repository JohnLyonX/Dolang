# 14. I/O、环境变量与配置

这一章把和宿主环境直接交互的能力收在一起。

这一类能力和“纯表达式计算”不同，因为它们会读取操作系统、项目配置或文件系统。

## 环境变量

### 语言表面：`$<<ENV(...)`

```dol
$ home = $<<ENV("HOME");
```

如果环境变量不存在，这类调用会失败，因此实际脚本里常常需要配合错误处理。

### 标准库：`std.env`

```dol
$mod std.env;
$>> env.get_or("APP_ENV", "dev");
```

当前 `std.env` 还包括：

- `get`
- `get_or`
- `has`
- `all`
- `set`
- `remove`

其中 `set` 和 `remove` 在 serve 模式下受限制。

### 常见写法

```dol
$mod std.env;

$>> env.has("HOME");
$>> env.get_or("APP_ENV", "dev");
```

## 项目配置

```dol
$ app = $<<CONFIG("APP_NAME");
```

`$<<CONFIG(...)` 依赖项目配置，并且只在 serve 模式下可用。

也就是说，如果你只是执行一个普通文件而没有对应项目上下文，不要把它当成通用配置读取方式。

## 文件读写

### 语言表面

```dol
$ content = $<<FILE("data.txt");
$>>FILE("out.txt", "hello");
```

这是语言层面的直接文件入口。

### 标准库

```dol
$mod std.fs;
fs.write_text("/tmp/demo.txt", "ok");
$>> fs.read_text("/tmp/demo.txt");
```

当前 `std.fs` 常用函数包括：

- `read_text`
- `read_lines`
- `write_text`
- `append_text`
- `delete`
- `exists`
- `size`
- `is_dir`
- `list`
- `mkdir`
- `mkdir_all`
- `rmdir`
- `copy`
- `rename`

### 读取文本

```dol
$mod std.fs;
$ content = fs.read_text("tests/fixtures/stdlib/hello.txt");
$>> content;
```

### 按行读取

```dol
$mod std.fs;
$ lines = fs.read_lines("tests/fixtures/stdlib/lines.txt");
$for line in lines {
    $>> line;
}
```

### 写入再读取

```dol
$mod std.fs;
fs.write_text("/tmp/dolang-demo.txt", "hello");
$>> fs.read_text("/tmp/dolang-demo.txt");
fs.delete("/tmp/dolang-demo.txt");
```

## 推荐用法

如果只是偶尔读一行输入，用 `$<<LINE(...)`。

如果是脚本和系统交互，优先考虑：

- `std.env`
- `std.fs`
- `$<<CONFIG(...)`

## 常见坑

- 缺失环境变量会报错，除非你用 `get_or`
- `$<<CONFIG(...)` 不是随时都能用
- 文件路径错误会导致运行时错误
- 有副作用的能力最好配合 `$try / $catch`

## 进一步参考

- `docs/runtime/intrinsics.md`
- `docs/spec/security-model.md`

## 下一章

继续看 [15-http-basics.md](15-http-basics.md)。
