# test-http-database

本项目用于验证 Dolang 在真实 HTTP + PostgreSQL + DDD 分层场景下的可用性与容灾路径。

## Recommended Layout

- `main.dol`: startup only
- `app/pages/admin.html`: admin shell returned by `$HTML().link(...)`
- `app/public/`: static CSS and JS for the admin SPA
- `app/router/*.dol`: mounted business API routers
- `config/`: runtime configuration and database connection
- `domains/*/data`: SQL and row fetching
- `domains/*/services`: use-case orchestration
- `shared/db/queries.dol`: shared query helper

## Run

```bash
cargo run -p dolang-cli -- serve test-http-database --routertab
```

本地数据库连接串固定为：

```text
host=/tmp dbname=postgres user=liangzhanbo
```

## Routes

- `GET /admin`
- `GET /assets/css/admin.css`
- `GET /assets/js/admin.js`
- `GET /api/users`
- `GET /api/users/:id`
- `POST /api/users`
- `PUT /api/users/:id`
- `DELETE /api/users/:id`
- `GET /api/categories`
- `GET /api/products`
- `GET /api/products/:id`
- `POST /api/products`
- `PUT /api/products/:id`
- `DELETE /api/products/:id`
- `GET /api/orders`
- `GET /api/users/:id/orders`
- `GET /api/orders/:id`
- `GET /api/orders/:id/items`
- `GET /api/orders/:id/detail`

## Verify

```bash
curl http://127.0.0.1:8082/admin
curl http://127.0.0.1:8082/assets/js/admin.js
curl http://127.0.0.1:8082/api/users
curl -X POST http://127.0.0.1:8082/api/users -H 'Content-Type: application/json' --data '{"name":"Alice Admin","email":"alice.admin@example.com"}'
curl http://127.0.0.1:8082/api/categories
curl http://127.0.0.1:8082/api/products
curl -X POST http://127.0.0.1:8082/api/products -H 'Content-Type: application/json' --data '{"name":"Keyboard","price":99.5,"category_id":1}'
curl http://127.0.0.1:8082/api/orders
curl http://127.0.0.1:8082/api/users/1/orders
curl http://127.0.0.1:8082/api/orders/1
curl http://127.0.0.1:8082/api/orders/1/items
curl http://127.0.0.1:8082/api/orders/1/detail
curl http://127.0.0.1:8082/api/orders/999
```

浏览器打开 `http://127.0.0.1:8082/admin` 后，可在单页后台里切换 `Products / Users / Orders`，并直接通过前端请求验证查询、创建、编辑、删除和订单详情查看。
