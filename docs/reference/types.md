# 类型系统

DaoLang v1.3 采用**渐进式类型系统**，结合了动态语言的灵活性和静态类型的安全性。

> **设计原则：能跑就不用加，怕出错再加。**

---

## 快速入门

### 动态模式（默认）

不写类型注解 → 完全动态，变量可以随时改类型：

```dao
$ x = 30;
x = "hello";   // ✅ 允许，类型从 Int 变为 String
x = true;      // ✅ 允许，类型从 String 变为 Bool
```

### 静态模式（可选）

写了类型注解 → 静态检查，类型保护生效：

```dao
$ x: Int = 30;
x = 100;       // ✅ 允许，同为 Int
x = "hello";   // ❌ 报错，类型不匹配
```

---

## 支持的数据类型

## 整数 (Integer)

表示整数值，支持正数和负数：

```dao
$>> 42;
42

$>> -17;
-17
```

**数值范围**：取决于实现，通常为 64 位有符号整数

**整数方法**：

```dao
$>> (42).to_str();      // 转换为字符串: "42"
$>> (42).to_float();    // 转换为浮点数: 42.0
$>> (1).to_bool();      // 转换为布尔: true
$>> (0).to_bool();      // 转换为布尔: false

$>> (42).type();        // 获取类型名称: Int
```

> 注意：整数方法调用需要用括号包裹，如 `(42).to_str()`

---

## 浮点数 (Float)

表示小数数值：

```dao
$>> 3.14;
3.14

$>> -2.5;
-2.5

$>> 0.5;
0.5
```

**浮点数方法**：

```dao
$>> (3.14).to_str();    // 转换为字符串: "3.14"
$>> (3.14).to_int();    // 转换为整数（截断）: 3

$>> (3.14).type();      // 获取类型名称: Float
```

> 注意：浮点数方法调用需要用括号包裹，如 `(3.14).to_int()`

---

## 字符串 (String)

使用双引号 `"..."` 包裹的文本：

```dao
$>> "Hello World";
Hello World

$>> "";
(空输出)

$>> "Dao" + "Lang";
Dolang
```

**特性**：
- 支持 `+` 运算符进行字符串连接
- 内部统一存储为字符串类型

**字符串方法**：

```dao
$ s = "Hello, World";

$>> s.len();              // 获取字符串长度: 12
$>> s.upper();            // 转大写: HELLO, WORLD
$>> s.lower();            // 转小写: hello, world

$>> "  hello  ".trim();   // 去除首尾空格: hello

$>> "hello".contains("ell");      // 判断包含: true
$>> "hello".starts_with("he");   // 判断前缀: true
$>> "hello".ends_with("lo");     // 判断后缀: true

$>> "hello".replace("l", "r");  // 替换: herro

$>> "a,b,c".split(",");         // 分割返回列表: [a, b, c]

$>> "hello".slice(1, 3);       // 截取子串: el

$>> "42".to_int();      // 转换为整数: 42
$>> "3.14".to_float();  // 转换为浮点数: 3.14

$>> "true".to_bool();   // 转换为布尔: true
$>> "false".to_bool();  // 转换为布尔: false

$>> "hello".type();     // 获取类型名称: String
```

**格式化字符串 (f-string)**：

```dao
$ name <& "World";
$>> f"Hello {name}";              // 输出: Hello World

$ age <& 25;
$>> f"I'm {age} years old";       // 输出: I'm 25 years old

$>> f"1 + 2 = {1 + 2}";          // 输出: 1 + 2 = 3

$ price <& 19.99;
$>> f"Price: {price.to_str()}";   // 输出: Price: 19.99
```

**f-string 特性**：
- 使用 `f"..."` 语法
- 花括号 `{...}` 内可以是变量、表达式或方法调用
- 支持类型自动转换（Int/Float/Bool 会自动调用 to_str()）

**错误示例**：

```dao
$>> f"Hello {}";
// [ERROR] Parse error: f-string syntax error: empty expression in "{}"

$>> f"Hello";
// [ERROR] f-string variable "undefined" is not defined
```

---

## 字符 (Char)

使用单引号 `'...'` 包裹的单个字符：

```dao
$>> 'a';
a

$>> 'Z';
Z
```

**注意**：字符类型在内部也以字符串形式存储

---

## 布尔 (Boolean)

表示真值，`true` 或 `false`：

```dao
$>> true;
true

$>> false;
false
```

**注意**：布尔类型主要用于条件判断

**布尔方法**：

```dao
$>> true.to_str();      // 转换为字符串: "true"
$>> false.to_str();     // 转换为字符串: "false"

$>> true.type();        // 获取类型名称: Bool
```

---

## 列表 (List)

表示一组有序的值集合：

```dao
$ arr = [1, 2, 3];
$>> arr[0];
1

$>> arr[2];
3
```

**空列表**：

```dao
$ empty = [];
$>> empty;
[]
```

**混合类型**：

```dao
$ mixed = [1, "hello", true];
$>> mixed[0];
1

$>> mixed[1];
hello

$>> mixed[2];
true
```

**嵌套列表**：

```dao
$ matrix = [[1, 2], [3, 4]];
$>> matrix[0][0];
1

$>> matrix[1][1];
4
```

**修改列表元素**：

```dao
$ arr = [1, 2, 3];
arr[1] = 99;
$>> arr[1];
99
```

**列表方法**：

```dao
$ arr = [1, 2, 3];
$>> arr.len();           // 获取列表长度: 3

$>> arr.contains(1);    // 检查是否包含元素: true
$>> arr.contains(5);   // 检查是否包含元素: false

$>> arr.push(4);        // 末尾添加元素: [1, 2, 3, 4]
$>> arr.pop();          // 移除末尾元素并返回: 4

$>> arr.reverse();       // 反转列表: [3, 2, 1]

$>> arr.join(", ");     // 拼接为字符串: "1, 2, 3"

$>> arr.type();         // 获取类型名称: List
```

---

## 字典 (Map)

表示键值对集合：

```dao
$ user = {"name": "Tom", "age": 18};
$>> user["name"];
Tom

$>> user["age"];
18
```

**空字典**：

```dao
$ empty = {};
$>> empty;
{}
```

**修改字典值**：

```dao
$ user = {"name": "Tom"};
user["email"] = "tom@example.com";
$>> user["email"];
tom@example.com
```

**嵌套字典**：

```dao
$ config = {"db": {"host": "localhost", "port": 5432}};
$>> config["db"]["host"];
localhost

$>> config["db"]["port"];
5432
```

**字典与列表混用**：

```dao
$ data = {"tags": [1, 2, 3], "name": "test"};
$>> data["tags"][0];
1

$>> data["name"];
test
```

**字典方法**：

```dao
$ user = {"name": "Tom", "age": 18};
$>> user.len();              // 获取键值对数量: 2

$>> user.keys();            // 获取所有键: [name, age]
$>> user.values();         // 获取所有值: [Tom, 18]

$>> user.contains_key("name");    // 检查键是否存在: true
$>> user.contains_key("email");  // 检查键是否存在: false

$>> user.remove("age");   // 移除键值对并返回新字典: {name: Tom}

$>> user.type();           // 获取类型名称: Map
```

---

## 类型注解（可选）

使用 `$ 变量名: 类型 = 值` 语法指定类型：

```dao
$ count: Int = 0;
$ name: String = "DaoLang";
$ price: Float = 9.9;
$ active: Bool = true;
```

### 支持的类型注解

| 类型注解 | 别名 | 说明 |
|---|---|---|
| `Int` | `Integer` | 整数 |
| `Float` | - | 浮点数 |
| `String` | `Str` | 字符串 |
| `Bool` | `Boolean` | 布尔值 |

---

## 动态模式 vs 静态模式

### 动态模式（默认）

变量声明时不指定类型，类型随赋值自动变化，无任何限制：

```dao
$ x = 30;
x = "hello";   // ✅ 允许，类型从 Int 变为 String
x = true;      // ✅ 允许，类型从 String 变为 Bool
x = 3.14;      // ✅ 允许，类型从 Bool 变为 Float
```

**适用场景**：快速验证想法、原型开发、摆摊阶段。

### 静态模式（显式声明）

变量声明时指定类型注解，后续赋值必须保持类型一致：

```dao
$ x: Int = 30;
x = 100;       // ✅ 允许，同为 Int
x = "hello";   // ❌ 报错，类型不匹配
x = 3.14;      // ❌ 报错，类型不匹配
```

**适用场景**：涉及金额、ID、状态码等关键变量，项目规模变大后。

---

### 常量类型注解

常量 `$@` 同样支持类型注解：

```dao
$@ MAX_RETRY: Int = 3;
$@ API_URL: String = "https://api.daoLang.co";
```

类型注解对常量而言是**文档性质**，增强可读性（常量本身已不可修改）。

---

### 两种模式混用

同一个文件里，两种模式可以共存，互不影响：

```dao
$ name = "DaoLang";   // 动态模式
$ price: Float = 9.9; // 静态模式

name = 123;           // ✅ 动态，允许
price = "free";       // ❌ 静态，报错
```

---

## 变量遮蔽 (Shadowing)

使用 `$` 前缀可以声明同名变量（遮蔽），允许改变类型：

```dao
$ a = 20;           // a = 20 (整数)
$ a = "hello";      // 遮蔽，新 a = "hello" (字符串)
$>> a;              // 输出: hello
```

---

## 类型注解 (静态模式)

Dolang 支持两种模式：**动态模式**（默认）和**静态模式**（类型注解）。

### 动态模式（默认）

变量声明时不指定类型，类型随赋值自动变化：

```dao
$ x = 30;
x = "hello";   // ✅ 允许，类型从 Int 变为 String
x = true;      // ✅ 允许，类型从 String 变为 Bool
```

### 静态模式（类型注解）

变量声明时指定类型，后续赋值必须保持类型一致：

```dao
$ x: Int <& 30;
x = 100;       // ✅ 允许，同为 Int
x = "hello";   // ❌ 报错，类型不匹配
```

**支持的类型注解**：

| 类型注解 | 说明 |
|---|---|
| `Int` / `Integer` | 整数 |
| `Float` | 浮点数 |
| `String` / `Str` | 字符串 |
| `Bool` / `Boolean` | 布尔值 |

**类型不匹配错误**：

```dao
$ x: Int <& 30;
x = "hello";
// [ERROR] runtime error: type mismatch: cannot assign hello to variable 'x' of type Int
```

---

## 统一的数据表示

所有数据在内部统一存储为字符串，简化实现：

```dao
$>> 42;        // 输出: 42
$>> "hello";   // 输出: hello
$>> 'a';       // 输出: a
```

---

## 运行时类型获取

使用 `.type()` 方法可以获取值的当前类型信息：

```dao
$>> 42.type();          // 输出: Int
$>> 3.14.type();        // 输出: Float
$>> "hello".type();     // 输出: String
$>> true.type();        // 输出: Bool
$>> [1,2,3].type();    // 输出: List
$>> {"a":1}.type();    // 输出: Map
```

> 注意：对于数值类型，方法调用需要用括号包裹，如 `(42).type()`

---

## 方法调用必须使用括号

Dolang 中所有方法调用**必须使用括号**：

```dao
$ s = "hello";
$>> s.upper();     // ✅ 正确: HELLO
$>> s.upper;      // ❌ 错误

$ arr = [1, 2, 3];
$>> arr.len();    // ✅ 正确: 3
$>> arr.len;     // ❌ 错误
```

**错误信息**：

```
[ERROR] Parse error at line 1:1
  found: Ident upper
  expected: (
  method 'upper' requires parentheses, use 'upper(...)' instead
```
