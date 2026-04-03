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
