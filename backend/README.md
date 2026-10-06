# OpenForum 后端

Java 25 + Spring Boot 4.1.1 构建的开放论坛系统后端

## 项目特性

- ☕ Java 25 最新语言特性
- 🚀 Spring Boot 4.1.1 生产级框架
- 🔐 Spring Security + JWT 认证
- 💾 Spring Data JPA + MySQL
- ⚡ Redis 缓存和会话管理
- 📚 RESTful API 设计

## 环境要求

- Java 25+
- Maven 3.8+
- MySQL 8.0+
- Redis (可选)

## 快速开始

### 数据库初始化

```sql
CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### 修改配置

编辑 `src/main/resources/application.yml`:

```yaml
spring:
  datasource:
    url: jdbc:mysql://localhost:3306/openforum
    username: root
    password: your_password
  redis:
    host: localhost
    port: 6379
  jwt:
    secret: your-secret-key
```

### 编译运行

```bash
# 编译
mvn clean compile

# 打包
mvn clean package

# 运行
java -jar target/openforum-backend-0.1.0.jar
```

或直接运行：

```bash
mvn spring-boot:run
```

服务访问地址：http://localhost:8080/api

## 项目结构

```
src/main/java/com/openforum/
├── config/                    # 配置类
│   ├── SecurityConfig.java   # Spring Security 配置
│   ├── WebConfig.java        # Web 配置
│   ├── JwtAuthenticationFilter.java # JWT 过滤器
│   └── ...
├── controller/               # API 控制器
│   ├── AuthController.java       # 认证相关
│   ├── UserController.java       # 用户相关
│   ├── SubForumController.java   # 论坛相关
│   ├── PostController.java       # 文章相关
│   ├── AuthenticationController.java # 用户认证相关
│   └── ...
├── service/                  # 业务逻辑
│   ├── AuthService.java
│   ├── UserService.java
│   ├── SubForumService.java
│   ├── PostService.java
│   ├── UserAuthenticationService.java
│   └── ...
├── repository/               # 数据访问层
│   ├── UserRepository.java
│   ├── SubForumRepository.java
│   ├── PostRepository.java
│   ├── UserAuthenticationRepository.java
│   ├── ForumMemberRepository.java
│   ├── CommentRepository.java
│   └── ...
├── entity/                   # 实体类
│   ├── User.java
│   ├── SubForum.java
│   ├── Post.java
│   ├── Comment.java
│   ├── UserAuthentication.java
│   ├── ForumMember.java
│   └── ...
├── dto/                      # 数据传输对象
│   ├── auth/
│   ├── user/
│   ├── forum/
│   ├── post/
│   ├── authentication/
│   └── ...
├── exception/                # 异常处理
│   ├── ApiException.java
│   └── GlobalExceptionHandler.java
├── util/                     # 工具类
│   └── JwtTokenProvider.java
└── OpenForumApplication.java # 启动类
```

## API 文档

### 认证相关

#### 用户注册
```
POST /api/auth/register
Content-Type: application/json

{
  "username": "user123",
  "email": "user@example.com",
  "password": "password123",
  "confirmPassword": "password123"
}
```

#### 用户登录
```
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

### 论坛相关

#### 创建论坛
```
POST /api/forums
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Python讨论",
  "slug": "python",
  "description": "Python编程讨论",
  "rules": "遵守社区规则"
}
```

#### 获取论坛详情
```
GET /api/forums/{slug}
```

#### 获取热门论坛
```
GET /api/forums/popular?page=0&size=10
```

### 文章相关

#### 发布文章
```
POST /api/posts?forumId=1
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Python异步编程入门",
  "content": "文章内容..."
}
```

#### 获取文章列表
```
GET /api/posts/forum/{forumId}?page=0&size=10
```

#### 点赞/点踩文章
```
POST /api/posts/{postId}/upvote
POST /api/posts/{postId}/downvote
```

### 认证相关

#### 提交认证信息
```
POST /api/authentications
Authorization: Bearer {token}
Content-Type: application/json

{
  "type": "ID_CARD",
  "authData": "认证数据..."
}
```

#### 获取用户认证状态
```
GET /api/authentications/me
Authorization: Bearer {token}
```

## 数据库设计

### 用户表 (users)
- id: 主键
- username: 用户名
- email: 邮箱
- password: 密码
- avatar: 头像URL
- bio: 个人简介
- is_active: 是否激活
- created_at: 创建时间
- updated_at: 更新时间

### 论坛表 (sub_forums)
- id: 主键
- slug: 唯一标识
- name: 论坛名称
- description: 描述
- creator_id: 创建者ID
- member_count: 成员数
- post_count: 文章数
- is_public: 是否公开
- rules: 规则
- required_auth_types: 要求的认证类型
- created_at: 创建时间
- updated_at: 更新时间

### 文章表 (posts)
- id: 主键
- title: 标题
- content: 内容
- author_id: 作者ID
- forum_id: 论坛ID
- upvotes: 点赞数
- downvotes: 点踩数
- comment_count: 评论数
- is_pinned: 是否置顶
- created_at: 创建时间
- updated_at: 更新时间

### 用户认证表 (user_authentications)
- id: 主键
- user_id: 用户ID
- type: 认证类型
- auth_data: 认证数据
- status: 审核状态
- rejection_reason: 拒绝原因
- created_at: 创建时间
- reviewed_at: 审核时间
- reviewed_by: 审核人ID

## 核心功能实现

### JWT 认证流程

1. 用户注册/登录时获取 Token
2. 客户端在请求头中携带 Token: `Authorization: Bearer {token}`
3. JwtAuthenticationFilter 验证 Token
4. 提取用户信息并设置到请求中

### 权限控制

- 用户创建的论坛只有本人可编辑
- 用户发布的文章只有本人可编辑/删除
- 论坛创建者可置顶文章
- 部分论坛要求特定认证才能发表

### AI 认证审核 (占位符)

`UserAuthenticationService` 中预留了审核逻辑接口，可集成：
- 阿里云内容安全
- 腾讯云内容安全
- 自定义 AI 模型

## 常见问题

### 如何修改密码加密方式？

编辑 `SecurityConfig.java` 中的 `passwordEncoder()` 方法

### 如何集成 OAuth 登录？

在 `AuthController` 中添加新的登录端点

### 如何实现评论功能？

已有 `Comment` 实体和 `CommentRepository`，可创建 `CommentService` 和 `CommentController`

### 如何使用 Redis 缓存？

使用 `@Cacheable`、`@CacheEvict` 等注解标注 Service 方法

## 打包和部署

### Docker 部署

```dockerfile
FROM maven:3.9-eclipse-temurin-25 AS builder
WORKDIR /app
COPY . .
RUN mvn clean package -DskipTests

FROM eclipse-temurin:25-jre
WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar
EXPOSE 8080
CMD ["java", "-jar", "app.jar"]
```

### 服务器部署

1. 准备 Java 25 环境
2. 部署 MySQL 和 Redis
3. 修改 `application.yml` 配置
4. 运行 jar 文件

## 许可证

MIT
