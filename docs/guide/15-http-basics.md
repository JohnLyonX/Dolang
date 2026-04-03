# 15. HTTP 基础

Dolang 当前内置了 HTTP 路由定义能力，可以直接和 `dolang serve` 配合使用。

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

## handler 的基本形状

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
- `List<T>`

### `List<T>` 返回例子

```dol
$Type User {
    id: Int
    name: Str
}

$GET("/users") list_users() -> List<User> {
    $# [
        User {
            id: 1,
            name: "Alice",
        },
        User {
            id: 2,
            name: "Bob",
        }
    ];
}
```

这里同样遵守类型章节的口径：返回集合时优先写 `List<T>`，不要再教学裸 `List`。

## 路径参数与 query 参数

```dol
$GET("/users/:id") get_user(id) -> JSON {
    $# $JSON { "id": id };
}
```

query string 会按同名变量注入：

```dol
$GET("/search") search() -> JSON {
    $# $JSON { "q": q, "limit": limit };
}
```

## Header 读取

```dol
$GET("/whoami") whoami() -> JSON {
    $ token = $HDR("Authorization");
    $# $JSON { "token": token };
}
```

这在 bearer token 场景下尤其常见：

```http
Authorization: Bearer <access_token>
```

## body 注入

当前主线明确这条规则：

- `POST`、`PUT`、`PATCH` 会把请求体注入到 `body`
- `GET`、`DELETE` 不把 `body` 当作主线能力来承诺

最小例子：

```dol
$POST("/echo") echo() -> JSON {
    $# $JSON { "body": body };
}
```

继续取字段：

```dol
$PATCH("/users") patch_user() -> JSON {
    $ name = body["name"];
    $# $JSON { "updated": true, "name": name };
}
```

## 认证与授权入口

`serve` 模式当前已经接通两类请求身份来源：

- cookie session
- bearer token

启用入口在 `package.toml` 的 `[server.auth]` 配置树里。最小例子：

```toml
[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie", "bearer"]

[server.auth.session]
enabled = true

[server.auth.jwt]
enabled = true
secret = "replace-me"
issuer = "dolang"
audience = "dolang"
```

路由进入 handler 前，runtime 会先解析当前 principal；如果你配置了授权规则，未认证请求会得到 `401`，权限不足会得到 `403`。

程序侧常用模块：

- `std.auth.session`
- `std.auth.jwt`
- `std.auth.guard`
- `std.auth.csrf`

完整配置和 API 细节请读：

- [../reference/http.md](../reference/http.md)
- [../reference/stdlib-api.md](../reference/stdlib-api.md)

## CSRF 基线

如果请求是通过 cookie session 认证的，并且方法属于：

- `POST`
- `PUT`
- `PATCH`
- `DELETE`

runtime 会默认校验 `X-CSRF-Token`。

使用方式：

1. 登录后读取 `session.current()["csrf_token"]`，或通过 `std.auth.csrf.token()` 获取 token
2. 后续带 cookie 的写请求附加 `X-CSRF-Token: <token>`

如果你走的是 bearer token 请求链路，则不要求这个 header。

## HTML 返回

### 直接内联

```dol
$GET("/") index() -> HTML {
    $# $HTML("<h1>Welcome</h1>");
}
```

### 链接外部文件：`$HTML().link(...)`

```dol
$GET("/") index() -> HTML {
    $# $HTML().link("pages.index");
}
```

记法：

- 使用模块式路径，如 `"pages.index"`
- 它对应外部页面资源，而不是 URL path
- 当前主线把它当作链接外部 HTML/CSS/JS/XML 文件的入口

如果你已经在组织多文件页面或路由模块，继续看 [16-http-organization.md](16-http-organization.md)。

## `std.http` 客户端

如果你不是在“接收请求”，而是要“调用别的服务”，用 `std.http`：

```dol
$mod std.http;

$ res = http.get("https://example.com");
$>> res["status"];
$>> res["body"];
```

如果 URL 不合法、连接失败或超时，错误会作为普通运行时错误抛出，可以配合 `$try / $catch`。

## 下一章

继续看 [16-http-organization.md](16-http-organization.md)。
