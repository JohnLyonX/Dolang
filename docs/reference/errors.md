# 错误参考

本文档列出 Dolang 常见的错误类型及其处理方式。

## 语法错误

Dolang 提供详细的解析错误信息，包括错误位置、找到的 token 和期望的内容：

```dao
$result = $fn(x,y) {
 x + y
}

result(a,b);
```

运行结果：
```
[ERROR] Parse error at line 1:1
  found: Ident
  unexpected token after expression
```

**错误信息包含**：
- 错误位置（行:列）
- 找到的 token 类型
- 期望的内容或错误描述

### 常见语法错误

- 缺少分号：`$>> "hello"` → 需要 `$>> "hello";`
- 缺少大括号：`$if true` → 需要 `$if true { }`
- 括号不匹配：函数调用参数未闭合

---

## 类型错误

### 动态模式（默认）- 重新赋值可改变类型

v1.3 起，动态模式下重新赋值可以改变类型：

```dao
$ a = 1;
a = "hello";   // ✅ 允许，类型从 Int 变为 String
```

### 静态模式 - 类型不匹配错误

使用类型注解后，重新赋值必须保持类型一致：

```dao
$ a: Int = 1;
a = "hello";
```

运行结果：
```
[ERROR] type error: variable 'a' is declared as 'Int', cannot assign 'String' value
  hint: use '$ a: String = ...' to redeclare with a new type
```

**解决方案**：
- 使用 `$` 重新声明（变量遮蔽）：`$ a = "hello";`
- 或重新指定类型：`$ a: String = "hello";`

---

### 声明时类型注解与值不匹配

类型注解和实际值的类型不一致：

```dao
$ x: Int = "hello";
```

运行结果：
```
[ERROR] type error: declared type 'Int' does not match value type 'String'
  hint: change the annotation or the value
```

---

### 未知类型注解

使用了不支持的类型名：

```dao
$ x: Number = 30;
```

运行结果：
```
[ERROR] type error: unknown type 'Number', supported types are: Int, Float, String, Bool
```

---

## 常量重赋值错误

常量不可修改：

```dao
$@ PI = 3.14;
PI = 3.14159;
```

运行结果：
```
[ERROR] runtime error: cannot reassign constant 'PI'
```

**解决方案**：使用 `$` 声明新变量

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

### 返回类型不匹配

```dao
$fn test() -> String {
    $# 123;
}
```

运行结果：
```
[ERROR] runtime error: function 'test' expects return type 'String' but got 'Int'
```

### 未声明返回类型但返回值

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

## 未定义变量错误

使用未声明的变量：

```dao
$>> x;
```

运行结果：
```
[ERROR] runtime error: variable 'x' not found
```

---

## 运行时错误

### 除零错误

除法和取模运算除以零时返回错误：

```dao
$>> 10 / 0;
```

运行结果：
```
[ERROR] runtime error: division by zero
```

---

## 错误处理最佳实践

1. **仔细阅读错误信息**：错误信息会指出错误位置和原因
2. **动态模式**：不需要类型约束时，直接 `$ x = value;` 即可随意改变类型
3. **静态模式**：需要类型保护时，使用 `$ x: Type = value;` 声明
4. **变量遮蔽**：需要改变类型时使用 `$` 重新声明
5. **声明返回类型**：函数有返回值时务必声明返回类型
6. **先声明后使用**：变量必须先声明再使用
