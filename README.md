# Dolang

Dolang is an interpreted scripting language for quickly turning ideas into runnable tools and HTTP services.

Current stdlib strengths include:

- fast lightweight HTTP services via `serve`
- outbound HTTP calls via `std.http`
- descriptor-driven unary gRPC client calls via `std.grpc`

## Workspace Overview

- `crates/dolang-frontend/`: AST, token, lexer, parser, diagnostics, syntax
- `crates/dolang-runtime/`: runtime, interpreter, module resolver, project loader
- `crates/dolang-cli/`: CLI entry, REPL, serve, test mode
- `crates/dolang-lsp/`: LSP crate, now directly depends on frontend
- `src/`: compatibility facade for the root `dolang` package
- `stdlib/`: standard library reserved namespace and contribution scaffold
- `examples/`: runnable sample projects and scripts
- `docs/`: guides, references, and contributor documentation
- `tests/`: integration tests and executable fixtures

## Quick Start

```bash
cargo build
cargo run
```

Useful commands:

```bash
cargo run -- run main.dol
cargo run -- serve
cargo run -- serve --routertab
cargo run -- test
```

## Development Baseline

Before opening a change, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

## Documentation

- Project docs: [docs/README.md](docs/README.md)
- gRPC client reference: [docs/reference/grpc.md](docs/reference/grpc.md)
- Contributor guide: [docs/contributing/dev-guide.md](docs/contributing/dev-guide.md)
- Language behavior index: [docs/spec/README.md](docs/spec/README.md)
- Standard library governance: [docs/stdlib/README.md](docs/stdlib/README.md)
- Refactor plan: [refactor-plan.md](refactor-plan.md)

## Examples

- HTTP auth sample: [examples/http-auth/README.md](examples/http-auth/README.md)
- HTTP to gRPC gateway sample: [examples/http-grpc-gateway/README.md](examples/http-grpc-gateway/README.md)

## License

MIT
