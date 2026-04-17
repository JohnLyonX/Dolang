# Dolang

[English](./README.md) · [中文](./README-CN.md)

> 专为 HTTP 服务而生的脚本语言。
> 一个文件、一条 `serve` 命令,就能起一个生产形态的 API ——
> 驻留内存不到 **10 MB**。

```dolang
$GET("/greet/{name}") greet(name) -> JSON {
    $# { "message": "你好," + name + "!" };
}
```

---

## 为什么选择 Dolang

Dolang 是一门具有**一级 HTTP 语法**的解释型语言。声明一个路由,运行 `dolang serve`,
服务立即在线——没有框架配置、没有服务器样板、没有依赖注入容器。

设计目标聚焦在一个狭窄但高价值的后端切片:**中小型 API 网关、边缘函数、
Sidecar 和内部工具**,团队想要 FastAPI 的手感,又想要接近 Rust 的资源占用。

开箱即用的能力:

- **一级 HTTP** — 用 `$GET`、`$POST`、`$PUT`、`$DELETE`、`$PATCH` 声明路由;路径参数、查询串、JSON 请求体自动注入到 handler 形参。
- **渐进式类型** — 需要时声明,不需要时省略。在 handler 边界与声明的返回类型上强制检查。
- **开箱即用的标准库** — 出站 HTTP(`std.http`)、PostgreSQL(`std.sql.postgres`)、JWT 守卫(`std.auth.guard`)、gRPC 客户端(`std.grpc`)、文件系统(`std.fs`)。
- **Multipart 上传** — 大小限制在项目级可配。
- **HTML 渲染** — `$HTML()` 构造器 + `-> HTML` 返回类型支持服务端渲染。
- **模块系统** — `$mod` 导入,项目级 `package.toml`,不依赖中央注册中心。
- **LSP 支持** — 第一天就通过语言服务器协议集成 IDE。
- **Rust 运行时,零 GC** — 解释执行,但构建在 Rust 工具链和内存分配器之上,没有 stop-the-world 停顿。

---

## 性能

三份服务实现完全一致的 `/hello` 和 `/greet/:name` 接口,返回同样的 JSON 负载。
同机器、同客户端采样,不做参数调优。完整测试套件在
[`benches/http-comparison/`](benches/http-comparison/),可一键复现。

### 驻留内存与冷启动

| 方案 | 驻留内存 (RSS) | 冷启动 |
|:-----|-----------:|-----:|
| **Dolang** | **9.8 MB** | 1011 ms |
| FastAPI + uvicorn | 43.3 MB | 3108 ms |
| Hono + Node.js | 63.0 MB | **76 ms** |

### 吞吐与延迟 —— `/hello`

| 方案 | QPS | p50 | p99 |
|:-----|----:|----:|----:|
| **Dolang** | **42,431** | **1.47 ms** | **1.84 ms** |
| FastAPI | 6,814 | 9.04 ms | 16.57 ms |
| Hono | 39,025 | 1.61 ms | 1.96 ms |

### 吞吐与延迟 —— `/greet/:name`

| 方案 | QPS | p50 | p99 |
|:-----|----:|----:|----:|
| **Dolang** | **40,997** | **1.51 ms** | 2.08 ms |
| FastAPI | 6,167 | 10.02 ms | 17.41 ms |
| Hono | 38,662 | 1.62 ms | **2.01 ms** |

### 数据解读——如实说

- **Dolang 的故事是内存,不是纯吞吐**。9.8 MB 的驻留内存意味着同样一台机器能塞下的 Dolang 实例大约是 FastAPI 的 **4 倍**、Hono 的 **6 倍**。对于边缘、Sidecar 和多租户部署,这是真正的差异点。
- **Dolang 与 Hono 在吞吐维度实际打平**。Python asyncio 压测客户端的上限约 40k QPS,两者都已经触及客户端上限,而不是各自服务端上限。用 `wrk` 或 `bombardier` 重跑会同时抬高两边的真实天花板。
- **Dolang 是 FastAPI 的明确替代**。约 6× 的吞吐、9× 更低的 p99、4× 更小的 RSS。原本用 Python 只是为了写简洁的路由,现在可以不付这笔成本。
- **冷启动是目前的公开短板**。Hono 76 ms 启动,Dolang 需要 1 秒左右。长驻服务无感,FaaS / 边缘场景请等待优化。降低冷启动已在路线图上。

完整方法论、局限说明和复现步骤见
[`benches/http-comparison/README.md`](benches/http-comparison/README.md)。

---

## 快速开始

**前置条件**:Rust 工具链(stable)。

```bash
git clone https://github.com/your-org/dolang
cd dolang
cargo build --release
```

执行脚本:

```bash
./target/release/dolang run main.dol
```

启动 HTTP 服务:

```bash
./target/release/dolang serve
./target/release/dolang serve --routertab   # 启动时打印路由注册表
```

运行项目测试:

```bash
./target/release/dolang test
```

---

## 一个完整的 Hello 服务

`main.dol`:

```dolang
$main() {
    $HTTP("/").link("app.router.hello_router");
}
```

`app/router/hello_router.dol`:

```dolang
$GET("/") index() -> JSON {
    $# { "status": "ok", "message": "欢迎使用 Dolang!" };
}

$GET("/hello/{name}") greet(name) -> JSON {
    $# { "greeting": "你好," + name + "!" };
}
```

```bash
dolang serve
# ➜  Local:   http://127.0.0.1:8080
```

---

## 项目配置

`package.toml` 是 Dolang 项目的唯一配置入口。

```toml
[project]
name    = "my-service"
version = "0.1.0"
entry   = "main.dol"

[server]
host                 = "0.0.0.0"
port                 = 8080
upload_max_size      = 50      # 请求体总大小上限(MB,默认 50)
upload_max_file_size = 10      # 单文件上限(MB,默认 10)

[server.auth]
enabled    = true
jwt_secret = "your-secret"
```

---

## 仓库结构

| 路径 | 说明 |
|------|------|
| `crates/dolang-frontend/` | 词法、语法、AST、诊断 |
| `crates/dolang-runtime/`  | 解释器、模块解析、项目加载 |
| `crates/dolang-cli/`      | CLI 入口、REPL、`serve`、测试运行器 |
| `crates/dolang-lsp/`      | LSP 实现 |
| `stdlib/`                 | 标准库命名空间 |
| `sample/`                 | 可运行的示例项目 |
| `benches/`                | 微基准与 HTTP 对比测试 |
| `docs/`                   | 教程、参考、贡献者文档 |
| `tests/`                  | 集成测试与规范用例 |

---

## 示例项目

| 示例 | 说明 |
|------|------|
| [`sample/upload-demo`](sample/upload-demo/) | Multipart 上传,按单文件与总大小设限 |
| [`sample/DataNest`](sample/DataNest/) | 全栈笔记应用:Dolang 网关 + Go gRPC 后端 |
| [`sample/auth-b2b-portal`](sample/auth-b2b-portal/) | JWT 保护的 B2B API 网关 |
| [`sample/test-http-database`](sample/test-http-database/) | HTTP + PostgreSQL 集成示例 |

---

## 文档入口

- **入门教程** — [docs/guide/README.md](docs/guide/README.md)
- **语法参考** — [docs/reference/syntax.md](docs/reference/syntax.md)
- **HTTP 参考** — [docs/reference/http.md](docs/reference/http.md)
- **gRPC 客户端** — [docs/reference/grpc.md](docs/reference/grpc.md)
- **项目系统** — [docs/reference/project-system.md](docs/reference/project-system.md)
- **标准库 API** — [docs/reference/stdlib-api.md](docs/reference/stdlib-api.md)
- **错误代码** — [docs/reference/errors.md](docs/reference/errors.md)
- **贡献者指南** — [docs/contributing/dev-guide.md](docs/contributing/dev-guide.md)

---

## 路线图与已知局限

这是一个早期项目。我们选择把已知的短板写在最上面,方便你判断 Dolang 是否适合
今天的需求。

- **冷启动约 1 秒**。已被观测、已在计划中降低。长驻服务无感;FaaS / 边缘场景建议等待优化。
- **标准库数据库覆盖**。已内置 PostgreSQL;MySQL、Redis、MongoDB 暂未提供。
- **尚无包注册中心**。项目通过本地模块 + `package.toml` 组合;社区注册中心正在论证,未实现。
- **单人开发节奏**。1.0 前的 minor 版本之间会有 breaking change。
- **可观测性**。tracing / metrics 仅提供最小钩子,OpenTelemetry 集成在规划中。

---

## 贡献

在提交变更前,请执行:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 与 [行为准则](CODE_OF_CONDUCT.md)。
安全问题的披露方式见 [SECURITY.md](SECURITY.md)。

---

## 许可证

MIT。详见 [LICENSE](LICENSE)。
