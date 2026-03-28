# SQL 标准库设计方案

## 背景

dolang 解释器目前已有 `std.http`（reqwest blocking）、`std.fs`、`std.time` 等原生模块，但尚无数据库支持。本文档梳理实现 SQL 标准库的三个备选方案，并记录选定方案的完整设计。

**关键结论：不需要先实现底层 TCP 层。** `rusqlite`（SQLite）是纯同步嵌入式，`tokio-postgres` 可用 `block_on` 包装成阻塞调用，与 `reqwest::blocking` 的实现模式完全一致。

---

## 三个备选方案

### 方案一：rusqlite（仅 SQLite）

用 `rusqlite` crate 实现 `std.sqlite` 模块。SQLite 嵌入式，无网络依赖，纯同步。

**优点：** 实现最简，无需 tokio，与现有阻塞架构完美契合。
**缺点：** 只支持 SQLite，后期支持 PostgreSQL 需重新设计。

---

### 方案二：sqlx + block_on 包装（统一模块）

`sqlx` 是 async 库，用 `tokio::runtime::Runtime::block_on()` 包装成阻塞调用。统一的 `std.sql` 模块，连接字符串前缀区分驱动（`sqlite://` / `postgres://`）。

**优点：** API 统一，扩展性好，与 `std.http` 实现模式一致。
**缺点：** 抽象层略重，需要 tokio 进入 runtime 依赖。

---

### 方案三：分驱动独立模块（选定方案）

SQLite 和 PostgreSQL 各自独立模块，分别用 `rusqlite` 和 `tokio-postgres` 实现，共享相同的 API 约定。

**优点：** 每个驱动独立优化，实现清晰，各自使用原生占位符风格。
**缺点：** 用户需要区分两个模块名，存在少量 API 重复。

---

## 选定方案设计（方案三）

### 架构总览

```
crates/dolang-runtime/src/stdlib_native/
├── sqlite.rs        ← rusqlite，同步，嵌入式
├── postgres.rs      ← tokio-postgres + block_on 包装
└── mod.rs           ← 注册 std.sqlite 和 std.postgres
```

**连接对象的表示：** dolang 没有指针/句柄类型。连接在 Rust 侧由 `RuntimeContext` 内的连接注册表（`SqlConnRegistry`）管理。`connect()` 返回一个连接 ID 字符串（如 `"conn:sqlite:0"`），后续所有操作通过该 ID 查找连接。从用户角度看，`conn` 是一个普通字符串 handle。

---

### API 设计

#### `std.sqlite`

| 函数 | 参数 | 返回值 |
|------|------|--------|
| `sqlite.connect(path)` | 文件路径字符串 | 连接 handle（`Str`） |
| `conn.query(sql, params)` | SQL 字符串，参数列表 | `List<Map>`，每行一个 Map |
| `conn.execute(sql, params)` | SQL 字符串，参数列表 | `Int`（影响行数） |
| `conn.close()` | — | `Null` |

占位符风格：`?`（rusqlite 原生）

#### `std.postgres`

| 函数 | 参数 | 返回值 |
|------|------|--------|
| `postgres.connect(url)` | 连接字符串 | 连接 handle（`Str`） |
| `conn.query(sql, params)` | SQL 字符串，参数列表 | `List<Map>`，每行一个 Map |
| `conn.execute(sql, params)` | SQL 字符串，参数列表 | `Int`（影响行数） |
| `conn.close()` | — | `Null` |

占位符风格：`$1, $2, ...`（postgres 原生）

#### 使用示例

```
$mod std.sqlite;
$ conn = sqlite.connect("./app.db");
$ rows = conn.query("SELECT * FROM users WHERE id = ?", [1]);
$ n    = conn.execute("INSERT INTO users (name, email) VALUES (?, ?)", ["Alice", "alice@example.com"]);
conn.close();

$mod std.postgres;
$ pg   = postgres.connect("postgres://user:pass@localhost/mydb");
$ rows = pg.query("SELECT * FROM users WHERE id = $1", [1]);
$ n    = pg.execute("UPDATE users SET name = $1 WHERE id = $2", ["Bob", 1]);
pg.close();
```

---

### Rust 实现细节

#### 新增依赖（`crates/dolang-runtime/Cargo.toml`）

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
tokio-postgres = "0.7"
tokio = { version = "1", features = ["rt"] }
```

> `bundled` feature 静态链接 SQLite，无需系统安装。tokio 只用 `rt`（单线程 runtime）。

#### 连接注册表

在 `RuntimeContext` 中新增 `SqlConnRegistry`：

```rust
pub struct SqlConnRegistry {
    sqlite: HashMap<String, rusqlite::Connection>,
    postgres: HashMap<String, tokio_postgres::Client>,
    counter: usize,
}
```

- `connect()` → 分配唯一 ID，存入注册表，返回 ID 字符串
- `query()` / `execute()` → 用 ID 取出连接执行
- `close()` → 从注册表移除，连接自动 Drop

#### PostgreSQL 阻塞包装

```rust
fn block_on<F: Future>(f: F) -> F::Output {
    tokio::runtime::Runtime::new().unwrap().block_on(f)
}
```

与 `reqwest::blocking` 实现模式一致。

#### DolangValue 类型映射

| SQL 类型 | DolangValue |
|----------|-------------|
| INTEGER / BIGINT | `Int(i64)` |
| REAL / FLOAT | `Float(f64)` |
| TEXT / VARCHAR | `Str(String)` |
| BOOLEAN | `Bool(bool)` |
| NULL | `Null` |
| 其他（JSON、DATE 等） | `Str(String)`（序列化为字符串） |

#### 错误处理

连接失败、SQL 语法错误、参数类型不匹配均通过现有 `Error` 类型抛出，在 dolang 里用 `$try` / `$catch` 捕获。

---

### 实现步骤

1. 在 `RuntimeContext` 中添加 `SqlConnRegistry` 及其方法
2. 实现 `stdlib_native/sqlite.rs`，注册 `std.sqlite`
3. 实现 `stdlib_native/postgres.rs`，注册 `std.postgres`
4. 在 `stdlib_native/mod.rs` 注册两个模块
5. 编写集成测试（SQLite 用临时文件，PostgreSQL 用 Docker 或 CI 环境变量跳过）
