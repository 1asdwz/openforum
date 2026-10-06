# OpenForum 设计文档

- 日期：2026-10-03
- 状态：已确认，进入实现
- 技术栈：Vue 3 + TypeScript + Vite + Pinia + Element Plus / Spring Boot 4.0.6 + Java 25
- 部署：Docker 容器 + 服务器 `43.157.5.182` 现有 nginx / MySQL 5.7 / Redis 7
- 访问入口：`https://apprest.mytruststores.com/forum`

> 本文是实现的唯一契约来源。后端、前端、部署脚本均以本文为准。

---

## 1. 背景与现状

原 `openforum/` 代码为前次自动生成，**无法编译**：

| 问题 | 说明 |
|------|------|
| 版本不存在 | `pom.xml` 声明 Spring Boot `4.1.1`，Maven Central 无此版本（最新 GA 为 `4.0.6`） |
| 缺失依赖 | `AIAuthenticationReviewer`/`AIConfig` 导入 `org.springframework.ai.*`，但 pom 无 Spring AI 依赖 |
| 重复类名 | `AuthenticationControllerV2.java` 内声明了 `AuthenticationController` 公有类 |
| API 过时 | `SecurityConfig` 使用 Spring Security 5 已移除的 `csrf().disable()` 链式 API |
| 目标版本不匹配 | `<release>25</release>` 但本机 JDK 21，javac 无法执行 |
| 大量级联错误 | `AuthenticationService` 等因上述问题产生数十处 `找不到符号` |

**决策：干净重写前后端**，保留功能设计，重写为可编译、可运行、可部署的代码。

---

## 2. 环境事实（实测）

### 2.1 本机
- JDK：Microsoft OpenJDK 21（JAVA_HOME）；另有 `C:\Program Files\Java\jdk-24`
- Maven 3.9.11；Node 24 / npm 11；Docker CLI 28（Docker Desktop 已安装）
- 编译 Java 25 需另装 Temurin JDK 25（下载到 `C:\Users\a7356\.trae\tools\`）

### 2.2 服务器 `43.157.5.182`（SSH 别名 `tx`，root，密钥 `~/.ssh/id_ed25519`）
- 主机名 `VM-0-13-centos`；**无 JDK**（必须容器运行）；内存 7.7G（可用约 2.7G）；磁盘 24G 可用
- 运行中容器：`nginx`(80/443)、`mall-admin`(8080)、`mall-portal`(8085)、`mysql`(3306, **5.7.44**)、`redis`(6379)、`mongo`、`rabbitmq`、`minio`、`record-service`、`mytruststores-shop`
- MySQL root 口令：`mallPass@151911`；已有生产库 `mall`
- nginx 静态根：`/mydata/nginx/html`（= 容器 `/usr/share/nginx/html`）；配置唯一文件 `/etc/nginx/conf.d/default.conf`（未纳入 git，**改动前必须备份**）
- `apprest.mytruststores.com` server 块：`/adminServer/`→8080、`/api/`→8085、`/file/`→9090

### 2.3 参考项目可复用密钥
- 火山方舟 Ark：`OPENAI_API_KEY=4484a50b-...`、`OPENAI_API_BASE=https://ark.cn-beijing.volces.com/api/v3/bots/`、`OPENAI_API_MODEL=bot-20250221230631-g8mrs`
- xAI：`XAI_API_KEY=xai-sWgAsu...`、`XAI_API_MODEL=grok-4-fast-reasoning`

> 本期 AI 审核默认**不启用**，走规则引擎 + 人工审核；AI 作为可插拔插件，配置上述任一密钥后启用。

---

## 3. 总体架构

```
浏览器
  │ HTTPS
  ▼
Cloudflare ──► 服务器 nginx (43.157.5.182:443)
                 ├── /forum/            → 静态 SPA (/mydata/nginx/html/forum)
                 └── /forum/api/        → proxy 10.9.0.13:8090 (openforum-backend)
                                              │
                       ┌──────────────────────┴──────────────────┐
                       ▼                                          ▼
                 MySQL 5.7 库 openforum                    Redis 7 (db 1)
```

- 后端容器 `openforum-backend`：Spring Boot 4.0.6 + Java 25，context-path `/api`，宿主端口 `8090`，`-Xmx384m`
- 前端：Vite `base=/forum/`，Router `history` + `base=/forum/`，API 基址 `/forum/api`
- SPA 回退：nginx `try_files $uri $uri/ /forum/index.html`

### 3.1 目录结构

```
openforum/
├── backend/          Spring Boot 应用
├── frontend/         Vue 3 应用
├── deploy/           服务器部署脚本与 nginx 片段
├── docker-compose.yml 本地开发用（MySQL8 + Redis + backend + frontend）
└── docs/superpowers/specs/
```

---

## 4. 数据模型

统一使用 `utf8mb4`，主键 `BIGINT AUTO_INCREMENT`，时间戳 `LocalDateTime`，`ddl-auto=update` 建表 + 启动种子数据。

### users
`id, username(uniq), email(uniq), password(bcrypt), avatar, bio, role(USER|ADMIN), is_active, created_at, updated_at`

### sub_forums
`id, slug(uniq), name, description, icon, theme_color, rules(TEXT), required_auth_types(VARCHAR 512, 逗号分隔如 "REAL_NAME,AGE_18"), template_type(DEFAULT|CUSTOM), template_config(JSON 字符串：板块/链接/公告), custom_page_html(LONGTEXT), custom_page_status(DRAFT|PENDING|APPROVED|REJECTED), custom_page_review_note, creator_id(FK users), member_count, post_count, is_public, created_at, updated_at`

### forum_members
`id, forum_id(FK), user_id(FK), role(OWNER|MODERATOR|MEMBER), joined_at`，`(forum_id,user_id)` 唯一

### posts
`id, forum_id(FK), author_id(FK), title, content(LONGTEXT), upvotes, downvotes, score, comment_count, is_pinned, is_removed, created_at, updated_at`

### comments
`id, post_id(FK), author_id(FK), parent_id(FK comments, 可空), content(LONGTEXT), upvotes, downvotes, score, is_removed, created_at, updated_at`

### votes
`id, user_id(FK), target_type(POST|COMMENT), target_id, value(1|-1)`，`(user_id,target_type,target_id)` 唯一

### user_authentications
`id, user_id(FK), type(枚举), auth_data(JSON 字符串), status(PENDING|APPROVED|REJECTED), review_method(RULE|AI|MANUAL), confidence, reason, rejection_reason, reviewed_by, reviewed_at, created_at, updated_at`，`(user_id,type)` 唯一

**认证类型枚举**：`REAL_NAME, ID_CARD, GENDER, AGE, NATIONALITY, DRIVER_LICENSE, MARRIAGE_CERT, PROPERTY_CERT, ASSETS, CUSTOM`

**门槛语法**：`required_auth_types` 逗号分隔，元素为 `TYPE` 或 `TYPE_OP_VALUE`（仅 `AGE` 支持比较：`AGE_18` 表示年龄≥18）。判定：用户存在该类型 APPROVED 认证即通过；`AGE_n` 需从 APPROVED 的 `AGE` 认证数据中读取 `age` 字段比较。

---

## 5. API 契约（context-path `/api`，nginx 前缀 `/forum`）

统一响应：`{ "code": 200, "message": "ok", "data": ... }`；错误：`{ "code": 4xx, "message": "..." }`

### 5.1 认证
| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/register` | body `{username,email,password}` → `{token,user}` |
| POST | `/auth/login` | body `{usernameOrEmail,password}` → `{token,user}` |
| GET | `/auth/me` | 当前用户 |

### 5.2 用户
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/{id}` | 公开资料（含已通过的认证勋章） |
| PUT | `/users/me` | `{avatar,bio}` |

### 5.3 子论坛
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/forums` | `?page&size&sort=hot\|new&q=` 分页 |
| GET | `/forums/popular` | `?limit=10` |
| GET | `/forums/recommended` | `?limit=10`（登录按认证/兴趣，未登录回退热门） |
| POST | `/forums` | `{slug,name,description,icon,themeColor,templateType,templateConfig,rules,requiredAuthTypes[]}` |
| GET | `/forums/{slug}` | 详情（含 `isMember`、`canPost`、`requiredAuthTypes`） |
| PUT | `/forums/{id}` | 坛主更新基本信息/规则/门槛/模板 |
| PUT | `/forums/{id}/custom-page` | 坛主提交自定义 HTML `{html}` → 存 DRAFT |
| POST | `/forums/{id}/custom-page/submit` | 提交审核 → PENDING |
| GET | `/forums/{id}/custom-page` | 坛主/管理员查看当前 HTML 与状态 |
| POST | `/forums/{id}/join` / `/leave` | 加入 / 退出 |
| GET | `/forums/{id}/members` | 成员列表 |

### 5.4 帖子
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/posts` | `?forumId&sort=hot\|new\|top&page&size` |
| GET | `/posts/{id}` | 详情（含 `canVote`） |
| POST | `/posts` | `{forumId,title,content}`（需登录 + 满足门槛） |
| PUT | `/posts/{id}` / DELETE | 作者或坛主 |
| POST | `/posts/{id}/vote` | `{value:1\|-1\|0}`（需登录 + 门槛） |
| POST | `/posts/{id}/pin` | 坛主置顶 |

### 5.5 评论
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/comments/post/{postId}` | 树形返回 |
| POST | `/comments` | `{postId,content,parentId?}` |
| DELETE | `/comments/{id}` | 作者或坛主 |
| POST | `/comments/{id}/vote` | `{value}` |

### 5.6 认证
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/authentications/types` | 类型元数据（label + 表单字段定义） |
| POST | `/authentications` | `{type,data{...}}` → 记录（规则引擎即时审核） |
| GET | `/authentications/me` | 我的认证列表 |
| GET | `/authentications/me/approved` | 已通过类型 |
| POST | `/authentications/{id}/retry` | 被拒后重提 |

### 5.7 管理员（role=ADMIN）
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/admin/authentications` | `?status=PENDING&page` |
| POST | `/admin/authentications/{id}/approve` / `/reject` | `{reason?}` |
| GET | `/admin/custom-pages` | `?status=PENDING` |
| POST | `/admin/custom-pages/{id}/approve` / `/reject` | `{reason?}` |
| GET | `/admin/users` | 用户列表 |

### 5.8 插件
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/plugins` | 已注册插件 `{key,name,type,enabled,description}` |
| PUT | `/plugins/{key}/enabled` | 管理员启停（`{enabled}`） |

---

## 6. 插件机制

后端 SPI：所有实现 `ForumPlugin` 的 Spring Bean 由 `PluginRegistry` 自动收集。

```java
public interface ForumPlugin {
    String getKey();          // 唯一标识
    String getName();         // 展示名
    PluginType getType();     // REVIEW | RECOMMENDATION | CONTENT_BLOCK
    boolean isEnabledByDefault();
    String getDescription();
}
```

内置实现：
- `RuleBasedReviewPlugin`（REVIEW，默认启用）：校验各认证类型的必填字段与格式（身份证 18 位、年龄范围、国籍非空等），输出 `{status, confidence, reason}`
- `AiReviewPlugin`（REVIEW，默认禁用）：读取 `openforum.ai.*` 配置，通过 Spring `RestClient` 调 OpenAI 兼容接口；未配置密钥时 `isAvailable()=false`，自动回退规则引擎
- `HotScoreRecommendationPlugin`（RECOMMENDATION，默认启用）：按 `member_count/post_count/近 7 天活跃` 计算推荐分

审核编排：`AuthenticationService` 依次调用所有启用的 REVIEW 插件，取最高置信结果；置信度 < 阈值（0.9）转人工 `PENDING`。

前端插件注册：`src/plugins/index.ts` 维护 `Map<key, () => Component>`；子论坛自定义页面渲染走 `CustomPageRenderer`（沙箱 iframe + 后端已净化 HTML）。

---

## 7. 子论坛自定义页面安全

1. 坛主提交 HTML → 后端用 **jsoup** `Safelist` 白名单净化（去 `<script>`/事件属性/`javascript:` 协议/iframe 等）
2. 净化后存 `custom_page_html`，状态 `PENDING`
3. 站点管理员在 `/admin/custom-pages` 审核 → `APPROVED` 才对外渲染
4. 前端用 `<iframe sandbox="allow-popups">` + `srcdoc` 渲染，与主站 DOM/Cookie 隔离
5. 未通过审核时子论坛使用默认模板（`template_type=DEFAULT` + `template_config`）

---

## 8. 前端设计（Reddit 风格）

- **固定顶栏**（全站，进入子论坛不变）：Logo、搜索框（回车跳 `/forum/search?q=`）、`+ 创建子论坛`、通知/用户头像菜单（登录/注册 或 个人资料/认证中心/我的论坛/管理后台/退出）
- **首页** `/forum/`：左侧栏（热门子论坛 / 为你推荐 / 我的论坛）+ 中部帖子流（排序 Tab：热门/最新/最高分）
- **子论坛页** `/forum/f/{slug}`：顶栏不变；头部（图标、名称、简介、成员数、加入/退出、发帖按钮）；下方渲染坛主模板或自定义页面
- **帖子详情** `/forum/post/{id}`：正文、投票、评论区（支持两级回复）
- **认证中心** `/forum/authentication`：按类型卡片提交，展示各认证状态
- **个人资料** `/forum/u/{id}`、**登录** `/forum/login`、**注册** `/forum/register`
- **坛主设置** `/forum/f/{slug}/settings`：基本信息/规则/门槛/模板/自定义页面编辑器
- **管理后台** `/forum/admin`：待审认证、待审自定义页面
- 视觉：深色/浅色可切换，卡片化、圆角、投票列，主色 `#FF4500`（Reddit 橙）

---

## 9. 部署方案（已实施）

### 9.1 数据层
- **不复用**服务器既有 MySQL 5.7：Hibernate 7 / Connector-J 9 要求 MySQL 8+，故新建独立容器
  - `docker run -d --name openforum-mysql --network openforum-net --restart unless-stopped -e MYSQL_ROOT_PASSWORD=... -e MYSQL_DATABASE=openforum -e MYSQL_USER=openforum -e MYSQL_PASSWORD='OpenForum@2026' -v /mydata/openforum/mysql:/var/lib/mysql -m 700m mysql:8.0 --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci`
- 复用服务器既有 `redis` 容器（数据库 1）：`docker network connect openforum-net redis`（应用当前未实际使用 Redis，仅保留自动配置）

### 9.2 应用层
- 本机 JDK 25 `mvn package` 产出 fat jar，仅上传 jar（不入服务器构建源码）
- `backend/Dockerfile` 改为运行期镜像：`FROM eclipse-temurin:25-jre` + `COPY app.jar`
- 在服务器 `docker build -t openforum-backend:latest`，运行：
  `docker run -d --name openforum-backend --network openforum-net --restart unless-stopped -p 8090:8080 -e SPRING_PROFILES_ACTIVE=prod -e DB_URL='jdbc:mysql://openforum-mysql:3306/openforum?...' -e DB_USER=openforum -e DB_PASSWORD='OpenForum@2026' -e REDIS_HOST=redis -e JWT_SECRET=... -m 512m --memory-swap 768m openforum-backend:latest`
- 容器同网 `openforum-net`，MySQL 不对外暴露端口

### 9.3 前端
- 产物上传 `/mydata/nginx/html/forum/`，**上传后必须** `chmod -R a+rX`（否则 scp 目录 700，nginx worker 无法读取）
- 静态资源目录 `assets/` → `static/`（`vite.config.ts: build.assetsDir`），避免被 CDN 长期缓存的旧 URL

### 9.4 nginx
在 `apprest.mytruststores.com` server 块（`/mydata/nginx/conf/conf.d/default.conf`，**改动前备份**）新增：

```nginx
location /forum/static/ {
    root /usr/share/nginx/html;
    try_files $uri =404;
    add_header Cache-Control "public, max-age=31536000, immutable";
}
location /forum/api/ {
    proxy_pass http://10.9.0.13:8090/api/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_read_timeout 120s;
}
location /forum/ {
    root /usr/share/nginx/html;
    try_files $uri $uri/ /forum/index.html;
    add_header Cache-Control "no-cache";
}
```

> 关键：`/forum/`（SPA 回退）必须显式 `Cache-Control: no-cache`，否则 Cloudflare 会对 `.js/.css` 按默认 4h（`max-age=14400`）缓存，一旦源站短暂 404/403 会被缓存成错误内容。

### 9.5 回滚
- 备份文件 `/mydata/nginx/conf/conf.d/default.conf.bak.<ts>`，失败即还原并 `nginx -s reload`
- 容器失败：`docker rm -f openforum-backend`，不影响既有服务
- 数据：容器卷 `/mydata/openforum/mysql`

### 9.6 线上地址
- 前端：`https://apprest.mytruststores.com/forum/`
- 接口：`https://apprest.mytruststores.com/forum/api/`
- 种子账号：`admin/admin123`（管理员）、`alice/alice123`、`bob/bob123`

---

## 10. 验收

1. `mvn -q clean package` 通过（JDK 25）
2. `npm run build` 通过
3. 本地 Docker Compose 起 MySQL8+Redis+backend，`/api/forums` 返回种子数据
4. 线上 `https://apprest.mytruststores.com/forum/` 打开首页
5. 全链路：注册 → 登录 → 建子论坛（设门槛）→ 提交认证 → 规则引擎通过 → 发帖 → 投票 → 评论 → 管理员审核自定义页面

---

## 11. 非目标（本期不做）

- 站内私信、实时通知（WebSocket）
- 对象存储图片上传（先用外链/头像 URL）
- 多语言 i18n
- 手机 App
- Elasticsearch 全文检索（先用 SQL LIKE）
