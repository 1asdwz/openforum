# 本地开发
[English](DEVELOPMENT.md) · **简体中文**

本文面向需要修改 OpenForum 代码的开发者，覆盖环境准备、启动、构建、测试与常见问题。

## 1. 工具链

| 工具 | 版本 | 用途 |
| --- | --- | --- |
| Rust | stable（Dockerfile 使用 1.93） | 后端 |
| MySQL | 8.0+ | 数据库 |
| Node.js / npm | 18+ | 前端 |
| Docker | 可选 | 容器化运行后端 / MinIO |
| MinIO | 可选 | 媒体上传与读取 |

推荐安装 `rustfmt` 与 `clippy`：

```bash
rustup component add rustfmt clippy
```

## 2. 首次启动

### 2.1 数据库

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

也可以只建空库，让后端启动时通过 `migrate.rs` 建表；但 `schema.sql` 包含完整基线，推荐直接导入。

### 2.2 后端

```bash
cd backend-rs

export DB_URL='jdbc:mysql://127.0.0.1:3306/openforum'
export DB_USER=root
export DB_PASSWORD='your-password'
export JWT_SECRET='local-dev-secret-key-at-least-32-bytes-long'

cargo run
```

启动日志应包含：

```text
Starting OpenForum Rust backend on port 8080, database 127.0.0.1:3306 / openforum
MySQL connection pool ready
Listening on http://0.0.0.0:8080 (API prefix /api)
```

首次启动会注入默认账号（`admin/admin123`、`alice/alice123`、`bob/bob123`）。

### 2.3 前端

```bash
cd frontend
npm install
npm run dev
```

访问 `http://localhost:5173/forum/`（默认 `base=/forum/`）。开发代理把 `/forum/api` 转发到 `http://localhost:8080`，因此后端无需配置 CORS。

如需同时调试 OAuth 授权页，直接访问：

```text
http://localhost:5173/forum/oauth/authorize?client_id=<id>&redirect_uri=http://localhost:3000/cb&state=dev
```

## 3. 常用命令

### 后端

```bash
cd backend-rs

cargo fmt                 # 格式化
cargo fmt --check         # 检查格式
cargo clippy -- -D warnings
cargo test                # 单元测试
cargo build --release     # 发布构建
```

测试覆盖了 OAuth 工具（PKCE RFC 向量、重定向校验、查询拼接）、路由表冲突检查、访问门禁与审核逻辑等纯函数部分；涉及数据库的流程需要本地 MySQL 手工验证。

### 前端

```bash
cd frontend

npm run dev          # 开发服务器
npm run type-check   # vue-tsc 类型检查
npm run build        # 生产构建
npm run preview      # 预览构建产物
```

> 前端目前没有配置单元测试脚本，提交前至少执行 `npm run type-check` 与 `npm run build`。

### 一键启动脚本

仓库根目录提供 `start.sh` / `stop.sh`（针对旧 Java + Docker Compose 环境），Rust 后端请使用上面的命令或 `docker build`。

## 4. 代码结构约定

### 4.1 后端

| 目录 / 文件 | 职责 |
| --- | --- |
| `routes/` | HTTP 层：解析参数、调用 repo / 领域模块、返回 `ApiResult` |
| `repo.rs` | SQL 查询与行映射 |
| `dto.rs` / `models.rs` | 对外 DTO 与数据库模型 |
| `gate.rs` / `auth_gate.rs` | 门禁规则模型与求值 |
| `security.rs` | JWT、密码哈希、`CurrentUser` 提取器 |
| `error.rs` | 统一错误与响应信封 |
| `config.rs` | 环境变量与启动校验 |
| `migrate.rs` | 幂等建表 |

新增接口的步骤：

1. 在 `routes/<module>.rs` 中实现 handler。
2. 在 `routes/mod.rs` 注册路由。
3. 如为公开接口，确认 `main.rs::is_public_path` 是否需要放行（默认要求登录）。
4. 更新 `backend-rs/api/openapi.yaml`。
5. 如涉及新表，更新 `migrate.rs` 与 `sql/schema.sql`。
6. 补充 `docs/API.md` 路由表。

### 4.2 前端

| 目录 | 职责 |
| --- | --- |
| `api/` | Axios 实例、接口封装、错误处理 |
| `views/` | 路由页面 |
| `components/` | 可复用组件 |
| `stores/` | Pinia 状态（`auth`、`ui`） |
| `router/` | 路由与 `requiresAuth` 守卫 |
| `types/` | 共享类型 |
| `styles/main.css` | 设计变量与全局样式 |

约定：

- 组件与页面使用 `<script setup lang="ts">`。
- 接口调用统一走 `src/api/index.ts`，不要在组件里直接拼 URL。
- 新增页面需在 `src/router/index.ts` 注册，并考虑是否需要登录。
- Element Plus 组件按需自动引入，无需手动 `import`。
- UI 文案使用英文，代码注释使用英文。

## 5. 配置与密钥

- 本地开发可以使用默认值，但 `JWT_SECRET` 必须 ≥ 32 字节。
- 不要把真实密钥写入 `.env` 并提交；使用 shell 环境变量或本地未跟踪文件。
- `.gitignore` 已忽略 `node_modules/`、`dist/`、`target/`、`*.jar`、`*.sql`（`backend-rs/sql/schema.sql` 除外）等。

## 6. 常见问题

| 现象 | 原因 | 处理 |
| --- | --- | --- |
| 启动报 `JWT_SECRET must be at least 32 bytes` | 密钥过短 | 换成长随机值 |
| 启动后立刻退出 | 数据库连接失败 | 检查 `DB_URL` / 账号密码 / 网络 |
| 上传返回 `File storage is not enabled` | `MINIO_ENABLED=false` | 启用并配置 MinIO |
| 上传返回 400 格式不支持 | MIME 不在白名单 | 使用 jpeg/png/webp/gif/mp4/webm |
| 前端接口 404 | `VITE_API_BASE_URL` 与代理路径不一致 | 开发用 `/forum/api` |
| 前端白屏 | `VITE_BASE_PATH` 与访问路径不一致 | 访问 `http://localhost:5173/forum/` |
| 401 `unauthorized` | 未带 Bearer 或令牌过期 | 重新登录 |
| 403 `forbidden` | 非管理员 / 非坛主 / 门禁未通过 | 查看响应 `data.missing` |
| 镜像构建失败于 `include_str!("../../api/openapi.yaml")` | 构建上下文缺少 `backend-rs/api` | 使用 `docker build ... backend-rs` |
| `cargo` 编译慢 | 全量依赖编译 | 首次属正常，后续增量 |

## 7. 调试建议

- 后端日志级别由 `RUST_LOG` 控制，默认 `info,sqlx=warn,tower_http=info`。需要看 SQL 时设置 `RUST_LOG=info,sqlx=debug`。
- 前端可在浏览器 Network 面板查看 `/forum/api/*` 请求与响应信封。
- 门禁问题优先检查 `sub_forums.browse_rule` / `operate_rule` 与用户的 `user_authentications`、`user_info` 记录。
- OAuth 问题检查 `oauth_clients.status`、回调地址精确匹配、`code_verifier` 与 challenge 是否配对。

## 8. 提交前检查

```bash
cd backend-rs && cargo fmt --check && cargo test
cd ../frontend && npm run type-check && npm run build
```

同时确认：

- 没有提交真实密钥或本地环境文件；
- 新增 / 修改的接口已更新 OpenAPI 与文档；
- 数据库变更同时更新了 `schema.sql` 与 `migrate.rs`。
