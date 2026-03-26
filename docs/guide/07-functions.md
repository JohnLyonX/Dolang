# 7. 函数

函数是 Dolang 组织逻辑、消除重复、拆分模块的主要方式。

## 定义函数

```dol
$fn greet(name) -> String {
    $# f"Hello, {name}";
}
```

调用：

```dol
$>> greet("World");
```

函数名后面是参数列表，函数体放在 `{ ... }` 里。

## 返回值

使用 `$#` 返回：

```dol
$fn add(a, b) -> Int {
    $# a + b;
}
```

调用：

```dol
$>> add(1, 2);
```

没有显式 `$#` 的函数会返回 `Null`。

```dol
$fn log(msg) {
    $>> msg;
}
```

## 可变参数

```dol
$fn sum(...nums) {
    $ total = 0;
    $for n in nums {
        total += n;
    }
    $# total;
}
```

调用时，多余参数会被收集进 `nums`：

```dol
$>> sum(1, 2, 3, 4);
```

## 递归

```dol
$fn factorial(n) {
    $if n <= 1 {
        $# 1;
    } $else {
        $# n * factorial(n - 1);
    }
}
```

递归函数最重要的是要有终止条件，否则会无限调用自身。

## 私有函数

```dol
_$fn helper(x) {
    $# x * 2;
}
```

`_$fn` 主要用于模块内部实现细节。

例如：

```dol
_$fn normalize(name) -> String {
    $# name.trim().lower();
}

$fn public_name(name) -> String {
    $# normalize(name);
}
```

## 匿名函数

当前实现支持匿名函数字面量：

```dol
$ double = $fn(x) {
    $# x * 2;
};

$>> double(5);
```

如果你需要把函数当作值传来传去，匿名函数会很有用。

## 返回类型注解

当前返回类型注解是运行时检查的一部分。常见类型包括：

- `Int`
- `Float`
- `String`
- `Bool`
- `JSON`

例如：

```dol
$fn classify(n) -> String {
    $if n > 5 {
        $# "pass";
    } $else {
        $# "fail";
    }
}
```

## 参数目前怎么写

当前主线文档里，函数参数先按“无参数类型注解”来教：

```dol
$fn add(a, b) -> Int {
    $# a + b;
}
```

也就是说，先把参数当作普通形参使用，不在这里引入未稳定的参数类型语法。

## 一个完整例子

```dol
_$fn twice(x) -> Int {
    $# x * 2;
}

$fn compute(n) -> Int {
    $ value = twice(n);
    $# value + 1;
}

$>> compute(10);
```

## 什么时候该把逻辑抽成函数

出现以下情况时，通常就该抽函数了：

- 同样逻辑重复两次以上
- 一段代码太长，不容易一眼看懂
- 你准备把逻辑导出到模块里复用

## 下一章

继续看 [08-collections-and-methods.md](08-collections-and-methods.md)。
