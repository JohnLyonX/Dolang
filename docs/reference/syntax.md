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
$>> fs.read_text("notes.txt");
```

文件能力主线推荐走 `std.fs`：

```dao
$mod std.fs;

$ text = fs.read_text("notes.txt");
$ lines = fs.read_lines("notes.txt");
fs.write("output.txt", text);
```

`$<<FILE(...)` / `$>>FILE(...)` 已移除；继续使用会报 `DOL-P008`，文件读写请改用 `std.fs`。

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
