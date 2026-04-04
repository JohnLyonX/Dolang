# Auth Sample Project Test Plan

## Goal

在项目根目录新增 `sample/` 目录，并在其中规划一个真实 Dolang `serve` 项目 `sample/auth-b2b-portal`，用来验证新内置 Auth 标准库在真实业务场景下的可用性。

这次先输出计划，不立即实现项目代码。计划必须覆盖：

- Cookie Session
- CSRF
- JWT Access Token
- JWT Refresh Token
- 路由认证与授权
- Postgres 持久化 session / refresh token store
- 真实业务登录流和受保护接口

## Chosen Scenario

本次样例采用一个“B2B 团队协作后台”场景：

- 浏览器后台员工通过用户名密码登录
- 登录后同时拿到两类身份态：
  - 浏览器使用 cookie session
  - API 客户端或脚本使用 JWT access token
- access token 过期后，通过 refresh token 刷新
- 管理后台的写操作必须通过 CSRF 校验
- 管理员与普通成员拥有不同权限

这个场景覆盖面足够大，但仍然可控，适合做 Auth 第一批项目级回归样例。

## Why This Scenario

相比纯 demo 登录页，这个场景更接近真实业务：

- 同时存在浏览器态和 API 态
- 同时覆盖 session 与 bearer 两条认证链路
- 同时覆盖登录、查看身份、写操作、刷新 token、登出
- 能测出 `401`、`403`、CSRF、防重复 refresh、session 销毁等关键行为

## Planned Project Layout

计划中的项目目录：

```text
sample/
  auth-b2b-portal/
    package.toml
    main.dol
    config/
      app_config.dol
      database.dol
    shared/
      db/
        queries.dol
    domains/
      auth/
        data/
          auth_user_types.dol
          auth_user_queries.dol
        services/
          auth_user_service.dol
          permission_service.dol
          auth_service.dol
      news/
        data/
          news_types.dol
          news_queries.dol
        services/
          news_service.dol
    README.md
    scripts/
      smoke.sh
```

职责建议：

- `package.toml`
  - `serve` 配置
  - auth 配置
  - postgres session store 配置
- `main.dol`
  - HTTP 路由注册
  - 简单 handler 装配
- `domains/auth/data/*`
  - `$Type AuthUser` / `PublicUser`
  - 用户查询 SQL
- `domains/auth/services/auth_service.dol`
  - 登录、刷新、登出
  - session / jwt 协调逻辑
- `domains/auth/services/permission_service.dol`
  - 从业务角色映射到 runtime principal roles / permissions
- `domains/news/data/*`
  - `$Type NewsArticleSummary` / `NewsArticle`
  - 新闻查询 SQL
- `domains/news/services/news_service.dol`
  - 新闻业务装配
- `scripts/smoke.sh`
  - 真实 curl 流程回归脚本

## Planned Business Routes

### Public Routes

- `POST /auth/login`
  - 用户名密码登录
  - 返回当前用户摘要
  - 响应体包含 `access_token`、`refresh_token`、`csrf_token`
  - 响应头写入 session cookie

- `POST /auth/refresh`
  - 使用 refresh token 轮换 access/refresh pair

### Authenticated Routes

- `GET /me`
  - 查看当前 principal、session、jwt claims
  - 用于验证 cookie / bearer 两条链路都能生效

- `POST /workspaces/current/switch`
  - 模拟一个浏览器后台写操作
  - 要求 cookie session 已登录
  - 要求携带 `X-CSRF-Token`

- `POST /auth/logout`
  - 销毁 session
  - 清理 cookie
  - 如果请求体中带 refresh token，则同时执行 refresh token revoke

### Role-Protected Routes

- `GET /admin/audit`
  - 仅 `admin` 角色允许访问

- `POST /api/projects`
  - 要求 `project:create` 权限
  - 用于验证 bearer token 场景下的权限检查

## Principal Mapping

业务用户表只保存最小身份数据：

- `user_id`
- `username`
- `password_hash`
- `display_name`
- `team_id`
- `role`
- `is_active`

运行时 principal 由项目层映射生成：

- `role = "admin"`
  - roles: `["admin"]`
  - permissions: `["project:create", "project:read", "audit:read", "news:create", "news:edit", "news:publish", "news:delete"]`

- `role = "editor"`
  - roles: `["editor"]`
  - permissions: `["news:create", "news:edit"]`

- `role = "member"`
  - roles: `["member"]`
  - permissions: `["project:read"]`

这样可以避免 Dolang 项目在首版样例里承担复杂 RBAC 表解析逻辑。

## Recommended Auth Configuration

计划中的 `package.toml` auth 配置建议如下：

```toml
[project]
name = "auth-b2b-portal"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie", "bearer"]

[server.auth.session]
enabled = true
cookie_name = "portal_session"
cookie_http_only = true
cookie_same_site = "lax"
cookie_secure = false
cookie_path = "/"
ttl_seconds = 86400
idle_timeout_seconds = 7200
rotation = "on_login"

[server.auth.session.store]
driver = "postgres"

[server.auth.session.store.postgres]
url_env = "SESSION_DATABASE_URL"
table = "auth_sessions"
refresh_table = "auth_refresh_tokens"

[server.auth.jwt]
enabled = true
issuer = "dolang-auth-sample"
audience = "auth-b2b-portal"
algorithm = "HS256"
secret_env = "JWT_SECRET"
access_ttl_seconds = 900
refresh_ttl_seconds = 2592000

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/me"
require = "authenticated"

[[server.auth.authorization.rules]]
method = "POST"
path = "/workspaces/current/switch"
require = "authenticated"

[[server.auth.authorization.rules]]
method = "POST"
path = "/auth/logout"
require = "authenticated"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin/*"
require = "authenticated"
roles_any = ["admin"]

[[server.auth.authorization.rules]]
method = "POST"
path = "/api/projects"
require = "authenticated"
permissions_all = ["project:create"]
```

## Postgres SQL Design

### Important Runtime Constraint

当前 runtime 对 Postgres auth store 的表名支持是：

- session store 表名可配置
- refresh token store 表名也可配置

本 sample 计划里仍然建议固定使用：

- session store 使用 `auth_sessions`
- refresh token store 使用 `auth_refresh_tokens`

这样可以和默认值保持一致。相关实现见：

- `crates/dolang-runtime/src/runtime/auth/session_postgres.rs`
- `crates/dolang-runtime/src/runtime/auth/refresh_postgres.rs`
- `crates/dolang-runtime/src/runtime/context.rs`

### Continuous SQL Script

下面这段 SQL 可以直接连续执行。  
导入前只需要把 `REPLACE_WITH_ARGON2_HASH` 替换成真实密码哈希。

runtime 当前使用的是 Argon2，不是 bcrypt。你可以直接用 sample 自带脚本生成：

```bash
PASSWORD='password123' cargo run -- run sample/auth-b2b-portal/scripts/hash_password.dol
```

约束说明：

- `app_users` 是 sample 项目自己的业务用户表
- `auth_sessions` 必须和 runtime 当前 session store 字段兼容
- `auth_refresh_tokens` 必须和 runtime 当前 refresh token store 字段兼容
- `news_categories` 与 `news_articles` 是 sample 的新闻业务表
- `role` 当前建议使用 `admin`、`editor`、`member`
- `claims_json` 当前保持 `TEXT`，不要改成 `JSONB`

```sql
CREATE TABLE IF NOT EXISTS app_users (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    team_id TEXT NOT NULL,
    role TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_app_users_team_id
    ON app_users (team_id);

CREATE INDEX IF NOT EXISTS idx_app_users_role
    ON app_users (role);

CREATE TABLE IF NOT EXISTS news_categories (
    category_id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS news_articles (
    article_id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    body TEXT NOT NULL,
    category_id TEXT NOT NULL REFERENCES news_categories(category_id),
    status TEXT NOT NULL CHECK (status IN ('draft', 'published')),
    author_user_id TEXT NOT NULL REFERENCES app_users(user_id),
    published_at BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_news_articles_status
    ON news_articles (status);

CREATE INDEX IF NOT EXISTS idx_news_articles_slug
    ON news_articles (slug);

CREATE INDEX IF NOT EXISTS idx_news_articles_category_id
    ON news_articles (category_id);

CREATE TABLE IF NOT EXISTS auth_sessions (
    session_id TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    roles_json TEXT NOT NULL,
    permissions_json TEXT NOT NULL,
    claims_json TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    idle_timeout_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_subject
    ON auth_sessions (subject);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_expires_at
    ON auth_sessions (expires_at);

CREATE TABLE IF NOT EXISTS auth_refresh_tokens (
    token_id TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_subject
    ON auth_refresh_tokens (subject);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_expires_at
    ON auth_refresh_tokens (expires_at);

CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_revoked_at
    ON auth_refresh_tokens (revoked_at);

INSERT INTO app_users (
    user_id,
    username,
    password_hash,
    display_name,
    team_id,
    role
) VALUES
    (
        'user_admin_1',
        'admin',
        'REPLACE_WITH_ARGON2_HASH',
        'Portal Admin',
        'team_blue',
        'admin'
    ),
    (
        'user_member_1',
        'member',
        'REPLACE_WITH_ARGON2_HASH',
        'Portal Member',
        'team_blue',
        'member'
    ),
    (
        'user_editor_1',
        'editor',
        'REPLACE_WITH_ARGON2_HASH',
        'Portal Editor',
        'team_blue',
        'editor'
    );

INSERT INTO news_categories (
    category_id,
    slug,
    name,
    sort_order
) VALUES
    ('cat_company', 'company', 'Company', 10),
    ('cat_product', 'product', 'Product', 20),
    ('cat_security', 'security', 'Security', 30);

INSERT INTO news_articles (
    article_id,
    slug,
    title,
    summary,
    body,
    category_id,
    status,
    author_user_id,
    published_at,
    created_at,
    updated_at
) VALUES
    (
        'article_001',
        'portal-launches-newsroom',
        'Portal Launches Newsroom',
        'A public newsroom is now available for customers and partners.',
        'The Dolang sample newsroom is now live and demonstrates published article delivery.',
        'cat_company',
        'published',
        'user_admin_1',
        1735689600,
        1735689600,
        1735689600
    ),
    (
        'article_002',
        'spring-release-roundup',
        'Spring Release Roundup',
        'The product team shipped a compact release with auth and admin improvements.',
        'This article exists to verify multiple public news cards and detail pages.',
        'cat_product',
        'published',
        'user_editor_1',
        1735776000,
        1735776000,
        1735776000
    ),
    (
        'article_003',
        'security-hardening-draft',
        'Security Hardening Draft',
        'This draft article should only be visible from the admin side before publish.',
        'The draft state is used to verify publish and unpublish visibility rules.',
        'cat_security',
        'draft',
        'user_editor_1',
        NULL,
        1735862400,
        1735862400
    );
```

## Planned Request Flows

### Flow A: Browser Login with Cookie Session

1. `POST /auth/login` with username/password
2. 读取 `app_users`
3. 使用 `std.auth.password.verify` 校验密码
4. 创建 session
5. 签发 access/refresh token pair
6. 返回：
   - `Set-Cookie`
   - `csrf_token`
   - `access_token`
   - `refresh_token`

### Flow B: Browser Authenticated Read

1. 带 cookie 请求 `GET /me`
2. runtime 从 cookie 解析 session
3. handler 返回 principal / session 摘要

### Flow C: Browser Authenticated Write with CSRF

1. 带 cookie 请求 `POST /workspaces/current/switch`
2. 缺少 `X-CSRF-Token` 时，返回 `403`
3. token 正确时，返回 `200`

### Flow D: Bearer Access Token

1. 从 `/auth/login` 返回体中拿 `access_token`
2. 请求 `GET /me` 或 `POST /api/projects`
3. runtime 通过 bearer 构建 principal

### Flow E: Refresh Token Rotation

1. `POST /auth/refresh`
2. 使用 refresh token 换新 pair
3. 第二次复用旧 refresh token 必须失败

### Flow F: Logout

1. `POST /auth/logout`
2. 销毁当前 session
3. 清理 cookie
4. 如果带 refresh token，则 revoke 该 token

## Validation Standards

这是本计划最重要的部分。样例项目完成后，至少要满足以下验收标准。

### A. Login Success

- 正确账号密码登录返回 `200`
- 响应头包含 `Set-Cookie`
- 响应体包含：
  - `access_token`
  - `refresh_token`
  - `csrf_token`
  - `user`

### B. Login Failure

- 错误密码返回 `401`
- 不写入 session cookie
- 不返回 access/refresh token

### C. Session Authentication

- 使用登录后的 cookie 请求 `GET /me` 返回 `200`
- `guard.authenticated()` 为 `true`
- `session.current()` 非空

### D. CSRF Enforcement

- 使用 cookie 对受保护写接口发起 `POST`
- 缺少 `X-CSRF-Token` 返回 `403`
- 错误 token 返回 `403`
- 正确 token 返回 `200`

### E. Bearer Authentication

- 使用 `Authorization: Bearer <access_token>` 访问 `GET /me` 返回 `200`
- `jwt.current()` 非空
- 不要求 `X-CSRF-Token`

### F. Authorization Rules

- `member` 访问 `/admin/audit` 返回 `403`
- `admin` 访问 `/admin/audit` 返回 `200`
- 无 `project:create` 权限访问 `/api/projects` 返回 `403`

### G. Refresh Token Rotation

- 第一次调用 `/auth/refresh` 返回新的 access/refresh pair
- 第二次复用旧 refresh token 返回 `401`
- 新 refresh token 可以继续使用

### H. Logout

- `POST /auth/logout` 返回 `200`
- 响应头清除 cookie
- 登出后使用旧 cookie 请求 `/me` 返回 `401`

### I. Postgres Persistence

- 成功登录后，`auth_sessions` 中出现新记录
- 签发 refresh token 后，`auth_refresh_tokens` 中出现新记录
- refresh 后，旧 token 的 `revoked_at` 被写入
- 登出或 session 销毁后，旧 session 不再可用

## Manual Smoke Test Matrix

建议最终至少跑下面这组真实请求：

```text
1. POST /auth/login                       -> 200
2. GET /me with cookie                    -> 200
3. POST /workspaces/current/switch        -> 403 without CSRF
4. POST /workspaces/current/switch        -> 200 with CSRF
5. GET /admin/audit as admin              -> 200
6. GET /admin/audit as member             -> 403
7. GET /me with bearer                    -> 200
8. POST /api/projects with member bearer  -> 403
9. POST /auth/refresh                     -> 200
10. POST /auth/refresh old token again    -> 401
11. POST /auth/logout                     -> 200
12. GET /me with old cookie               -> 401
```

## Suggested Implementation Order

### Phase 1. Scaffold Sample Project

- 创建 `sample/auth-b2b-portal`
- 写 `package.toml`
- 写最小 `README.md`

### Phase 2. Add Business Login Source

- 接好 `app_users` 查询
- 接好密码校验
- 完成 `/auth/login`

### Phase 3. Add Session + JWT Dual Flow

- 写 session 登录态
- 写 JWT issue pair
- 完成 `/me`

### Phase 4. Add CSRF + Protected Write Route

- 完成 `/workspaces/current/switch`
- 补 cookie 写请求 CSRF 验证用例

### Phase 5. Add Admin + Permission Checks

- 完成 `/admin/audit`
- 完成 `/api/projects`

### Phase 6. Add Refresh + Logout Flow

- 完成 `/auth/refresh`
- 完成 `/auth/logout`
- 验证 refresh revoke 与 session destroy

### Phase 7. Add Smoke Script

- 写 `scripts/smoke.sh`
- 固化真实 curl 回归流程

## Out of Scope for First Sample

以下内容先不纳入首版样例：

- OAuth / OIDC
- 多租户 IAM
- 邮箱验证码
- 找回密码
- 用户注册
- 管理员后台 UI
- Redis session store

## Final Recommendation

先做一个项目级、真实但收敛的样例：

- 项目名：`sample/auth-b2b-portal`
- 业务形态：B2B 团队协作后台
- 登录入口：用户名密码
- 双认证链路：cookie session + JWT bearer
- 持久化：Postgres
- 验证重点：CSRF、401/403、refresh rotation、logout、权限控制

这版样例做完后，就可以把 Auth 模块从“内置标准库能力”提升到“真实项目可验证能力”。
