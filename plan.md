# Dolang 语言开发计划

## 项目概述

Dolang 是一个用 Rust 编写的轻量级解释型编程语言，采用独特的 `$` 符号前缀风格设计。

**当前版本**: v1.0
**核心语言**: Rust

---

## 已实现功能 (v1.0)

### ✅ 数据类型
- [x] 整数 (Integer): 支持正负整数
- [x] 浮点数 (Float): 支持小数
- [x] 字符串 (String): 双引号包裹
- [x] 字符 (Char): 单引号包裹
- [x] 布尔 (Boolean): `true` / `false`

### ✅ 变量与常量
- [x] 变量声明: `$ variable = value;`
- [x] 变量重新赋值: `variable = value;` (无 `$` 前缀)
- [x] 变量遮蔽 (Shadowing): 支持同名变量重新声明不同类型
- [x] 常量声明: `$@ constant = value;`
- [x] 常量不可重新赋值 (运行时错误保护)
- [x] 变量直接读取: `variable` (无需 `$.` 前缀)

### ✅ 动态类型系统
- [x] 运行时类型推断
- [x] 类型检查 (重新赋值需相同类型)
- [x] 类型错误提示

### ✅ 运算符
- [x] 算术运算符: `+`, `-`, `*`, `/`, `%`
- [x] 比较运算符: `==`, `!=`, `>`, `<`, `>=`, `<=`
- [x] 逻辑运算符: `&&`, `||`, `!`
- [x] 一元负号: `-value`

### ✅ 语句
- [x] 打印语句: `$>> expression;`
- [x] 变量读取: `variable` (直接使用变量名)
- [x] 单行注释: `// 注释内容`
- [x] 多行注释: `/* 注释内容 */`

### ✅ 流程控制
- [x] 条件语句: `$if` / `$elif` / `$else`
- [x] 循环语句: `$while` / `$loop` / `$for`
- [x] 控制流: `$break` / `$continue`

### ✅ REPL
- [x] 交互式解释器
- [x] 历史记录 (上下箭头)
- [x] 光标移动 (左右箭头)
- [x] 退格支持
- [x] Ctrl+C 中断
- [x] 彩色错误信息

---

## 版本规划

| 版本 | 目标 | 状态 |
|------|------|------|
| v0.6 | 变量遮蔽、类型系统 | ✅ 已完成 |
| v0.7 | 布尔、比较、逻辑、算术运算 | ✅ 已完成 |
| v0.8 | 条件语句 ($if/$else/$elif) | ✅ 已完成 |
| v0.9 | 循环语句 ($while/$loop/$for) | ✅ 已完成 |
| v1.0 | 函数系统 ($fn) | ✅ 已完成 |
| v1.1 | 复合赋值运算符 | ✅ 已完成 |
| v1.2 | List/Map 数据结构 + 类型打印 | ✅ 已完成 |
| v1.3 | 链式调用与内置方法 + 类型转换 | ✅ 已完成 |
| v1.4 | for-in 遍历语法 | ✅ 已完成 |
| v1.5 | 渐进式类型系统（静态模式） | ✅ 已完成 |
| v1.6 | 标准输入/输出/错误 | ✅ 已完成 |

---

## P1 - 函数系统

### 1. `$fn` 普通函数 ✅
- [x] 函数定义: `$fn name(params) -> return_type { body }`
- [x] 无返回值函数: `$fn name(params) { body }`
- [x] 函数调用: `name(args)`
- [x] 返回语句: `$# expression;`
- [x] 作用域规则: 函数内外变量隔离
- [x] 递归支持
- [x] 参数传递规则

---

## P1 - 语法增强

### 2. 复合赋值运算符 (Compound Assignment Operators) ✅ 已完成
- [x] `+=` 加法复合赋值: `i += 1;`
- [x] `-=` 减法复合赋值: `i -= 1;`
- [x] `*=` 乘法复合赋值: `i *= 2;`
- [x] `/=` 除法复合赋值: `i /= 2;`
- [x] `%=` 取模复合赋值: `i %= 3;`

**实现要点:**
1. 在词法分析器 (lexer) 添加新 token 类型: `PlusAssign`, `MinusAssign`, `MulAssign`, `DivAssign`, `ModAssign`
2. 识别复合赋值运算符模式: `<=` 改为 `<+=`, `<-=`, `<*=`, `</=`, `<%=`
3. 在解析器处理: 解析为 `Assign(VarLookup(i), BinaryExpr(VarLookup(i), Plus, Number(1)))`
4. 添加运行时处理: 读取变量值 → 计算 → 重新赋值

**使用场景:**
```dao
$ i = 10;
i += 1;    // i = 11
i -= 3;    // i = 8
i *= 2;    // i = 16
i /= 4;    // i = 4
i %= 3;    // i = 1
```

---

### 3. List 类型 ✅ 已完成

**语法:**
- [x] 空 List: `$ arr = [];`
- [x] 带初始值: `$ arr = [1, 2, 3];`
- [x] 混合类型: `$ arr = [1, "hello", true];`
- [x] 嵌套 List: `$ arr = [[1, 2], [3, 4]];`
- [x] 下标读取: `arr[0]`
- [x] 下标赋值: `arr[0] = 99;`
- [x] 越界访问报错

**实现要点:**
1. `Value` 枚举新增变体: `List(Vec<Value>)`
2. Lexer 新增 token: `LBracket` (`[`)、`RBracket` (`]`)
3. Parser 新增列表字面量解析: `[expr, expr, ...]`
4. Parser 新增下标访问表达式: `expr[expr]`
5. AST 新增节点: `ListLiteral(Vec<Expr>)`、`IndexAccess { object: Box<Expr>, index: Box<Expr> }`
6. Interpreter eval 处理 `ListLiteral` → 求值每个元素 → 构建 `Value::List`
7. Interpreter eval 处理 `IndexAccess` → 取出 List → 验证下标 → 返回元素
8. Interpreter exec 处理下标赋值 → 取出 List → 验证下标 → 修改元素

**使用场景:**
```dao
$ arr = [1, 2, 3];
$>> arr[0];          // 1
$>> arr[2];          // 3
arr[1] = 99;
$>> arr[1];          // 99

$ matrix = [[1, 2], [3, 4]];
$>> matrix[0][1];    // 2
```

**错误场景:**
```dao
$ arr = [1, 2, 3];
$>> arr[5];
// [ERROR] runtime error: index out of bounds: list length is 3 but index is 5
```

---

### 4. Map 类型 ✅ 已完成

**语法:**
- [x] 空 Map: `$ maps = {};`
- [x] 带初始值: `$ maps = {"key": "value"};`
- [x] 多键值对: `$ maps = {"name": "Tom", "age": 18};`
- [x] 嵌套 Map: `$ maps = {"db": {"host": "localhost", "port": 5432}};`
- [x] Map 嵌套 List: `$ maps = {"tags": [1, 2, 3]};`
- [x] key 读取: `maps["name"]`
- [x] key 赋值: `maps["email"] = "tom@example.com";`
- [x] 访问不存在的 key 报错

**实现要点:**
1. `Value` 枚举新增变体: `Map(IndexMap<String, Value>)`
2. Cargo.toml 新增依赖: `indexmap = "2"`（保持插入顺序）
3. Lexer 新增 token: `LBrace` (`{`)、`RBrace` (`}`)、`Colon` (`:`)
4. Parser 新增 Map 字面量解析: `{ string: expr, string: expr, ... }`
5. AST 新增节点: `MapLiteral(Vec<(String, Expr)>)`
6. Interpreter eval 处理 `MapLiteral` → 求值每个 value → 构建 `Value::Map`
7. 下标访问复用 `IndexAccess` 节点，eval 时区分 List/Map 分支处理
8. Map 赋值: key 不存在则插入，存在则覆盖

**使用场景:**
```dao
$ user = {"name": "Tom", "age": 18};
$>> user["name"];           // Tom
$>> user["age"];            // 18

user["email"] = "tom@example.com";
$>> user["email"];          // tom@example.com

$ config = {"db": {"host": "localhost"}};
$>> config["db"]["host"];   // localhost

$ data = {"tags": [1, 2, 3]};
$>> data["tags"][0];        // 1
```

**错误场景:**
```dao
$ maps = {"name": "Tom"};
$>> maps["age"];
// [ERROR] runtime error: key 'age' not found in map
```

---

### 5. 类型打印 ✅ 已完成

**语法:**
- [x] List 打印: `$>> list;` → `[1, 2, 3]`
- [x] Map 打印: `$>> map;` → `{"name": "Tom", "age": 18}`
- [x] 嵌套结构正确缩进输出

**实现要点:**
1. `Value` 实现 `Display` trait
2. `List` 递归格式化每个元素，用 `, ` 分隔，外层加 `[` `]`
3. `Map` 递归格式化每个键值对，用 `: ` 连接，用 `, ` 分隔，外层加 `{` `}`
4. 字符串值输出时加双引号，数字/布尔不加

**输出示例:**
```dao
$ arr = [1, "hello", true];
$>> arr;
// [1, "hello", true]

$ user = {"name": "Tom", "age": 18};
$>> user;
// {"name": "Tom", "age": 18}

$ nested = {"tags": [1, 2, 3]};
$>> nested;
// {"tags": [1, 2, 3]}
```

---

### 6. 链式调用与内置方法基础设施 ✅ 已完成

**实现顺序:**
```
6.1 链式调用基础设施（MethodCall AST + Parser + Interpreter 分发）
    ↓
6.2 字符串内置方法
    ↓
6.3 List / Map 内置方法
    ↓
6.4 for-in 遍历语法
```

---

#### 6.1 链式调用基础设施 ✅ 已完成

**实现要点:**
1. AST 新增 `MethodCall` 节点: `{ object: Box<Expr>, method: String, args: Vec<Expr> }`
2. Lexer 新增 Dot token: `.`
3. Parser expr.rs 后缀循环新增 Dot 分支处理
4. Interpreter eval.rs 新增 `MethodCall` 求值逻辑
5. 类型分发：根据 Value 类型路由到对应方法处理器
6. 无效方法调用报错: `[ERROR] runtime error: 'Int' has no method 'push'`

**数据流:**
```
arr.push(1).push(2).len()
        ↓ Parser
MethodCall(MethodCall(MethodCall(arr, push,[1]), push,[2]), len,[])
        ↓ Interpreter（由内向外求值）
eval arr        → Value::List([])
eval .push(1)   → Value::List([1])
eval .push(2)   → Value::List([1, 2])
eval .len()     → Value::Int(2)
```

---

#### 6.2 字符串内置方法 ✅ 已完成

**方法列表:**
- [x] `s.len()` 返回字符串长度: `"hello".len()` → `5`
- [x] `s.upper()` 转大写: `"hello".upper()` → `"HELLO"`
- [x] `s.lower()` 转小写: `"HELLO".lower()` → `"hello"`
- [x] `s.trim()` 去除首尾空格: `"  hi  ".trim()` → `"hi"`
- [x] `s.contains(sub)` 判断包含: `"hello".contains("ell")` → `true`
- [x] `s.starts_with(prefix)` 判断前缀: `"hello".starts_with("he")` → `true`
- [x] `s.ends_with(suffix)` 判断后缀: `"hello".ends_with("lo")` → `true`
- [x] `s.replace(old, new)` 替换: `"hello".replace("l", "r")` → `"herro"`
- [x] `s.split(sep)` 分割返回 List: `"a,b,c".split(",")` → `["a", "b", "c"]`
- [x] `s.slice(start, end)` 截取子串: `"hello".slice(1, 3)` → `"el"`

**使用场景:**
```dao
$ s = "Hello, World";
$>> s.len();                         // 12
$>> s.upper();                       // HELLO, WORLD
$>> s.lower();                       // hello, world
$>> s.contains("World");             // true
$>> s.replace("World", "Dolang");    // Hello, Dolang
$>> s.split(",");                    // ["Hello", " World"]
$>> s.split(",").len();              // 2  ← 链式调用
```

---

#### 6.3 List / Map 内置方法 ✅ 已完成

**List 方法:**
- [x] `list.push(val)` 末尾追加，返回 List（支持链式）
- [x] `list.pop()` 移除末尾元素，返回被移除的值
- [x] `list.len()` 返回元素数量
- [x] `list.contains(val)` 判断是否包含某值
- [x] `list.reverse()` 反转列表，返回新 List（支持链式）
- [x] `list.join(sep)` 拼接为字符串

**Map 方法:**
- [x] `map.keys()` 返回所有 key 的 List
- [x] `map.values()` 返回所有 value 的 List
- [x] `map.contains_key(key)` 判断 key 是否存在
- [x] `map.remove(key)` 删除指定 key，返回被删除的值
- [x] `map.len()` 返回键值对数量

**使用场景:**
```dao
// List
$ arr = [3, 1, 2];
$>> arr.len();                   // 3
$>> arr.contains(1);             // true
$>> arr.push(4).push(5).len();   // 5  ← 链式调用
$>> arr.reverse();               // [2, 1, 3]
$>> arr.join(", ");              // "3, 1, 2"

// Map
$ user = {"name": "Tom", "age": 18};
$>> user.len();                  // 2
$>> user.keys();                 // ["name", "age"]
$>> user.values();               // ["Tom", 18]
$>> user.contains_key("name");   // true
$>> user.keys().len();           // 2  ← 链式调用
```

---

#### 6.4 for-in 遍历语法 ✅ 已完成

**语法:**
- [x] List 遍历: `$for item in list { ... }`
- [x] Map key 遍历: `$for key in map { ... }`
- [x] 字符串遍历: `$for char in str { ... }`
- [x] 遍历非可迭代类型时报错

**实现要点:**
1. Lexer 新增关键字 `in`
2. AST 新增节点: `ForIn { var: String, iterable: Box<Expr>, body: Vec<Stmt> }`
3. Parser stmt.rs 在解析 `$for` 时，检测是否存在 `in` 关键字，分支到 ForIn 解析路径
4. Interpreter exec.rs 处理 ForIn：求值 iterable → 遍历每个元素 → 绑定到 var → 执行 body
5. `$break / $continue` 在 `$for-in` 内正常工作

**使用场景:**
```dao
// List 遍历
$ arr = [1, 2, 3];
$for item in arr {
    $>> item;
}
// 1
// 2
// 3

// Map 遍历 key
$ user = {"name": "Tom", "age": 18};
$for key in user {
    $>> key;
}
// name
// age

// 字符串遍历
$ s = "abc";
$for ch in s {
    $>> ch;
}
// a
// b
// c
```

---

### 7. 类型转换函数 ✅ 已完成

**内置函数:**
- [x] `to_int()` String / Float → Int
- [x] `to_float()` String / Int → Float
- [x] `to_str()` Int / Float / Bool → String
- [x] `to_bool()` String / Int → Bool
- [x] `type()` 查看当前类型（作为链式方法调用）

**错误规范:**
- 类型不支持转换: `type mismatch: {FromType} cannot convert to {ToType}`
- 值无法解析: `cannot convert "{value}" to {ToType}`
- 参数错误: `{fn}() requires exactly {n} argument`
- 附加说明: 换行缩进补充 `expected ...`

**统一格式:** `[ERROR] runtime error: {错误描述}`

---

#### 7.1 to_int() - String / Float → Int ✅ 已完成

**成功场景:**
```dao
$>> "42".to_int();       // → 42
$>> "3.14".to_int();     // → 3 (截断小数)
$>> 3.14.to_int();       // → 3
```

**失败场景:**
```dao
$>> "abc".to_int();
// [ERROR] runtime error: cannot convert "abc" to Int

$>> "".to_int();
// [ERROR] runtime error: cannot convert "" to Int

$>> true.to_int();
// [ERROR] runtime error: type mismatch: Bool cannot convert to Int
```

**实现要点:**
1. Interpreter eval.rs 新增 `CallBuiltin` 处理分支
2. 方法分发：检测 `to_int` 方法名
3. String 参数：调用 `str::parse::<i64>()`，失败则报错
4. Float 参数：截断小数部分 `as i64`
5. Bool 参数：报错 `type mismatch`

---

#### 7.2 to_float() - String / Int → Float ✅ 已完成

**成功场景:**
```dao
$>> "3.14".to_float();   // → 3.14
$>> 42.to_float();       // → 42.0
```

**失败场景:**
```dao
$>> "abc".to_float();
// [ERROR] runtime error: cannot convert "abc" to Float

$>> "".to_float();
// [ERROR] runtime error: cannot convert "" to Float

$>> true.to_float();
// [ERROR] runtime error: type mismatch: Bool cannot convert to Float
```

**实现要点:**
1. 方法分发：检测 `to_float` 方法名
2. String 参数：调用 `str::parse::<f64>()`，失败则报错
3. Int 参数：转换为 `f64`
4. Bool 参数：报错 `type mismatch`

---

#### 7.3 to_str() - Int / Float / Bool → String ✅ 已完成

**成功场景:**
```dao
$>> 42.to_str();       // → "42"
$>> 3.14.to_str();     // → "3.14"
$>> true.to_str();     // → "true"
$>> false.to_str();    // → "false"
```

**实现要点:**
1. 方法分发：检测 `to_str` 方法名
2. Int 参数：调用 `to_string()`
3. Float 参数：调用 `to_string()`
4. Bool 参数：返回 "true" 或 "false"
5. 理论上所有类型都能转字符串，无需报错

---

#### 7.4 to_bool() - String / Int → Bool ✅ 已完成

**成功场景:**
```dao
$>> "true".to_bool();    // → true
$>> "false".to_bool();   // → false
$>> 1.to_bool();         // → true
$>> 0.to_bool();         // → false
```

**失败场景:**
```dao
$>> "abc".to_bool();
// [ERROR] runtime error: cannot convert "abc" to Bool
//                        expected "true" or "false"

$>> 2.to_bool();
// [ERROR] runtime error: cannot convert 2 to Bool
//                        expected 0 or 1

$>> 3.14.to_bool();
// [ERROR] runtime error: type mismatch: Float cannot convert to Bool
```

**实现要点:**
1. 方法分发：检测 `to_bool` 方法名
2. String 参数：匹配 "true" / "false"，否则报错 + 附加说明
3. Int 参数：仅允许 0 / 1，其他值报错 + 附加说明
4. Float 参数：报错 `type mismatch`

---

#### 7.5 type() - 查看当前类型 ✅ 已完成

**成功场景:**
```dao
$>> type(42);          // → "Int"
$>> type(3.14);        // → "Float"
$>> type("hello");     // → "String"
$>> type(true);         // → "Bool"
$>> type([1,2,3]);      // → "List"
$>> type({"a": 1});    // → "Map"
```

**失败场景:**
```dao
$>> type();
// [ERROR] runtime error: type() requires exactly 1 argument
```

**实现要点:**
1. 作为内置函数实现（非方法），通过函数名分发
2. 参数校验：必须恰好 1 个参数
3. 根据 Value 枚举变体返回对应类型名字符串
4. List/Map 类型也返回 "List" / "Map"

---

## P1 - 渐进式类型系统 ✅ 已完成

### 设计原则

> **能跑就不用加，怕出错再加。**

---

### 8.1 动态模式（默认）✅

变量声明时**不指定类型**，类型随赋值自动变化，无任何限制。

```dao
$ x = 30;
x = "hello";   // ✅ 允许，类型从 Int 变为 String
x = true;      // ✅ 允许，类型从 String 变为 Bool
x = 3.14;      // ✅ 允许，类型从 Bool 变为 Float
```

**适用场景**：快速验证想法、原型开发。

---

### 8.2 静态模式（显式声明）✅ 已完成

变量声明时**指定类型注解**，后续赋值必须保持类型一致。

```dao
$ x: Int = 30;
x = 100;       // ✅ 允许，同为 Int
x = "hello";   // ❌ 报错，类型不匹配
```

**适用场景**：涉及金额、ID、状态码等关键变量。

**实现要点:**
1. Parser: 解析 `:` 后面的类型标识符 ✅
2. AST: `VarDeclStmt` / `ConstDeclStmt` 新增 `type_annotation` 字段 ✅
3. Interpreter: 赋值时检查类型是否匹配，不匹配则报错 ✅

---

### 8.3 语法规范 ✅ 已完成

**类型注解语法**：
```dao
$ 变量名: 类型 = 值;
```

**支持的类型注解**：

| 类型注解 | 说明 |
|---|---|
| `Int` / `Integer` | 整数 |
| `Float` | 浮点数 |
| `String` / `Str` | 字符串 |
| `Bool` / `Boolean` | 布尔值 |

**常量类型注解**：
```dao
$@ MAX_RETRY: Int = 3;
$@ API_URL: String = "https://api.dolang.co";
```

---

### 8.4 错误信息规范 ✅ 已完成

**静态模式类型不匹配**：
```
[ERROR] type error: variable 'x' is declared as 'Int', cannot assign 'String' value
  hint: use '$ x: String = ...' to redeclare with a new type
```

**类型注解与初始值不匹配**：
```
[ERROR] type error: declared type 'Int' does not match value type 'String'
  hint: change the annotation or the value
```

**未知类型注解**：
```
[ERROR] type error: unknown type 'Xxx', supported types are: Int, Float, String, Bool
```

---

### 8.5 实现优先级

| 优先级 | 内容 | 状态 |
|---|---|---|
| P0 | 放开动态模式重赋值限制 | ✅ 已完成 |
| P0 | 解析类型注解语法 `$ x: Int = 30` | ✅ 已完成 |
| P1 | 静态模式类型检查 | ✅ 已完成 |
| P1 | 错误信息更新 | ✅ 已完成 |
| P2 | 常量类型注解支持 | ✅ 已完成 |

---

*DaoLang Team · Gradual Typing Spec v1.0 · 2026-03-01*



### 9. 标准输出、格式化输出、标准错误、标准输入 ✅ 已完成

#### 9.1 普通输出 ✅
- [x] 字符串输出，默认换行: `$>> "Hello";`
- [x] 变量输出: `$>> name;`
- [x] 表达式输出: `$>> 1 + 2;`
- [x] 拼接输出: `$>> "Hello" + name;`

---

#### 9.2 格式化输出 (f-string) ✅ 已完成

**语法:**

- [x] 基础 f-string: `$>> f"Hello {name}";`
- [x] 多变量: `$>> f"My name is {name}, I am {age} years old.";`
- [x] 表达式: `$>> f"1 + 1 = {1 + 1}";`
- [x] 方法调用: `$>> f"price: {price.to_str()}";`

**实现要点:**
1. Lexer 新增 token: `FString` ✅
2. Parser 新增 f-string 字面量解析: 识别 `f"..."` 语法 ✅
3. 解析花括号内的表达式为独立 AST 节点 ✅
4. Interpreter eval 时拼接字符串，支持变量/表达式/方法调用 ✅
5. 类型自动转换: Int/Float/Bool 自动调用 to_str() ✅

**测试结果:**
```dao
$ name = "John";
$ age = 18;
$ score = 99.5;

$>> f"姓名: {name}";     // 姓名: John
$>> f"年龄: {age}";       // 年龄: 18
$>> f"分数: {score}";     // 分数: 99.5
$>> f"1 + 1 = {1 + 1}";  // 1 + 1 = 2
$>> f"price: {score.to_str()}";  // price: 99.5
```

**错误规范 (已实现):**
- 花括号未闭合: ✅ `[ERROR] lexer error: f-string syntax error: unclosed "{"`
- 空花括号: ✅ `[ERROR] parse error: f-string syntax error: empty expression in "{}"`
- 花括号多余: ✅ `[ERROR] lexer error: f-string syntax error: unexpected "}"`

**待完善:**
- 变量未定义时报错信息优化 (目前显示 "invalid expression")
- 转义字符支持 (`\n`, `\t`, `\"` 等)

---

#### 9.3 stderr 输出 ✅ 已完成（部分）

**语法:**

- [x] `$>>ERR("something went wrong");` → 输出到 stderr
- [x] `$>>ERR(f"variable {name} is not defined");` → 支持 f-string

**实现要点:**
1. Lexer: `ERR` 识别为普通 Ident ✅
2. Parser: 解析到 `$>>` 后检查下一个 Token，大写标识符进入特殊模式 ✅
3. Interpreter: 识别 ERR 调用，输出到 stderr ✅

**测试结果:**
```dao
$>>ERR("something went wrong");   // 输出到 stderr
$ name = "John";
$>>ERR(f"variable {name} is not defined");  // stderr: variable John is not defined
```

**错误规范 (部分实现):**

- 语法错误: ✅ `[ERROR] parse error: ERR requires parentheses: $>>ERR("msg")`
- 参数缺失/类型检查: ⚠️ 需要完善（目前接受任意参数）

**待完善:**

- 运行时参数检查（参数数量、类型）

**上下文关键字 (已实现):**
- `ERR` → stderr 输出 ✅

  ---

  

## 9.4 stdin 输入 ✅ 已完成

### 设计说明

`$<<` 为输入方向关键符，数据从外部流入变量，与输出符 `$>>` 方向相反。

------

### 9.4.1 环境变量读取 `$<<ENV` ✅ 已完成

**语法**：

```dao
$ 变量名 = $<<ENV("KEY");
```

**示例**：

```dao
$ home = $<<ENV("HOME");
$>> home;
// /Users/john

$ port = $<<ENV("PORT");
$>> port;
// 8080
```

**错误规范**：

- 参数为空：`[ERROR] parse error: ENV requires a key: $<<ENV("KEY")`
- 无参数：`[ERROR] parse error: ENV requires a key: $<<ENV("KEY")`
- 环境变量不存在：`[ERROR] runtime error: environment variable 'KEY' is not defined`
- 参数类型非 String：`[ERROR] parse error: ENV key must be a String`

---



# 9.4 标准输入

## 9.4.2 读取 `$<<LINE` ✅ 已完成

**语法**：

```dao
$ 变量名 = $<<LINE();           // 无提示，等待用户输入
$ 变量名 = $<<LINE("提示文字"); // 有提示，显示后等待用户输入
$<<LINE();                      // 不赋值，pause 效果，等待用户按回车后继续
$<<LINE("提示文字");            // 不赋值带提示，等待用户按回车后继续
```

请注意: 

1. LINE() 也属于内置方法; 但是需要匹配LINE()内置方法的前面是否有 $<< 关键符;
2. 若用户使用 $<<LINE(); 并没有赋值, 同样是pause效果, 只不过读取后输入会被丢弃



**示例**：

```dao
// 无提示
$ input = $<<LINE();
$>> input;

// 有提示
$ name = $<<LINE("请输入你的名字: ");
$>> f"你好, {name}";
// 请输入你的名字: John
// 你好, John

// 结合类型转换
$ age = $<<LINE("请输入年龄: ");
age = age.to_int();
$>> f"明年你 {age + 1} 岁";

// 不赋值，pause 效果
$>> "程序执行完毕，按回车退出";
$<<LINE();
$>> "已退出";
```

**错误规范**：

- 参数超过一个：`[ERROR] parse error: LINE accepts at most one argument`
- 参数类型非 String：`[ERROR] parse error: LINE prompt must be a String`
- 读取 EOF（Ctrl+D）：`[ERROR] runtime error: unexpected EOF on stdin`

------

### 实现要点

1. Lexer：`ENV`、`LINE` 识别为上下文关键字，仅在 `$<<` 后生效
2. Parser：解析 `$<<` 后检查下一个 Token，进入输入模式
3. Interpreter：
   - `ENV` → 调用 `std::env::var()` 读取环境变量
   - `LINE` → 有参数时先输出提示（不换行），再读取 stdin 一行
   - `LINE` 不赋值 → 同样等待用户输入，输入后丢弃，继续执行
4. 返回值统一为 `String` 类型，用户按需调用 `to_int()` / `to_float()` 转换

------

### 上下文关键字汇总

| 关键字 | 语法                            | 功能            | 状态     |
| ------ | ------------------------------- | --------------- | -------- |
| `ENV`  | `$<<ENV("KEY")`                 | 读取环境变量    | ✅ 已完成 |
| `LINE` | `$<<LINE()` / `$<<LINE("提示")` | 读取 stdin 一行 | ✅ 已完成 |

------

### 测试结果

#### ENV 测试

```dao
// 1. 读取存在的环境变量
$ home = $<<ENV("HOME");
$>> home;
// 预期: /Users/xxxx（当前用户目录）

// 2. 读取不存在的环境变量
$ foo = $<<ENV("NOT_EXIST");
// 预期: [ERROR] runtime error: environment variable 'NOT_EXIST' is not defined

// 3. 空参数
$ a = $<<ENV("");
// 预期: [ERROR] parse error: ENV requires a key: $<<ENV("KEY")

// 4. 无参数
$ a = $<<ENV();
// 预期: [ERROR] parse error: ENV requires a key: $<<ENV("KEY")

// 5. f-string 结合输出
$ home = $<<ENV("HOME");
$>> f"当前用户目录: {home}";
// 预期: 当前用户目录: /Users/xxxx
```

#### LINE 测试

```dao
// 1. 无提示输入
$ input = $<<LINE();
$>> input;
// 预期: 用户输入什么输出什么

// 2. 有提示输入
$ name = $<<LINE("请输入你的名字: ");
$>> f"你好, {name}";
// 预期终端交互:
// 请输入你的名字: John
// 你好, John

// 3. 输入后类型转换（合法）
$ age = $<<LINE("请输入年龄: ");
age = age.to_int();
$>> f"明年你 {age + 1} 岁";
// 预期终端交互:
// 请输入年龄: 18
// 明年你 19 岁

// 4. 输入后类型转换（非法）
$ age = $<<LINE("请输入年龄: ");
age = age.to_int();
// 预期终端交互:
// 请输入年龄: hello
// [ERROR] runtime error: cannot convert "hello" to Int

// 5. 不赋值，pause 效果
$>> "程序执行完毕，按回车退出";
$<<LINE();
$>> "已退出";
// 预期终端交互:
// 程序执行完毕，按回车退出
// （等待用户按回车）
// 已退出

// 6. 不赋值带提示
$<<LINE("按回车继续...");
$>> "继续执行";
// 预期终端交互:
// 按回车继续...
// （等待用户按回车）
// 继续执行

// 7. 参数超过一个
$ a = $<<LINE("提示1", "提示2");
// 预期: [ERROR] parse error: LINE accepts at most one argument

// 8. EOF 输入（Ctrl+D）
$ a = $<<LINE();
// 预期: [ERROR] runtime error: unexpected EOF on stdin
```

---

#### 8.4 转义字符 📋 待开始

**支持的转义序列:**
- `\n` 换行
- `\t` 制表符
- `\\` 反斜杠
- `\"` 双引号
- `\r` 回车
- `\0` 空字符

**使用场景:**
```dao
$>> "Hello\nWorld";         →  Hello
                               World
$>> "Hello\tWorld";         →  Hello   World
$>> "He said \"Hi\"";       →  He said "Hi"
$>> "C:\\path\\file";       →  C:\path\file
$>> f"Hello\n{name}";      →  f-string 同样支持转义
```

**错误规范:**
- 无效转义: `[ERROR] parse error: invalid escape sequence "\q"`
- 单引号转义: `[ERROR] parse error: Char type does not support escape sequences`

---

#### 8.5 完整使用示例

```dao
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

**输出结果:**
```
=== 用户信息 ===
姓名: John
年龄: 18
分数: 99.5
及格: true
```

**stderr:**
```
以上为测试输出
```

---

#### 实现优先级

```
P0   $>> 普通输出        →  ✅ 已完成
P0   $>> f"" 格式化      →  ✅ 已完成
P0   $>>ERR()            →  ✅ 已完成
P1   转义字符支持        →  📋 待实现
P1   ERR 参数检查        →  📋 待完善
```

---

*Dolang Team · Standard Output Spec Plan 2*

---

## 附录: 

---

## 贡献指南

欢迎提交 Issue 和 PR！

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/xxx`)
3. 提交更改 (`git commit -m 'Add xxx'`)
4. 推送分支 (`git push origin feature/xxx`)
5. 创建 Pull Request
