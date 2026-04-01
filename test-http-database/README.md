# test-http-database

本项目用于验证 Dolang 在真实 HTTP + PostgreSQL + DDD 分层场景下的可用性与容灾路径。

## Recommended Layout

- `main.dol`: startup only
- `app/http/routes.dol`: route assembly
- `config/`: runtime configuration and database connection
- `domains/*/data`: SQL and row fetching
- `domains/*/services`: use-case orchestration
- `domains/*/routers`: HTTP adapters

## Run

```bash
cargo run -p dolang-cli -- serve test-http-database --routertab
```

本地数据库连接串固定为：

```text
host=/tmp dbname=postgres user=liangzhanbo
```

## Routes

- `GET /api/users`
- `GET /api/categories`
- `GET /api/products`
- `GET /api/users/:id/orders`
- `GET /api/orders/:id`
- `GET /api/orders/:id/items`
- `GET /api/orders/:id/detail`

## Verify

```bash
curl http://127.0.0.1:8082/api/users
curl http://127.0.0.1:8082/api/categories
curl http://127.0.0.1:8082/api/products
curl http://127.0.0.1:8082/api/users/1/orders
curl http://127.0.0.1:8082/api/orders/1
curl http://127.0.0.1:8082/api/orders/1/items
curl http://127.0.0.1:8082/api/orders/1/detail
curl http://127.0.0.1:8082/api/orders/999
```
