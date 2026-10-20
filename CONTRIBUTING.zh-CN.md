# 贡献指南
[English](CONTRIBUTING.md) · **简体中文**

感谢你参与 OpenForum。本文说明开发环境、代码规范与提交流程。

## 1. 开始之前

- 阅读 [README.zh-CN.md](README.zh-CN.md) 与 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.zh-CN.md)，了解项目结构。
- 新功能请基于 `backend-rs/`（Rust）与 `frontend/`（Vue 3）开发。
- `backend/` 是已下线的 Java 实现，仅作历史参考，不接受新功能。
- 较大的改动请先开 Issue 讨论，避免重复劳动。

## 2. 开发环境

```bash
# 后端
cd backend-rs
cargo build

# 前端
cd frontend
npm install
npm run dev
```

数据库与完整启动步骤见 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.zh-CN.md)。

## 3. 代码规范

### 3.1 Rust 后端

- 使用 `cargo fmt` 统一格式，提交前执行 `cargo fmt --check`。
- 使用 `cargo clippy -- -D warnings` 检查常见问题。
- 类型 / 函数：`snake_case`；结构体 / 枚举：`PascalCase`；常量：`UPPER_SNAKE_CASE`。
- HTTP handler 只做参数解析、授权与响应组装，SQL 放入 `repo.rs` 或对应模块。
- 错误统一使用 `ApiError` / `ApiResult`，不要直接构造响应体。
- 注释与日志使用英文；不要提交调试用的 `println!` 或临时文件。

```rust
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = repo::require_forum(&state.pool, id).await?;
    Ok(j::ok(dto::forum_detail(&forum)))
}
```

### 3.2 Vue / TypeScript 前端

- 页面与组件使用 `<script setup lang="ts">`。
- 组件文件：`PascalCase.vue`；变量 / 函数：`camelCase`；常量：`UPPER_SNAKE_CASE`。
- 接口调用统一放在 `src/api/`，不要在组件内直接写 URL。
- 复用 `src/types/` 中的类型，避免 `any`。
- UI 文案使用英文，样式优先复用 `styles/main.css` 中的设计变量。
- 提交前必须通过 `npm run type-check` 与 `npm run build`。

### 3.3 数据库

- 修改表结构时同时更新 `backend-rs/sql/schema.sql` 与 `backend-rs/src/migrate.rs`。
- 迁移必须幂等，保证已有数据库可平滑升级。
- 在 PR 中说明兼容性与回滚方式。

### 3.4 API 与文档

- 新增或修改接口时更新 `backend-rs/api/openapi.yaml` 与 [docs/API.md](docs/API.zh-CN.md)。
- OAuth 相关改动同时更新 [docs/OAUTH.md](docs/OAUTH.zh-CN.md)。
- 文档使用 Markdown，包含可运行的示例。

## 4. 分支与提交

```bash
git clone https://github.com/1asdwz/openforum.git
cd openforum
git checkout -b feature/your-feature-name
```

分支命名：

- `feature/`：新功能
- `bugfix/`：缺陷修复
- `docs/`：文档
- `refactor/`：重构

提交信息使用 Conventional Commits：

```text
feat: add forum bookmark API

- add bookmark table and repository
- expose POST /api/forums/:id/bookmark
- document the endpoint in openapi.yaml
```

常用类型：`feat`、`fix`、`docs`、`style`、`refactor`、`perf`、`test`、`chore`。

## 5. 测试与检查

```bash
# 后端
cd backend-rs
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# 前端
cd frontend
npm run type-check
npm run build
```

涉及数据库、MinIO 或 OAuth 的改动，请在 PR 中附上手工验证步骤与结果。

## 6. Pull Request

PR 描述请包含：

- 变更目的与背景
- 主要改动点
- 验证方式（命令、截图或接口示例）
- 关联 Issue
- 是否涉及数据库 / 配置 / 破坏性变更

代码审查清单：

- [ ] 逻辑正确，边界条件已处理
- [ ] 没有硬编码密钥或环境相关配置
- [ ] 没有调试代码与临时文件
- [ ] 已通过格式化、静态检查与测试
- [ ] 接口 / 数据库 / 文档同步更新
- [ ] 提交信息清晰

## 7. 报告 Issue

Bug 报告请包含：

- 环境（操作系统、浏览器、版本 / commit）
- 复现步骤
- 预期行为与实际行为
- 相关日志、截图或接口响应

功能建议请说明使用场景与期望行为。安全问题请勿公开提交 Issue，先私下联系维护者。

## 8. 文档位置

| 文档 | 内容 |
| --- | --- |
| [README.zh-CN.md](README.zh-CN.md) / [README.md](README.md) | 项目介绍与快速开始 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.zh-CN.md) | 架构与设计决策 |
| [docs/API.md](docs/API.zh-CN.md) | API 约定与路由总表 |
| [docs/OAUTH.md](docs/OAUTH.zh-CN.md) | OAuth 2.0 接入 |
| [docs/DATABASE.md](docs/DATABASE.zh-CN.md) | 数据库结构 |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.zh-CN.md) | 部署 |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.zh-CN.md) | 本地开发 |

