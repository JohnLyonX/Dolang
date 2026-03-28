# DOC-001 更新类型语法文档

## 背景

两项破坏性语法变更已落地，需要同步更新文档：

1. **`JSON<T>` 已删除** → 改用 `List<T>`（支持泛型元素类型校验）
2. **`_fieldname` 隐藏字段语法已删除** → 改用 `@HIDE fieldname` 注解

---

## 变更一：`List<T>` 替代 `JSON<T>`

### 旧语法（已不可用）
```dolang
$fn getUsers() -> JSON<User> { ... }
$GET("/users") list() -> JSON<User> { ... }
```

### 新语法
```dolang
$fn find_all_users() -> List<User> { ... }
$GET("/users") getUsers() -> List<User> { ... }
```

### 需要更新的文件

| 文件 | 行号 | 问题描述 |
|---|---|---|
| `docs/reference/types.md` | L631 | 说明"不支持 JSON<User> 泛型"→ 改为说明 List<T> 是支持的泛型写法 |
| `docs/reference/gradual-typing.md` | L79 | 同上 |
| `docs/guide/types.md` | L123 | 同上 |
| `docs/spec/syntax.md` | L49 | 同上 |
| `docs/guide/09-gradual-typing.md` | L113 | 同上 |

---

## 变更二：`@HIDE` 注解替代 `_` 前缀

### 旧语法（已不可用）
```dolang
$Type User {
    id: Int
    name: String
    _password: String   // ❌ 不再支持
    _salt: String       // ❌ 不再支持
}

$ user = User {
    _password: "hash",  // ❌ 不再支持
    _salt: "salt",
};

$>> user._password;     // ❌ 旧访问方式
```

### 新语法
```dolang
$Type User {
    id: Int
    name: String
    @HIDE password: String  // ✓ 注解标记隐藏字段
    @HIDE salt: String
}

$ user = User {
    password: "hash",   // ✓ 构造时使用干净名称
    salt: "salt",
};

$>> user.password;      // ✓ 访问时使用干净名称
// HTTP JSON 响应中 password / salt 字段自动被过滤
```

### 行为变化

| 场景 | 旧行为 | 新行为 |
|---|---|---|
| `$Type` 字段声明 | `_fieldname: Type` | `@HIDE fieldname: Type` |
| 构造实例 | `_fieldname: value` | `fieldname: value` |
| 代码内访问 | `obj._fieldname` | `obj.fieldname` |
| HTTP JSON 输出 | 自动过滤 `_` 前缀字段 | 自动过滤 `@HIDE` 注解字段 |
| `$>>` 打印 | 显示 `_fieldname: value` | 显示 `fieldname: value` |
| 解析器行为 | 允许 `_fieldname` | **报错**，提示使用 `@HIDE` |

### 需要更新的文件

| 文件 | 行号 | 问题描述 |
|---|---|---|
| `docs/reference/types.md` | L19, L28, L36, L584, L600, L622, L658, L669 | `_` 前缀定义示例 + 规则说明 → 改用 `@HIDE` |
| `docs/reference/gradual-typing.md` | L49, L63 | `_password: String` 示例 + `_` 前缀规则 |
| `docs/reference/http.md` | L57, L65, L66 | `_password: "hash"` 构造示例 + 序列化说明 |
| `docs/reference/functions.md` | L86 | `_` 前缀过滤描述 |
| `docs/guide/types.md` | L76, L85, L94 | `_password: String` 示例 + `_` 前缀规则 |
| `docs/guide/09-gradual-typing.md` | L76, L91 | `_password: String` 示例 + `_` 前缀规则 |
| `docs/guide/15-http-basics.md` | L83 | `_password: "hash"` 构造示例 |
| `docs/spec/semantics.md` | L44 | `_` 前缀字段 JSON 隐藏规则 |
| `docs/CHANGELOG.md` | L24 | `_` 前缀字段功能描述 |

---

## 文档更新要点

更新以上文件时，统一遵循以下措辞：

**隐藏字段规则（新）：**
> 在 `$Type` 定义中，使用 `@HIDE` 注解标记需要隐藏的字段。被标记的字段在 HTTP 响应 JSON 中自动过滤，但在 Dolang 代码内部可通过字段名正常读写。`@HIDE` 是 HTTP 序列化层的关注点，与 `@CORS`、`@SET_HDR` 属于同一类注解。

**List<T> 规则（新）：**
> 函数或 HTTP handler 若返回多个相同类型的对象，使用 `-> List<T>` 标注返回类型。运行时会逐元素校验列表内容类型是否匹配。目前只支持 `List<T>` 一种泛型形式，其他泛型写法（如 `Map<T>`）会在解析阶段报错。

---

## 优先级

`docs/reference/` 优先（参考文档最常被查阅），其次 `docs/guide/`，最后 `docs/spec/`。
