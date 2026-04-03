# Development Report

## 背景

本次工作目标是在仓库根目录创建纯 Dolang 项目 `test-http-database`，使用本地 PostgreSQL `postgres` 数据库，验证以下真实场景：

- 单表查询
- 一对多查询
- 多表聚合查询
- `main.dol` 入口装配
- `.link()` 路由组织
- 内联 `$HTTP { ... }` 路由组织

项目开发过程中，不只落了业务代码，也暴露并修复了若干个 Dolang `serve` 模式下的底层问题。

---

## 1. `serve` 模式下 `std.postgres` 会触发 runtime panic

### 现象

`run` 模式下，下面这种脚本是可用的：

```dol
$mod std.postgres;

$ conn = postgres.connect("host=/tmp dbname=postgres user=liangzhanbo");
$ rows = conn.query("SELECT 1 AS n", []);
$>> rows;
$>> conn.close();
```

但同样的逻辑一旦放进 `serve` 模式的 HTTP handler 中，请求会触发 panic：

```text
Cannot start a runtime from within a runtime
```

### 根因

同步 `postgres::Client` 在 HTTP/Tokio worker 线程里 drop 时，会走内部 runtime 的 `block_on`，从而触发嵌套 runtime panic。

### 修复

增加了 `PostgresClientHandle`，把真实 `postgres::Client` 的 drop 移到普通线程执行，避免在 Axum/Tokio worker 上直接析构。

涉及文件：

- [sql_registry.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/sql_registry.rs)
- [sql_intrinsics.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/sql_intrinsics.rs)

### 验证

新增并通过了 `serve` 场景回归测试：

- `serve_mode_postgres_route_can_query_and_close_connection_when_url_is_present`

---

## 2. `.link()` 路由请求执行时，导入的 `$Type` 不会传播

### 现象

linked router 可以成功注册路由，但请求真正执行到返回 `User` 之类的 typed object 时，会报：

```text
type 'User' is not defined
```

### 根因

`.link()` 加载模块时，会在隔离的 `module_context` 里执行模块文件并注册 `$Type`。  
但请求执行时 clone 的是父 `RuntimeContext`，不是当时的 `module_context`，所以用户类型只存在于隔离上下文，没有合并回主上下文。

### 修复

在 linked module route load 完成后，把新注册的类型从模块上下文合并回父 `RuntimeContext`。

涉及文件：

- [http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/exec/http.rs)
- [context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs)

### 验证

新增并通过了 linked-route 类型传播回归测试：

- `serve_mode_linked_route_can_return_imported_user_type`

---

## 3. 模块函数内部再调用同模块其他函数时，会丢失模块环境

### 现象

像 `order_service.get_order_detail(id)` 这样的函数，在内部继续调用：

- `get_order_by_id(id)`
- `list_order_items(id)`

这类同模块裸函数调用时，可能报出与模块环境相关的错误，典型表现是：

- 看不到 `order_queries`
- 看不到模块级 `$mod` 导入
- 业务层被 catch 后只看到统一错误 `database unavailable`

### 根因

通过 `module_proxy.some_fn()` 调函数时，`call_module_fn()` 会把模块环境注入执行上下文。  
但函数体内部继续裸调同模块其他函数时，走的是普通 `call_fn()`，它没有自动带上定义该函数时的模块环境。

### 修复

给 `RuntimeFn` 增加 `module_env`，在函数声明时捕获当时可见的模块环境；之后无论是顶层调用还是模块内部互调，都先把这份环境注入局部执行态。

涉及文件：

- [env.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/env.rs)
- [functions.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/exec/functions.rs)
- [literals.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/eval/literals.rs)

---

## 4. PostgreSQL `numeric` 列当前 runtime 不支持直接映射

### 现象

查询 `products.price` 或 `order_items.price` 时，底层报错：

```text
sql.query: postgres column 'price' has unsupported type 'numeric'
```

### 根因

当前 Dolang Postgres runtime 只支持一部分 PostgreSQL 标量类型，没有直接支持 `numeric` 到 Dolang `Float` 的自动映射。

### 处理方式

不修改解释器语义，直接在项目 SQL 层显式 cast：

```sql
price::double precision AS price
```

涉及文件：

- [product_queries.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/domains/products/data/product_queries.dol)
- [order_queries.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/domains/orders/data/order_queries.dol)

---

## 5. PostgreSQL 参数类型与 `INT4` 列比较时会出现序列化失败

### 现象

带参数查询订单时，底层报：

```text
postgres query failed: error serializing parameter 0
```

### 根因

Dolang `Int` 目前走 `i64`，而 `orders.user_id`、`orders.id`、`order_items.order_id`、`users.id` 这类列是 PostgreSQL `INT4`。  
在当前 runtime 路径里，直接比较会命中参数序列化兼容问题。

### 处理方式

项目 SQL 层直接把列侧 cast 到 `bigint`：

```sql
WHERE user_id::bigint = $1
```

涉及文件：

- [order_queries.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/domains/orders/data/order_queries.dol)

---

## 6. `$Type` 结构表达有当前语法边界

### 现象

尝试定义这种结构时：

```dol
$Type OrderDetail {
    order: Order
    user: User
    items: List<OrderItem>
}
```

解析失败。

### 判断

当前 Dolang 虽然支持 `$Type` 和 `List<T>` 返回类型，但对 `$Type` 字段里嵌套自定义类型 / `List<T>` 的结构表达还不适合作为稳定主线依赖。

### 处理方式

`GET /api/orders/:id/detail` 改为返回 `Map`，内部值仍然可以包含：

- `Order`
- `User`
- `List<OrderItem>`

这保持了真实聚合接口能力，同时不硬顶当前 `$Type` 语法边界。

涉及文件：

- [order_types.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/domains/orders/data/order_types.dol)
- [order_service.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/domains/orders/services/order_service.dol)
- [main.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/main.dol)

---

## 7. 本地已安装 `dolang` 二进制是旧的

### 现象

直接跑：

```bash
dolang serve test-http-database
```

会出现类似：

```text
module not found: 'std.postgres'
```

### 根因

机器上安装的 `dolang` CLI 不是当前仓库代码构建出的版本，因此看不到新加的数据库标准库能力。

### 处理方式

整个开发和验证过程统一改为使用仓库内当前代码构建的 CLI：

```bash
cargo run -p dolang-cli -- serve test-http-database
```

---

## 最终结果

在修完以上问题后，`test-http-database` 已经可以在真实 `serve` 模式下工作，并通过了以下接口验证：

- `GET /api/users`
- `GET /api/categories`
- `GET /api/products`
- `GET /api/users/1/orders`
- `GET /api/orders/1`
- `GET /api/orders/1/items`
- `GET /api/orders/1/detail`
- `GET /api/orders/999`

同时两条关键回归测试已通过：

- `serve_mode_postgres_route_can_query_and_close_connection_when_url_is_present`
- `serve_mode_linked_route_can_return_imported_user_type`
