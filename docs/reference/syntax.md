# 语法参考

本文档按“能快速查关键写法”的方式整理 Dolang 常用语法。

## 语句结束符

所有语句必须以 `;` 结尾：

```dao
$>> "Hello";
```

## 声明与函数

```dao
$ value = 1;
$ value: Int = 1;
$@ NAME = "dolang";

$fn greet(name) -> String {
    $# name;
}
```

参数类型注解当前不是主线承诺语法；reference 也不把它写成稳定用法。

## 输入与输出

```dao
$>> "Hello";
$>>ERR("bad input");
$ name = $<<LINE();
$ home = $<<ENV("HOME");
$ text = $<<FILE("notes.txt");
$ lines = $<<FILE("notes.txt", "LINES");
```

关于 `$<<FILE(...)`，统一口径如下：

- `$<<FILE(path)`：按全文读取
- `$<<FILE(path, "LINES")`：按行读取
- 如果你需要更明确、更完整的文件 API，优先改用 `std.fs.read_text(...)` / `std.fs.read_lines(...)`

## 控制流

```dao
$if cond {
} $elif other {
} $else {
}

$while cond {
}

$loop {
    $break;
}

$for item in items {
    $continue;
}
```

## 模块

```dao
$mod helper;
$mod math.*;
$mod std.math;
```

`$mod a.*;` 导入直接子模块命名空间，不会把函数平铺到当前作用域。

## 类型与返回

```dao
$Type User {
    id: Int
    name: Str
}

$fn list_users() -> List<User> {
    $# [
        User {
            id: 1,
            name: "Alice",
        }
    ];
}
```

返回集合时，主线推荐写 `List<T>`，而不是裸 `List`。

## HTTP

```dao
$GET("/hello") hello() -> String {
    $# "world";
}

$POST("/echo") echo() -> JSON {
    $# $JSON { "body": body };
}

$HTTP("/api").link("routers.api");
```

## 构造器与入口

```dao
$# $JSON { "ok": true };
$# $HTML("<h1>Hello</h1>");
$# $HTML().link("pages.index");
$# $RES(201, {"created": true});

$main() {
    $>> "starting";
}
```
