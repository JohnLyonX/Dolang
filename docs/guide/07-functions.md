# 7. 函数

函数是 Dolang 组织逻辑、消除重复、拆模块的主要方式。

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

## 返回值和返回类型

使用 `$#` 返回：

```dol
$fn add(a, b) -> Int {
    $# a + b;
}
```

没有显式 `$#` 的函数会返回 `Null`：

```dol
$fn log(msg) {
    $>> msg;
}
```

当前主线可稳定讲解的返回类型包括：

- `Int`
- `Float`
- `String`
- `Bool`
- `List`
- `Map`
- `JSON`
- `HTML`
- `Response`
- `List<T>`，其中 `T` 可以是内建类型或已定义的用户类型

需要注意的是，当前主线只把 `List<T>` 当作可教学的泛型返回写法，不把 `JSON<T>` 当作同级语法来承诺。

## 参数目前怎么写

当前主线**不承诺函数参数类型注解是可用语法**。也就是说，这一版文档统一只教下面这种签名：

```dol
$fn add(a, b) -> Int {
    $# a + b;
}
```

如果你需要约束输入，当前推荐做法是：

- 用返回类型约束输出
- 在函数体内用 `.type()`、条件判断或断言做输入检查
- 在对外接口层用更小的函数拆分而不是依赖参数注解

## 可变参数

```dol
$fn sum(...nums) -> Int {
    $ total = 0;
    $for n in nums {
        total += n;
    }
    $# total;
}
```

## 递归

```dol
$fn factorial(n) -> Int {
    $if n <= 1 {
        $# 1;
    } $else {
        $# n * factorial(n - 1);
    }
}
```

## 私有函数

```dol
_$fn normalize(name) -> String {
    $# name.trim().lower();
}

$fn public_name(name) -> String {
    $# normalize(name);
}
```

`_$fn` 只给当前模块内部使用。

## 匿名函数

```dol
$ double = $fn(x) {
    $# x * 2;
};

$>> double(5);
```

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

## 下一章

继续看 [08-collections-and-methods.md](08-collections-and-methods.md)。
