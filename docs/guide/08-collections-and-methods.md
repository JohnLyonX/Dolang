# 8. 集合与常用方法

这一章集中讲三类最常用的值：字符串、列表、map。每一类都给出“创建、读取、修改、查询”的基本方式。

## 字符串

```dol
$ s = "hello";
$>> s.len();
$>> s.contains("ell");
$>> s.starts_with("he");
$>> s.ends_with("lo");
$>> s.slice(1, 3);
```

再看几个常用方法：

```dol
$ s = "  Hello, World  ";
$>> s.trim();
$>> s.upper();
$>> s.lower();
$>> s.replace("World", "Dolang");
$>> s.split(",");
$>> "42".to_int();
$>> "3.14".to_float();
$>> "true".to_bool();
```

字符串方法适合“当前值本身”的直接处理。

## 列表

```dol
$ nums = [3, 1, 4];
$>> nums.len();
nums.push(5);
$>> nums.pop();
$>> nums.contains(1);
```

如果你希望变量在后续赋值时继续保持列表类型，也可以显式声明：

```dol
$ nums: List = [3, 1, 4];
```

更多常用方法：

```dol
$ nums = [3, 1, 4, 1, 5];
$>> nums.index_of(4);
$>> nums.last_index_of(1);
$>> nums.slice(1, 4);
$>> nums.first();
$>> nums.last();
$>> nums.is_empty();
$>> nums.unique();
$>> nums.count(1);
$>> nums.join("-");
```

可变方法：

```dol
$ nums = [3, 1, 4];
nums.push(5);
nums.insert(1, 9);
nums.remove_at(0);
nums.reverse();
nums.sort();
$>> nums;
```

## Map

```dol
$ user = {"name": "Tom", "age": 25};
$>> user["name"];
$>> user.keys();
$>> user.values();
$>> user.contains_key("age");
```

同样可以显式声明：

```dol
$ user: Map = {"name": "Tom", "age": 25};
```

还可以删除 key：

```dol
$ user = {"name": "Tom", "age": 25};
user.remove("age");
$>> user;
```

## 索引访问

```dol
$ items = ["a", "b", "c"];
$>> items[0];
```

map 也可以通过 key 访问：

```dol
$ user = {"name": "Tom"};
$>> user["name"];
```

## 方法和标准库的区别

两类能力都会出现：

- 值方法：`"hello".len()`、`nums.push(1)`
- 标准库函数：`str.trim("  hi  ")`、`math.sqrt(9.0)`

值方法适合日常直接操作，`std.*` 模块适合整理通用函数。

一个最直观的对比：

```dol
$mod std.str;

$ text = "  hello  ";
$>> text.trim();
$>> str.trim(text);
```

这两种写法都能表达“去空白”，只是组织方式不同。

## 链式调用

当前不少表达式都可以直接链起来写：

```dol
$>> "a,b,c".split(",").len();
$>> "  hello  ".trim().upper();
```

这种写法适合短链路处理，但链太长时仍然建议拆成中间变量。

## 一个完整例子

```dol
$ user = {"name": "Alice", "tags": ["lang", "tool", "lang"]};
$ tags = user["tags"];

$>> user["name"].upper();
$>> tags.unique();
$>> tags.join(", ");
```

## 迁移期参考

- [stdlib.md](stdlib.md)
- [examples.md](examples.md)

## 下一章

继续看 [09-gradual-typing.md](09-gradual-typing.md)。
