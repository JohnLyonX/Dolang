# HTTP 超函数参考

Dolang 提供了完整的 HTTP 超函数支持，用于快速构建 Web API 服务。

## 目录

- [HTTP 方法](#http-方法)
- [返回类型](#返回类型)
- [HTTP 块](#http-块)
- [请求上下文](#请求上下文)
- [响应返回](#响应返回)
- [模块挂载](#模块挂载)
- [测试模式](#测试模式)

---

## HTTP 方法

### 基本语法

```dao
$METHOD("/path") handler_name(param1, param2) -> ReturnType {
    // handler body
    $# return_value;
}
```

### 支持的方法

| 方法 | 说明 | 适用场景 |
|------|------|----------|
| `$GET` | GET 请求 | 查询数据 |
| `$POST` | POST 请求 | 创建数据 |
| `$PUT` | PUT 请求 | 更新数据 |
| `$DELETE` 或 `$DEL` | DELETE 请求 | 删除数据 |
| `$PATCH` | PATCH 请求 | 部分更新 |

### 示例

```dao
// GET 请求 - 查询用户
$GET("/users/:id") get_user(id) -> JSON {
    $ result = $JSON {
        "id": id,
        "name": "John"
    };
    $# result;
}

// POST 请求 - 创建用户
$POST("/users") create_user(body) -> JSON {
    $ result = $JSON {
        "status": "created",
        "data": body
    };
    $# result;
}

// 带路径参数
$GET("/users/:id/posts/:post_id") get_user_post(id, post_id) -> JSON {
    $# $JSON { "user_id": id, "post_id": post_id };
}

// Query 参数自动提取
$GET("/search") search_handler(q, limit) -> JSON {
    $# $JSON { "query": q, "limit": limit };
}
```

---

## 返回类型

HTTP 路由支持以下返回类型：

| 返回类型 | 说明 | Content-Type |
|----------|------|--------------|
| `JSON` | JSON 响应（默认） | `application/json` |
| `HTML` | HTML 响应 | `text/html; charset=utf-8` |
| `String` | 纯文本响应 | `text/plain` |

### JSON 类型（默认）

```dao
$GET("/api/users") get_users() -> JSON {
    $# $JSON { "users": ["tom", "jerry"] };
}
```

### HTML 类型

可以使用 `$HTML()` 构造器或 `-> HTML` 返回类型：

```dao
// 使用 $HTML() 构造器（推荐）
$GET("/pages/home") home_page() -> HTML {
    $# $HTML("<h1>Welcome</h1><p>This is an HTML page</p>");
}

// 使用 -> HTML 返回类型
$GET("/pages/index") index_page() -> HTML {
    $# "<html><head><title>My App</title></head><body><h1>Hello</h1></body></html>";
}
```

#### $HTML().link() - 链接外部文件

可以使用 `$HTML().link()` 方法链接外部 HTML/CSS/JS/XML 文件：

```dao
// 链接外部 HTML 文件（使用模块路径格式）
$GET("/") index() -> HTML {
    $# $HTML().link("pages.index");
}

// 链接外部文件，内联内容优先
$GET("/page") page() -> HTML {
    $# $HTML("<h1>Override</h1>").link("pages.index");
}

// 支持的文件类型：html, htm, css, js, xml
$GET("/style") style() -> HTML {
    $# $HTML().link("assets.style");
}
```

**通配符支持**：

```dao
// 匹配目录下所有文件（包含子目录）
$GET("/all") all() -> HTML {
    $# $HTML().link("pages.*");
}

// 链式调用多个 link
$GET("/multi") multi() -> HTML {
    $# $HTML().link("pages.*").link("css.*");
}
```

**注意**：
- 模块路径格式：`"pages.index"` 对应 `pages/index.html`
- 不支持路径格式：`"pages/index"` 会报错
- 通配符 `"pages.*"` 会递归匹配 pages 目录下所有 html/htm/css/js/xml 文件
- 如果有内联内容，优先使用内联内容，忽略 link
- 多个 link 会按顺序合并内容

### String 类型

```dao
$GET("/text") text_handler() -> String {
    $# "Hello World";
}
```

---

## HTTP 块

### 基本块语法

可以使用 `$HTTP { }` 块在文件中定义多个路由：

```dao
$HTTP {
    $GET("/users") list_users() -> JSON { ... }
    $GET("/users/:id") get_user(id) -> JSON { ... }
    $POST("/users") create_user(body) -> JSON { ... }
}
```

### 带路径前缀的块

可以使用 `$HTTP(path) { }` 为块内的路由添加路径前缀：

```dao
$HTTP("/v1/api/") {
    $GET("/users") list_users() -> JSON { ... }
    $GET("/users/:id") get_user(id) -> JSON { ... }
}
// 实际路由: /v1/api/users, /v1/api/users/:id
```

路径前缀会自动处理斜杠去重：

```dao
$HTTP("/v1/api/") { ... }   // 前缀末尾有斜杠
$GET("/users") { ... }       // 路由开头有斜杠
// 结果: /v1/api/users (正确，去重)
```

---

## 请求上下文

### $HDR - 获取请求头

```dao
$GET("/api/data") api_handler() -> JSON {
    $ token = $HDR("Authorization");
    $# $JSON { "token": token };
}
```

### $<< - 获取请求体

对于 POST/PUT 请求，可以通过 `body` 参数获取 JSON body：

```dao
$POST("/users") create_user(body) -> JSON {
    $ name = body.name;
    $# $JSON { "created": true, "name": name };
}
```

---

## 响应返回

### $# - 返回响应

```dao
// 返回 JSON
$GET("/api/users") get_users() -> JSON {
    $# $JSON { "users": ["tom", "jerry"] };
}

// 返回字符串
$GET("/hello") hello() -> String {
    $# "Hello World";
}

// 返回空响应
$DELETE("/users/:id") delete_user(id) -> JSON {
    $# $JSON { "deleted": true };
}
```

### $>> - 输出日志

```dao
$GET("/api/test") test_handler() -> JSON {
    $>> "Processing request...";
    $# $JSON { "ok": true };
}
```

---

## 模块挂载

### $HTTP(path).link(module) - 挂载外部模块

可以将其他 `.dol` 文件中的路由挂载到指定路径前缀下：

**main.dol:**
```dao
$mod routers.api;

$main() {
    $HTTP("/v1/api/").link("routers.api");
}
```

**routers/api.dol:**
```dao
$GET("/users") list_users() -> JSON {
    $# $JSON { "users": ["tom", "jerry"] };
}

$GET("/users/:id") get_user(id) -> JSON {
    $# $JSON { "id": id, "name": "John" };
}
```

挂载后路由：
- `GET /v1/api/users` → list_users
- `GET /v1/api/users/:id` → get_user

### 路径斜杠处理

系统会自动处理路径斜杠，去除重复的斜杠：

```dao
$HTTP("/v1/api/").link("routers.api")  // 前缀末尾有斜杠
$GET("/users")                          // 路由开头有斜杠
// 结果: /v1/api/users (正确)
```

---

## 模块声明

### $mod - 导入模块

```dao
$mod dao.user;
$mod services.auth;
```

这会加载对应的 `.dol` 文件：
- `dao.user` → `dao/user.dol`
- `services.auth` → `services/auth.dol`

---

## $main() 入口

`$main()` 函数是服务器入口点，只能在 `main.dol` 中定义：

```dao
$main() {
    // 初始化逻辑
    $>> "Server starting...";

    // 可以直接定义路由
    $GET("/hello") hello() -> String {
        $# "Hello";
    }

    // 也可以挂载模块
    $HTTP("/api/").link("routers.api");
}
```

---

## 测试模式

### dolang test

```bash
# 测试所有路由
dolang test main.dol

# 测试特定路由
dolang test main.dol --route GET /hello

# 带 body 测试
dolang test main.dol --route POST /api/users --body '{"name":"tom"}'
```

---

## 静态文件服务

### $STATIC - 静态文件服务

可以使用 `$STATIC()` 函数来服务静态文件（CSS、JS、图片等）：

```dao
$main() {
    // 方式1: 指定 URL 前缀和目录
    $STATIC("/css", "css");      // /css/style.css -> css/style.css
    $STATIC("/js", "js");        // /js/main.js -> js/main.js
    $STATIC("/images", "img");   // /images/logo.png -> img/logo.png

    // 方式2: 只指定目录（默认 URL 前缀为 /static）
    $STATIC("public");           // /static/index.html -> public/index.html

    // 方式3: 带点号格式的目录
    $STATIC("/dolangcss", "css.dolang"); // /dolangcss/style.css -> css/dolang.css
}
```

**语法：**
- `$STATIC("dir")` - 单参数，URL 前缀默认为 `/static`
- `$STATIC("/url-prefix", "dir")` - 双参数，指定 URL 前缀和目录
- `dir` 支持点号格式：`css.dolang` 会转换为 `css/` 目录

**示例：**

```dao
// 项目结构
// my-app/
// ├── main.dol
// ├── css/
// │   └── style.css
// └── js/
//     └── main.js

$main() {
    $STATIC("/css", "css");
    $STATIC("/js", "js");

    $GET("/") index() -> HTML {
        $# $HTML().link("pages.index");
    }
}
```

访问：
- `GET /css/style.css` → 返回 `css/style.css`
- `GET /js/main.js` → 返回 `js/main.js`

---

## CLI 命令

| 命令 | 说明 |
|------|------|
| `dolang` | 启动 REPL 模式 |
| `dolang run <file>` | 运行 .dol 文件 |
| `dolang serve [path]` | 启动 HTTP 服务器 |
| `dolang serve --routertab` | 启动并显示路由表 |
| `dolang test [file]` | 运行测试 |

---

## 完整示例

**项目结构：**
```
my-app/
├── main.dol
└── routers/
    ├── api.dol
    └── users.dol
```

**main.dol:**
```dao
$mod routers.api;
$mod routers.users;

$main() {
    // 挂载 API 路由到 /v1/api
    $HTTP("/v1/api/").link("routers.api");

    // 挂载用户路由到 /v1/users
    $HTTP("/v1/users/").link("routers.users");

    // 本地定义一个路由
    $GET("/health") health() -> JSON {
        $# $JSON { "status": "ok" };
    }
}
```

**routers/api.dol:**
```dao
$GET("/info") get_info() -> JSON {
    $# $JSON { "version": "1.0.0" };
}
```

**routers/users.dol:**
```dao
$GET("/list") list_users() -> JSON {
    $# $JSON { "users": ["tom", "jerry"] };
}

$POST("/create") create_user(body) -> JSON {
    $# $JSON { "created": true, "data": body };
}
```

**启动服务：**
```bash
dolang serve .
# 或显示路由表
dolang serve . --routertab
```

**注册路由：**
```
GET /health         → health
GET /v1/api/info    → get_info
GET /v1/users/list   → list_users
POST /v1/users/create → create_user
```
