# 15. HTTP 基础

Dolang 当前内置了 HTTP 路由定义能力，可以直接和 `dolang serve` 配合使用。
除此之外，也已经有 `std.http` 原生模块，可以在脚本或 handler 内主动发出同步 HTTP 请求。

## 定义路由

```dol
$GET("/hello") hello() -> String {
    $# "world";
}
```

启动：

```bash
dolang serve main.dol
```

如果是项目目录，也可以：

```bash
dolang serve .
```

默认监听配置会受项目里的 `[server]` 影响。

支持的方法：

- `$GET`
- `$POST`
- `$PUT`
- `$DEL`
- `$PATCH`

## `std.http` 客户端

如果你不是在“接收请求”，而是要“调用别的服务”，用 `std.http`：

```dol
$mod std.http;

$ res = http.get("https://example.com");
$>> res["status"];
$>> res["body"];
```

返回值统一是一个 `Map`：

```text
{
  "status": Int,
  "body": String,
  "headers": Map
}
```

也可以传请求头：

```dol
$mod std.http;

$ res = http.get("https://example.com", {
    "Authorization": "Bearer demo-token"
});
$>> res["status"];
```

POST / PUT 请求用于发送 JSON 风格的 `Map`：

```dol
$mod std.http;

$ created = http.post("https://example.com/users", {
    "name": "Alice",
    "role": "admin"
});
$>> created["status"];
```

如果 URL 不合法、连接失败或超时，错误会作为普通运行时错误抛出，可以配合 `$try / $catch`：

```dol
$mod std.http;

$try {
    $ res = http.get("not-a-url");
    $>> res["status"];
} $catch err {
    $>> "request failed";
}
```

## handler 长什么样

一个 handler 一般包含四部分：

1. HTTP 方法关键字
2. 路径
3. 函数名和参数
4. 返回类型和函数体

```dol
$GET("/users/:id") get_user(id) -> JSON {
    $# $JSON { "id": id };
}
```

## 返回类型

常见返回类型：

- `String`
- `JSON`
- `HTML`

```dol
$GET("/health") health() -> String {
    $# "ok";
}
```

可以把它理解成“告诉运行时该怎么把返回值变成 HTTP 响应”。

## JSON 返回

```dol
$GET("/user") get_user() -> JSON {
    $# $JSON { "name": "Alice", "age": 30 };
}
```

`$JSON { ... }` 是最直接的 JSON 构造方式。

## `$Type` 和 `JSON<User>`

如果你前面已经看过类型章节，这里就可以把 `$Type` 和 HTTP 返回放在一起使用。

```dol
$Type User {
    id: Int
    name: Str
    email: Str?
}

$GET("/users/:id") get_user(id) -> JSON<User> {
    $# $JSON {
        "id": id,
        "name": "Alice"
    };
}
```

这种写法的价值主要在于文档性和可读性：

- 路由明确返回 JSON
- 这个 JSON 想表达的是 `User` 结构
- 阅读接口时更容易理解返回体大概长什么样

当前这层先按“结构意图声明”理解，不把它讲成完整的强校验系统。

## HTML 返回

```dol
$GET("/") index() -> HTML {
    $# $HTML("<h1>Welcome</h1>");
}
```

如果返回类型是 `HTML`，运行时会按 HTML 响应处理。

## 路径参数

```dol
$GET("/users/:id") get_user(id) -> JSON {
    $# $JSON { "id": id };
}
```

当请求路径是 `/users/42` 时，`id` 会被注入为 `"42"`。

## query 参数

当前 runtime 会把 query string 注入为同名变量：

```dol
$GET("/search") search() -> JSON {
    $# $JSON { "q": q, "limit": limit };
}
```

例如访问：

```text
/search?q=hello&limit=10
```

这时 `q` 和 `limit` 可直接读取。

## Header 读取

```dol
$GET("/whoami") whoami() -> JSON {
    $ token = $HDR("Authorization");
    $# $JSON { "token": token };
}
```

`$HDR(...)` 适合取：

- `Authorization`
- `Content-Type`
- 自定义请求头

## `@SET_HDR(...)` 响应头注解

如果你要给响应统一附加静态 header，用 `@SET_HDR(...)`，而不是在 handler 里手工拼接。

```dol
@SET_HDR({
    "X-Frame-Options": "DENY",
    "Cache-Control": "max-age=3600"
})
$GET("/health") health() -> String {
    $# "ok";
}
```

要点：

- `@SET_HDR(...)` 只接受一个对象字面量
- key 必须是字符串 header 名
- value 也必须是字符串字面量
- 一个注解里可以声明多个 header

不符合这个形态会直接报解析错误，例如：

```dol
@SET_HDR("X-Frame-Options", "DENY")     // 非法
@SET_HDR({ name: "X-Frame-Options" })   // 非法
```

如果没有写 `@SET_HDR(...)`，响应不会额外注入这些自定义 header。

## `@CORS(...)`

如果服务需要浏览器跨域访问，可以直接在路由前或 HTTP 组织节点前写 `@CORS(...)`。

最简单的写法是允许所有来源：

```dol
@CORS("*")
$GET("/public") public_data() -> JSON {
    $# $JSON { "ok": true };
}
```

需要更细的配置时，用单个对象字面量：

```dol
@CORS({
    origins: ["https://app.example.com"],
    methods: ["GET", "POST"],
    headers: ["Authorization", "Content-Type"],
    max_age: 600,
    credentials: true
})
$GET("/profile") profile() -> JSON {
    $# $JSON { "ok": true };
}
```

要点：

- `@CORS("*")` 适合公开接口
- `@CORS({ ... })` 适合白名单接口
- 对象字段当前支持 `origins`、`methods`、`headers`、`max_age`、`credentials`
- 这些参数都必须是静态字面量，不能传变量或运行时表达式

如果没有写 `@CORS(...)`，服务不会额外返回 `Access-Control-Allow-*` 响应头。

## body

当前 runtime 会把请求体放进 `body` 变量。

在真实服务模式下，JSON body 会被解析后注入；在 `dolang test --body ...` 里，body 的输入路径和服务模式略有差异，因此写文档时最好先用“读取 body 并回显”的最小例子说明：

```dol
$POST("/echo") echo() -> JSON {
    $# $JSON { "body": body };
}
```

如果你想从 body 中取字段，可以继续往下读：

```dol
$POST("/users") create_user() -> JSON {
    $ name = body["name"];
    $ age = body["age"];
    $# $JSON {
        "created": true,
        "name": name,
        "age": age
    };
}
```

在 handler 内部也可以直接使用 `List` / `Map` 显式声明：

```dol
$GET("/check") check() -> JSON {
    $ items: List = [1, 2, 3];
    $ config: Map = {"name": "demo", "ok": true};
    $# $JSON {
        "items_type": items.type(),
        "config_type": config.type(),
        "second": items[1],
        "ok": config["ok"]
    };
}
```

这类写法在 `serve` 模式下可以正常注册并返回结果。

## 读取项目配置

HTTP 服务里也经常会用到项目配置：

```dol
$GET("/config") show_config() -> JSON {
    $ app = $<<CONFIG("APP_NAME");
    $# $JSON { "app": app };
}
```

这要求你在项目配置里已经写好了对应的 `[env]` 字段。

## 返回指定状态码

```dol
$GET("/users/:id") get_user(id) -> JSON {
    $# $RES(200, {"id": id});
}
```

`$RES(status, body)` 用来显式返回指定 HTTP 状态码。

另一个常见例子：

```dol
$POST("/users") create_user() -> JSON {
    $# $RES(201, {"created": true});
}
```

## 路由里的错误处理

HTTP handler 里也可以配合 `$try / $catch` 和 `$throw`：

```dol
$GET("/safe") safe() -> JSON {
    $try {
        $throw "inner error";
    } $catch err {
        $# $JSON { "caught": err };
    }
}
```

`$>>ERR(...)` 也可以用于打印服务端日志，而不影响正常响应：

```dol
$GET("/debug") debug() -> JSON {
    $>>ERR("debug info");
    $# $JSON { "ok": true };
}
```

## 一个完整例子

```dol
$GET("/users/:id") get_user(id) -> JSON {
    $ token = $HDR("Authorization");
    $# $RES(200, {
        "id": id,
        "token": token
    });
}
```

## 迁移期参考

- [web.md](web.md)
- `docs/reference/http.md`

## 下一章

继续看 [16-http-organization.md](16-http-organization.md)。
