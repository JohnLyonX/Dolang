# 9. 渐进类型

Dolang 当前采用“默认动态，显式约束”的使用方式。

## 默认动态

不写类型注解时，变量可以改变值类型：

```dol
$ x = 30;
x = "hello";
x = true;
```

## 显式类型注解

```dol
$ count: Int = 0;
$ name: String = "Alice";
$ active: Bool = true;
$ items: List = [1, 2, 3];
$ config: Map = {"name": "demo"};
```

常量也支持类型注解：

```dol
$@ MAX: Int = 100;
```

当前主线可依赖的声明类型名：

- `Int`
- `Float`
- `String`
- `Str`
- `Bool`
- `Boolean`
- `List`
- `Map`

当前不要把 `Json`、`Response` 写成“变量/常量声明可稳定使用”的类型名。

## 函数参数类型注解的态度

当前主线文档统一采用保守口径：**不教学、不承诺参数类型注解语法**。

也就是说，这一版只写：

```dol
$fn add(a, b) -> Int {
    $# a + b;
}
```

如果你需要更强约束，优先使用：

- 返回类型
- 局部变量显式声明
- `.type()` / `std.core.check`
- [10-error-handling.md](10-error-handling.md) 里的错误处理方式

## 返回类型和 `List<T>`

返回类型是这一章最重要的稳定能力之一。

### 正确写法

```dol
$Type User {
    id: Int
    name: Str
}

$fn list_users() -> List<User> {
    $# [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ];
}
```

这里要记住两条：

- `List<T>` 是当前主线可用写法
- 裸 `List` 不应再作为返回类型写法来教学

`T` 可以是内建类型，也可以是先定义好的用户类型。

## `$Type`

`$Type` 用来描述结构，不等于 class：

```dol
$Type User {
    id: Int
    name: Str
    email: Str?
}
```

它最适合和 `List<T>`、接口返回说明、阅读文档时的结构表达配合使用。

## HTTP 返回里的 `List<T>`

在 HTTP 章节里同样推荐这样写：

```dol
$Type User {
    id: Int
    name: Str
}

$GET("/users") list_users() -> List<User> {
    $# [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ];
}
```

如果类型或返回校验失败，它会表现为用户可见错误；如何排查请继续看 [10-error-handling.md](10-error-handling.md)。

## 运行时类型查询

```dol
$>> 42.type();
$>> "hello".type();
$>> [1, 2].type();
$>> {"a": 1}.type();
```

## `std.core.check`

```dol
$mod std.core.check;

$>> check.type_of(42);
check.assert(1 + 1 == 2, "math broken");
```

## 下一章

继续看 [10-error-handling.md](10-error-handling.md)。
