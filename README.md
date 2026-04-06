# Dolang

[English](./README.md) · [中文](./README-CN.md)

**Dolang** is a lightweight interpreted scripting language designed to turn ideas into runnable HTTP services and tools with minimal boilerplate.

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
