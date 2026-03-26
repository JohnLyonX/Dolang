# 3. 第一个程序

这一章用三个最短例子建立对 Dolang 的第一印象：脚本、REPL、HTTP。

## 第一个脚本

创建 `hello.dol`：

```dol
$>> "Hello, Dolang!";
```

运行：

```bash
dolang run hello.dol
```

## 第一个 REPL 交互

进入 REPL：

```bash
dolang
```

输入：

```text
>> $ name = "builder";
>> $>> f"hello, {name}";
hello, builder
```

REPL 适合做这些事情：

- 试验表达式
- 试验字符串与列表方法
- 快速复现小问题

## 第一个 HTTP 路由

创建 `main.dol`：

```dol
$GET("/hello") hello() -> String {
    $# "world";
}
```

启动：

```bash
dolang serve main.dol
```

这个例子说明 Dolang 当前不仅能跑脚本，也能直接定义并运行简单 HTTP 路由。

## 单文件与项目目录

单文件适合：

- 小脚本
- 小型 demo
- 单路由接口

项目目录适合：

- 需要 `package.toml`
- 需要多个模块
- 需要统一 server 配置

项目目录的例子可以参考 [12-projects-and-package.md](12-projects-and-package.md)。

## 迁移期参考

旧版快速开始仍保留在 [getting-started.md](getting-started.md)。

## 下一章

继续看 [04-basic-syntax.md](04-basic-syntax.md)。
