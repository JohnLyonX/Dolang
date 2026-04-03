# HTTP Auth Example

这个示例演示 `serve` 模式下的完整认证主线：

- `POST /login` 同时创建 session cookie 和 JWT access/refresh pair
- `POST /refresh` 使用 refresh token 轮换签发新的 token pair
- `POST /logout` 销毁当前 session 并清理 cookie
- `GET /me` 查看当前 principal / session / JWT claims
- `GET /admin` 演示基于角色的受保护路由

## Run

```bash
cargo run -- serve examples/http-auth
```

## Try It

先登录并保存 cookie：

```bash
curl -i \
  -c /tmp/dolang-auth.cookie \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"password123"}' \
  http://127.0.0.1:8080/login
```

响应里会同时返回：

- `Set-Cookie: dolang_session=...`
- `csrf_token`
- `access_token`
- `refresh_token`

使用 session cookie 访问：

```bash
curl -i \
  -b /tmp/dolang-auth.cookie \
  http://127.0.0.1:8080/admin
```

如果你要发起基于 session cookie 的写请求，还需要带上 `X-CSRF-Token`：

```bash
curl -i \
  -X POST \
  -b /tmp/dolang-auth.cookie \
  -H "Content-Type: application/json" \
  -H "X-CSRF-Token: <csrf_token>" \
  -d '{}' \
  http://127.0.0.1:8080/logout
```

使用 bearer token 访问：

```bash
curl -i \
  -H "Authorization: Bearer <access_token>" \
  http://127.0.0.1:8080/admin
```

刷新 token：

```bash
curl -i \
  -H 'Content-Type: application/json' \
  -d '{"refresh_token":"<refresh_token>"}' \
  http://127.0.0.1:8080/refresh
```

登出并清除 cookie：

```bash
curl -i \
  -X POST \
  -b /tmp/dolang-auth.cookie \
  http://127.0.0.1:8080/logout
```

## Notes

- 这个示例为了本地开发把 `cookie_secure = false`；如果你要设置 `cookie_same_site = "none"`，必须同时启用 `cookie_secure = true`
- `JWT` secret 这里只是示例值，生产环境请改成安全密钥或 `secret_env`
- cookie session 的 `POST` / `PUT` / `PATCH` / `DELETE` 请求默认会校验 `X-CSRF-Token`
