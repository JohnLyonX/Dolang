# DataNest Sample

DataNest 是一个双服务展示案例：

- Dolang：HTTP gateway + JWT auth
- Go：gRPC note service
- PostgreSQL：持久化

## 目录

- `database.sql`
  PostgreSQL 初始化脚本
- `proto/datanest.proto`
  gRPC 契约
- `descriptors/datanest.pb`
  Dolang gateway 使用的 descriptor
- `gateway/`
  Dolang HTTP gateway
- `go-service/`
  Go gRPC 服务

## 首版范围

- `POST /auth/register`
- `POST /auth/login`
- `GET /api/notes`
- `GET /api/notes/:id`
- `GET /api/notes/search?q=...`
- `GET /api/tags`
- `POST /api/notes`
- `POST /api/tags`
- `POST /api/notes/:id/tags`

## 环境变量

### Gateway

- `DATANEST_DATABASE_URL`
- `DATANEST_JWT_SECRET`
- `DATANEST_GRPC_TARGET`

### Go service

- `DATANEST_DATABASE_URL`
- `DATANEST_GRPC_ADDR`

## 初始化数据库

先执行：

```bash
psql -f sample/DataNest/database.sql
```

如果你已经手动创建并连接了 `datanest` 数据库，也可以只执行建表部分。

## 启动 Go gRPC 服务

```bash
cd sample/DataNest/go-service
DATANEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/datanest?sslmode=disable \
DATANEST_GRPC_ADDR=:50051 \
go run ./cmd/server
```

## 启动 Dolang gateway

```bash
DATANEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/datanest?sslmode=disable \
DATANEST_JWT_SECRET=dev-secret \
DATANEST_GRPC_TARGET=http://127.0.0.1:50051 \
cargo run -- serve sample/DataNest/gateway
```

## 联调顺序

### 1. 注册

```bash
curl -s -X POST http://127.0.0.1:8080/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"password123"}'
```

### 2. 登录

```bash
curl -s -X POST http://127.0.0.1:8080/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}'
```

把返回里的 `access_token` 记下来。

### 3. 创建笔记

```bash
curl -s -X POST http://127.0.0.1:8080/api/notes \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer <token>" \
  -d '{"title":"First Note","content":"hello datanest"}'
```

### 4. 创建标签

```bash
curl -s -X POST http://127.0.0.1:8080/api/tags \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer <token>" \
  -d '{"name":"work"}'
```

### 5. 绑定标签

```bash
curl -s -X POST http://127.0.0.1:8080/api/notes/<note_id>/tags \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer <token>" \
  -d '{"tag_id":"<tag_id>"}'
```

### 6. 查询与搜索

```bash
curl -s -H "Authorization: Bearer <token>" http://127.0.0.1:8080/api/notes
curl -s -H "Authorization: Bearer <token>" "http://127.0.0.1:8080/api/notes/search?q=hello"
```

## 当前边界

- 只支持 unary gRPC
- Dolang 侧使用 descriptor，不直接读取 `.proto`
- 认证由 Dolang gateway 负责
- Go service 不自行做 JWT 登录
- 这是展示案例，不是完整产品
