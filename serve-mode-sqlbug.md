# Serve Mode SQL Bug

## 结论

当前 Dolang 的 `std.postgres` 在 `run` 模式下可用，但在 `serve` 模式的 HTTP handler 中调用时会触发运行时 panic，因此无法支撑真实的 HTTP + PostgreSQL 项目。

这不是 `test-http-database` 项目代码本身的业务逻辑错误，而是当前解释器 / runtime 在 HTTP 场景下集成 PostgreSQL 的底层问题。

## 现象

在 `test-http-database` 中，请求：

```bash
curl http://127.0.0.1:8082/api/users
```

返回：

```json
{"error":"error[DOL-R001]: uncaught throw: database unavailable"}
```

继续做最小化复现后，确认 HTTP 请求实际会触发底层 panic：

```text
Cannot start a runtime from within a runtime
```

## 已确认的事实

### 1. `run` 模式下 PostgreSQL 可用

使用最小脚本：

```dol
$mod std.postgres;

$ conn = postgres.connect("host=/tmp dbname=postgres user=liangzhanbo");
$ rows = conn.query("SELECT id, name, email, created_at::text AS created_at FROM users ORDER BY id", []);
$>> rows;
$>> conn.close();
```

通过：

```bash
cargo run -p dolang-cli -- run /tmp/test_postgres_run.dol
```

可以正常返回 `users` 表数据。

### 2. `serve` 模式下 HTTP handler 中调用 PostgreSQL 会失败

最小 HTTP 复现项目：

```dol
$mod std.postgres;

$GET("/users") users() -> List<Map> {
    $ conn = postgres.connect("host=/tmp dbname=postgres user=liangzhanbo");
    $ rows = conn.query("SELECT id, name, email, created_at::text AS created_at FROM users ORDER BY id", []);
    $>> conn.close();
    $# rows;
}
```

启动：

```bash
cargo run -p dolang-cli -- serve /tmp/http-pg-debug
```

请求：

```bash
curl http://127.0.0.1:8090/users
```

服务端报错：

```text
thread 'tokio-rt-worker' panicked at ... postgres ... connection.rs
Cannot start a runtime from within a runtime.
```

### 3. `test-http-database` 的 users 路由已经挂载成功

当前项目入口 [main.dol](/Users/liangzhanbo/CodeStudio/dolang/test-http-database/main.dol) 已经能注册：

```text
GET /api/users -> get_users
```

说明：

- 项目结构本身能被 `serve` 加载
- `.link()` 路由挂载正常
- `users` 域模块解析正常
- 失败点不在路由注册层，而在请求执行到 PostgreSQL 时

## 根因判断

目前证据表明：

1. SQL 语句本身没问题
2. PostgreSQL 连接串本身没问题
3. `std.postgres` 在非 HTTP 的 `run` 模式下能工作
4. 问题只在 `serve` 模式的 HTTP runtime / Tokio runtime 中出现

因此根因应是：

- 当前 `std.postgres` 的实现方式与 `serve` 模式下的异步 runtime 不兼容
- 在 HTTP handler 执行数据库连接或查询时，触发了嵌套 runtime / blocking runtime 问题

## 当前影响

影响范围是明确的：

- 可以继续写纯 Dolang 项目结构、DDD 分层、router/service/data 代码
- 可以在 `run` 模式下验证 SQL 查询逻辑
- 不能在当前实现下把真实 PostgreSQL 查询放进 `serve` 模式的 HTTP handler 中运行

所以在“不修改解释器源码”的前提下：

- `test-http-database` 可以继续作为代码骨架存在
- 但无法成为真正可运行的 HTTP + PostgreSQL 验证项目

## 对当前任务的含义

这次阻断不是项目层 bug，而是解释器能力边界：

- `std.postgres` 已经“部分可用”
- 但还没有达到“可在 HTTP 服务模式中稳定使用”的程度

如果要真正跑通 `test-http-database`，需要修复解释器 / runtime 中 `serve` 模式与 PostgreSQL 的集成问题。
