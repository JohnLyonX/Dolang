# Syntax Cheatsheet

只放最小可识别片段，不展开成长教程。

## 声明与赋值

```dol
$ value = 1;
$ value: Int = 1;
$@ NAME = "dolang";
value = 2;
value += 1;
```

## 输入与输出

```dol
$>> "hello";
$>>ERR("bad input");
$ name = $<<LINE();
$ home = $<<ENV("HOME");
$>> fs.read_text("notes.txt");
```

```dol
$mod std.fs;
$ text = fs.read_text("notes.txt");
$ lines = fs.read_lines("notes.txt");
fs.write("out.txt", text);
fs.append("out.txt", "\nmore");
```

## 控制流

```dol
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

$for i = 0; i < 10; i = i + 1 {
}
```

## 函数

```dol
$fn greet(name) -> String {
    $# name;
}

_$fn helper(x) -> Int {
    $# x * 2;
}

$ anon = $fn(x) {
    $# x + 1;
};
```

## 错误处理

```dol
$try {
    $throw "oops";
} $catch err {
    $>>ERR(err);
}
```

## 模块

```dol
$mod helper;
$mod shared.tool;
$mod math.*;
$mod std.math;
```

## 类型系统

```dol
$ items: List = [1, 2, 3];
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

## HTTP handler

```dol
$GET("/hello") hello() -> String {
    $# "world";
}

$POST("/echo") echo() -> JSON {
    $# $JSON { "body": body };
}
```

## HTTP 组织

```dol
$HTTP("/api") {
    $GET("/ping") ping() -> JSON {
        $# $JSON { "ok": true };
    }
}

$HTTP("/api").link("routers.api");
$STATIC("/static", "static");
```

## 构造器

```dol
$# $JSON { "ok": true };
$# $HTML("<h1>Hello</h1>");
$# $HTML().link("pages.index");
$# $RES(201, {"created": true});
```

## 注解

```dol
@SET_HDR({ "Cache-Control": "no-store" })
@CORS("*")
```

## 程序入口与退出

```dol
$main() {
    $>> "starting";
}

exit;
```
