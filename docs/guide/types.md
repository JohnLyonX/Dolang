# 类型注解

Dolang 支持**渐进类型注解**：注解完全可选，不影响程序运行，但能提升代码可读性。

---

## 变量类型注解

在变量名后加 `: 类型` 进行注解：

```dolang
$ count: Int = 0;
$ name: String = "Alice";
$ ratio: Float = 0.5;
$ active: Bool = true;
$ items: List = [1, 2, 3];
$ config: Map = {"key": "value"};
```

不带注解的写法同样有效：

```dolang
$ count = 0;
$ name = "Alice";
```

---

## 函数返回类型注解

在参数列表后加 `-> 类型`：

```dolang
$fn add(a, b) -> Int {
    $# a + b;
}

$fn greet(name) -> String {
    $# f"Hello, {name}!";
}

$fn is_even(n) -> Bool {
    $# n % 2 == 0;
}
```

没有返回值的函数可以省略注解：

```dolang
$fn log(msg) {
    $>> msg;
}
```

---

## 支持的类型名

| 类型名 | 说明 |
|--------|------|
| `Int` | 整数 |
| `Float` | 浮点数 |
| `String` | 字符串 |
| `Bool` | 布尔值 |
| `List` | 列表 |
| `Map` | 映射 |
| `Null` | 空值 |
| `Any` | 任意类型（不限制） |

---

## 运行时类型查询

通过 `.type()` 方法在运行时获取值的类型名：

```dolang
$>> 42.type();          // Int
$>> 3.14.type();        // Float
$>> "hello".type();     // String
$>> true.type();        // Bool
$>> [1, 2].type();      // List
$>> {"a": 1}.type();    // Map
$>> null.type();        // Null
```

配合条件判断：

```dolang
$fn describe(val) -> String {
    $ t = val.type();
    $if t == "Int" {
        $# "这是整数";
    } $else $if t == "String" {
        $# "这是字符串";
    } $else {
        $# f"类型: {t}";
    }
}

$>> describe(42);        // 这是整数
$>> describe("hello");   // 这是字符串
```

---

## 常量

使用 `$@` 声明不可修改的常量：

```dolang
$@ MAX_SIZE = 100;
$@ PI = 3.14159;
$@ APP_NAME = "Dolang";
```

常量支持类型注解：

```dolang
$@ MAX_SIZE: Int = 100;
$@ PI: Float = 3.14159;
```

尝试修改常量会报运行时错误：

```dolang
$@ X = 10;
X = 20;    // 错误：不能修改常量
```

---

## `std.core.check` 类型工具

```dolang
$mod std.core.check;

check.type_of(42);        // "Int"
check.type_of("hello");   // "String"
check.is_null(null);      // true
check.is_null(0);         // false

check.assert(1 + 1 == 2, "数学出错了");    // 通过
check.assert(false, "断言失败");           // 抛出异常
```
