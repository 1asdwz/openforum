/**
 * 项目技术栈说明文档
 * 
 * 前端技术栈
 * ===========
 * - Vue 3.x：渐进式 JavaScript 框架
 * - TypeScript：JavaScript 的超集，提供类型检查
 * - Vite：新一代前端构建工具，提供快速开发体验
 * - Element Plus：Vue 3 UI 组件库，提供丰富的企业级组件
 * - Pinia：Vue 官方推荐的状态管理库
 * - Vue Router：Vue 官方路由管理
 * - Axios：HTTP 客户端库
 * 
 * 后端技术栈
 * ===========
 * - Java 25：最新的 Java 版本，提供最新的语言特性
 * - Spring Boot 4.1.1：快速开发 Java 应用
 * - Spring Data JPA：简化数据访问层代码
 * - Spring Security：提供认证和授权
 * - Spring Web：构建 RESTful API
 * - MySQL 8.0：关系数据库
 * - Redis：内存数据库，用于缓存和会话
 * - JWT：无状态认证方案
 * - Lombok：减少样板代码
 * 
 * 开发工具
 * ===========
 * - Docker：容器化部署
 * - Docker Compose：多容器编排
 * - Maven：Java 构建工具
 * - npm：Node.js 包管理工具
 * 
 * 部署架构
 * ===========
 * Client (Vue SPA) 
 *     ↓
 * Nginx (反向代理 + 静态文件)
 *     ↓
 * Spring Boot Backend (REST API)
 *     ↓
 * MySQL + Redis
 */
