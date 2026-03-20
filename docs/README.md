# Dolang

> *"我们不关心工程优雅，我们关心最快把想法变成收入。"*

<p align="center">
  <img src="https://img.shields.io/badge/version-v1.7-blue" alt="Version">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
</p>

Dolang 是面向**独立开发者**的快速变现脚本语言。

## 核心定位

- **目标用户**：独立开发者（同时身兼产品经理、设计师、工程师、运营）
- **核心场景**：快速验证想法、直接暴露成可调用的服务
- **设计哲学**：今天学，今天用，明天就能收钱

## 核心特点

### $ 符号哲学

```dao
$ a = 1;        // 声明变量
$@ PI = 3.14;   // 声明常量
$>> a;          // 打印输出
$<<LINE();      // 读取输入
```

`$` 是全世界最通用的财富符号——美金、财富、钱。我们用它来声明变量、声明常量、打印输出、读取输入，把每一行代码都和「赚钱」这件事绑定在一起。

**写代码，就是赚钱。**

### 极简语法

7 个符号，5 分钟上手：

| 语法 | 功能 |
|------|------|
| `$` | 变量声明 |
| `$@` | 常量声明 |
| `$>>` | 打印输出 |
| `$<<` | 读取输入 |
| `$if/$elif/$else` | 条件语句 |
| `$while/$loop/$for` | 循环语句 |
| `$fn/$#` | 函数定义与返回 |

### 渐进式类型

```dao
$ x = 30;           // 动态模式，类型随时可变
x = "hello";        // ✅ 允许

$ y: Int = 30;     // 静态模式，类型固定
y = "hello";        // ❌ 报错
```

### 快速验证

从想法到跑起来的时间，就是你需要的全部。

---

## 安装

```bash
# 克隆项目
git clone https://github.com/your-repo/daolang.git
cd daolang

# 编译
cargo build

# 运行 REPL
cargo run
```

## CLI 命令

```bash
dolang                    # REPL 交互模式
dolang run <file>        # 运行单个 .dol 文件
dolang serve [path]      # 服务模式（默认当前目录）
dolang serve --routertab # 服务模式，显示路由表
dolang test [file]       # 测试模式
```

## HTTP 超函数

Dolang 内置完整的 HTTP 超函数支持，详见 [HTTP 参考文档](./reference/http.md)：

```dao
// 定义 GET 路由
$GET("/users/:id") get_user(id) -> JSON {
    $# $JSON { "id": id, "name": "John" };
}

// 定义 POST 路由
$POST("/users") create_user(body) -> JSON {
    $# $JSON { "created": true };
}

// 定义 HTML 页面（使用 $HTML() 构造器）
$GET("/pages/home") home_page() -> HTML {
    $# $HTML("<h1>Welcome to Dolang</h1><p>Hello World!</p>");
}

// 链接外部 HTML 文件
$GET("/") index() -> HTML {
    $# $HTML().link("pages.index");
}

// 挂载外部模块
$HTTP("/v1/api/").link("routers.api");
```

### 主要特性

- `$GET` / `$POST` / `$PUT` / `$DELETE` / `$PATCH` - HTTP 方法
- `$HTTP { }` / `$HTTP(path) { }` - HTTP 块语法（支持路径前缀）
- `$HTTP(prefix).link(module)` - 路由模块挂载
- `$HDR("Header")` - 获取请求头
- `$HTML(...)` / `$JSON(...)` - 响应构造器
- `$HTML().link()` - 链接外部 HTML/CSS/JS/XML 文件
- `$#` - 返回响应
- `-> HTML` / `-> JSON` / `-> String` - 返回类型
- `$>>` - 日志输出
- `$main()` - 服务入口点
- `$mod` - 语言模块导入

说明：

- `$mod foo.bar;` 用于导入普通语言模块，导入后通过文件名命名空间访问，例如 `bar.answer()`
- `$HTTP(...).link("routers.api")` 只负责挂载 HTTP 路由，不导入普通函数

## 快速开始

```
Dolang REPL v1.7
>> $>> "Hello, Dolang!";
Hello, Dolang!
>>
```

## 项目系统

Dolang 支持完整的项目系统：

```bash
# 创建项目目录
mkdir my-project
cd my-project

# 创建 package.toml
echo 'name = "my-project"
version = "0.1.0"
DB_URL = "postgres://localhost/db"' > package.toml

# 创建 main.dol
echo '$main() {
    $>> "Server started!";
    $ db = $<<CONFIG("DB_URL");
    $>> f"Database: {db}";
}' > main.dol

# 运行服务模式
dolang serve
```

### 退出 REPL

- 输入 `exit`
- 输入 `exit()`
- 输入 `quit`
- 快捷键 Ctrl+D（当输入为空时）

---

## 与 PHP 的区别

| | PHP | Dolang |
|---|---|---|
| 定位 | 建房子 | 摆摊 |
| 目标 | 工程构建 | 变现验证 |
| 学习成本 | 需要框架、路由、模板 | 7 个符号，5 分钟上手 |
| 时代背景 | 30 年前的 Web | 独立开发者的今天 |
| 态度 | 严肃的工程语言 | 反严肃，追求最快落地 |

**PHP 是建房子的，Dolang 是摆摊的。**

摆摊不需要地基，不需要设计图纸，需要的是今天就能开张，明天就能收钱。

---

## 未来方向

Dolang 目前是一个解释型脚本语言的起点。

我们下一步想做的，是让「变现」这个动作成为语言的一等公民——让支付、Webhook、API 暴露不再是靠第三方库拼凑出来的，而是内建在语言的表达能力里。

我们想要的终态是：

> 一个 `.dal` 文件，一条命令，直接是一个能收钱的服务。

---

*Dolang Team · v1.7 · MIT License*
