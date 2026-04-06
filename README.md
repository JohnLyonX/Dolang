# Dolang

**Dolang** is a lightweight interpreted scripting language designed to turn ideas into runnable HTTP services and tools with minimal boilerplate.

[English](#english) · [中文](#中文)

---

## English

### What is Dolang?

Dolang is an interpreted language with a clean, expressive syntax built for rapid HTTP service development. Write a route handler, run `dolang serve`, and your service is live — no framework configuration, no dependency wiring.

```dolang
$GET("/hello") greet(name) -> JSON {
    $# { "message": "Hello, " + name + "!" };
}
```

### Features

- **HTTP services** — declare routes with `$GET`, `$POST`, etc.; serve instantly with `dolang serve`
- **Gradual typing** — optional type annotations; untyped code just works
- **gRPC client** — descriptor-driven unary calls via `std.grpc`
- **Outbound HTTP** — call external APIs via `std.http`
- **File uploads** — multipart form handling with configurable size limits
- **SQL** — connect to PostgreSQL and run queries with `std.sql.postgres`
- **JWT auth** — built-in `std.auth.guard` for token validation and principal extraction
- **Module system** — organize code with `$mod` imports and project-level `package.toml`
- **HTML rendering** — serve HTML pages with `$HTML()` and `-> HTML` return type
- **LSP support** — language server for IDE integration

### Quick Start

**Prerequisites:** Rust toolchain (stable)

```bash
git clone https://github.com/your-org/dolang
cd dolang
cargo build --release
```

Run a script:

```bash
cargo run -- run main.dol
```

Start an HTTP service:

```bash
cargo run -- serve
cargo run -- serve --routertab    # print route table on startup
```

Run tests:

```bash
cargo run -- test
```

### Hello Service

Create `main.dol`:

```dolang
$main() {
    $HTTP("/").link("app.router.hello_router");
}
```

Create `app/router/hello_router.dol`:

```dolang
$GET("/") index() -> JSON {
    $# { "status": "ok", "message": "Welcome to Dolang!" };
}

$GET("/hello/{name}") greet(name) -> JSON {
    $# { "greeting": "Hello, " + name + "!" };
}
```

Start the server:

```bash
dolang serve
# Listening on http://0.0.0.0:8080
```

### Project Structure (`package.toml`)

```toml
[project]
name = "my-service"
version = "0.1.0"
entry = "main.dol"

[server]
host = "0.0.0.0"
port = 8080
upload_max_size = 50        # total request body limit (MB), default 50
upload_max_file_size = 10   # per-file limit (MB), default 10

[server.auth]
enabled = true
jwt_secret = "your-secret"
```

### Workspace Layout

| Path | Description |
|------|-------------|
| `crates/dolang-frontend/` | AST, lexer, parser, diagnostics |
| `crates/dolang-runtime/` | Interpreter, module resolver, project loader |
| `crates/dolang-cli/` | CLI entry, REPL, serve, test runner |
| `crates/dolang-lsp/` | Language server protocol implementation |
| `stdlib/` | Standard library namespace scaffold |
| `sample/` | Runnable sample projects |
| `docs/` | Guides, references, contributor docs |
| `tests/` | Integration tests and spec fixtures |

### Sample Projects

| Sample | Description |
|--------|-------------|
| [`sample/upload-demo`](sample/upload-demo/) | Multipart file upload with size limits |
| [`sample/DataNest`](sample/DataNest/) | Full-stack notes app: Dolang gateway + Go gRPC backend |
| [`sample/auth-b2b-portal`](sample/auth-b2b-portal/) | JWT-protected B2B API gateway |
| [`sample/test-http-database`](sample/test-http-database/) | HTTP + PostgreSQL integration |

### Documentation

- **Get started:** [docs/guide/README.md](docs/guide/README.md)
- **Syntax reference:** [docs/reference/syntax.md](docs/reference/syntax.md)
- **HTTP reference:** [docs/reference/http.md](docs/reference/http.md)
- **gRPC client:** [docs/reference/grpc.md](docs/reference/grpc.md)
- **Project system:** [docs/reference/project-system.md](docs/reference/project-system.md)
- **Standard library:** [docs/reference/stdlib-api.md](docs/reference/stdlib-api.md)
- **Error codes:** [docs/reference/errors.md](docs/reference/errors.md)
- **Contributor guide:** [docs/contributing/dev-guide.md](docs/contributing/dev-guide.md)

### Development

Before submitting a change:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

### License

MIT

---

## 中文

### Dolang 是什么？

Dolang 是一门轻量级解释型脚本语言，专为快速构建 HTTP 服务和工具而设计。语法简洁，声明一个路由处理函数，执行 `dolang serve`，服务即刻运行——无需框架配置，无需依赖注入。

```dolang
$GET("/hello") greet(name) -> JSON {
    $# { "message": "你好，" + name + "！" };
}
```

### 核心特性

- **HTTP 服务** — 使用 `$GET`、`$POST` 等声明路由，`dolang serve` 一键启动
- **渐进式类型** — 类型注解可选，不写也能正常运行
- **gRPC 客户端** — 基于描述符文件的一元调用，通过 `std.grpc` 使用
- **出站 HTTP** — 通过 `std.http` 调用外部 API
- **文件上传** — 支持 multipart 表单，上传大小限制可在配置文件中指定
- **SQL** — 通过 `std.sql.postgres` 连接 PostgreSQL 并执行查询
- **JWT 鉴权** — 内置 `std.auth.guard`，支持 Token 验证和主体信息提取
- **模块系统** — 通过 `$mod` 导入，配合 `package.toml` 组织项目结构
- **HTML 渲染** — 使用 `$HTML()` 构造器和 `-> HTML` 返回类型提供页面服务
- **LSP 支持** — 语言服务器协议，支持 IDE 集成

### 快速开始

**前提条件：** Rust 工具链（stable）

```bash
git clone https://github.com/your-org/dolang
cd dolang
cargo build --release
```

运行脚本：

```bash
cargo run -- run main.dol
```

启动 HTTP 服务：

```bash
cargo run -- serve
cargo run -- serve --routertab    # 启动时打印路由表
```

运行测试：

```bash
cargo run -- test
```

### Hello 服务示例

创建 `main.dol`：

```dolang
$main() {
    $HTTP("/").link("app.router.hello_router");
}
```

创建 `app/router/hello_router.dol`：

```dolang
$GET("/") index() -> JSON {
    $# { "status": "ok", "message": "欢迎使用 Dolang！" };
}

$GET("/hello/{name}") greet(name) -> JSON {
    $# { "greeting": "你好，" + name + "！" };
}
```

启动服务：

```bash
dolang serve
# Listening on http://0.0.0.0:8080
```

### 项目配置（`package.toml`）

```toml
[project]
name = "my-service"
version = "0.1.0"
entry = "main.dol"

[server]
host = "0.0.0.0"
port = 8080
upload_max_size = 50        # 请求体总大小上限（MB），默认 50
upload_max_file_size = 10   # 单个文件大小上限（MB），默认 10

[server.auth]
enabled = true
jwt_secret = "your-secret"
```

### 工作区结构

| 路径 | 说明 |
|------|------|
| `crates/dolang-frontend/` | AST、词法分析、语法解析、诊断信息 |
| `crates/dolang-runtime/` | 解释器、模块解析、项目加载 |
| `crates/dolang-cli/` | CLI 入口、REPL、serve、测试运行器 |
| `crates/dolang-lsp/` | 语言服务器协议实现 |
| `stdlib/` | 标准库命名空间脚手架 |
| `sample/` | 可运行的示例项目 |
| `docs/` | 教程指南、参考文档、贡献者文档 |
| `tests/` | 集成测试与规范验证 |

### 示例项目

| 示例 | 说明 |
|------|------|
| [`sample/upload-demo`](sample/upload-demo/) | Multipart 文件上传，支持大小限制配置 |
| [`sample/DataNest`](sample/DataNest/) | 全栈笔记应用：Dolang 网关 + Go gRPC 后端 |
| [`sample/auth-b2b-portal`](sample/auth-b2b-portal/) | JWT 保护的 B2B API 网关 |
| [`sample/test-http-database`](sample/test-http-database/) | HTTP + PostgreSQL 集成示例 |

### 文档入口

- **入门教程：** [docs/guide/README.md](docs/guide/README.md)
- **语法参考：** [docs/reference/syntax.md](docs/reference/syntax.md)
- **HTTP 参考：** [docs/reference/http.md](docs/reference/http.md)
- **gRPC 客户端：** [docs/reference/grpc.md](docs/reference/grpc.md)
- **项目系统：** [docs/reference/project-system.md](docs/reference/project-system.md)
- **标准库 API：** [docs/reference/stdlib-api.md](docs/reference/stdlib-api.md)
- **错误代码：** [docs/reference/errors.md](docs/reference/errors.md)
- **贡献者指南：** [docs/contributing/dev-guide.md](docs/contributing/dev-guide.md)

### 开发规范

提交变更前，请运行：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

### 许可证

MIT
