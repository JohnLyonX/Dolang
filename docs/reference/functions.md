# 函数系统

Dolang 支持完整的函数功能，包括函数定义、参数传递、返回值、递归等。

## 函数定义

使用 `$fn` 声明函数，`$#` 返回值：

```dao
$fn add(a, b) {
    $ result = a + b;
    $# result;
}
```

**语法**：

```dao
$fn 函数名(参数1, 参数2, ...) {
    函数体;
    $# 返回值;
}
```

**使用技巧**：

- 函数名在全局唯一，不能重复定义
- 参数是函数的局部变量，外部无法直接访问
- `$#` 语句会立即返回，后面的代码不会执行

---

## 函数参数传递

参数按值传递，函数内部修改参数不会影响外部变量：

```dao
$fn increment(x) {
    x = x + 1;
    $>> x;
}

$ a = 10;
increment(a);    // 输出: 11
$>> a;           // 输出: 10 (a 未改变)
```

---

## 返回语句

使用 `$#` 返回值：

```dao
$fn double(x) {
    $# x * 2;
}

$>> double(5);
10
```

---

## 返回类型声明

可以使用 `-> Type` 指定函数返回类型：

```dao
$fn double(x) -> Int {
    $# x * 2;
}

$>> double(5);
10
```

支持的类型：
- `Int` / `Integer` - 整数类型
- `Float` - 浮点数类型
- `String` - 字符串类型
- `Bool` / `Boolean` - 布尔类型

**返回类型不匹配时报错**：

```dao
$fn test() -> String {
    $# 123;
}
```

运行结果：
```
[ERROR] runtime error: function 'test' expects return type 'String' but got 'Int'
```

**函数有返回值但未声明返回类型时报错**：

```dao
$fn add(x, y) {
    $# x + y;
}
```

运行结果：
```
[ERROR] runtime error: function 'add' has no return type declared but returns a value
```

---

## 无返回值函数

```dao
$fn greet(name) {
    $>> "Hello, " + name;
}

greet("Dolang");
```

运行结果：
```
Hello, Dolang
```

---

## 函数调用

```dao
$fn add(a, b) {
    $# a + b;
}

$ sum = add(1, 2);
$>> sum;
3
```

---

## 匿名函数

Dolang 支持将函数赋值给变量，实现函数作为一等公民（first-class functions）：

```dao
$a = 20;
$b = 30;

$result = $fn(x,y) {
 $>> x + y;
};

result(a,b);
50
```

**语法**：

```dao
$变量名 = $fn(参数1, 参数2, ...) {
    函数体;
};
```

- 匿名函数不需要函数名
- 函数体末尾需要分号结束
- 可以像调用普通函数一样调用通过变量持有的函数

**带返回值的匿名函数**：

```dao
$result = $fn(x, y) -> Int {
    $# x + y;
};

$>> result(10, 20);
30
```

**无返回值的匿名函数**：

```dao
$result = $fn(x, y) {
    $>> x + y;
};

result(10, 20);
30
```

---

## 递归支持

Dolang 支持递归函数：

```dao
$fn factorial(n) {
    $if n <= 1 {
        $# 1;
    } $else {
        $# n * factorial(n - 1);
    }
}

$>> factorial(5);
120
```

---

## 作用域规则

函数内部有自己的作用域，外部变量无法直接访问：

```dao
$ x = 10;

$fn test() {
    $>> x;  // ERROR: variable 'x' not found
}

test();
```

要访问外部变量，需要通过参数传递：

```dao
$fn print_val(x) {
    $>> x;
}

$ x = 10;
print_val(x);  // 输出: 10
```

---

## 函数调用错误

### 调用未定义的函数

```dao
foo();
```

运行结果：
```
[ERROR] runtime error: function 'foo' is not defined
```

### 函数重定义错误

```dao
$fn add(a, b) {
    $# a + b;
}
$fn add(a, b) {
    $# a;
}
```

运行结果：
```
[ERROR] runtime error: function 'add' is already defined
```

---

## 方法调用

Dolang 支持对数据进行方法调用，方法调用**必须使用括号**。

### 语法

```dao
对象.方法名(参数);
```

### 示例

```dao
$ s = "hello";
$>> s.upper();        // 输出: HELLO
$>> s.lower();        // 输出: hello
$>> s.len();          // 输出: 5

$ arr = [1, 2, 3];
$>> arr.len();        // 输出: 3
$>> arr.push(4);     // 输出: [1, 2, 3, 4]

$ n = 123;
$>> n.to_str();       // 输出: "123"
```

### 错误：方法调用必须使用括号

```dao
$ s = "hello";
$>> s.upper;          // ❌ 错误
```

运行结果：
```
[ERROR] Parse error at line 1:1
  found: Ident upper
  expected: (
  method 'upper' requires parentheses, use 'upper(...)' instead
```

---

## 标准输入

Dolang 支持从标准输入读取数据，使用 `$<<` 关键字。

### 读取环境变量 `$<<ENV`

```dao
$ home = $<<ENV("HOME");
$>> home;             // 输出: /Users/xxx
```

**错误情况**：

```dao
$ x = $<<ENV("NOT_EXIST");
```

运行结果：
```
[ERROR] runtime error: environment variable 'NOT_EXIST' is not defined
```

### 读取 stdin `$<<LINE`

```dao
// 无提示输入
$ name = $<<LINE();
$>> name;

// 有提示输入
$ age = $<<LINE("请输入年龄: ");
$>> age;
```

### 不赋值（pause 效果）

单独使用 `$<<LINE()` 会等待用户输入，按回车后继续，不保存输入：

```dao
$>> "按回车继续...";
$<<LINE();
$>> "程序继续执行";
```
