# Contributing Guide

[English](CONTRIBUTING.md) · [简体中文](CONTRIBUTING.zh-CN.md)

Thanks for contributing to OpenForum. This document covers the development environment, code style, and submission workflow.

## 1. Before You Start

- Read [README.md](README.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand the project structure.
- Build new features on `backend-rs/` (Rust) and `frontend/` (Vue 3).
- `backend/` is the retired Java implementation. It is kept for historical reference only and does not accept new features.
- For larger changes, open an Issue first to discuss and avoid duplicate work.

## 2. Development Environment

```bash
# Backend
cd backend-rs
cargo build

# Frontend
cd frontend
npm install
npm run dev
```

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for database setup and the full startup steps.

## 3. Code Style

### 3.1 Rust Backend

- Format code with `cargo fmt`. Run `cargo fmt --check` before committing.
- Check for common issues with `cargo clippy -- -D warnings`.
- Types / functions: `snake_case`; structs / enums: `PascalCase`; constants: `UPPER_SNAKE_CASE`.
- HTTP handlers should only parse parameters, authorize, and assemble responses. Put SQL in `repo.rs` or the relevant module.
- Use `ApiError` / `ApiResult` for all errors. Do not construct response bodies directly.
- Write comments and logs in English. Do not commit debug `println!` calls or temporary files.

```rust
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = repo::require_forum(&state.pool, id).await?;
    Ok(j::ok(dto::forum_detail(&forum)))
}
```

### 3.2 Vue / TypeScript Frontend

- Use `<script setup lang="ts">` for pages and components.
- Component files: `PascalCase.vue`; variables / functions: `camelCase`; constants: `UPPER_SNAKE_CASE`.
- Put all API calls in `src/api/`. Do not hardcode URLs inside components.
- Reuse types from `src/types/` and avoid `any`.
- Write UI copy in English. Reuse the design variables in `styles/main.css` where possible.
- `npm run type-check` and `npm run build` must pass before committing.

### 3.3 Database

- When changing the table schema, update both `backend-rs/sql/schema.sql` and `backend-rs/src/migrate.rs`.
- Migrations must be idempotent so existing databases can upgrade smoothly.
- Describe compatibility and rollback steps in the PR.

### 3.4 API and Documentation

- When adding or changing endpoints, update `backend-rs/api/openapi.yaml` and [docs/API.md](docs/API.md).
- For OAuth-related changes, also update [docs/OAUTH.md](docs/OAUTH.md).
- Write documentation in Markdown and include runnable examples.

## 4. Branches and Commits

```bash
git clone https://github.com/1asdwz/openforum.git
cd openforum
git checkout -b feature/your-feature-name
```

Branch naming:

- `feature/`: new features
- `bugfix/`: bug fixes
- `docs/`: documentation
- `refactor/`: refactoring

Use Conventional Commits for commit messages:

```text
feat: add forum bookmark API

- add bookmark table and repository
- expose POST /api/forums/:id/bookmark
- document the endpoint in openapi.yaml
```

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`.

## 5. Tests and Checks

```bash
# Backend
cd backend-rs
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Frontend
cd frontend
npm run type-check
npm run build
```

For changes involving the database, MinIO, or OAuth, include manual verification steps and results in the PR.

## 6. Pull Request

The PR description should include:

- Purpose and background of the change
- Main changes
- Verification approach (commands, screenshots, or endpoint examples)
- Related Issue
- Whether the change involves the database, configuration, or breaking changes

Code review checklist:

- [ ] Logic is correct and edge cases are handled
- [ ] No hardcoded secrets or environment-specific configuration
- [ ] No debug code or temporary files
- [ ] Formatting, static checks, and tests pass
- [ ] API / database / documentation are updated together
- [ ] Commit messages are clear

## 7. Reporting Issues

A bug report should include:

- Environment (operating system, browser, version / commit)
- Steps to reproduce
- Expected behavior and actual behavior
- Relevant logs, screenshots, or API responses

For feature requests, describe the use case and expected behavior. Do not report security issues publicly in an Issue. Contact the maintainers privately first.

## 8. Documentation Locations

| Document | Contents |
| --- | --- |
| [README.md](README.md) | Project introduction and quick start |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture and design decisions |
| [docs/API.md](docs/API.md) | API conventions and route overview |
| [docs/OAUTH.md](docs/OAUTH.md) | OAuth 2.0 integration |
| [docs/DATABASE.md](docs/DATABASE.md) | Database schema |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Deployment |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Local development |

