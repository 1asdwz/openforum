# API 参考
[English](API.md) · **简体中文**

本文说明 OpenForum HTTP API 的通用约定与路由总表。机器可读的完整定义（参数、请求体、响应结构）以 [backend-rs/api/openapi.yaml](../backend-rs/api/openapi.yaml) 为准，也可直接访问线上 `GET /api/developers/docs`。

## 1. 基础信息

| 项目 | 值 |
| --- | --- |
| 线上地址 | `https://openforum.yingrest.com/api` |
| 本地地址 | `http://localhost:8080/api` |
| 协议 | HTTPS |
| 编码 | UTF-8，请求 / 响应均为 JSON（上传接口为 `multipart/form-data`） |
| 鉴权 | `Authorization: Bearer <JWT>` |
| OpenAPI | `GET /api/developers/docs`（公开，`application/yaml`） |
| 文档页 | `/developers/docs` |

所有路由都挂载在 `/api` 前缀下；下文表格省略该前缀。

## 2. 响应格式

### 2.1 成功

```json
{
  "code": 200,
  "message": "ok",
  "data": { }
}
```

### 2.2 业务错误

HTTP 状态码与 `code` 一致，`data` 通常为 `null`：

```json
{
  "code": 400,
  "message": "Invalid request",
  "data": null
}
```

### 2.3 门禁拒绝

论坛访问规则不满足时返回 `403`，并在 `data` 中给出结构化信息：

```json
{
  "code": 403,
  "message": "You do not yet meet the browse requirements for this forum",
  "data": {
    "gate": "BROWSE",
    "forumId": 1,
    "forumName": "Example",
    "loginRequired": false,
    "missing": [
      { "fieldKey": "REAL_NAME", "label": "Real name", "reason": "NOT_VERIFIED", "required": "MEDIUM" }
    ]
  }
}
```

### 2.4 过滤器层错误

未登录 / 无权限由安全门禁直接返回，**没有 `data` 字段**：

```json
{ "code": 401, "message": "unauthorized" }
{ "code": 403, "message": "forbidden" }
```

### 2.5 OAuth 标准端点

`/oauth/token`、`/oauth/revoke` 按 RFC 6749 返回裸对象，并带 `Cache-Control: no-store`：

```json
{ "error": "invalid_grant", "error_description": "Authorization code has expired" }
```

`/oauth/userinfo` 同样不套信封。详见 [OAUTH.md](OAUTH.zh-CN.md)。

## 3. 分页与排序

列表接口使用 0 基页码：

| 参数 | 默认 | 说明 |
| --- | --- | --- |
| `page` | `0` | 页码，从 0 开始 |
| `size` | `20` | 每页数量 |
| `sort` | `hot` | 排序方式（`hot` / `new` 等，按接口而定） |
| `q` | 空 | 搜索关键词 |

响应：

```json
{
  "content": [],
  "page": 0,
  "size": 20,
  "totalElements": 0,
  "totalPages": 0
}
```

## 4. 认证与权限标记

| 标记 | 含义 |
| --- | --- |
| 匿名 | 无需令牌 |
| 登录 | 需要有效 JWT |
| 管理员 | 需要 `role=ADMIN` |
| 应用所有者 | 仅能操作自己创建的 OAuth 应用 |
| OAuth 客户端 | 需要 `client_id` + `client_secret`（或 PUBLIC 客户端 + PKCE） |

> 公共路径在门禁层匿名放行，但处理器仍可能要求登录（例如 `GET /auth/me`、`POST /oauth/authorize`）。

## 5. 路由总表

### 5.1 认证

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| POST | `/auth/register` | 匿名 | 注册 |
| POST | `/auth/login` | 匿名 | 用户名 / 邮箱 + 密码登录 |
| POST | `/auth/google` | 匿名 | Google ID Token 登录 |
| GET | `/auth/me` | 登录 | 当前用户 |

### 5.2 用户与资料

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| PUT | `/users/me` | 登录 | 更新个人资料 |
| GET | `/users/:id` | 匿名 | 用户公开主页数据 |
| GET | `/users/:id/posts` | 匿名 | 用户帖子 |
| GET | `/users/:id/comments` | 匿名 | 用户评论 |
| GET | `/users/me/homepage` | 登录 | 我的主页配置 |
| PUT | `/users/me/homepage` | 登录 | 保存我的主页 |
| POST | `/users/me/homepage/reset` | 登录 | 重置主页 |
| GET | `/users/:id/homepage` | 匿名 | 公开主页（受状态控制） |
| GET | `/users/:id/homepage-data` | 匿名 | 主页组件所需数据 |
| POST | `/users/:id/homepage/report` | 登录 | 举报主页 |
| GET | `/info/fields` | 登录 | 资料字段目录 |
| GET | `/info/me` | 登录 | 我的资料 |
| PUT | `/info/me/:fieldKey` | 登录 | 更新单个资料字段 |

### 5.3 子论坛

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/forums` | 匿名 | 论坛列表（分页） |
| POST | `/forums` | 登录 | 创建论坛 |
| GET | `/forums/popular` | 匿名 | 热门论坛 |
| GET | `/forums/recommended` | 匿名 | 推荐论坛 |
| GET | `/forums/:id` | 匿名 | 论坛详情（受浏览门禁） |
| PUT | `/forums/:id` | 登录（坛主） | 更新论坛 |
| POST | `/forums/:id/join` | 登录 | 加入 |
| POST | `/forums/:id/leave` | 登录 | 退出 |
| GET | `/forums/:id/members` | 匿名 | 成员列表 |
| GET | `/forums/:id/moderators` | 匿名 | 版主列表 |
| POST | `/forums/:id/moderators` | 坛主 | 添加版主 |
| PUT | `/forums/:id/moderators/:userId` | 坛主 | 调整版主权限 |
| DELETE | `/forums/:id/moderators/:userId` | 坛主 | 移除版主 |

### 5.4 论坛主页

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/forums/:id/homepage` | 匿名 | 公开主页 |
| PUT | `/forums/:id/homepage` | 坛主 | 保存主页 |
| GET | `/forums/:id/homepage/mine` | 登录 | 我的可编辑版本 |
| POST | `/forums/:id/homepage/reset` | 坛主 | 重置 |
| POST | `/forums/:id/homepage/report` | 登录 | 举报 |
| GET | `/forums/:id/homepage-data` | 匿名 | 主页组件数据 |

### 5.5 帖子与评论

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/posts` | 匿名 | 列表；`forumId`、`q`、`sort`、`page`、`size` |
| POST | `/posts` | 登录 | 发帖（受操作门禁） |
| GET | `/posts/:id` | 匿名 | 详情（受浏览门禁） |
| PUT | `/posts/:id` | 作者 / 管理员 | 编辑 |
| DELETE | `/posts/:id` | 作者 / 管理员 | 删除 |
| POST | `/posts/:id/vote` | 登录 | 投票，`value` 为 `1` / `-1` / `0` |
| POST | `/posts/:id/pin` | 坛主 / 版主 | 置顶 |
| GET | `/comments/post/:postId` | 匿名 | 评论树 |
| POST | `/comments` | 登录 | 发表评论 |
| DELETE | `/comments/:id` | 作者 / 管理员 | 删除评论 |
| POST | `/comments/:id/vote` | 登录 | 评论投票 |

### 5.6 媒体

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| POST | `/media` | 登录 | 上传图片 / 视频，字段名 `file` |
| GET | `/media/:id` | 匿名 | 读取帖子媒体 |

支持类型：`image/jpeg`、`image/png`、`image/webp`、`image/gif`、`video/mp4`、`video/webm`；图片 ≤ 10MB，视频 ≤ 50MB。

### 5.7 认证中心

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/authentications/types` | 匿名 | 认证类型与等级元数据 |
| GET | `/authentications/challenge` | 登录 | 创建挑战（nonce / 有效期） |
| POST | `/authentications/media` | 登录 | 上传认证材料（`kind=DOCUMENT\|SELFIE\|VIDEO`） |
| GET | `/authentications/media/:mediaId` | 本人 / 管理员 | 读取认证材料 |
| POST | `/authentications` | 登录 | 提交认证 |
| GET | `/authentications/me` | 登录 | 我的认证记录 |
| GET | `/authentications/me/approved` | 登录 | 已通过的认证 |
| POST | `/authentications/:id/retry` | 登录 | 重新提交 |

### 5.8 管理后台

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/admin/authentications` | 管理员 | 待审 / 全部认证 |
| POST | `/admin/authentications/:id/approve` | 管理员 | 通过 |
| POST | `/admin/authentications/:id/reject` | 管理员 | 拒绝 |
| GET | `/admin/users` | 管理员 | 用户列表 |
| GET | `/admin/forum-homepages` | 管理员 | 论坛主页列表 |
| POST | `/admin/forum-homepages/:forumId/block` | 管理员 | 屏蔽 |
| POST | `/admin/forum-homepages/:forumId/unblock` | 管理员 | 解除屏蔽 |
| GET | `/admin/user-homepages` | 管理员 | 用户主页列表 |
| POST | `/admin/user-homepages/:userId/block` | 管理员 | 屏蔽 |
| POST | `/admin/user-homepages/:userId/unblock` | 管理员 | 解除屏蔽 |

### 5.9 插件

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/plugins` | 匿名 | 插件列表与开关状态 |
| PUT | `/plugins/:key/enabled` | 管理员 | 切换开关（仅内存生效） |

### 5.10 开发者平台

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/developers/apps` | 登录 | 我的应用列表 |
| POST | `/developers/apps` | 登录 | 创建应用 |
| GET | `/developers/apps/:id` | 应用所有者 | 应用详情 |
| PUT | `/developers/apps/:id` | 应用所有者 | 更新应用 |
| DELETE | `/developers/apps/:id` | 应用所有者 | 删除应用 |
| POST | `/developers/apps/:id/secret` | 应用所有者 | 重置密钥 |
| GET | `/developers/docs` | 匿名 | OpenAPI 3 YAML |

创建应用请求体：

```json
{
  "name": "My App",
  "description": "Optional",
  "homepageUrl": "https://example.com",
  "redirectUris": ["https://example.com/callback"],
  "clientType": "CONFIDENTIAL"
}
```

约束：每用户最多 10 个应用，每应用最多 5 个回调地址；回调必须是 HTTPS（`http://localhost` / `http://127.0.0.1` 例外）。

### 5.11 OAuth 2.0

| 方法 | 路径 | 权限 | 说明 |
| --- | --- | --- | --- |
| GET | `/oauth/authorize/info` | 匿名 | 授权页展示信息 |
| POST | `/oauth/authorize` | 登录 | 同意 / 拒绝授权 |
| POST | `/oauth/token` | OAuth 客户端 | 换取 / 刷新令牌 |
| GET | `/oauth/userinfo` | Bearer | 用户信息 |
| POST | `/oauth/revoke` | OAuth 客户端 | 撤销刷新令牌 |

完整参数与流程见 [OAUTH.md](OAUTH.zh-CN.md)。

## 6. 错误码

| HTTP | 含义 | 典型场景 |
| --- | --- | --- |
| 400 | 参数错误 | 字段缺失、格式不支持、超出配额 |
| 401 | 未认证 | 缺少或无效令牌、客户端认证失败 |
| 403 | 无权限 | 非管理员、非坛主、门禁未通过 |
| 404 | 资源不存在 | ID 不存在、路由未匹配（`Resource not found`） |
| 500 | 服务器错误 | 数据库 / 存储异常（不返回内部细节） |

OAuth 端点的错误码遵循 RFC 6749：`invalid_request`、`invalid_client`、`invalid_grant`、`unsupported_grant_type`、`server_error`。

## 7. 调用示例

```bash
BASE=http://localhost:8080/api

# 登录
TOKEN=$(curl -s -X POST "$BASE/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"admin","password":"admin123"}' \
  | python -c 'import sys,json;print(json.load(sys.stdin)["data"]["token"])')

# 当前用户
curl -s "$BASE/auth/me" -H "Authorization: Bearer $TOKEN"

# 论坛列表
curl -s "$BASE/forums?page=0&size=10&sort=hot"

# 发帖
curl -s -X POST "$BASE/posts" \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"forumId":1,"title":"Hello","content":"First post"}'
```

> 登录成功时返回 `{"token": "...", "user": {...}}`，即 `data.token`。

## 8. 版本与兼容性

- 当前 API 未使用 URL 版本号，`/api` 即当前稳定版本。
- 新增字段视为向后兼容；删除或改变语义会通过文档与变更记录说明。
- 旧 Java 实现已下线，其 `/forum/rs/api` 前缀仅存在于历史部署配置中。
