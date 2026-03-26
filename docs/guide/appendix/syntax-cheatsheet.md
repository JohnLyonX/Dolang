# Syntax Cheatsheet

## 声明

```dol
$ value = 1;
$@ NAME = "dolang";
$fn greet(name) -> String {
    $# name;
}
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
}
```

## 模块

```dol
$mod helper;
$mod std.math;
```

## HTTP

```dol
$GET("/hello") hello() -> String {
    $# "world";
}
```
