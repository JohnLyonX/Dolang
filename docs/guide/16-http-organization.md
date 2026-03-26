# 16. HTTP 组织方式

当路由开始增多时，就需要把 HTTP 代码组织起来。

## `$HTTP { ... }`

```dol
$HTTP {
    $GET("/ping") ping() -> String {
        $# "pong";
    }
}
```

适合把同一个文件里的多条路由聚到一起：

```dol
$HTTP {
    $GET("/ping") ping() -> String {
        $# "pong";
    }

    $GET("/health") health() -> String {
        $# "ok";
    }
}
```

## 带前缀的 HTTP 块

```dol
$HTTP("/api/v1") {
    $GET("/users") list_users() -> JSON {
        $# $JSON { "ok": true };
    }
}
```

这样可以少写重复前缀，适合按版本或按业务域组织路由。

## 在块上挂 `@SET_HDR(...)`

块级 `@SET_HDR(...)` 适合写整个路由组都要带上的静态响应头：

```dol
@SET_HDR({
    "X-Frame-Options": "DENY",
    "Cache-Control": "no-cache"
})
$HTTP("/api") {
    $GET("/ping") ping() -> String {
        $# "pong";
    }
}
```

路由级也可以继续声明：

```dol
@SET_HDR({ "Cache-Control": "no-store" })
$GET("/admin") admin() -> String {
    $# "ok";
}
```

合并规则：

- 块级 header 会传给块内路由
- 同名 header 由更靠近路由的声明覆盖
- 不同名 header 会一起保留

## 组织 `@CORS(...)`

`@CORS(...)` 可以放在三个层级：

- `$main() {}` 前，表示全局默认策略
- `$HTTP(...)` 前，表示某个路由组的策略
- 具体路由前，表示单条路由策略

例如：

```dol
@CORS("*")
$main() {}

@CORS({
    origins: ["https://api.example.com"],
    methods: ["GET", "POST"]
})
$HTTP("/api") {
    $GET("/users") users() -> JSON {
        $# $JSON { "ok": true };
    }
}
```

优先级是：

- 路由级 `@CORS` 覆盖块级
- 块级 `@CORS` 覆盖全局
- 这里是整份配置替换，不做字段级合并

## 链接外部路由模块

```dol
$HTTP("/v1/api").link("routers.api");
```

`.link()` 只用于挂载 HTTP 路由，不等于普通 `$mod` 导入。

可以把它理解成：

- `$mod` 导入普通模块函数
- `$HTTP(...).link(...)` 挂载路由定义

例如：

`routers/api.dol`：

```dol
$GET("/status") status() -> String {
    $# "running";
}
```

`main.dol`：

```dol
$HTTP("/v1/api").link("routers.api");
```

## 静态资源

```dol
$STATIC("/static", "static");
```

也支持单参数形式：

```dol
$STATIC("static");
```

它适合把某个目录作为静态文件入口暴露出去。

## HTML 外部文件链接

```dol
$GET("/") index() -> HTML {
    $# $HTML().link("pages.index");
}
```

这适合：

- 首页 HTML
- 简单静态页面
- 把 HTML/CSS/JS 作为文件组织而不是全部写进字符串

## 路由拆分建议

当项目开始变大时，可以按这个顺序组织：

1. 先把多个路由放进 `$HTTP { ... }`
2. 再把业务域拆到不同文件
3. 在入口文件里用 `.link()` 挂载
4. 静态资源单独交给 `$STATIC(...)`

## 一个完整例子

```dol
@CORS("*")
$main() {}

@SET_HDR({ "X-Frame-Options": "DENY" })
$HTTP("/api") {
    $GET("/ping") ping() -> JSON {
        $# $JSON { "ok": true };
    }
}

$HTTP("/site").link("routers.site");
$STATIC("/static", "static");
```

## 进一步参考

- `docs/reference/http.md`
- `docs/spec/modules.md`

## 下一章

继续看 [17-testing-and-debugging.md](17-testing-and-debugging.md)。
