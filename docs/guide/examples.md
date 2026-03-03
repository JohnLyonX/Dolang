# 实用示例

本章节提供真实使用场景的示例，帮助你快速上手 Dolang。

## 1. 快速计算

REPL 可以直接作为计算器使用：

```dao
>> $>> 1 + 2 * 3;
7

>> $>> (1 + 2) * 3;
9

>> $>> 10 / 3;
3.3333333333333335

>> $>> 10 % 3;
1
```

## 2. 字符串处理

```dao
>> $ name = "Dolang";
>> $>> "Hello, " + name + "!";
Hello, Dolang!

>> $ s = "Hello, World";
>> $>> s.upper();
HELLO, WORLD

>> $>> s.lower();
hello, world

>> $>> s.contains("World");
true

>> $>> "  hello  ".trim();
hello

>> $>> "a,b,c".split(",");
[a, b, c]
```

## 3. 条件表达式

```dao
>> $ x = 10;
>> $if x > 5 {
>>     $>> "big";
>> } $else {
>>     $>> "small";
>> }
big
```

### 多分支条件

```dao
>> $ age = 18;
>> $if age >= 18 {
>>     $>> "adult";
>> } $elif age >= 12 {
>>     $>> "teenager";
>> } $else {
>>     $>> "child";
>> }
```

## 4. 循环计算

### while 循环

```dao
>> $ sum = 0;
>> $while sum < 100 {
>>     sum = sum + 1;
>> }
>> $>> sum;
100
```

### for 循环

```dao
>> $ sum = 0;
>> $for i = 0; i < 100; i = i + 1 {
>>     sum = sum + i;
>> }
>> $>> sum;
4950
```

### 无限循环

```dao
>> $ i = 0;
>> $loop {
>>     $>> i;
>>     i = i + 1;
>>     $if i >= 5 {
>>         $break;
>>     }
>> }
0
1
2
3
4
```

## 5. 使用函数封装逻辑

### 基本函数

```dao
>> $fn double(x) {
>>     $# x * 2;
>> }
>> $>> double(5);
10
```

### 递归函数

```dao
>> $fn factorial(n) {
>>     $if n <= 1 {
>>         $# 1;
>>     } $else {
>>         $# n * factorial(n - 1);
>>     }
>> }
>> $>> factorial(5);
120
```

### 斐波那契数列

```dao
>> $fn fib(n) {
>>     $if n <= 1 {
>>         $# n;
>>     } $else {
>>         $# fib(n - 1) + fib(n - 2);
>>     }
>> }
>> $>> fib(10);
55
```

## 6. 列表操作

```dao
>> $ arr = [1, 2, 3];
>> $>> arr.len();
3

>> $>> arr.push(4);
[1, 2, 3, 4]

>> $>> arr.reverse();
[4, 3, 2, 1]

>> $>> arr.pop();
4
```

## 7. 字典操作

```dao
>> $ user = {"name": "Tom", "age": 18};
>> $>> user["name"];
Tom

>> $>> user.keys();
[name, age]

>> $>> user.contains_key("name");
true
```

## 8. for-in 遍历

### 遍历列表

```dao
>> $ arr = [1, 2, 3];
>> $for item in arr {
>>     $>> item;
>> }
1
2
3
```

### 遍历字典

```dao
>> $ user = {"name": "Tom", "age": 18};
>> $for key in user {
>>     $>> key;
>> }
name
age
```

### 遍历字符串

```dao
>> $ s = "abc";
>> $for ch in s {
>>     $>> ch;
>> }
a
b
c
```

## 9. 匿名函数

```dao
>> $ add = $fn(a, b) { $# a + b; };
>> $>> add(1, 2);
3
```

## 10. 链式调用

Dolang 支持方法链式调用：

```dao
>> $>> "a,b,c".split(",").len();     // 3

>> $>> [1, 2, 3].reverse().len();   // 3

>> $>> [1, 2, 3].push(4).len();    // 4

>> $ user = {"name": "Tom", "age": 18};
>> $>> user.keys().len();            // 2
```

---

这些示例涵盖了 Dolang 的主要用法，你可以根据自己的需求组合使用。
