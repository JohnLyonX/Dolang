# 10. 错误处理

Dolang 当前同时有解析错误、运行时错误和显式抛出的异常。

可以先把它们粗分成三类：

- 代码还没跑起来就出错：解析错误
- 跑到一半因为值或环境问题出错：运行时错误
- 代码里主动抛出：`$throw`

## try / catch

```dol
$try {
    $throw "oops";
} $catch err {
    $>> err;
}
```

如果 `try` 块内部出错，就会进入 `catch` 块。

还可以把“可能失败的宿主能力调用”包起来：

```dol
$mod std.fs;
$try {
    $ content = fs.read_text("missing.txt");
    $>> content;
} $catch err {
    $>> "caught!";
}
```

## throw

```dol
$throw "bad input";
```

如果没有被捕获，会变成未捕获错误。

它适合：

- 主动拒绝非法输入
- 在工具函数里显式抛出失败
- 配合 `check.assert(...)` 之类的封装

例如：

```dol
$fn must_positive(n) {
    $if n <= 0 {
        $throw "n must be positive";
    }
    $# n;
}
```

## 常见错误来源

- 未定义变量
- 类型不匹配
- 模块找不到
- 文件读取失败
- JSON 解析失败

### 1. 未定义变量

```dol
$>> missing;
```

### 2. 类型不匹配

```dol
$ value: Int = "text";
```

### 3. 模块找不到

```dol
$mod missing.module;
```

### 4. 文件不存在

```dol
$mod std.fs;
$ content = fs.read_text("missing.txt");
```

## 如何阅读错误

排查时建议按这个顺序看：

1. 是 parse error 还是 runtime error
2. 出错的是哪一行
3. 是语法问题、值类型问题，还是模块/文件问题
4. 能否先在 REPL 或最小 `.dol` 文件里复现

## try/catch 的实际价值

如果你不捕获错误，程序会直接失败。

如果你捕获错误，就可以：

- 输出更友好的消息
- 给默认值
- 跳过失败分支继续执行

例如环境变量读取失败时：

```dol
$mod std.env;
$try {
    $ token = env.get("APP_TOKEN");
    $>> token;
} $catch err {
    $>> "env error caught";
}
```

## 一个完整例子

```dol
$mod std.json;

$fn parse_or_default(raw) -> String {
    $try {
        $ value = json.parse(raw);
        $# json.stringify(value);
    } $catch err {
        $# "{}";
    }
}

$>> parse_or_default("{\"ok\":true}");
```

## 进一步参考

- `docs/reference/errors.md`
- [common-errors.md](appendix/common-errors.md)

## 下一章

继续看 [11-modules.md](11-modules.md)。
