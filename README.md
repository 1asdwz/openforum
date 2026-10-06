# OpenForum 开放式论坛系统

一个类似 Reddit 的开放式论坛平台，支持子论坛创建、用户认证、权限控制和插件扩展。

## 🎯 核心功能

### 1. 子论坛系统
- ✅ 用户可创建自己的子论坛
- ✅ 坛主可编辑论坛规则和页面
- ✅ 支持使用默认模板或自定义 HTML
- ✅ 热门论坛推荐和个性化推荐

### 2. 用户信息认证
支持多种认证类型：
- ✅ 实名认证
- ✅ 身份证认证
- ✅ 驾驶证认证
- ✅ 结婚证认证
- ✅ 房产证认证
- ✅ 资产认证
- ✅ 国籍认证
- ✅ 年龄认证

认证特性：
- ✅ AI 自动审核（占位符）
- ✅ 管理员手动审核
- ✅ 用户可查看认证状态

### 3. 权限控制
- ✅ 用户可浏览论坛而无需认证
- ✅ 部分论坛可设置认证要求
- ✅ 只有满足要求的用户才能发表/投票
- ✅ 论坛创建者权限管理

### 4. 首页功能
- ✅ 论坛列表展示
- ✅ 热门论坛推荐
- ✅ 个性化论坛推荐
- ✅ Reddit 风格导航栏
- ✅ 全站搜索功能
- ✅ 用户菜单

### 5. 文章系统
- ✅ 发布、编辑、删除文章
- ✅ 点赞/点踩投票
- ✅ 评论功能（基础框架）
- ✅ 文章排序和分页
- ✅ 置顶文章功能

### 6. 用户系统
- ✅ 注册和登录
- ✅ 个人资料管理
- ✅ 用户头像和简介
- ✅ 用户统计信息
- ✅ JWT Token 认证

## 📋 技术栈

### 前端
- **框架**: Vue 3.x
- **构建工具**: Vite
- **语言**: TypeScript
- **状态管理**: Pinia
- **UI 框架**: Element Plus
- **HTTP 客户端**: Axios
- **路由**: Vue Router

### 后端
- **语言**: Java 25
- **框架**: Spring Boot 4.1.1
- **ORM**: Spring Data JPA
- **认证**: Spring Security + JWT
- **数据库**: MySQL 8.0
- **缓存**: Redis
- **API**: RESTful

## 📁 项目结构

```
openforum/
├── frontend/                 # Vue 前端项目
│   ├── src/
│   │   ├── components/      # 可复用组件
│   │   ├── views/           # 页面文件
│   │   ├── stores/          # Pinia 状态管理
│   │   ├── api/             # API 客户端
│   │   ├── router/          # 路由配置
│   │   ├── App.vue
│   │   └── main.ts
│   ├── index.html
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── package.json
│   └── README.md
│
├── backend/                  # Java Spring Boot 后端
│   ├── src/main/java/com/openforum/
│   │   ├── config/          # 配置类
│   │   ├── controller/      # API 控制器
│   │   ├── service/         # 业务逻辑
│   │   ├── repository/      # 数据访问
│   │   ├── entity/          # 实体类
│   │   ├── dto/             # 数据传输对象
│   │   ├── exception/       # 异常处理
│   │   ├── util/            # 工具类
│   │   └── OpenForumApplication.java
│   ├── src/main/resources/
│   │   └── application.yml  # 配置文件
│   ├── pom.xml
│   └── README.md
│
├── docs/                     # 文档
├── .gitignore
└── README.md                # 本文件
```

## 🚀 快速开始

### 前置要求

- Java 25+
- Node.js 18+
- MySQL 8.0+
- Redis (可选)

### 后端启动

```bash
cd backend

# 1. 修改配置
# 编辑 src/main/resources/application.yml
# 配置数据库和 Redis 连接

# 2. 编译运行
mvn spring-boot:run

# 或打包后运行
mvn clean package
java -jar target/openforum-backend-0.1.0.jar
```

后端服务地址：`http://localhost:8080/api`

### 前端启动

```bash
cd frontend

# 1. 安装依赖
npm install

# 2. 开发模式
npm run dev

# 3. 构建生产版本
npm run build
```

前端访问地址：`http://localhost:5173`

## 📖 API 文档

详见各模块 README.md：
- [后端 API 文档](./backend/README.md)
- [前端使用指南](./frontend/README.md)

## 🔧 配置说明

### 后端配置 (application.yml)

```yaml
spring:
  datasource:
    url: jdbc:mysql://localhost:3306/openforum
    username: root
    password: password
  redis:
    host: localhost
    port: 6379
  jwt:
    secret: your-secret-key-change-in-production
    expiration: 86400000  # 24小时
```

### 前端配置 (.env.development)

```
VITE_API_BASE_URL=http://localhost:8080/api
VITE_APP_NAME=OpenForum Dev
```

## 🎨 主要页面

1. **首页** - 热门论坛、个性化推荐
2. **登录/注册** - 用户认证
3. **个人资料** - 用户信息管理
4. **论坛详情** - 浏览文章、发布文章
5. **信息认证** - 用户认证管理
6. **论坛创建** - 创建新论坛

## 🔐 安全特性

- ✅ JWT Token 认证
- ✅ 密码加密存储 (BCrypt)
- ✅ 权限验证和访问控制
- ✅ CORS 跨域配置
- ✅ SQL 注入防护 (JPA)
- ✅ CSRF 防护

## 🚢 部署

### Docker 部署

```bash
# 构建后端镜像
cd backend
docker build -t openforum-backend:latest .

# 构建前端镜像
cd frontend
docker build -t openforum-frontend:latest .

# 使用 docker-compose
docker-compose up -d
```

### 手动部署

1. 部署 MySQL 和 Redis
2. 修改后端配置文件
3. 启动后端服务
4. 构建并部署前端静态文件
5. 配置 Nginx 反向代理

## 🔌 插件扩展

系统预留了扩展点：
- 认证审核模块（可集成 AI 服务）
- 评论系统（基础框架已建立）
- 推荐算法（可自定义）
- 内容审核（可扩展）

## 🐛 常见问题

**Q: 如何重置密码？**
A: 暂不支持，可通过数据库直接修改或扩展实现

**Q: 如何实现标签和分类？**
A: 可扩展 SubForum 实体添加标签字段

**Q: 如何处理大文件上传？**
A: 可集成对象存储（OSS/S3）

**Q: 支持多语言吗？**
A: 当前仅支持中文，可通过 i18n 扩展

## 📝 待实现功能

- [ ] 完整的评论系统
- [ ] 用户通知系统
- [ ] 站内消息功能
- [ ] 高级搜索功能
- [ ] 论坛分析仪表板
- [ ] 用户分级系统
- [ ] 标签系统
- [ ] 关注功能
- [ ] 收藏夹功能

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 👨‍💻 作者

CC - AI 助理

---

**最后更新**: 2024年10月

🎉 感谢使用 OpenForum！
