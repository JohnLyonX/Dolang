# Auth B2B Portal Sample

这个 sample 用来验证 Dolang `serve` 模式下的真实 auth 项目链路：

- 用户名密码登录
- Cookie session
- CSRF
- JWT access token
- refresh token 轮换
- 基于角色和权限的路由保护
- 公共新闻站与新闻后台
- Postgres 持久化 session / refresh token store

项目结构按 [test-http-database](/Users/liangzhanbo/CodeStudio/dolang/test-http-database) 的分层约定组织：

- `config/*`
- `shared/db/*`
- `domains/auth/data/*`
- `domains/auth/services/*`
- `domains/news/data/*`
- `domains/news/services/*`
- `app/router/*`

## Required Env

- `SESSION_DATABASE_URL`
  - runtime auth store 使用
- `APP_DATABASE_URL`
  - 业务用户查询使用
  - 如果未提供，项目会回退到 `SESSION_DATABASE_URL`
- `JWT_SECRET`
  - JWT 签名密钥

## Database

先导入项目根目录 [test-auth-plan.md](/Users/liangzhanbo/CodeStudio/dolang/test-auth-plan.md) 里的连续 SQL。

建议种子账号：

- `admin / password123`
- `editor / password123`
- `member / password123`

runtime 当前要求的是 Argon2 hash，不是 bcrypt。

你需要把种子 SQL 里的 `REPLACE_WITH_ARGON2_HASH` 替换成 `password123` 对应的真实 Argon2 hash。可以直接用 sample 自带脚本生成：

```bash
PASSWORD='password123' cargo run -- run sample/auth-b2b-portal/scripts/hash_password.dol
```

如果你已经导入过旧数据，直接更新 `app_users.password_hash` 即可。

## Run

```bash
cargo run -- serve sample/auth-b2b-portal
```

浏览器入口：

- `http://127.0.0.1:8080/`
- `http://127.0.0.1:8080/console`

控制台页面可直接点击验证：

- login
- `GET /me` via cookie / bearer
- `POST /auth/refresh`
- `POST /auth/logout`
- `GET /admin/audit` via cookie / bearer
- `POST /workspaces/current/switch` via cookie / bearer
- `POST /api/projects` via cookie / bearer

新闻页面：

- `GET /news`
- `GET /news/:slug`
- `GET /news-admin`

角色矩阵：

- `admin`
  - `project:create`
  - `audit:read`
  - `news:create`
  - `news:edit`
  - `news:publish`
- `editor`
  - `news:create`
  - `news:edit`
  - `news:publish`
- `member`
  - 只能访问公共新闻页和已授权的只读能力

新闻业务验证重点：

- `GET /api/news` 只返回 `published`
- `editor` 可创建草稿并发布新闻
- `member` 调 `POST /api/admin/news` 应得到 `403`
- cookie 写请求继续要求 `X-CSRF-Token`
- bearer 写请求不应受浏览器 cookie 干扰

## Main Routes

- `GET /`
- `GET /console`
- `GET /news`
- `GET /news/:slug`
- `GET /news-admin`
- `POST /auth/login`
- `POST /auth/refresh`
- `POST /auth/logout`
- `GET /me`
- `POST /workspaces/current/switch`
- `GET /admin/audit`
- `POST /api/projects`
- `GET /api/news`
- `GET /api/news/:slug`
- `GET /api/admin/news`
- `POST /api/admin/news`
- `PUT /api/admin/news/:id`
- `POST /api/admin/news/:id/publish`
- `POST /api/admin/news/:id/unpublish`
