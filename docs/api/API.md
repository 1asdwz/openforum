# OpenForum API 接口手册

> 本文档基于现有 Spring Boot(Java) 实现整理，供后续以 Rust(Axum) 重写时逐一对照。
> 规范文件见同目录 [`openapi.yaml`](./openapi.yaml)。

## 1. 概述

OpenForum 是一个论坛后端，提供用户认证、子论坛（板块）、帖子、评论、媒体、多类型多等级的用户认证中心、
以及管理后台与插件管理等能力。

- 技术栈：Spring Boot + Spring Security + JWT(HS256) + JPA(MySQL) + MinIO(可选) + Redis + 插件化审核/推荐。
- 统一响应信封：`{"code":200,"message":"ok","data":<T>}`。
- 所有业务接口位于 servlet `context-path` `/api` 之下。

## 2. Base URL 与部署前缀

| 部署形态 | 示例 Base URL | 说明 |
| --- | --- | --- |
| Java 版（经网关） | `https://<host>/forum/api/` | 后端自身 `context-path` 为 `/api` |
| Rust 版（经网关，目标） | `https://<host>/forum/rs/api/` | 与 Java 版路径保持一致 |

两版的业务路径（`/auth/login`、`/posts` 等）完全一致，仅网关前缀不同。
因此本手册及 `openapi.yaml` 中统一以 `/api` 为服务根，`paths` 使用 `/auth/login` 这类相对路径。
下文各接口路径均为「相对于服务根 `/api`」的写法，调用时需拼接实际 Base URL。

## 3. 统一响应格式

所有 JSON 接口返回 `ApiResponse`：

```json
{
  "code": 200,
  "message": "ok",
  "data": { }
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| code | int | 成功固定为 `200`；出错时为对应 HTTP 状态码 |
| message | string | 成功为 `ok`；出错为中文错误文案 |
| data | T / null | 业务数据；出错时为 `null` |

出错示例：

```json
{ "code": 403, "message": "需要坛主或管理员权限", "data": null }
```

> 例外：`GET /media/{id}` 与 `GET /authentications/media/{mediaId}` 直接返回二进制流，不套用该信封。

## 4. 分页格式

分页接口的 `data` 为 `PageResponse`：

```json
{
  "content": [],
  "page": 0,
  "size": 20,
  "totalElements": 0,
  "totalPages": 0
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| content | T[] | 当前页记录 |
| page | int | 当前页码（从 **0** 开始） |
| size | int | 每页大小 |
| totalElements | long | 总记录数 |
| totalPages | int | 总页数 |

## 5. 鉴权说明

### 5.1 获取与携带 JWT

- 通过 `POST /auth/register`、`POST /auth/login`、`POST /auth/google` 获取 `token`。
- 后续请求在请求头携带：

```
Authorization: Bearer <token>
```

- JWT 为 HS256 签名，Payload 含 `sub`（用户 ID）、`username`、`role`，默认有效期 **168 小时**
  （`openforum.jwt.expiration-hours`，可用环境变量 `JWT_EXPIRATION_HOURS` 覆盖）。
- Token 无效或缺失时，受保护接口返回 `401`。

### 5.2 权限分级

| 级别 | 说明 |
| --- | --- |
| 公开 | 无需登录 |
| 登录 | 需要有效 JWT |
| ADMIN | 需要 JWT 且 `role=ADMIN`，对应 `/admin/**` 及 `PUT /plugins/{key}/enabled` |
| 坛主/管理 | 需为该子论坛的 `OWNER`/`MODERATOR`，或系统管理员 |

### 5.3 需要 ADMIN 的接口清单

| 方法 | 路径 |
| --- | --- |
| GET | `/admin/authentications` |
| POST | `/admin/authentications/{id}/approve` |
| POST | `/admin/authentications/{id}/reject` |
| GET | `/admin/custom-pages` |
| POST | `/admin/custom-pages/{id}/approve` |
| POST | `/admin/custom-pages/{id}/reject` |
| GET | `/admin/users` |
| PUT | `/plugins/{key}/enabled` |

## 6. 错误码表

`code` 与 HTTP 状态码一致，源自 `ApiException` 与 `GlobalExceptionHandler`。

| code | 典型 message | 触发场景 |
| --- | --- | --- |
| 400 | 参数校验失败 / `字段名 校验文案` | 请求体字段校验失败（`MethodArgumentNotValidException`） |
| 400 | `IllegalArgumentException` 的 message | 非法参数 |
| 400 | 用户名已被占用 | 注册用户名重复 |
| 400 | 邮箱已被占用 | 注册邮箱重复 |
| 400 | Google 登录未配置 | 未配置 Google client-id |
| 400 | Google 账号邮箱未验证 | Google 邮箱未验证 |
| 400 | slug 只能包含小写字母、数字、-、_（2-80 位） | 创建子论坛 slug 非法 |
| 400 | slug 已存在 | 创建子论坛 slug 冲突 |
| 400 | html 不能为空 | 保存自定义页 HTML 为空 |
| 400 | 请先保存自定义页面 HTML | 未保存就提交审核 |
| 400 | 坛主不能退出自己的子论坛 | 坛主调用 leave |
| 400 | 投票值只能是 1、-1 或 0 | 投票值非法 |
| 400 | 父评论不存在 | 回复的父评论不存在 |
| 400 | 父评论不属于该帖子 | 父评论与帖子不匹配 |
| 400 | 该类型认证已存在，如需重新提交请使用重试接口 | 重复提交认证 |
| 400 | 仅被拒绝的认证或提升等级时可以重新提交 | 非法重试 |
| 400 | 未知认证类型: X | 认证类型非法 |
| 400 | 未知认证等级: X | 认证等级非法 |
| 400 | 非法的状态: X | status 参数非法 |
| 400 | 文件类型 kind 不能为空 / 未知的文件类型: X | 认证媒体 kind 非法 |
| 400 | 上传文件不能为空 | 上传空文件 |
| 400 | 文件存储未启用，无法上传媒体 | 未启用 MinIO 上传帖子媒体 |
| 400 | 文件存储未启用，无法上传认证材料 | 未启用 MinIO 上传认证材料 |
| 400 | 文件存储未启用 | 读取媒体时存储未启用 |
| 400 | 文件格式不支持，图片仅允许 jpeg/png/webp/gif，视频仅允许 mp4/webm | 帖子媒体格式不支持 |
| 400 | 视频格式不支持，仅允许 webm/mp4 / 图片格式不支持，仅允许 jpeg/png/webp | 认证媒体格式不支持 |
| 400 | 视频过大，上限 NMB / 图片过大，上限 NMB | 超出大小上限 |
| 400 | 读取上传文件失败 | 读取 multipart 失败 |
| 400 | 缺失活体挑战标识 / 活体挑战无效或不属于当前用户 | 活体挑战校验失败 |
| 400 | 活体挑战已使用，请重新开始视频认证 / 活体挑战已过期，请重新开始视频认证 | 挑战失效 |
| 400 | 缺少活体检测结果 / 活体检测未通过，请重新进行视频认证 | 活体检测失败 |
| 400 | 该等级需要上传证件材料 / 该等级需要完成视频认证 | 认证材料缺失 |
| 400 | 至少需要上传 1 张证件照片 | 无证件照 |
| 401 | 请先登录 | 未登录 |
| 401 | 账号或密码错误 | 登录失败 |
| 401 | Google 登录凭证无效 | Google 凭证无效 |
| 403 | 需要管理员权限 | 非 ADMIN 访问管理接口（service 层） |
| 403 | 账号已被禁用 | 账号被禁用 |
| 403 | 需要坛主或管理员权限 | 无板块管理权限 |
| 403 | 只有作者或坛主可以操作 | 编辑/删除帖子无权限 |
| 403 | 只有作者或坛主可以删除评论 | 删除评论无权限 |
| 403 | 需要完成认证: ... | 未满足板块认证门槛 |
| 403 | 无权操作该认证 | 操作他人认证记录 |
| 403 | 无权访问该媒体文件 | 读取他人认证媒体 |
| 404 | 用户不存在 | 用户不存在 |
| 404 | 子论坛不存在 | 子论坛不存在 |
| 404 | 帖子不存在 | 帖子不存在或已删除 |
| 404 | 评论不存在 | 评论不存在或已删除 |
| 404 | 认证记录不存在 | 认证记录不存在 |
| 404 | 媒体文件不存在 | 媒体不存在 |
| 404 | 插件不存在: X | 插件 key 不存在 |
| 404 | 资源不存在 | 静态资源/路径不存在（`NoResourceFoundException`） |
| 500 | 服务器内部错误 | 未捕获异常 |

## 7. 枚举取值表

| 枚举 | 取值 | 说明 |
| --- | --- | --- |
| UserRole | `USER`, `ADMIN` | 用户角色 |
| ForumRole | `OWNER`, `MODERATOR`, `MEMBER` | 子论坛成员角色 |
| AuthType | `REAL_NAME`, `ID_CARD`, `GENDER`, `AGE`, `NATIONALITY`, `DRIVER_LICENSE`, `MARRIAGE_CERT`, `PROPERTY_CERT`, `ASSETS`, `CUSTOM` | 认证类型 |
| AuthLevel | `LOW`(低), `MEDIUM`(中), `HIGH`(高), `EXTREME`(极高) | 认证等级（证据强度） |
| AuthStatus | `PENDING`, `APPROVED`, `REJECTED` | 认证状态 |
| CustomPageStatus | `DRAFT`, `PENDING`, `APPROVED`, `REJECTED` | 自定义页状态 |
| TemplateType | `DEFAULT`, `CUSTOM` | 子论坛模板类型 |
| MediaKind | `DOCUMENT`, `SELFIE`, `VIDEO` | 认证媒体类型 |
| ReviewMethod | `RULE`, `AI`, `MANUAL`, `SYSTEM` | 审核方式 |
| ReviewAction | `SUBMIT`, `APPROVE`, `REJECT`, `RETRY` | 审核动作 |
| VoteTargetType | `POST`, `COMMENT` | 投票目标类型 |
| PluginType | `REVIEW`, `DOCUMENT_REVIEW`, `RECOMMENDATION`, `CONTENT_BLOCK` | 插件类型 |

### 7.1 板块认证门槛语法（`requiredAuthTypes`）

`requiredAuthTypes` 为字符串数组，元素含义：

| 形式 | 含义 |
| --- | --- |
| `TYPE` | 该类型认证已通过且等级 ≥ `LOW`（兼容旧数据） |
| `TYPE:LEVEL` | 该类型认证已通过且等级 ≥ `LEVEL`，如 `REAL_NAME:HIGH` |
| `AGE_n` | `AGE` 认证已通过且年龄 ≥ n，如 `AGE_18` |

### 7.2 认证类型字段定义（`GET /authentications/types`）

| 类型 | 中文标签 | 字段(name/label/type) |
| --- | --- | --- |
| REAL_NAME | 实名认证 | `fullName`/真实姓名/text |
| ID_CARD | 身份证认证 | `idNumber`/身份证号/text |
| GENDER | 性别认证 | `gender`/性别/select |
| AGE | 年龄认证 | `age`/年龄/number |
| NATIONALITY | 国籍认证 | `nationality`/国籍/text |
| DRIVER_LICENSE | 驾照认证 | `licenseNumber`/驾照号/text |
| MARRIAGE_CERT | 婚姻认证 | `spouseName`/配偶姓名/text、`certificateNumber`/结婚证号/text |
| PROPERTY_CERT | 房产认证 | `certificateNumber`/产权证号/text、`propertyAddress`/房产地址/text |
| ASSETS | 资产认证 | `amount`/资产金额/number |
| CUSTOM | 自定义认证 | `description`/说明/textarea |

> 每个类型同时返回 `levels`（LOW/MEDIUM/HIGH/EXTREME 的 requirements 与 tips）。

## 8. 接口详情

> 下文路径均相对于服务根 `/api`；`Authorization` 行为 Bearer JWT。
> 「权限」列出该接口所需的最低要求；响应示例中的 `data` 为精简示例。

### 8.1 认证（Auth）

#### 8.1.1 注册

- 方法与路径：`POST /auth/register`
- 权限：公开
- 说明：注册成功后直接返回 token 与用户信息。

请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| username | string | 是 | 用户名，长度 3-50 |
| email | string | 是 | 邮箱 |
| password | string | 是 | 密码，长度 6-64 |

响应示例：

```json
{
  "code": 200,
  "message": "ok",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiJ9...",
    "user": {
      "id": 4,
      "username": "carol",
      "email": "carol@example.com",
      "avatar": null,
      "bio": null,
      "role": "USER",
      "createdAt": "2026-10-08T12:34:56",
      "authentications": []
    }
  }
}
```

典型错误：400「用户名已被占用」、400「邮箱已被占用」、400 参数校验失败。

#### 8.1.2 登录

- 方法与路径：`POST /auth/login`
- 权限：公开

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| usernameOrEmail | string | 是 | 用户名或邮箱 |
| password | string | 是 | 密码 |

响应：同 `AuthResponse`。典型错误：401「账号或密码错误」、403「账号已被禁用」。

#### 8.1.3 Google 登录

- 方法与路径：`POST /auth/google`
- 权限：公开

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| credential | string | 是 | Google ID Token |

响应：同 `AuthResponse`（首次登录自动创建账号）。典型错误：400「Google 登录未配置」、
400「Google 账号邮箱未验证」、401「Google 登录凭证无效」。

#### 8.1.4 当前用户

- 方法与路径：`GET /auth/me`
- 权限：登录

响应：`UserDTO`（含 `email` 与已通过的认证类型列表 `authentications`）。

### 8.2 用户（Users）

#### 8.2.1 用户公开资料

- 方法与路径：`GET /users/{id}`
- 权限：公开
- 参数：`id`（path，integer int64）用户 ID
- 响应：`UserDTO`，其中 `email` 为 `null`
- 典型错误：404「用户不存在」

#### 8.2.2 更新个人资料

- 方法与路径：`PUT /users/me`
- 权限：登录
- 说明：仅更新非 null 字段。

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| avatar | string | 否 | 头像 URL |
| bio | string | 否 | 简介 |

- 响应：更新后的 `UserDTO`

#### 8.2.3 用户的帖子

- 方法与路径：`GET /users/{id}/posts`
- 权限：公开

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| id | path | int64 | 是 | - | 用户 ID |
| sort | query | string | 否 | `new` | `new`/`top`/`hot` |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |

- 响应：`PageResponse<PostSummaryDTO>`

#### 8.2.4 用户的评论

- 方法与路径：`GET /users/{id}/comments`
- 权限：公开

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| id | path | int64 | 是 | - | 用户 ID |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |

- 响应：`PageResponse<CommentDTO>`，每条评论带 `postTitle`（所属帖子标题），`children` 为空。

### 8.3 子论坛（Forums）

#### 8.3.1 列表

- 方法与路径：`GET /forums`
- 权限：公开

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |
| sort | query | string | 否 | `hot` | `new` 按创建时间倒序；其他值按推荐分数 |
| q | query | string | 否 | - | 匹配 name 或 slug（不区分大小写） |

- 响应：`PageResponse<ForumSummaryDTO>`

#### 8.3.2 热门

- 方法与路径：`GET /forums/popular`
- 权限：公开
- 参数：`limit`（query，int，默认 `10`）
- 响应：`ForumSummaryDTO[]`（按成员数倒序，再按帖子数倒序）

#### 8.3.3 推荐

- 方法与路径：`GET /forums/recommended`
- 权限：公开
- 参数：`limit`（query，int，默认 `10`）
- 响应：`ForumSummaryDTO[]`（按推荐插件分数；登录用户已加入的板块加权）

#### 8.3.4 创建

- 方法与路径：`POST /forums`
- 权限：登录
- 说明：`slug` 需匹配 `^[a-z0-9_-]{2,80}$`；创建者自动成为 OWNER。

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| slug | string | 是 | 唯一标识（小写字母/数字/-/_，2-80） |
| name | string | 是 | 名称 |
| description | string | 否 | 简介 |
| icon | string | 否 | 图标 |
| themeColor | string | 否 | 主题色，缺省 `#FF4500` |
| templateType | string | 否 | `DEFAULT`/`CUSTOM`，缺省 `DEFAULT` |
| templateConfig | string | 否 | 模板配置（JSON 字符串） |
| rules | string | 否 | 版规 |
| requiredAuthTypes | string[] | 否 | 认证门槛，见 7.1 |

- 响应：`ForumDetailDTO`
- 典型错误：400「slug 只能包含…」、400「slug 已存在」、401「请先登录」

#### 8.3.5 详情（按 slug）

- 方法与路径：`GET /forums/{slug}`
- 权限：公开
- 参数：`slug`（path，string）
- 响应：`ForumDetailDTO`；`customPageHtml` 仅在自定义页 `APPROVED` 时返回
- 典型错误：404「子论坛不存在」

#### 8.3.6 更新

- 方法与路径：`PUT /forums/{id}`
- 权限：登录 + 坛主/管理员
- 参数：`id`（path，int64）
- 请求体：同「创建」去掉 `slug`（字段均可选，仅更新非 null 字段）
- 响应：`ForumDetailDTO`
- 典型错误：403「需要坛主或管理员权限」

#### 8.3.7 读取自定义页面

- 方法与路径：`GET /forums/{id}/custom-page`
- 权限：登录 + 坛主/管理员
- 参数：`id`（path，int64）
- 响应：`CustomPageDTO`
- 典型错误：401「请先登录」、403「需要坛主或管理员权限」

#### 8.3.8 保存自定义页面（草稿）

- 方法与路径：`PUT /forums/{id}/custom-page`
- 权限：登录 + 坛主/管理员
- 参数：`id`（path，int64）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| html | string | 是 | 页面 HTML（会去除危险标签/事件） |

- 说明：保存后状态置为 `DRAFT`，清空审核备注。
- 响应：`CustomPageDTO`
- 典型错误：400「html 不能为空」、403「需要坛主或管理员权限」

#### 8.3.9 提交自定义页面审核

- 方法与路径：`POST /forums/{id}/custom-page/submit`
- 权限：登录 + 坛主/管理员
- 参数：`id`（path，int64）
- 说明：状态置为 `PENDING`。
- 响应：`CustomPageDTO`
- 典型错误：400「请先保存自定义页面 HTML」

#### 8.3.10 加入子论坛

- 方法与路径：`POST /forums/{id}/join`
- 权限：登录
- 参数：`id`（path，int64）
- 响应：`data` 为 `null`

#### 8.3.11 退出子论坛

- 方法与路径：`POST /forums/{id}/leave`
- 权限：登录
- 参数：`id`（path，int64）
- 响应：`data` 为 `null`
- 典型错误：400「坛主不能退出自己的子论坛」

#### 8.3.12 成员列表

- 方法与路径：`GET /forums/{id}/members`
- 权限：公开
- 参数：`id`（path，int64）
- 响应：`MemberDTO[]`（按加入时间升序）
- 典型错误：404「子论坛不存在」

### 8.4 帖子（Posts）

#### 8.4.1 列表

- 方法与路径：`GET /posts`
- 权限：公开

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| forumId | query | int64 | 否 | - | 子论坛 ID |
| q | query | string | 否 | - | 标题关键字（非空时忽略 forumId） |
| sort | query | string | 否 | `hot` | `new`/`top`/`hot` |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |

- 说明：置顶帖始终排在最前。
- 响应：`PageResponse<PostSummaryDTO>`

#### 8.4.2 发帖

- 方法与路径：`POST /posts`
- 权限：登录（并需通过目标板块认证门槛）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| forumId | int64 | 是 | 目标子论坛 ID |
| title | string | 是 | 标题（最长 300） |
| content | string | 是 | 正文 |

- 响应：`PostDetailDTO`
- 典型错误：403「需要完成认证: ...」、404「子论坛不存在」

#### 8.4.3 详情

- 方法与路径：`GET /posts/{id}`
- 权限：公开
- 参数：`id`（path，int64）
- 响应：`PostDetailDTO`（未登录时 `myVote=0`、`canVote=false`）
- 典型错误：404「帖子不存在」

`PostDetailDTO` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | int64 | 帖子 ID |
| forumId | int64 | 子论坛 ID |
| forumSlug | string | 子论坛 slug |
| forumName | string | 子论坛名称 |
| title | string | 标题 |
| content | string | 正文 |
| upvotes | int | 赞同数 |
| downvotes | int | 反对数 |
| score | int | 得分（upvotes - downvotes） |
| commentCount | int | 评论数 |
| isPinned | boolean | 是否置顶 |
| author | AuthorDTO | 作者（id/username/avatar） |
| canVote | boolean | 当前用户是否可投票 |
| myVote | int | 当前用户投票值（1/-1/0） |
| createdAt | datetime | 创建时间 |
| updatedAt | datetime | 更新时间 |

#### 8.4.4 更新

- 方法与路径：`PUT /posts/{id}`
- 权限：登录 + 作者或坛主/管理员
- 参数：`id`（path，int64）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| title | string | 否 | 标题（非空才更新） |
| content | string | 否 | 正文 |

- 响应：`PostDetailDTO`
- 典型错误：403「只有作者或坛主可以操作」

#### 8.4.5 删除（软删除）

- 方法与路径：`DELETE /posts/{id}`
- 权限：登录 + 作者或坛主/管理员
- 参数：`id`（path，int64）
- 响应：`data` 为 `null`；同时清除该帖投票

#### 8.4.6 投票

- 方法与路径：`POST /posts/{id}/vote`
- 权限：登录（并需通过目标板块认证门槛）
- 参数：`id`（path，int64）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| value | int | 是 | `1` 赞同 / `-1` 反对 / `0` 取消 |

- 响应：`PostDetailDTO`
- 典型错误：400「投票值只能是 1、-1 或 0」

#### 8.4.7 置顶切换

- 方法与路径：`POST /posts/{id}/pin`
- 权限：登录 + 坛主/管理员
- 参数：`id`（path，int64）
- 说明：切换 `isPinned`。
- 响应：`PostDetailDTO`
- 典型错误：403「需要坛主或管理员权限」

### 8.5 评论（Comments）

#### 8.5.1 评论树

- 方法与路径：`GET /comments/post/{postId}`
- 权限：公开
- 参数：`postId`（path，int64）
- 响应：`CommentDTO[]`（顶级评论数组，`children` 为嵌套子回复，按创建时间升序）

`CommentDTO` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | int64 | 评论 ID |
| postId | int64 | 帖子 ID |
| authorId | int64 | 作者 ID |
| authorName | string | 作者用户名 |
| authorAvatar | string | 作者头像 |
| parentId | int64 | 父评论 ID |
| content | string | 内容 |
| upvotes | int | 赞同数 |
| downvotes | int | 反对数 |
| score | int | 得分 |
| createdAt | datetime | 创建时间 |
| children | CommentDTO[] | 子回复 |
| postTitle | string | 所属帖子标题（仅用户评论列表返回） |

#### 8.5.2 发表评论

- 方法与路径：`POST /comments`
- 权限：登录

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| postId | int64 | 是 | 帖子 ID |
| content | string | 是 | 内容 |
| parentId | int64 | 否 | 父评论 ID |

- 响应：`CommentDTO`
- 典型错误：400「父评论不存在」、400「父评论不属于该帖子」、404「帖子不存在」

#### 8.5.3 删除评论（软删除）

- 方法与路径：`DELETE /comments/{id}`
- 权限：登录 + 作者或坛主/管理员
- 参数：`id`（path，int64）
- 响应：`data` 为 `null`
- 典型错误：403「只有作者或坛主可以删除评论」、404「评论不存在」

#### 8.5.4 评论投票

- 方法与路径：`POST /comments/{id}/vote`
- 权限：登录
- 参数：`id`（path，int64）
- 请求体：`{ "value": 1 }`（`1`/`-1`/`0`）
- 响应：`CommentDTO`
- 典型错误：400「投票值只能是 1、-1 或 0」

### 8.6 媒体（Media）

#### 8.6.1 上传帖子媒体

- 方法与路径：`POST /media`，`Content-Type: multipart/form-data`
- 权限：登录
- 表单字段：`file`（binary，必填）
- 支持类型：图片 `image/jpeg`、`image/png`、`image/webp`、`image/gif`；视频 `video/mp4`、`video/webm`
- 响应：

```json
{ "code": 200, "message": "ok", "data": { "id": 12, "contentType": "image/png", "sizeBytes": 20480 } }
```

- 典型错误：400「文件存储未启用，无法上传媒体」、400 格式/大小超限

#### 8.6.2 读取媒体

- 方法与路径：`GET /media/{id}`
- 权限：公开
- 参数：`id`（path，int64）
- 响应：原始二进制流（`Content-Type` 按存储类型；`Cache-Control: public, max-age=31536000, immutable`）
- 典型错误：404「媒体文件不存在」、400「文件存储未启用」

### 8.7 认证中心（Authentications）

认证中心支持「类型 × 等级」的正交模型：`AuthType` 表示认证什么，`AuthLevel` 表示证据强度
（`LOW`/`MEDIUM`/`HIGH`/`EXTREME`）。

#### 8.7.1 认证类型元数据

- 方法与路径：`GET /authentications/types`
- 权限：公开
- 响应：`AuthTypeMetaDTO[]`（含 `fields` 与 `levels`，见 7.2）

#### 8.7.2 申领活体检测挑战

- 方法与路径：`GET /authentications/challenge`
- 权限：登录
- 说明：随机抽取 3 个动作（`BLINK`/`OPEN_MOUTH`/`TURN_LEFT`/`TURN_RIGHT`/`NOD`），默认 10 分钟有效、一次性。
- 响应示例：

```json
{
  "code": 200,
  "message": "ok",
  "data": {
    "challengeId": "8f3c...",
    "actions": ["BLINK", "TURN_LEFT", "NOD"],
    "nonce": "a1b2c3...",
    "expiresAt": "2026-10-08T12:44:56"
  }
}
```

#### 8.7.3 上传认证材料

- 方法与路径：`POST /authentications/media`，`Content-Type: multipart/form-data`
- 权限：登录

| 参数 | 位置 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- | --- |
| kind | query | string | 是 | `DOCUMENT`/`SELFIE`/`VIDEO` |
| file | formData | binary | 是 | 材料文件 |

- 限制：图片仅 `image/jpeg`/`image/png`/`image/webp`（默认 ≤10MB）；视频仅 `video/webm`/`video/mp4`（默认 ≤50MB）。
- 响应：

```json
{ "code": 200, "message": "ok", "data": { "mediaId": 30, "kind": "DOCUMENT", "contentType": "image/jpeg", "sizeBytes": 102400 } }
```

- 典型错误：400「文件类型 kind 不能为空」、400「未知的文件类型: X」、400「文件存储未启用，无法上传认证材料」

#### 8.7.4 读取认证材料

- 方法与路径：`GET /authentications/media/{mediaId}`
- 权限：登录（仅归属者或管理员）
- 参数：`mediaId`（path，int64）
- 响应：原始二进制流（`Content-Disposition: inline`）
- 典型错误：403「无权访问该媒体文件」、404「媒体文件不存在」

#### 8.7.5 提交认证

- 方法与路径：`POST /authentications`
- 权限：登录
- 说明：结构化审核流水线：结构校验 →（LOW 直接规则引擎）→ 专业审核插件 →
  自动/人工决策；HIGH 及以上需要视频活体挑战；EXTREME 需要移动 App 防伪录制。

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| type | string | 是 | AuthType |
| level | string | 否 | AuthLevel，缺省 `LOW` |
| data | object | 否 | 业务字段（如 `fullName`、`age`） |
| mediaIds | int64[] | 否 | 已上传材料媒体 ID（MEDIUM 及以上必填） |
| videoMediaId | int64 | 否 | 视频媒体 ID（HIGH 及以上必填） |
| challengeId | string | 否 | 活体挑战 ID（HIGH 及以上必填） |
| liveness | object | 否 | 活体检测结果 |
| attestation | object | 否 | 防伪录制结果（EXTREME 必填） |

- 响应示例：

```json
{
  "code": 200,
  "message": "ok",
  "data": {
    "id": 7,
    "userId": 2,
    "type": "REAL_NAME",
    "typeLabel": "实名认证",
    "status": "APPROVED",
    "reviewMethod": "RULE",
    "confidence": 0.95,
    "reason": "规则校验通过",
    "rejectionReason": null,
    "authData": { "fullName": "爱丽丝" },
    "createdAt": "2026-10-08T12:40:00",
    "reviewedAt": "2026-10-08T12:40:01",
    "level": "LOW",
    "media": [],
    "liveness": null,
    "attestation": null
  }
}
```

- 典型错误：400「该类型认证已存在，如需重新提交请使用重试接口」、400「未知认证类型: X」、
  400「该等级需要上传证件材料」、400「该等级需要完成视频认证」、400「至少需要上传 1 张证件照片」

#### 8.7.6 我的认证记录

- 方法与路径：`GET /authentications/me`
- 权限：登录
- 响应：`AuthenticationDTO[]`（按创建时间倒序）

#### 8.7.7 我已通过的认证类型

- 方法与路径：`GET /authentications/me/approved`
- 权限：登录
- 响应：`string[]`（已 APPROVED 的类型名，升序）

#### 8.7.8 重新提交认证

- 方法与路径：`POST /authentications/{id}/retry`
- 权限：登录（仅记录归属者）
- 参数：`id`（path，int64）
- 请求体：与提交相同，字段均可选（缺省沿用原记录）
- 响应：`AuthenticationDTO`
- 典型错误：403「无权操作该认证」、400「仅被拒绝的认证或提升等级时可以重新提交」、404「认证记录不存在」

### 8.8 管理后台（Admin，需 ADMIN）

#### 8.8.1 认证审核列表

- 方法与路径：`GET /admin/authentications`

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| status | query | string | 否 | `PENDING` | AuthStatus |
| level | query | string | 否 | - | AuthLevel 精确过滤 |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |

- 响应：`PageResponse<AuthenticationDTO>`（按创建时间升序）

#### 8.8.2 通过认证

- 方法与路径：`POST /admin/authentications/{id}/approve`
- 参数：`id`（path，int64）
- 请求体：`{ "reason": "..." }`（可省略，缺省「管理员通过」）
- 响应：`AuthenticationDTO`

#### 8.8.3 驳回认证

- 方法与路径：`POST /admin/authentications/{id}/reject`
- 参数：`id`（path，int64）
- 请求体：`{ "reason": "..." }`（可省略，缺省「管理员驳回」，写入 `rejectionReason`）
- 响应：`AuthenticationDTO`

#### 8.8.4 自定义页审核列表

- 方法与路径：`GET /admin/custom-pages`

| 参数 | 位置 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- | --- |
| status | query | string | 否 | `PENDING` | CustomPageStatus |
| page | query | int | 否 | `0` | 页码 |
| size | query | int | 否 | `20` | 每页大小 |

- 响应：`PageResponse<CustomPageDTO>`（按更新时间倒序）
- 典型错误：400「非法的状态: X」

#### 8.8.5 通过自定义页

- 方法与路径：`POST /admin/custom-pages/{id}/approve`
- 参数：`id`（path，int64，子论坛 ID）
- 说明：状态置 `APPROVED`，子论坛模板类型置 `CUSTOM`
- 响应：`CustomPageDTO`

#### 8.8.6 驳回自定义页

- 方法与路径：`POST /admin/custom-pages/{id}/reject`
- 参数：`id`（path，int64）
- 请求体：`{ "reason": "..." }`（可省略，缺省「管理员驳回」）
- 响应：`CustomPageDTO`

#### 8.8.7 用户列表

- 方法与路径：`GET /admin/users`
- 参数：`page`（默认 `0`）、`size`（默认 `20`）
- 响应：`PageResponse<UserDTO>`（按创建时间倒序，`email` 为 `null`）

### 8.9 插件（Plugins）

#### 8.9.1 插件列表

- 方法与路径：`GET /plugins`
- 权限：公开
- 响应：`PluginDTO[]`（`key`/`name`/`type`/`enabled`/`description`）

#### 8.9.2 启用/停用插件

- 方法与路径：`PUT /plugins/{key}/enabled`
- 权限：ADMIN
- 参数：`key`（path，string）
- 请求体：`{ "enabled": true }`
- 响应：`PluginDTO`
- 典型错误：404「插件不存在: X」

## 9. 种子账号（DataSeeder）

首次启动（且库中子论坛为空）时自动写入以下演示账号与 5 个演示板块：

| 用户名 | 密码 | 角色 | 说明 |
| --- | --- | --- | --- |
| admin | admin123 | ADMIN | 站点管理员 |
| alice | alice123 | USER | 普通用户（已通过 REAL_NAME、AGE 认证） |
| bob | bob123 | USER | 普通用户（已通过 REAL_NAME；NATIONALITY 待审） |

演示板块：`general`（综合讨论区）、`tech`（技术交流）、`life`（生活百态）、
`adult`（成人话题，需 `REAL_NAME,AGE_18`）、`creative`（创意工坊）。

> admin 账号可用于测试管理后台接口。

## 10. 常见调用流程示例

以下示例以 `BASE=https://<host>/forum/api`（Rust 版为 `/forum/rs/api`）为前缀。

### 10.1 注册 → 登录 → 发帖 → 投票

```bash
# 1) 注册（可直接获得 token）
curl -s -X POST "$BASE/auth/register" \
  -H 'Content-Type: application/json' \
  -d '{"username":"carol","email":"carol@example.com","password":"carol123"}'

# 2) 登录（也可用用户名或邮箱）
TOKEN=$(curl -s -X POST "$BASE/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"carol","password":"carol123"}' | jq -r .data.token)

# 3) 查看板块，拿到 forumId（如 general）
curl -s "$BASE/forums?q=general" -H "Authorization: Bearer $TOKEN"

# 4) 发帖
curl -s -X POST "$BASE/posts" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"forumId":1,"title":"大家好","content":"我的第一条帖子"}'

# 5) 给帖子投票（1 赞同 / -1 反对 / 0 取消）
curl -s -X POST "$BASE/posts/1/vote" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"value":1}'
```

### 10.2 提交实名认证（LOW 等级，仅需填写信息）

```bash
# 提交（LOW 直接走规则引擎，通常立即 APPROVED）
curl -s -X POST "$BASE/authentications" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"type":"REAL_NAME","level":"LOW","data":{"fullName":"卡罗尔"}}'
```

### 10.3 提交 HIGH 等级认证（含材料与活体挑战）

```bash
# 1) 申领活体挑战
curl -s "$BASE/authentications/challenge" -H "Authorization: Bearer $TOKEN"
#    → 记下 challengeId 与 actions

# 2) 上传证件材料
curl -s -X POST "$BASE/authentications/media?kind=DOCUMENT" \
  -H "Authorization: Bearer $TOKEN" -F 'file=@id-card.jpg'
#    → 记下 mediaId

# 3) 提交（videoMediaId / challengeId / liveness 按前端采集结果填写）
curl -s -X POST "$BASE/authentications" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{
        "type":"ID_CARD",
        "level":"HIGH",
        "data":{"idNumber":"110101199001011234"},
        "mediaIds":[30],
        "videoMediaId":31,
        "challengeId":"8f3c...",
        "liveness":{"passed":true,"actions":[{"action":"BLINK","passed":true}]}
      }'
```

### 10.4 管理后台审核

```bash
# 使用 admin 登录获取 ADMIN token
ADMIN=$(curl -s -X POST "$BASE/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"admin","password":"admin123"}' | jq -r .data.token)

# 待审认证列表
curl -s "$BASE/admin/authentications?status=PENDING" -H "Authorization: Bearer $ADMIN"

# 通过 / 驳回
curl -s -X POST "$BASE/admin/authentications/7/approve" \
  -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' -d '{"reason":"材料清晰"}'
curl -s -X POST "$BASE/admin/authentications/8/reject" \
  -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' -d '{"reason":"照片模糊"}'
```

---

> 附：本文档与 `openapi.yaml` 中的字段、错误文案、默认值均以 Java 源码为准。
> 若 Java 实现调整，请同步更新这两个文件。







