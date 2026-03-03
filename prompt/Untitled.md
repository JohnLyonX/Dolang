# Dolang 标准输出规范

------

## 语法设计

### 普通输出

```
$>> "Hello";               →  输出字符串，默认换行
$>> name;                  →  输出变量
$>> 1 + 2;                 →  输出表达式结果
$>> "Hello" + name;        →  拼接输出
```

### 格式化输出

```
$>> f"Hello {name}";
$>> f"My name is {name}, I am {age} years old.";
$>> f"1 + 1 = {1 + 1}";          →  支持表达式
$>> f"price: {price.to_str()}";   →  支持方法调用
```

### stderr 输出

```
$>>ERR("something went wrong");
$>>ERR(f"variable {name} is not defined");
```

------

## Lexer 保留标识符

以下大写标识符在 $>> 之后具有特殊含义，不可用作变量名：

```
ERR    →  stderr 输出
FILE   →  文件输出（IO 阶段实现）
ALL    →  保留
```

------

## 错误规范

### 格式化字符串错误

**变量未定义**

```
$ name = "John";
$>> f"Hello {age}";

[ERROR] runtime error: f-string variable "age" is not defined
```

**花括号未闭合**

```
$>> f"Hello {name";

[ERROR] parse error: f-string syntax error: unclosed "{"
```

**空花括号**

```
$>> f"Hello {}";

[ERROR] parse error: f-string syntax error: empty expression in "{}"
```

**花括号多余关闭**

```
$>> f"Hello }";

[ERROR] parse error: f-string syntax error: unexpected "}"
```

### stderr 错误

**参数缺失**

```
$>>ERR();

[ERROR] runtime error: ERR requires exactly 1 argument
```

**参数类型错误**

```
$>>ERR(123);

[ERROR] runtime error: ERR argument must be a String
```

### 类型输出错误

**不可输出的类型（预留）**

```
$>> null;

[ERROR] runtime error: cannot print null value
```

------

## 行为规范

### 换行

```
$>> "Hello";         →  输出 Hello\n，默认换行
$>>ERR("msg");       →  输出到 stderr，默认换行
```

### f-string 作用域

```
f-string 直接读取当前作用域的变量，不需要显式传入：

$ name = "John";
$fn greet() {
    $>> f"Hello {name}";   →  ERROR: name 不在函数作用域内
}
```

函数作用域隔离规则不变，f-string 遵守现有作用域规则。

### 类型自动转换

```
$ age = 18;
$>> f"I am {age} years old.";   →  自动调用 to_str()，不报错
$>> f"price: {3.14}";           →  浮点自动转字符串
$>> f"flag: {true}";            →  布尔自动转字符串
```

------

## 完整使用示例

```
$ name = "John";
$ age = 18;
$ score = 99.5;

$>> "=== 用户信息 ===";
$>> f"姓名: {name}";
$>> f"年龄: {age}";
$>> f"分数: {score}";
$>> f"及格: {score >= 60}";

$>>ERR("以上为测试输出");
```

输出结果：

```
=== 用户信息 ===
姓名: John
年龄: 18
分数: 99.5
及格: true
```

stderr：

```
以上为测试输出
```

------

## 实现优先级

```
P0   $>> 普通输出        →  已有，确认行为一致
P0   $>> f"" 格式化      →  核心新功能
P1   $>>ERR()            →  stderr 分离
P2   $>>FILE()           →  IO 阶段实现，本阶段跳过
```

------

## 保留标识符冲突报错

```
$ ERR = "hello";

[ERROR] runtime error: "ERR" is a reserved identifier
```

------

*Dolang Team · Standard Output Spec Plan 2*