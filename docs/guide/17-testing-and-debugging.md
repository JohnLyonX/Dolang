# 17. 测试与调试

这一章介绍当前最实用的调试路径。

## 用 `dolang test` 跑路由

```bash
dolang test
dolang test main.dol
dolang test --route GET /hello
```

带请求体：

```bash
dolang test --route POST /echo --body '{"name":"Tom"}'
```

你可以先把它理解成“对当前 HTTP 路由做最小验证”的入口。

## 推荐测试顺序

建议按这个顺序排查问题：

1. 先跑 `dolang run file.dol`
2. HTTP 场景再跑 `dolang serve` 或 `dolang test`
3. 最后把问题缩成最小样例

## 用 `--route` 缩小问题面

如果一个文件里有多条路由，优先只测出问题的那一条：

```bash
dolang test --route GET /hello
```

这样比一次性跑全部路由更容易定位问题。

如果你的路由定义本身带参数，占位匹配要按“已注册路由路径”理解，而不是先把它当成完整集成测试工具。

## 用 REPL 做最小复现

REPL 适合：

- 试表达式
- 试类型行为
- 试字符串和列表方法

## 用打印定位问题

```dol
$>> "debug";
$>> value;
$>>ERR(f"unexpected: {value}");
```

打印最适合放在：

- 分支判断前后
- 文件读取之后
- 返回之前
- catch 块里

## 用 try/catch 包住不稳定点

例如文件读取：

```dol
$mod std.fs;
$try {
    $ content = fs.read_text("missing.txt");
    $>> content;
} $catch err {
    $>>ERR(err);
}
```

## 用最小文件定位问题

当问题不明显时，建议新建一个最小 `.dol` 文件，只保留：

- 触发问题的最短输入
- 一到两个函数
- 一次运行命令

例如不要一上来就在完整项目里查问题，先缩成：

```dol
$ value: Int = "text";
```

或者：

```dol
$mod missing.module;
```

## 参考测试资产

仓库里已有很多可参考样例：

- `tests/spec/valid/`
- `tests/spec/invalid/`
- `tests/fixtures/`

这些目录最适合拿来做两件事：

- 看“正确写法”长什么样
- 看“错误最小复现”应该缩到什么程度

## 下一章

继续看 [18-patterns-and-recipes.md](18-patterns-and-recipes.md)。
