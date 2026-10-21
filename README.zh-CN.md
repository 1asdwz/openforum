# OpenForum

[English](README.md) · **简体中文**

一个开源的 AI 原生社区论坛系统：成员可自建子论坛并发布富文本帖子，支持嵌套评论、投票、实名认证中心，每个论坛可自定义准入门槛，论坛与个人主页可写自定义代码，并提供一套可自建前端的 OAuth 2.0 开放接口。

线上实例：<https://openforum.yingrest.com>

[![Rust](https://img.shields.io/badge/backend-Rust%20%2B%20axum-000000)](https://github.com/1asdwz/openforum)
![Vue](https://img.shields.io/badge/frontend-Vue%203%20%2B%20TypeScript-42b883)
![MySQL](https://img.shields.io/badge/database-MySQL%208-4479A1)

---

## 目录

- [亮点](#亮点)
- [项目简介](#项目简介)
- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [仓库结构](#仓库结构)
- [快速开始](#快速开始)
- [环境变量](#环境变量)
- [默认账号](#默认账号)
- [API 与开放平台](#api-与开放平台)
- [部署](#部署)
- [技术文档](#技术文档)
- [安全说明](#安全说明)
- [贡献](#贡献)

## 亮点

**类似 Reddit 的子论坛，由成员自建。** 站点本身是社区集合：任何登录用户都能创建子论坛（`f/{slug}`），设置名称、图标、主题色、公开或私密、版规，并成为坛主。坛主可任命版主并分配权限，每个子论坛都有独立的帖子流、成员列表与可选的自定义主页。

**AI 原生。** 认证材料会经过 AI 视觉审核（`ai-vision-review`）与启发式检查（`heuristic-document`），论坛与帖子排序由 `hot-score-recommendation` 计算。插件注册表还内置 `ai-review` 与 `rule-based-review`，所有插件都能在管理后台运行时开关。AI 调用走任意 OpenAI 兼容接口（`OPENAI_API_BASE`）；未配置密钥时视觉审核返回不可用，材料留在人工审核。

**每个论坛自定义准入门槛。** 论坛可分别配置浏览规则与操作规则，条件覆盖认证等级（Low / Medium / High / Extreme）、年龄、国籍、地区和资料字段。规则在界面中编辑，无需改代码；每次读写都会判定，未通过时返回 `403` 并附带缺失条件，前端可直接告诉用户还差什么。

```json
{
  "version": 1,
  "conditions": [
    { "field": "REAL_NAME", "authLevel": "MEDIUM" },
    { "field": "AGE", "op": "between", "min": 18, "max": 60 },
    { "field": "COUNTRY", "op": "in", "values": ["CN", "SG"] }
  ]
}
```

**论坛与个人主页都能写代码。** 坛主和用户都可以发布自己的 HTML 主页（支持行内样式），用 `<of-*>` 自定义元素读取实时数据——`<of-profile>`、`<of-stats>`、`<of-posts>`、`<of-members>` 以及论坛对应的元素。提交的标记先经过白名单净化，再放入没有同源权限的沙箱 iframe 渲染，因此自定义代码无法读取 Cookie、存储或父页面。

**后端优先，前端可自建。** 自带 Vue 3 应用只是其中一个客户端。它展示的每个界面都对应 `/api` 下已文档化的 REST 接口，OpenAPI 规范可通过 `GET /api/developers/docs` 获取；配合 OAuth 2.0（授权码 + PKCE），第三方 Web 或移动 App 可以自行实现登录与全部接口调用。

## 项目简介

OpenForum 是一个可直接部署的论坛社区系统，设计目标是把「论坛」做成可扩展的社区基础设施，而不只是一个发帖页面：

- **内容层**：子论坛、富文本帖子、Markdown、图片/视频、嵌套评论、投票与热度排序。
- **身份层**：账号体系 + 实名认证中心，认证结果分为 Low / Medium / High / Extreme 四个等级，可被论坛访问规则直接引用。
- **权限层**：论坛可配置浏览规则与操作规则（按认证等级、年龄、地区等条件组合），并支持坛主 / 版主的多项管理权限。
- **表达层**：用户与论坛均可自定义主页（受控 HTML 沙箱渲染），并提供模板与引导。
- **扩展层**：AI 插件层（文本与图片审核、材料识别、热度排序）与 OAuth 2.0 开放平台，第三方程序可代表用户调用全部公开 API。

前端为单页应用，后端为 Rust（axum）实现，数据存储使用 MySQL 8，媒体文件使用 MinIO（S3 兼容）。

## 功能特性

| 模块 | 能力 |
| --- | --- |
| 账号 | 注册、登录、Google 登录、JWT 鉴权、个人资料 |
| 子论坛 | 成员自建社区（`f/{slug}`）：创建 / 编辑、加入 / 退出、成员与版主管理、公开 / 私有、主题色与图标 |
| 帖子 | 发布、编辑、删除、置顶、投票、图片与视频、分页与排序 |
| 评论 | 树形嵌套、投票、删除、按帖子聚合 |
| 搜索 | 按标题 / 内容 / 论坛检索 |
| 认证中心 | 实名、身份证、年龄、国籍、驾照、婚姻、房产、资产等认证类型；材料上传、活体检测、AI / 规则 / 人工审核、等级体系 |
| 访问门禁 | 浏览规则与操作规则可视化编辑；按认证等级、资料字段、逻辑组合判定 |
| 自定义主页 | 用户主页与论坛主页；受控 HTML（支持行内样式）、`<of-*>` 元素、模板、举报、管理员屏蔽 |
| AI 与插件 | `ai-review`、`ai-vision-review`、`heuristic-document`、`hot-score-recommendation`、`rule-based-review`；运行时可开关 |
| 自建客户端 | 完整 REST API、公开 OpenAPI 规范、OAuth 2.0；自带 Vue 应用只是参考实现之一 |
| 管理后台 | 认证审核、用户列表、主页内容审核与屏蔽 |
| 开发者平台 | 创建 OAuth 应用、重置密钥、查看公开 API 文档 |
| OAuth 2.0 | 授权码 + PKCE（S256）、访问令牌、刷新令牌轮换、userinfo、revoke |

## 技术栈

### 后端（生产实现，`backend-rs/`）

| 组件 | 选型 |
| --- | --- |
| 语言 / 运行时 | Rust 2021、Tokio |
| Web 框架 | axum 0.7（multipart、macros） |
| 数据访问 | sqlx 0.8（MySQL、chrono、json，编译期无宏数据库依赖） |
| 鉴权 | jsonwebtoken（HS256）+ bcrypt |
| 对象存储 | MinIO / S3（`rust-s3`，路径风格访问） |
| HTML 净化 | ammonia |
| 图像处理 | `image`（JPEG / PNG / WebP / GIF） |
| 可观测性 | tracing + tracing-subscriber |
| 反向代理中间件 | tower / tower-http（CORS、Trace、BodyLimit） |

### 前端（`frontend/`）

Vue 3 + TypeScript + Vite 5 + Pinia + Vue Router + Element Plus + Axios + Tiptap。

### 数据与部署

MySQL 8、MinIO；前端部署于 Cloudflare Pages，后端以 Docker 容器运行。

### 历史实现（`backend/`）

`backend/` 是早期的 Java 25 / Spring Boot 4.1.1 实现，线上已由 Rust 后端替代。保留它是为了对照旧的表结构、接口语义与迁移过程，**新功能请只修改 `backend-rs/`**。

## 仓库结构

```text
openforum/
├── backend-rs/              # Rust 后端（线上实现）
│   ├── api/openapi.yaml     # OpenAPI 3 规范（公开下发）
│   ├── deploy/              # 环境变量示例
│   ├── sql/schema.sql       # 完整建表脚本（21 张表）
│   └── src/
│       ├── main.rs          # 启动、路由挂载、安全门禁中间件
│       ├── routes/          # 各业务模块 HTTP 处理
│       ├── repo.rs          # 数据访问
│       ├── oauth.rs         # OAuth 基础能力（PKCE、令牌、客户端）
│       ├── access.rs        # 坛主 / 版主权限
│       ├── gate.rs          # 访问规则引擎
│       ├── info.rs          # 用户资料字段定义
│       ├── plugins.rs       # 插件注册表
│       ├── storage.rs       # MinIO 存储
│       └── ...
├── frontend/                # Vue 3 单页应用
│   ├── public/_worker.js    # Cloudflare Pages 同源 API 代理
│   └── src/{views,components,api,router,stores}
├── backend/                 # 旧 Java 实现（仅供参考）
├── docs/                    # 技术文档
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── OAUTH.md
│   ├── DATABASE.md
│   ├── DEPLOYMENT.md
│   └── DEVELOPMENT.md
├── docker-compose.yml       # 旧 Java 后端的本地编排
└── CONTRIBUTING.md
```

## 快速开始

### 前置条件

- Rust stable（Dockerfile 使用 1.93）
- MySQL 8.0+
- Node.js 18+ 与 npm
- 可选：MinIO（媒体上传 / 读取所必需）

### 1. 准备数据库

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

`backend-rs/sql/schema.sql` 是完整的 21 张表结构。首次启动时后端还会执行幂等迁移（补充用户资料、主页、OAuth 相关表）并注入种子数据。

### 2. 启动后端

```bash
cd backend-rs

export SERVER_PORT=8080
export DB_URL='jdbc:mysql://127.0.0.1:3306/openforum'
export DB_USER=root
export DB_PASSWORD='your-password'
export JWT_SECRET='replace-with-a-random-secret-at-least-32-bytes'

# 可选：开启媒体存储
export MINIO_ENABLED=true
export MINIO_ENDPOINT='http://127.0.0.1:9000'
export MINIO_ACCESS_KEY=minioadmin
export MINIO_SECRET_KEY=minioadmin
export MINIO_BUCKET=openforum-auth

cargo run
```

后端监听 `http://localhost:8080`，API 前缀为 `/api`。健康检查可直接请求任意公开接口，例如：

```bash
curl http://localhost:8080/api/plugins
```

> `JWT_SECRET` 少于 32 字节时进程会拒绝启动。生产环境务必替换默认值。

### 3. 启动前端

```bash
cd frontend
npm install
npm run dev
```

开发服务器为 `http://localhost:5173`，会把 `/forum/api` 代理到 `http://localhost:8080`（见 [vite.config.ts](frontend/vite.config.ts)）。

生产构建：

```bash
npm run type-check
npm run build
```

### 4. Docker 运行后端

把示例环境文件复制为本地配置并修改数据库地址，然后构建运行：

```bash
cp backend-rs/deploy/openforum-rs.env.example backend-rs/.env.local
# 编辑 backend-rs/.env.local：至少修改 DB_URL、DB_PASSWORD、JWT_SECRET

docker build -t openforum-rs:latest backend-rs
docker run -d --name openforum-rs \
  -p 8080:8080 \
  --env-file backend-rs/.env.local \
  openforum-rs:latest
```

`docker-compose.yml` 编排的是旧 Java 后端 + Redis + MySQL + 前端，仅用于历史环境复现；Rust 后端不依赖 Redis。

## 环境变量

后端全部配置来自环境变量，完整示例见 [openforum-rs.env.example](backend-rs/deploy/openforum-rs.env.example)。

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `SERVER_PORT` | `8080` | 监听端口 |
| `DB_URL` | `jdbc:mysql://localhost:3306/openforum` | MySQL 连接串，兼容 `jdbc:mysql://` 前缀 |
| `DB_USER` | `root` | 数据库用户 |
| `DB_PASSWORD` | `root123456` | 数据库密码 |
| `JWT_SECRET` | 内置开发密钥 | JWT 签名密钥，**必须 ≥ 32 字节** |
| `JWT_EXPIRATION_HOURS` | `168` | 登录令牌有效期（小时） |
| `MINIO_ENABLED` | `false` | 是否启用对象存储 |
| `MINIO_ENDPOINT` | `http://localhost:9000` | MinIO 地址 |
| `MINIO_ACCESS_KEY` | `minioadmin` | MinIO Access Key |
| `MINIO_SECRET_KEY` | `minioadmin` | MinIO Secret Key |
| `MINIO_BUCKET` | `openforum-auth` | 存储桶名称 |
| `AI_ENABLED` | `false` | 是否启用 AI 审核插件 |
| `OPENAI_API_KEY` | 空 | AI 服务密钥 |
| `OPENAI_API_BASE` | `https://api.openai.com/v1` | AI 服务地址 |
| `OPENAI_API_MODEL` | `gpt-4o-mini` | 模型名称 |
| `AI_TIMEOUT_SECONDS` | `20` | AI 请求超时 |
| `GOOGLE_CLIENT_ID` | 空 | Google 登录 Client ID |
| `AUTH_CHALLENGE_TTL_MINUTES` | `10` | 认证挑战有效期（分钟） |
| `AUTH_MAX_DOCUMENT_COUNT` | `5` | 单次认证最多材料数 |
| `AUTH_MAX_IMAGE_BYTES` | `10485760` | 图片大小上限（10 MB） |
| `AUTH_MAX_VIDEO_BYTES` | `52428800` | 视频大小上限（50 MB） |
| `AUTH_AUTO_APPROVE_THRESHOLD` | `0.85` | 自动审核通过阈值 |

前端变量（`frontend/.env.*`）：

| 变量 | 说明 |
| --- | --- |
| `VITE_API_BASE_URL` | API 基地址，开发默认 `/forum/api` |
| `VITE_APP_NAME` | 站点名称 |
| `VITE_GOOGLE_CLIENT_ID` | Google Client ID；留空则隐藏 Google 登录 |
| `VITE_BASE_PATH` | 构建时的部署基础路径，默认 `/forum/`，独立域名部署用 `/` |

## 默认账号

数据库为空时，后端会自动注入以下账号（仅用于本地开发，请在部署前修改密码）：

| 用户名 | 密码 | 角色 |
| --- | --- | --- |
| `admin` | `admin123` | ADMIN |
| `alice` | `alice123` | USER |
| `bob` | `bob123` | USER |

## API 与开放平台

- **API 基地址**：`https://openforum.yingrest.com/api`（本地 `http://localhost:8080/api`）
- **统一响应信封**：`{"code":200,"message":"ok","data":...}`
- **鉴权**：`Authorization: Bearer <JWT>`
- **在线文档**：站点内 `/developers/docs`，同时公开下发 `GET /api/developers/docs`（OpenAPI 3 YAML）
- **规范文件**：[backend-rs/api/openapi.yaml](backend-rs/api/openapi.yaml)
- **自建前端**：自带 Vue 单页应用完全基于这些接口构建，任何 Web 或移动端前端都可以替换它。

示例：

```bash
# 登录
curl -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"admin","password":"admin123"}'

# 调用需登录的接口
curl http://localhost:8080/api/auth/me \
  -H 'Authorization: Bearer <access_token>'
```

### 第三方应用接入（OAuth 2.0）

1. 登录后进入 `/developers` 创建应用，配置回调地址（HTTPS，开发环境允许 `http://localhost` / `http://127.0.0.1`）。
2. 选择客户端类型：
   - `CONFIDENTIAL`：服务端应用，使用 `client_secret`（Basic 认证）。
   - `PUBLIC`：浏览器 / 移动端应用，无密钥，**强制 PKCE**。
3. 用户授权后，应用用授权码换取访问令牌，并以 `Authorization: Bearer` 调用全部公开 API。

| 端点 | 说明 |
| --- | --- |
| `GET /api/oauth/authorize/info` | 授权页展示的应用信息 |
| `POST /api/oauth/authorize` | 同意 / 拒绝授权，返回跳转地址 |
| `POST /api/oauth/token` | `authorization_code` / `refresh_token` |
| `GET /api/oauth/userinfo` | 当前令牌对应的用户信息 |
| `POST /api/oauth/revoke` | 撤销刷新令牌 |

访问令牌有效期 2 小时，刷新令牌 30 天且每次刷新都会轮换。完整流程、参数与错误码见 [docs/OAUTH.md](docs/OAUTH.zh-CN.md)。

## 部署

### 前端（Cloudflare Pages）

```bash
cd frontend
VITE_BASE_PATH=/ VITE_API_BASE_URL=/api npm run build
npx wrangler pages deploy dist --project-name openforum --branch main
```

`frontend/public/_worker.js` 会把同源的 `/api/*` 代理到 `BACKEND_BASE`（在 Pages 项目中配置环境变量），其余请求走静态资源并回退到 `index.html`。

### 后端（GitHub Actions + Docker）

[.github/workflows/deploy-rs.yml](.github/workflows/deploy-rs.yml) 在 `backend-rs/**` 变更时触发：构建镜像 → 压缩 → SCP 到服务器 → 以 `--env-file` 启动容器（默认映射 `8091:8080`）。

需要在仓库 Secrets 中配置 `SSH_PRIVATE_KEY`，并在服务器上准备环境变量文件。也可以完全手动部署：

```bash
docker build -t openforum-rs:latest backend-rs
docker run -d --name openforum-rs --restart unless-stopped \
  -p 8091:8080 --env-file /path/to/openforum-rs.env openforum-rs:latest
```

更多细节（反向代理、Nginx、旧 Java 流水线）见 [docs/DEPLOYMENT.md](docs/DEPLOYMENT.zh-CN.md)。

## 技术文档

| 文档 | 内容 |
| --- | --- |
| [ARCHITECTURE.md](docs/ARCHITECTURE.zh-CN.md) | 系统架构、请求生命周期、模块划分、关键设计决策 |
| [API.md](docs/API.zh-CN.md) | 接口约定、路由总表、错误码、分页 |
| [OAUTH.md](docs/OAUTH.zh-CN.md) | OAuth 2.0 授权码 + PKCE 全流程与示例 |
| [DATABASE.md](docs/DATABASE.zh-CN.md) | 21 张表的结构说明与初始化方式 |
| [DEPLOYMENT.md](docs/DEPLOYMENT.zh-CN.md) | Cloudflare Pages、Docker、Nginx、CI/CD |
| [DEVELOPMENT.md](docs/DEVELOPMENT.zh-CN.md) | 本地开发、构建、测试、排错 |
| [CONTRIBUTING.md](CONTRIBUTING.zh-CN.md) | 贡献流程与代码规范 |

## 安全说明

- 不要提交任何真实密钥：`JWT_SECRET`、数据库密码、`MINIO_SECRET_KEY`、`OPENAI_API_KEY`、`GOOGLE_CLIENT_ID` 均通过环境变量注入。
- `JWT_SECRET` 必须 ≥ 32 字节，生产环境请使用高熵随机值。
- 自定义主页的 HTML 会经过净化，并在受限沙箱中渲染。
- 认证材料存放于私有 MinIO 桶，仅通过鉴权后的媒体接口读取。
- 发现安全问题请不要公开提交 Issue，先通过仓库维护者私下联系。

## 贡献

欢迎 Issue 与 Pull Request。开始之前请阅读 [CONTRIBUTING.md](CONTRIBUTING.zh-CN.md)：

```bash
# 后端
cd backend-rs && cargo fmt && cargo test && cargo build --release

# 前端
cd frontend && npm run type-check && npm run build
```

