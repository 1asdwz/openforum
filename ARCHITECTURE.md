# 项目架构设计

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                     用户浏览器                               │
└────────────────────┬────────────────────────────────────────┘
                     │ HTTP/HTTPS
┌────────────────────▼────────────────────────────────────────┐
│                    Nginx 反向代理                            │
│  • 静态文件服务 (SPA)                                       │
│  • API 路由转发                                              │
│  • 负载均衡                                                  │
└────────────────────┬────────────────────────────────────────┘
                     │ REST API
┌────────────────────▼────────────────────────────────────────┐
│                  Spring Boot 后端                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Controller 层 (REST API)                │  │
│  │  • AuthController                                     │  │
│  │  • UserController                                     │  │
│  │  • SubForumController                                 │  │
│  │  • PostController                                     │  │
│  │  • AuthenticationController                           │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Service 层 (业务逻辑)                    │  │
│  │  • AuthService                                        │  │
│  │  • UserService                                        │  │
│  │  • SubForumService                                    │  │
│  │  • PostService                                        │  │
│  │  • CommentService                                     │  │
│  │  • UserAuthenticationService                          │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Repository 层 (数据访问)                     │  │
│  │  • UserRepository                                     │  │
│  │  • SubForumRepository                                 │  │
│  │  • PostRepository                                     │  │
│  │  • CommentRepository                                  │  │
│  │  • UserAuthenticationRepository                       │  │
│  │  • ForumMemberRepository                              │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Security & Config 层 (安全配置)              │  │
│  │  • JwtAuthenticationFilter                            │  │
│  │  • SecurityConfig                                     │  │
│  │  • JwtTokenProvider                                   │  │
│  └──────────────────────────────────────────────────────┘  │
└────────┬────────────────────────┬──────────────────────────┘
         │                        │
         │ JDBC                   │ Redis
         │                        │
┌────────▼────────┐      ┌────────▼──────────┐
│    MySQL 8.0    │      │    Redis 7.0      │
│                 │      │                   │
│  • 用户数据      │      │  • Session 缓存   │
│  • 论坛数据      │      │  • 热点数据缓存   │
│  • 文章评论      │      │  • 速率限制       │
│  • 认证记录      │      │                   │
└─────────────────┘      └───────────────────┘
```

## 数据库设计

### 核心表结构

#### users (用户表)
```
id (PK)
username (UNIQUE)
email (UNIQUE)
password
avatar
bio
is_active
created_at
updated_at
```

#### sub_forums (子论坛表)
```
id (PK)
slug (UNIQUE)
name
description
icon
custom_page_html (可选)
creator_id (FK -> users)
member_count
post_count
is_public
rules
required_auth_types
created_at
updated_at
```

#### posts (文章表)
```
id (PK)
title
content (LONGTEXT)
author_id (FK -> users)
forum_id (FK -> sub_forums)
upvotes
downvotes
comment_count
is_pinned
created_at
updated_at
```

#### comments (评论表)
```
id (PK)
content (LONGTEXT)
post_id (FK -> posts)
author_id (FK -> users)
parent_comment_id (FK -> comments, 可选)
upvotes
created_at
updated_at
```

#### user_authentications (用户认证表)
```
id (PK)
user_id (FK -> users)
type (ENUM)
auth_data (JSON)
status (ENUM: PENDING, APPROVED, REJECTED)
rejection_reason
created_at
reviewed_at
reviewed_by
```

#### forum_members (论坛成员表)
```
id (PK)
forum_id (FK -> sub_forums)
user_id (FK -> users)
joined_at
```

## API 设计规范

### RESTful 路由设计

```
认证相关
POST   /api/auth/register           # 注册
POST   /api/auth/login              # 登录

用户相关
GET    /api/users/{userId}          # 获取用户信息
PUT    /api/users/{userId}          # 更新用户信息
GET    /api/users/{userId}/forums   # 获取用户创建的论坛
GET    /api/users/{userId}/posts    # 获取用户发布的文章

论坛相关
POST   /api/forums                  # 创建论坛
GET    /api/forums/{slug}           # 获取论坛详情
GET    /api/forums/popular          # 获取热门论坛
GET    /api/forums/search           # 搜索论坛
PUT    /api/forums/{forumId}/rules  # 更新论坛规则
POST   /api/forums/{forumId}/join   # 加入论坛
POST   /api/forums/{forumId}/leave  # 离开论坛

文章相关
POST   /api/posts                   # 发布文章
GET    /api/posts/{postId}          # 获取文章详情
GET    /api/posts/forum/{forumId}   # 获取论坛的文章
GET    /api/posts/author/{userId}   # 获取用户的文章
PUT    /api/posts/{postId}          # 编辑文章
DELETE /api/posts/{postId}          # 删除文章
POST   /api/posts/{postId}/upvote   # 点赞
POST   /api/posts/{postId}/downvote # 点踩
POST   /api/posts/{postId}/pin      # 置顶

评论相关
POST   /api/comments                # 发布评论
GET    /api/comments/post/{postId}  # 获取文章评论
DELETE /api/comments/{commentId}    # 删除评论
POST   /api/comments/{commentId}/upvote # 评论点赞

认证相关
POST   /api/authentications         # 提交认证
GET    /api/authentications/me      # 获取个人认证
GET    /api/authentications/me/approved # 获取已通过认证
GET    /api/authentications/pending # 获取待审核认证（管理员）
POST   /api/authentications/{id}/approve # 审核通过
POST   /api/authentications/{id}/reject  # 审核拒绝
```

## 认证流程

```
1. 用户注册/登录
   ↓
2. 服务器生成 JWT Token
   ↓
3. 客户端存储 Token (localStorage)
   ↓
4. 客户端在请求头中附带 Token
   Authorization: Bearer {token}
   ↓
5. JwtAuthenticationFilter 验证 Token
   ↓
6. 提取用户信息，允许请求继续
```

## 权限控制流程

```
用户请求 → 权限检查
   ↓
是否需要认证？
   ├─ 否 → 继续
   └─ 是 → 检查 Token
      ├─ 有效 → 继续
      └─ 无效 → 返回 401

是否需要特定角色？
   ├─ 否 → 继续
   └─ 是 → 检查用户认证
      ├─ 满足 → 执行操作
      └─ 不满足 → 返回 403
```

## 缓存策略

```
热点数据缓存 (Redis)
├─ 热门论坛列表 (1小时)
├─ 用户信息 (30分钟)
├─ 热门文章 (2小时)
├─ 论坛规则 (24小时)
└─ 用户认证状态 (15分钟)

缓存失效场景
├─ 数据更新时立即失效
├─ 定时过期失效
└─ 手动清除
```

## 错误处理

```
异常层级
├─ ApiException (业务异常)
│  ├─ NotFoundException (404)
│  ├─ UnauthorizedException (401)
│  ├─ ForbiddenException (403)
│  └─ BadRequestException (400)
├─ DataIntegrityViolationException (数据完整性)
└─ Exception (系统异常)

响应格式
{
  "code": 200,
  "message": "Success",
  "data": {}
}

错误响应
{
  "code": 400,
  "message": "用户已存在"
}
```

## 安全性考虑

```
认证安全
├─ JWT Token 有过期时间 (24小时)
├─ 密码使用 BCrypt 加密
├─ Token 存储在 httpOnly Cookie (可选)
└─ HTTPS 传输

授权安全
├─ 检查用户权限
├─ 验证资源所有权
├─ 速率限制防止滥用
└─ 审计日志记录

数据安全
├─ 参数验证和清理
├─ SQL 注入防护 (JPA)
├─ CORS 配置
└─ 敏感信息加密
```

## 扩展性设计

```
可扩展模块
├─ AI 审核集成点 (UserAuthenticationService)
├─ 推荐算法接口 (SubForumService)
├─ 评论系统框架 (CommentService)
├─ 消息通知系统 (待实现)
├─ 搜索引擎集成 (Elasticsearch)
└─ 文件存储集成 (OSS/S3)

插件架构
├─ 内容审核插件
├─ 推荐系统插件
├─ 支付系统插件
└─ 第三方登录插件
```

---

这个架构设计充分考虑了可扩展性、安全性和性能，支持未来的功能增强和大规模部署。
