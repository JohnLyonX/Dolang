# 9. 渐进类型

Dolang 当前采用“默认动态，显式约束”的使用方式。

## 默认动态

不写类型注解时，变量可以改变值类型：

```dol
$ x = 30;
x = "hello";
x = true;
```

这表示：

- 不写类型注解时，Dolang 不把变量锁死在某一个类型上
- 这对快速脚本和原型开发很方便

例如：

```dol
$ value = 1;
$>> value.type();

value = "done";
$>> value.type();
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

这类注解的作用是：

- 声明时检查右值类型
- 后续赋值时继续检查

例如：

```dol
$ count: Int = 1;
count = 2;
```

下面这种写法会报错：

```dol
$ count: Int = 1;
count = "two";
```

## 当前可依赖的声明类型名

按当前实现，变量/常量声明里稳定可写入主线文档的类型名包括：

- `Int`
- `Float`
- `String`
- `Str`
- `Bool`
- `Boolean`
- `List`
- `Map`

我实际验证过这组声明在三种入口都能工作：

- REPL
- 文件模式：`cargo run -- run xxx.dol`
- serve 模式：`cargo run -- serve .`

例如：

```dol
$ items: List = [1, 2, 3];
$ config: Map = {"name": "demo", "ok": true};

$>> items.type();
$>> config.type();
$>> items[0];
$>> config["name"];
```

仍然不要把下面这些提前写成“变量/常量声明已稳定支持”的类型名：

- `Json`
- `Response`

## 函数返回类型

```dol
$fn greet(name) -> String {
    $# f"Hello, {name}";
}
```

函数返回类型会做运行时检查。

另一个例子：

```dol
$fn is_even(n) -> Bool {
    $# n % 2 == 0;
}
```

## `$Type` 与 `JSON<User>`

当前实现支持 `$Type` 声明形状，也支持 `JSON<User>` 这样的返回标注语法。

最小例子：

```dol
$Type User {
    id: Int
    name: Str
}
```

更完整一点的写法：

```dol
$Type User {
    id: Int
    name: Str
    email: Str?
    age: Int?
}
```

这里可以先记住这些规则：

- 字段类型通常写成首字母大写的类型名
- 类型后加 `?` 表示可选字段
- `$Type` 用来描述结构，不等于 class
- 当前主线文档不把它讲成带 getter / setter / 方法的对象系统

### 和函数返回结合

```dol
$fn get_user(id) -> JSON<User> {
    $# $JSON { "id": id, "name": "Alice" };
}
```

### 和 HTTP handler 结合

```dol
$GET("/users/:id") get_user(id) -> JSON<User> {
    $# $JSON { "id": id, "name": "Alice" };
}
```

`JSON<User>` 更适合先理解成“返回值想表达的是 `User` 形状的 JSON”。

这能帮助你把数据结构写得更清楚，但不要把它误解成已经等同于成熟静态类型系统或完整强校验模型。

推荐阅读方式：

1. 在这一章先知道 `$Type` 是什么
2. 到 HTTP 章节再看 `JSON<User>` 的接口写法

## 运行时类型查询

除了类型注解，Dolang 还支持在运行时查询值类型：

```dol
$>> 42.type();
$>> 3.14.type();
$>> "hello".type();
$>> true.type();
$>> [1, 2].type();
$>> {"a": 1}.type();
$>> null.type();
```

这在调试动态值时很有用。

对 `List` 和 `Map` 来说，显式声明与运行时类型查询可以直接配合：

```dol
$ items: List = [1, 2, 3];
$ config: Map = {"name": "demo"};

$>> items.type();
$>> config.type();
```

例如：

```dol
$fn describe(val) -> String {
    $ t = val.type();
    $if t == "Int" {
        $# "这是整数";
    } $elif t == "String" {
        $# "这是字符串";
    } $else {
        $# f"类型: {t}";
    }
}
```

## `std.core.check`

如果你想把类型判断和简单断言写得更清楚，可以用 `std.core.check`：

```dol
$mod std.core.check;

$>> check.type_of(42);
$>> check.is_null(null);
check.assert(1 + 1 == 2, "math broken");
```

它适合：

- 小型断言
- 调试时检查值类型
- 写更清晰的工具函数

## 什么时候该加类型

比较适合优先加类型注解的地方：

- 计数器
- 金额
- 状态值
- 配置项
- 对外返回值

不需要一上来给所有变量都加类型，Dolang 的设计也不是鼓励你这么做。

## 迁移期参考

- [types.md](types.md)
- `docs/spec/semantics.md`

## 下一章

继续看 [10-error-handling.md](10-error-handling.md)。
