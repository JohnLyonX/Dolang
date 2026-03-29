# DOC-016: 第 15 章缺少 `$HTML().link()` 和 PATCH/PUT body 注入说明

**优先级：** P3
**影响章节：** `docs/guide/15-http-basics.md`

## 问题描述

### 1. `$HTML().link()` 未在 HTTP 基础章节出现

`$HTML().link("pages.index")` 用于链接外部 HTML 文件，是实际项目中比 `$HTML("...")` 内联字符串更常用的写法。第 15 章没有提到它，第 16 章有一句提及，但没有完整说明：

- `pages.index` 的路径解析规则是什么（相对于项目根？）
- 链接的文件必须是 `.html` 吗？
- 是静态文件还是每次请求都读取？

### 2. PATCH / PUT 的 body 注入是否和 POST 相同

第 15 章说明了 POST body 自动解析为 `body` 变量（JSON），但没有明确 `$PATCH` 和 `$PUT` 是否有相同行为。用户可能对"只有 POST 有 body"产生误解。

## 期望改进

### 补充 `$HTML().link()` 示例：
```dol
$GET("/") index() -> HTML {
    $# $HTML().link("pages.index");
    // 对应文件：pages/index.html（相对项目根）
}
```
并说明文件路径解析规则和热更新/缓存行为。

### 补充 body 注入适用方法说明：
> `body` 变量自动注入适用于 POST、PUT、PATCH 三种方法，GET 和 DELETE 没有 `body` 注入。
