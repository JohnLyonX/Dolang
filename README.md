# Dolang

[English](./README.md) · [中文](./README-CN.md)

> A scripting language built for HTTP services.
> One file, one `serve` command, a production-shaped API —
> in under **10 MB of resident memory**.

```dolang
$GET("/greet/{name}") greet(name) -> JSON {
    $# { "message": "Hello, " + name + "!" };
}
```

---

## Why Dolang

Dolang is an interpreted language with a first-class HTTP syntax. Declare a
route, run `dolang serve`, and your service is live — no framework wiring, no
server boilerplate, no dependency injection container.

The design optimises for a narrow, high-value slice of backend work:
**small- to medium-sized API gateways, edge functions, sidecars, and internal
tools** where a team wants the feel of FastAPI with the footprint of Rust.

What you get out of the box:

- **First-class HTTP** — declare routes with `$GET`, `$POST`, `$PUT`, `$DELETE`, `$PATCH`; path params, query strings, and JSON bodies are injected into handler parameters automatically.
- **Gradual typing** — annotate when it helps, skip when it doesn't. Types are enforced at handler boundaries and on declared returns.
- **Batteries-included stdlib** — outbound HTTP (`std.http`), PostgreSQL (`std.sql.postgres`), JWT guard (`std.auth.guard`), gRPC client (`std.grpc`), filesystem (`std.fs`).
- **Multipart uploads** — size limits configurable at the project level.
- **HTML rendering** — `$HTML()` and `-> HTML` for server-rendered pages.
- **Module system** — `$mod` imports, project-level `package.toml`, no central registry lock-in.
- **LSP** — day-one IDE integration through the Language Server Protocol.
- **Rust runtime, no GC** — interpreted, but built on the Rust toolchain and allocator. No stop-the-world pauses.

---

## Performance

All three services implement identical `/hello` and `/greet/:name` endpoints
with the same JSON payload. Measured on the same machine with the same client
harness. The full suite lives in [`benches/http-comparison/`](benches/http-comparison/)
and can be reproduced in one command.

### Resident Memory and Cold Start

| Target | Resident memory (RSS) | Cold start |
|:-------|----------------------:|-----------:|
| **Dolang** | **9.8 MB** | 1011 ms |
| FastAPI + uvicorn | 43.3 MB | 3108 ms |
| Hono + Node.js | 63.0 MB | **76 ms** |

### Throughput and Latency — `/hello`

| Target | QPS | p50 | p99 |
|:-------|----:|----:|----:|
| **Dolang** | **42,431** | **1.47 ms** | **1.84 ms** |
| FastAPI | 6,814 | 9.04 ms | 16.57 ms |
| Hono | 39,025 | 1.61 ms | 1.96 ms |

### Throughput and Latency — `/greet/:name`

| Target | QPS | p50 | p99 |
|:-------|----:|----:|----:|
| **Dolang** | **40,997** | **1.51 ms** | 2.08 ms |
| FastAPI | 6,167 | 10.02 ms | 17.41 ms |
| Hono | 38,662 | 1.62 ms | **2.01 ms** |

### What these numbers mean — honestly

- **Dolang's story is memory, not raw speed.** At 9.8 MB resident, one box fits
  roughly **4× more Dolang services than FastAPI**, **6× more than Hono**.
  For edge, sidecar, and multi-tenant deployments this is the differentiator.
- **Dolang and Hono are statistically tied on throughput.** The Python
  asyncio load generator caps at ~40k QPS; both servers are pressing against
  the client ceiling, not their own. A re-run with `wrk` or `bombardier` would
  likely lift both numbers.
- **Dolang is a clear replacement for FastAPI.** ~6× the throughput, ~9× lower
  p99, ~4× smaller RSS. For teams writing Python just to get terse route
  handlers, Dolang delivers the same ergonomics without the cost.
- **Cold start is our open problem.** Hono boots in 76 ms; Dolang takes 1 s.
  For long-lived servers this does not matter. For FaaS / edge workers it
  does. Profiling and reducing this is on the roadmap.

Full methodology, caveats, and reproduction instructions are in
[`benches/http-comparison/README.md`](benches/http-comparison/README.md).

---

## Quick Start

**Prerequisites**: Rust toolchain (stable).

```bash
git clone https://github.com/your-org/dolang
cd dolang
cargo build --release
```

Run a script:

```bash
./target/release/dolang run main.dol
```

Start an HTTP service:

```bash
./target/release/dolang serve
./target/release/dolang serve --routertab   # print the registered route table on startup
```

Run the project's tests:

```bash
./target/release/dolang test
```

---

## A Complete Hello Service

`main.dol`:

```dolang
$main() {
    $HTTP("/").link("app.router.hello_router");
}
```

`app/router/hello_router.dol`:

```dolang
$GET("/") index() -> JSON {
    $# { "status": "ok", "message": "Welcome to Dolang!" };
}

$GET("/hello/{name}") greet(name) -> JSON {
    $# { "greeting": "Hello, " + name + "!" };
}
```

```bash
dolang serve
# ➜  Local:   http://127.0.0.1:8080
```

---

## Project Configuration

`package.toml` is the single source of truth for a Dolang project.

```toml
[project]
name    = "my-service"
version = "0.1.0"
entry   = "main.dol"

[server]
host                 = "0.0.0.0"
port                 = 8080
upload_max_size      = 50      # total request body limit in MB (default 50)
upload_max_file_size = 10      # per-file limit in MB (default 10)

[server.auth]
enabled    = true
jwt_secret = "your-secret"
```

---

## Workspace Layout

| Path | Description |
|------|-------------|
| `crates/dolang-frontend/` | Lexer, parser, AST, diagnostics |
| `crates/dolang-runtime/`  | Interpreter, module resolver, project loader |
| `crates/dolang-cli/`      | CLI entry, REPL, `serve`, test runner |
| `crates/dolang-lsp/`      | Language Server Protocol implementation |
| `stdlib/`                 | Standard library namespaces |
| `sample/`                 | Runnable sample projects |
| `benches/`                | Microbenchmarks and HTTP comparison suite |
| `docs/`                   | Guides, references, contributor documentation |
| `tests/`                  | Integration tests and spec fixtures |

---

## Sample Projects

| Sample | Description |
|--------|-------------|
| [`sample/upload-demo`](sample/upload-demo/) | Multipart file upload with per-file and total size limits |
| [`sample/DataNest`](sample/DataNest/) | Full-stack notes application: Dolang gateway + Go gRPC backend |
| [`sample/auth-b2b-portal`](sample/auth-b2b-portal/) | JWT-protected B2B API gateway |
| [`sample/test-http-database`](sample/test-http-database/) | HTTP + PostgreSQL integration |

---

## Documentation

- **Getting started** — [docs/guide/README.md](docs/guide/README.md)
- **Syntax reference** — [docs/reference/syntax.md](docs/reference/syntax.md)
- **HTTP reference** — [docs/reference/http.md](docs/reference/http.md)
- **gRPC client** — [docs/reference/grpc.md](docs/reference/grpc.md)
- **Project system** — [docs/reference/project-system.md](docs/reference/project-system.md)
- **Standard library** — [docs/reference/stdlib-api.md](docs/reference/stdlib-api.md)
- **Error codes** — [docs/reference/errors.md](docs/reference/errors.md)
- **Contributor guide** — [docs/contributing/dev-guide.md](docs/contributing/dev-guide.md)

---

## Roadmap & Known Limitations

This is an early-stage project. We publish the known sharp edges so you can
decide whether Dolang fits your needs today.

- **Cold start (~1 s)**. Measurable and targeted for reduction. Long-lived
  servers aren't affected; FaaS / edge workloads should wait for optimisation.
- **Stdlib database coverage**. PostgreSQL is supported; MySQL, Redis, and
  MongoDB are not yet shipped.
- **No package registry**. Projects compose via local modules and
  `package.toml`. A community registry is being scoped, not yet implemented.
- **Single-author velocity**. Expect breaking changes between minor releases
  until the 1.0 line stabilises.
- **Observability**. Tracing and metrics hooks are minimal. OpenTelemetry
  integration is planned.

---

## Contributing

Before submitting a change:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Security reports go via
[SECURITY.md](SECURITY.md).

---

## License

MIT. See [LICENSE](LICENSE).
