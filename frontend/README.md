# OpenForum 前端

Vue 3 + TypeScript + Vite 构建的开放论坛系统前端

## 项目特性

- 🎨 现代化 UI 设计，使用 Element Plus 组件库
- ⚡ Vite 快速构建和热更新
- 🔐 JWT 认证和权限管理
- 📱 响应式设计，支持移动设备
- 🚀 生产级别的代码质量

## 环境要求

- Node.js 18+
- npm 或 yarn

## 快速开始

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run dev
```

访问 http://localhost:5173

### 生产构建

```bash
npm run build
```

### 代码检查

```bash
npm run lint
npm run type-check
```

## 项目结构

```
src/
├── components/          # 可复用组件
│   └── NavBar.vue      # 导航栏
├── views/              # 页面
│   ├── Home.vue        # 首页
│   ├── Login.vue       # 登录
│   ├── Register.vue    # 注册
│   ├── Profile.vue     # 个人资料
│   ├── ForumDetail.vue # 论坛详情
│   └── Authentication.vue # 认证管理
├── stores/             # Pinia 状态管理
│   └── userStore.ts    # 用户状态
├── api/                # API 客户端
│   └── client.ts       # Axios 实例
├── router/             # 路由配置
│   └── index.ts
├── App.vue             # 根组件
└── main.ts             # 入口文件
```

## 核心功能

### 认证

- 用户注册和登录
- JWT Token 管理
- 路由权限保护

### 论坛管理

- 浏览热门论坛
- 创建新论坛
- 加入/离开论坛
- 编辑论坛规则

### 文章系统

- 发布文章
- 编辑和删除文章
- 文章投票（点赞/点踩）
- 文章分类和排序

### 用户认证

- 多种认证类型支持
- 认证信息管理
- 认证状态查询

## 配置

### API 地址

开发环境：`.env.development`
生产环境：`.env.production`

### 修改 API 基础 URL

编辑 `src/api/client.ts` 中的 `API_BASE_URL`

## 常见问题

### 如何修改主题色？

编辑 `src/App.vue` 中的 CSS 变量

### 如何添加新页面？

1. 在 `src/views/` 创建新的 Vue 文件
2. 在 `src/router/index.ts` 添加路由
3. 在导航栏中添加链接

### API 请求超时

修改 `src/api/client.ts` 中的 `timeout` 值

## 打包和部署

### Docker 部署

```dockerfile
FROM node:18 as builder
WORKDIR /app
COPY . .
RUN npm install && npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## 许可证

MIT
