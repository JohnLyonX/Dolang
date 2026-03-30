# 函数系统

> 迁移提示
>
> 本页是旧 reference 页面，仍有部分历史示例。
> 当前主线函数口径请优先参考 [../guide/07-functions.md](../guide/07-functions.md)、[../guide/09-gradual-typing.md](../guide/09-gradual-typing.md) 和 [syntax.md](syntax.md)。
> 如果本页示例与这些页面冲突，以主线和当前实现为准。

Dolang 支持完整的函数功能，包括函数定义、参数传递、返回值、递归等。

## 函数定义

使用 `$fn` 声明函数，`$#` 返回值：

```dao
$fn add(a, b) {
    $ result = a + b;
    $# result;
}
```

**语法**：

```dao
$fn 函数名(参数1, 参数2, ...) {
    函数体;
    $# 返回值;
}
```

**使用技巧**：

- 函数名在全局唯一，不能重复定义
- 参数是函数的局部变量，外部无法直接访问
- `$#` 语句会立即返回，后面的代码不会执行

---

## 函数参数传递

参数按值传递，函数内部修改参数不会影响外部变量：

```dao
$fn increment(x) {
    x = x + 1;
    $>> x;
}

$ a = 10;
increment(a);    // 输出: 11
$>> a;           // 输出: 10 (a 未改变)
```

---

## 可变参数函数

Dolang 支持可变参数函数，使用 `...参数名` 语法收集多余参数到列表。

### 基本用法

```dao
$fn sum(...nums) {
    $# nums;
};

$>> sum(1, 2, 3);
[1, 2, 3]
```

**语法**：

```dao
$fn 函数名(...参数名) {
    函数体;
}
```

### 混合普通参数和可变参数

可变参数必须是最后一个参数：

```dao
$fn greet(name, ...others) {
    $# [name, others];
};

$>> greet("Tom");
[Tom, []]
$>> greet("Tom", "Jerry", "Bob");
[Tom, [Jerry, Bob]]
```

### 获取参数个数

使用 `.len()` 方法获取传入的参数个数：

```dao
$fn countArgs(...args) {
    $# args.len();
};

$>> countArgs();
0
$>> countArgs(1);
1
$>> countArgs(1, 2, 3);
3
```

### 匿名可变参数函数

```dao
$result = $fn(...args) {
    $# args;
};

$>> result(1, 2);
[1, 2]
$>> result();
[]
```

---

## 返回语句

使用 `$#` 返回值：

```dao
$fn double(x) {
    $# x * 2;
}

$>> double(5);
10
```

---

## 返回类型声明

可以使用 `-> Type` 指定函数返回类型：

```dao
$fn double(x) -> Int {
    $# x * 2;
}

$>> double(5);
10
```

支持的类型：
- `Int` / `Integer` - 整数类型
- `Float` - 浮点数类型
- `String` - 字符串类型
- `Bool` / `Boolean` - 布尔类型

### HTTP 返回类型

在 HTTP 路由中，还可以使用以下返回类型：

- `JSON` - JSON 响应（默认）
- `HTML` - HTML 页面响应
- `String` - 纯文本响应

```dao
// JSON 响应
$GET("/api/users") get_users() -> JSON {
    $# $JSON { "users": ["tom", "jerry"] };
}

// HTML 响应
$GET("/pages/home") home() -> HTML {
    $# "<h1>Welcome</h1><p>Hello World!</p>";
}
```

### 历史写法提示：`JSON<TypeName>`

以下是旧 reference 中曾出现过的签名形式：`$fn getUser(id) -> JSON<User> { ... }`

当前主线不再把 `JSON<TypeName>` 当作稳定教学写法。阅读函数签名时，优先使用：

- 普通 `-> JSON`
- 或集合返回写成 `-> List<T>`

涉及 `$Type` 的结构表达，请以 [../guide/09-gradual-typing.md](../guide/09-gradual-typing.md) 为准。

---

**返回类型不匹配时报错**：

```dao
$fn test() -> String {
    $# 123;
}
```

运行结果：
```
[ERROR] runtime error: function 'test' expects return type 'String' but got 'Int'
```

**函数有返回值但未声明返回类型时报错**：

```dao
$fn add(x, y) {
    $# x + y;
}
```

运行结果：
```
[ERROR] runtime error: function 'add' has no return type declared but returns a value
```

---

## 无返回值函数

```dao
$fn greet(name) {
    $>> "Hello, " + name;
}

greet("Dolang");
```

运行结果：
```
Hello, Dolang
```

---

## 函数调用

```dao
$fn add(a, b) {
    $# a + b;
}

$ sum = add(1, 2);
$>> sum;
3
```

---

## 匿名函数

Dolang 支持将函数赋值给变量，实现函数作为一等公民（first-class functions）：

```dao
$a = 20;
$b = 30;

$result = $fn(x,y) {
 $>> x + y;
};

result(a,b);
50
```

**语法**：

```dao
$变量名 = $fn(参数1, 参数2, ...) {
    函数体;
};
```

- 匿名函数不需要函数名
- 函数体末尾需要分号结束
- 可以像调用普通函数一样调用通过变量持有的函数

**带返回值的匿名函数**：

```dao
$result = $fn(x, y) -> Int {
    $# x + y;
};

$>> result(10, 20);
30
```

**无返回值的匿名函数**：

```dao
$result = $fn(x, y) {
    $>> x + y;
};

result(10, 20);
30
```

---

## 递归支持

Dolang 支持递归函数：

```dao
$fn factorial(n) {
    $if n <= 1 {
        $# 1;
    } $else {
        $# n * factorial(n - 1);
    }
}

$>> factorial(5);
120
```

---

## 作用域规则

函数内部有自己的作用域，外部变量无法直接访问：

```dao
$ x = 10;

$fn test() {
    $>> x;  // ERROR: variable 'x' not found
}

test();
```

要访问外部变量，需要通过参数传递：

```dao
$fn print_val(x) {
    $>> x;
}

$ x = 10;
print_val(x);  // 输出: 10
```

---

## 函数调用错误

### 调用未定义的函数

```dao
foo();
```

运行结果：
```
[ERROR] runtime error: function 'foo' is not defined
```

### 函数重定义错误

```dao
$fn add(a, b) {
    $# a + b;
}
$fn add(a, b) {
    $# a;
}
```

运行结果：
```
[ERROR] runtime error: function 'add' is already defined
```

---

## 方法调用

Dolang 支持对数据进行方法调用，方法调用**必须使用括号**。

### 语法

```dao
对象.方法名(参数);
```

### 示例

```dao
$ s = "hello";
$>> s.upper();        // 输出: HELLO
$>> s.lower();        // 输出: hello
$>> s.len();          // 输出: 5

$ arr = [1, 2, 3];
$>> arr.len();        // 输出: 3
$>> arr.push(4);     // 输出: [1, 2, 3, 4]

$ n = 123;
$>> n.to_str();       // 输出: "123"
```

### 错误：方法调用必须使用括号

```dao
$ s = "hello";
$>> s.upper;          // ❌ 错误
```

运行结果：
```
[ERROR] Parse error at line 1:1
  found: Ident upper
  expected: (
  method 'upper' requires parentheses, use 'upper(...)' instead
```

---

## 标准输入

Dolang 支持从标准输入读取数据，使用 `$<<` 关键字。

### 读取环境变量 `$<<ENV`

```dao
$ home = $<<ENV("HOME");
$>> home;             // 输出: /Users/xxx
```

**错误情况**：

```dao
$ x = $<<ENV("NOT_EXIST");
```

运行结果：
```
[ERROR] runtime error: environment variable 'NOT_EXIST' is not defined
```

### 读取 stdin `$<<LINE`

```dao
// 无提示输入
$ name = $<<LINE();
$>> name;

// 有提示输入
$ age = $<<LINE("请输入年龄: ");
$>> age;
```

### 不赋值（pause 效果）

单独使用 `$<<LINE()` 会等待用户输入，按回车后继续，不保存输入：

```dao
$>> "按回车继续...";
$<<LINE();
$>> "程序继续执行";
```

---

## 文件读写

Dolang 支持文件读写操作，使用 `$>>FILE` 创建文件对象，`$<<FILE` 读取文件。

### 写入文件 `$>>FILE`

`$>>FILE` 支持两种语法：

#### 1. 直接写入语法（推荐）

```dolang
// 覆盖写入
$>>FILE("output.txt", "Hello World");

// 追加写入
$>>FILE("output.txt", "第二行", "A");

// 使用变量
$ content = "Hello";
$>>FILE("output.txt", content, "W");
```

**语法**：
```dolang
$>>FILE(path, content);           // 覆盖写入
$>>FILE(path, content, "W");     // 覆盖写入
$>>FILE(path, content, "A");     // 追加写入
$>>FILE(path, content, "a");     // 追加写入
$>>FILE(path, "", "DEL");        // 删除文件（content 为空时）
```

#### 2. 链式调用语法（对象方式）

```dolang
// 创建文件对象
$ f = $>>FILE("output.txt");
$ f.content("Hello World");

// 追加写入
$ f = $>>FILE("output.txt", "A");
$ f.content("\n第二行");

// 删除文件
$>>FILE("temp.txt", "DEL");
```

**语法**：
```dolang
$ f = $>>FILE(path);              // 创建文件对象（默认覆盖写入）
$ f = $>>FILE(path, "W");        // 覆盖写入
$ f = $>>FILE(path, "A");        // 追加写入
$>>FILE(path, "DEL");            // 删除文件

$ f.content("内容");              // 写入内容
```

### 读取文件 `$<<FILE`

`$<<FILE("path")` 返回一个 File 对象，支持链式调用：

```dao
// 获取文件对象
$ f = $<<FILE("data.txt");

// 检查文件是否存在
$>> f.exists();

// 读取全部内容
$ content = f.read();

// 获取文件大小
$>> f.size();

// 判断是否为目录
$>> f.is_dir();
```

**语法**：

```dao
$ f = $<<FILE(path);              // 获取文件对象
$ f = $<<FILE(path, "LINES");    // 设置为行读取模式
$ f.exists()                      // Bool - 文件是否存在
$ f.read()                       // String - 读取全部内容
$ f.read_lines()                 // List - 读取所有行（等效于设置 "LINES" 模式后调用 read()）
$ f.size()                      // Int - 文件大小
$ f.is_dir()                    // Bool - 是否为目录
```

### 示例

```dao
// 写入文件
$ f = $>>FILE("data.txt");
$ f.content("line1\nline2\nline3");

// 检查文件是否存在
$ f = $<<FILE("data.txt");
$if f.exists() {
    // 读取内容
    $ content = f.read();
    $>> content;
}

// 设置行读取模式
$ f2 = $<<FILE("data.txt", "LINES");
$ lines = f2.read();
$for line in lines {
    $>> line;
}
```

### 错误情况

```dao
// 读取不存在的文件
$ f = $<<FILE("not_exist.txt");
$>> f.read();
[ERROR] runtime error: cannot read file: Not found (os error 2)

// 读取目录
$ f = $<<FILE("./src");
$>> f.read();
[ERROR] runtime error: 'src' is a directory, not a file
```

---

## JSON 和 HTML 方法

JSON 和 HTML 类型支持以下内置方法：

### JSON 方法

```dolang
$ j = $JSON {"name": "John", "age": 30};

// 转换为字符串
$ str = j.to_str();

// 获取类型名
$ t = j.type();
```

**可用方法**：
- `.to_str()` - 将 JSON 对象转换为字符串表示
- `.type()` - 返回类型名 "Json"

### HTML 方法

```dolang
$ h = $HTML("Hello World");

// 转换为字符串
$ str = h.to_str();

// 获取类型名
$ t = h.type();

// 链接外部文件
$ h = $HTML().link("pages.index");
```

**可用方法**：
- `.to_str()` - 将 HTML 内容转换为字符串
- `.type()` - 返回类型名 "Html"
- `.link("module.path")` - 链接外部 HTML/CSS/JS/XML 文件

---

## 模块系统

Dolang 支持模块系统，允许将代码拆分到多个文件中，通过 `$mod` 关键字导入。

### 模块声明 `$mod`

使用 `$mod 路径;` 导入模块：

```dao
$mod dao.user;
```

**语法**：

```dao
$mod 模块路径;
$mod 目录路径.*;
```

模块路径支持点分隔符：
- `$mod dao.user;` 会加载 `dao/user.dol` 文件
- `$mod services.*;` 会导入 `services/` 目录下的直接子模块

### 访问方式

模块导入后通过文件名命名空间访问，不会平铺到全局函数表：

```dao
$mod dao.user;

$main() {
    $ result = user.get_user("123");
    $>> result;
};
```

### 公开与私有

- `$fn` 默认公开
- `_$fn` 默认私有
- 私有函数不会被 `$mod` 导入
- `$mod` 当前只导入函数，不导入变量、常量、HTTP 路由或 `$main`

### 模块搜索路径

模块按以下顺序搜索：
1. 当前目录：`dao/user.dol`
2. modules 目录：`modules/dao/user.dol`

### 示例

**项目结构**：

```
my-project/
├── main.dol
├── package.toml
└── dao/
    └── user.dol
```

**dao/user.dol**（模块文件）：

```dao
$fn get_user(id) -> String {
    $# "User-" + id;
};

_$fn format_email(name, email) -> String {
    $# "Created: " + name + " (" + email + ")";
};

$fn create_user(name, email) -> String {
    $# format_email(name, email);
};
```

**main.dol**（主文件）：

```dao
$mod dao.user;

$main() {
    $ result = user.get_user("123");
    $>> result;           // 输出: User-123

    $ created = user.create_user("Tom", "tom@example.com");
    $>> created;         // 输出: Created: Tom (tom@example.com)
};
```

### 错误情况

```dao
$mod non.existent.module;
```

运行结果（如果模块文件不存在）：
```
[ERROR] runtime error: module not found: 'non.existent.module' (tried: non/existent/module.dol, modules/non/existent/module.dol)
```

如果尝试访问私有函数：

```dao
$mod dao.user;
$ value = user.format_email("Tom", "tom@example.com");
```

会报模块函数不存在错误。

---

## 项目系统

Dolang 支持项目系统，提供服务模式、配置管理和主入口功能。

### CLI 命令

```bash
dolang               # REPL 交互模式
dolang run <file>   # 运行单个 .dol 文件
dolang serve [path] # 服务模式（默认当前目录）
```

### 服务模式 `dolang serve`

服务模式用于运行项目，会自动加载配置和执行主入口：

```bash
dolang serve .           # 从当前目录加载
dolang serve main.dol    # 指定主入口文件
```

### 主入口 `$main`

使用 `$main()` 定义程序入口，仅在服务模式下执行：

```dao
$main() {
    $>> "Server started!";
    $>> "Loading configuration...";
};
```

**注意**：
- `$main()` 只能在 `main.dol` 文件中使用
- 只有在 `dolang serve` 模式下才会执行

### 配置文件 package.toml

在项目根目录创建 `package.toml` 文件：

```toml
name = "my-project"
version = "0.1.0"
DB_URL = "postgres://localhost/db"
API_KEY = "your-api-key"
port = 3000
host = "0.0.0.0"
```

**配置项**：
| 键 | 说明 | 默认值 |
|---|---|---|
| name | 项目名称 | - |
| version | 项目版本 | - |
| entry | 入口文件 | main.dol |
| port | 服务器端口 | 8080 |
| host | 服务器地址 | 0.0.0.0 |
| 其他 | 环境变量 | - |

### 配置读取 `$<<CONFIG`

在服务模式下，可以使用 `$<<CONFIG("KEY")` 读取配置：

```dao
$mod dao.user;

$main() {
    $ db_url = $<<CONFIG("DB_URL");
    $>> f"Connecting to: {db_url}";

    $ api_key = $<<CONFIG("API_KEY");
    $>> f"API Key loaded: {api_key}";
};
```

**注意**：`$<<CONFIG` 仅在服务模式下可用

运行结果：
```
$ dolang serve .
Loaded project: my-project v0.1.0
Connecting to: postgres://localhost/db
API Key loaded: your-api-key
```

### 错误情况

```dao
// 在 REPL 模式下使用 $<<CONFIG
$ a = $<<CONFIG("DB_URL");
```

运行结果：
```
[ERROR] runtime error: $<<CONFIG() is only available in serve mode
```
