# 数据库
[English](DATABASE.md) · **简体中文**

OpenForum 使用 MySQL 8.0+，默认字符集 `utf8mb4`。完整基线结构位于 [`backend-rs/sql/schema.sql`](../backend-rs/sql/schema.sql)，共 21 张表。

## 1. 初始化

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

说明：

- `schema.sql` 由生产库导出，只包含 `CREATE TABLE`，不含业务数据。
- 脚本使用了 `utf8mb4_0900_ai_ci` 等 MySQL 8 排序规则，**不支持 MySQL 5.7**。
- 后端启动时会执行 `migrate.rs` 中的幂等 `CREATE TABLE IF NOT EXISTS`，补齐资料、主页与 OAuth 相关表；因此即使只导入部分结构也能启动。
- 当 `sub_forums` 为空时，`seeder.rs` 会注入演示论坛与默认账号（`admin` / `alice` / `bob`）。
- 首次启动后无需再手工执行任何 SQL。

### 连接配置

后端通过以下环境变量连接：

```bash
DB_URL=jdbc:mysql://127.0.0.1:3306/openforum
DB_USER=root
DB_PASSWORD=your-password
```

`DB_URL` 兼容 `jdbc:mysql://` 前缀，解析出 host、port 与 database，其余连接参数由 `sqlx` 设置（字符集 `utf8mb4`）。

## 2. 表总览

| 分组 | 表 | 作用 |
| --- | --- | --- |
| 账号 | `users` | 账号、密码哈希、角色、头像、Google ID |
| 账号 | `user_profiles` | 旧版结构化资料（性别、城市、职业、房产等） |
| 资料 | `info_fields` | 资料字段目录（分组、类型、是否可认证、门禁操作符） |
| 资料 | `user_info` | 用户资料值，按字段存储，附带认证状态与等级 |
| 认证 | `user_authentications` | 认证申请、材料、审核结果与等级 |
| 认证 | `auth_challenges` | 提交前的挑战（nonce、动作、过期时间） |
| 认证 | `auth_media` | 认证材料对象（证件、自拍、视频） |
| 认证 | `auth_review_logs` | 审核日志（动作、方式、置信度、原因） |
| 论坛 | `sub_forums` | 子论坛、门禁规则、模板与统计 |
| 论坛 | `forum_members` | 成员、版主、坛主及权限位 |
| 内容 | `posts` | 帖子、投票数、评论数、置顶 / 删除标记 |
| 内容 | `post_media` | 帖子媒体对象 |
| 内容 | `comments` | 树形评论（`parent_id` 自关联） |
| 内容 | `votes` | 帖子 / 评论投票记录 |
| 主页 | `user_homepages` | 用户自定义主页 HTML 与状态 |
| 主页 | `user_homepage_reports` | 用户主页举报 |
| 主页 | `forum_homepages` | 论坛自定义主页 HTML 与状态 |
| 主页 | `forum_homepage_reports` | 论坛主页举报 |
| OAuth | `oauth_clients` | 第三方应用与密钥哈希 |
| OAuth | `oauth_authorization_codes` | 授权码哈希与 PKCE challenge |
| OAuth | `oauth_refresh_tokens` | 刷新令牌哈希与撤销状态 |

## 3. 关键表结构

### 3.1 `users`

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `bigint` PK, AUTO_INCREMENT | 用户 ID（JWT `sub`） |
| `username` | `varchar(50)` UNIQUE | 登录名 |
| `email` | `varchar(120)` UNIQUE | 邮箱，可为空 |
| `password` | `varchar(120)` | bcrypt 哈希 |
| `role` | `enum('ADMIN','USER')` | 全局角色 |
| `avatar` | `varchar(500)` | 头像 URL / 对象键 |
| `bio` | `text` | 简介 |
| `google_id` | `varchar(64)` UNIQUE | Google 账号绑定 |
| `is_active` | `bit(1)` | 是否启用 |
| `created_at` / `updated_at` | `datetime(6)` | 时间戳 |

### 3.2 `sub_forums`

| 列 | 说明 |
| --- | --- |
| `slug` | URL 唯一标识 |
| `name` / `description` / `icon` / `theme_color` | 展示信息 |
| `is_public` | 是否公开 |
| `member_count` / `post_count` | 冗余计数 |
| `template_type` | `DEFAULT` / `CUSTOM` |
| `template_config` | 版式配置 JSON |
| `required_auth_types` | 旧版认证要求（逗号分隔） |
| `browse_rule` / `operate_rule` | 浏览 / 操作门禁规则 JSON |
| `access_rules` | 组合规则（新版，`longtext`） |
| `gate_version` | 门禁版本号 |
| `custom_page_html` / `custom_page_status` / `custom_page_review_note` | 旧版自定义页 |

### 3.3 `forum_members`

`UNIQUE(forum_id, user_id)`。`role` 为 `MEMBER` / `MODERATOR` / `OWNER`。

`permissions` 为权限位字符串，取值见 `backend-rs/src/access.rs`：

| 权限 | 含义 |
| --- | --- |
| `EVERYTHING` | 全部权限 |
| `USERS` | 成员管理 |
| `CONFIG` | 论坛设置 |
| `FLAIR` | 标签 / 装扮 |
| `MAIL` | 站内信 |
| `POSTS_COMMENTS` | 内容管理 |
| `WIKI` | 百科 / 资料页 |

### 3.4 `posts` / `comments` / `votes`

- `posts.is_pinned`、`posts.is_removed`、`comments.is_removed` 使用 `bit(1)`。
- `posts.score`、`upvotes`、`downvotes`、`comment_count` 为冗余统计，写入时同步更新。
- `comments.parent_id` 为空表示顶层评论。
- `votes` 通过 `UNIQUE(user_id, target_type, target_id)` 保证一人一票；`value` 为 `1` / `-1`。

### 3.5 `user_authentications`

| 列 | 说明 |
| --- | --- |
| `type` | `REAL_NAME`、`ID_CARD`、`AGE`、`NATIONALITY`、`DRIVER_LICENSE`、`MARRIAGE_CERT`、`PROPERTY_CERT`、`ASSETS`、`CITY`、`EDUCATION`、`GENDER`、`INTERESTS`、`OCCUPATION`、`CUSTOM` |
| `level` | `LOW` / `MEDIUM` / `HIGH` / `EXTREME` |
| `status` | `PENDING` / `APPROVED` / `REJECTED` |
| `method` | `AI` / `MANUAL` / `RULE` / `SYSTEM` |
| `confidence` | 审核置信度（0–1） |
| `auth_data` | 提交的业务数据 JSON |
| `media` | 材料引用 JSON |
| `liveness` | 活体检测结果 |
| `attestation` | 用户承诺 / 声明 |
| `reviewed_by` / `reviewed_at` / `rejection_reason` | 人工复核信息 |

`UNIQUE(user_id, type)`：同一用户同一类型只保留一条记录。

### 3.6 资料体系

- `info_fields`：字段目录，`field_key` 主键，`group_key` 属于 `IDENTITY` / `LOCATION` / `CAREER` / `PROPERTY` / `INTEREST`，`gate_ops` 声明可用于门禁的操作符。
- `user_info`：`UNIQUE(user_id, field_key)`，`value_json` 存实际值，`auth_level` / `auth_status` / `auth_id` 关联认证结果。

### 3.7 OAuth 表

| 表 | 关键列 |
| --- | --- |
| `oauth_clients` | `client_id` UNIQUE、`secret_hash`、`redirect_uris`（JSON 数组）、`client_type`、`scopes`、`owner_id`、`status` |
| `oauth_authorization_codes` | 主键 `code_hash`、`client_id`、`user_id`、`redirect_uri`、`code_challenge`、`expires_at`、`used_at` |
| `oauth_refresh_tokens` | `token_hash` UNIQUE、`client_id`、`user_id`、`scope`、`expires_at`、`revoked_at` |

所有令牌与授权码只存 SHA-256 哈希，明文不落库。

## 4. 索引

| 表 | 索引 | 用途 |
| --- | --- | --- |
| `users` | `username`、`email`、`google_id` | 登录与账号绑定 |
| `sub_forums` | `slug` UNIQUE | 路由查询 |
| `forum_members` | `(forum_id, user_id)` UNIQUE | 成员判定 |
| `user_info` | `(user_id, field_key)` UNIQUE、`field_key` | 资料读取与门禁 |
| `user_authentications` | `(user_id, type)` UNIQUE | 认证查询 |
| `votes` | `(user_id, target_type, target_id)` UNIQUE | 防重复投票 |
| `*_homepage_reports` | `(target_id, created_at)` | 按时间查询举报 |
| `oauth_clients` | `client_id` UNIQUE、`owner_id` | 客户端查找 |
| `oauth_authorization_codes` | `code_hash` PK、`client_id`、`expires_at` | 换令牌与过期清理 |
| `oauth_refresh_tokens` | `token_hash` UNIQUE、`client_id`、`user_id` | 刷新与撤销 |

## 5. 约定与注意事项

- **无外键约束**：表间关系由应用层维护（沿用旧 JPA 结构），删除用户 / 论坛时需自行清理关联数据。
- **布尔值**：历史表使用 `bit(1)`，新表使用 `tinyint` / `varchar`；读取时统一转换为 Rust `bool` / 字符串。
- **时间**：历史表为 `datetime(6)`，新表为 `datetime`，均不带时区，应用侧使用 `NaiveDateTime`。OAuth 相关逻辑使用 UTC，部分历史代码使用服务器本地时间；部署时请把服务器时区固定为 UTC，避免跨时区偏差。
- **枚举**：部分列使用 MySQL `enum`，修改取值需要 `ALTER TABLE`。
- **JSON**：规则、模板、资料值以 `TEXT` / `LONGTEXT` 保存 JSON 字符串，由 Rust 侧序列化。
- **排序规则**：同一库中同时存在 `utf8mb4_unicode_ci` 与 `utf8mb4_0900_ai_ci`，跨表比较时注意大小写与重音规则差异。

## 6. 备份与恢复

```bash
# 备份
mysqldump -uroot -p --single-transaction --routines --triggers openforum > openforum-backup.sql

# 恢复
mysql -uroot -p openforum < openforum-backup.sql
```

媒体文件不在数据库中，需单独备份 MinIO 桶。

## 7. 结构变更流程

当前没有版本化迁移工具，修改表结构时请同步以下位置：

1. 更新 `backend-rs/sql/schema.sql`（基线）。
2. 在 `backend-rs/src/migrate.rs` 中添加幂等迁移，保证旧库可平滑升级。
3. 更新本文档与 `backend-rs/api/openapi.yaml` 中受影响的字段。
4. 在 PR 描述中说明兼容性与回滚方式。
