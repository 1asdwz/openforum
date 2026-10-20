# Local Development

[English](DEVELOPMENT.md) · [简体中文](DEVELOPMENT.zh-CN.md)

This document is intended for developers who need to modify the OpenForum code, and covers environment setup, startup, build, test, and common issues.

## 1. Toolchain

| Tool | Version | Purpose |
| --- | --- | --- |
| Rust | stable (Dockerfile uses 1.93) | Backend |
| MySQL | 8.0+ | Database |
| Node.js / npm | 18+ | Frontend |
| Docker | Optional | Containerized backend / MinIO |
| MinIO | Optional | Media upload and retrieval |

Installing `rustfmt` and `clippy` is recommended:

```bash
rustup component add rustfmt clippy
```

## 2. First Startup

### 2.1 Database

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

Alternatively, you can create only an empty database and let the backend create tables through `migrate.rs` at startup; however, `schema.sql` contains the complete baseline, so importing it directly is recommended.

### 2.2 Backend

```bash
cd backend-rs

export DB_URL='jdbc:mysql://127.0.0.1:3306/openforum'
export DB_USER=root
export DB_PASSWORD='your-password'
export JWT_SECRET='local-dev-secret-key-at-least-32-bytes-long'

cargo run
```

The startup log should contain:

```text
Starting OpenForum Rust backend on port 8080, database 127.0.0.1:3306 / openforum
MySQL connection pool ready
Listening on http://0.0.0.0:8080 (API prefix /api)
```

On first startup, default accounts are seeded (`admin/admin123`, `alice/alice123`, `bob/bob123`).

### 2.3 Frontend

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173/forum/` (default `base=/forum/`). The development proxy forwards `/forum/api` to `http://localhost:8080`, so the backend does not need CORS configured.

To debug the OAuth authorization page as well, visit directly:

```text
http://localhost:5173/forum/oauth/authorize?client_id=<id>&redirect_uri=http://localhost:3000/cb&state=dev
```

## 3. Common Commands

### Backend

```bash
cd backend-rs

cargo fmt                 # Format
cargo fmt --check         # Check formatting
cargo clippy -- -D warnings
cargo test                # Unit tests
cargo build --release     # Release build
```

The tests cover the pure-function parts such as OAuth utilities (PKCE RFC vectors, redirect validation, query building), route table conflict checks, access gating and moderation logic; flows that involve the database need to be verified manually against a local MySQL.

### Frontend

```bash
cd frontend

npm run dev          # Development server
npm run type-check   # vue-tsc type check
npm run build        # Production build
npm run preview      # Preview the build output
```

> The frontend currently has no unit test script configured; before committing, run at least `npm run type-check` and `npm run build`.

### One-Click Startup Scripts

The repository root provides `start.sh` / `stop.sh` (for the legacy Java + Docker Compose environment). For the Rust backend, use the commands above or `docker build`.

## 4. Code Structure Conventions

### 4.1 Backend

| Directory / File | Responsibility |
| --- | --- |
| `routes/` | HTTP layer: parse parameters, call repo / domain modules, return `ApiResult` |
| `repo.rs` | SQL queries and row mapping |
| `dto.rs` / `models.rs` | External DTOs and database models |
| `gate.rs` / `auth_gate.rs` | Gate rule models and evaluation |
| `security.rs` | JWT, password hashing, `CurrentUser` extractor |
| `error.rs` | Unified errors and response envelope |
| `config.rs` | Environment variables and startup validation |
| `migrate.rs` | Idempotent table creation |

Steps for adding a new endpoint:

1. Implement the handler in `routes/<module>.rs`.
2. Register the route in `routes/mod.rs`.
3. If it is a public endpoint, check whether `main.rs::is_public_path` needs to allow it (login is required by default).
4. Update `backend-rs/api/openapi.yaml`.
5. If new tables are involved, update `migrate.rs` and `sql/schema.sql`.
6. Add the route to the route table in `docs/API.md`.

### 4.2 Frontend

| Directory | Responsibility |
| --- | --- |
| `api/` | Axios instance, API wrappers, error handling |
| `views/` | Routed pages |
| `components/` | Reusable components |
| `stores/` | Pinia state (`auth`, `ui`) |
| `router/` | Routes and the `requiresAuth` guard |
| `types/` | Shared types |
| `styles/main.css` | Design variables and global styles |

Conventions:

- Components and pages use `<script setup lang="ts">`.
- All API calls go through `src/api/index.ts`; do not assemble URLs directly in components.
- New pages must be registered in `src/router/index.ts`, and whether login is required should be considered.
- Element Plus components are auto-imported on demand, so no manual `import` is needed.
- UI copy uses English, and code comments use English.

## 5. Configuration and Secrets

- Local development can use the default values, but `JWT_SECRET` must be at least 32 bytes.
- Do not write real secrets into `.env` and commit them; use shell environment variables or local untracked files.
- `.gitignore` already ignores `node_modules/`, `dist/`, `target/`, `*.jar`, `*.sql` (except `backend-rs/sql/schema.sql`), etc.

## 6. Troubleshooting

| Symptom | Cause | Resolution |
| --- | --- | --- |
| Startup reports `JWT_SECRET must be at least 32 bytes` | Secret is too short | Replace it with a long random value |
| Exits immediately after startup | Database connection failed | Check `DB_URL` / credentials / network |
| Upload returns `File storage is not enabled` | `MINIO_ENABLED=false` | Enable and configure MinIO |
| Upload returns 400 unsupported format | MIME is not in the allowlist | Use jpeg/png/webp/gif/mp4/webm |
| Frontend API returns 404 | `VITE_API_BASE_URL` does not match the proxy path | Use `/forum/api` in development |
| Frontend shows a blank page | `VITE_BASE_PATH` does not match the access path | Visit `http://localhost:5173/forum/` |
| 401 `unauthorized` | No Bearer token or token expired | Log in again |
| 403 `forbidden` | Not an admin / not a forum owner / gate check failed | Inspect `data.missing` in the response |
| Image build fails at `include_str!("../../api/openapi.yaml")` | The build context is missing `backend-rs/api` | Use `docker build ... backend-rs` |
| `cargo` compiles slowly | Full dependency compilation | Normal on the first build; incremental afterwards |

## 7. Debugging Tips

- The backend log level is controlled by `RUST_LOG`, defaulting to `info,sqlx=warn,tower_http=info`. To see SQL, set `RUST_LOG=info,sqlx=debug`.
- On the frontend, you can inspect `/forum/api/*` requests and response envelopes in the browser Network panel.
- For gate issues, first check `sub_forums.browse_rule` / `operate_rule` and the user's `user_authentications` and `user_info` records.
- For OAuth issues, check `oauth_clients.status`, exact callback URL matching, and whether `code_verifier` matches the challenge.

## 8. Pre-Commit Checks

```bash
cd backend-rs && cargo fmt --check && cargo test
cd ../frontend && npm run type-check && npm run build
```

Also confirm that:

- no real secrets or local environment files are committed;
- new / modified endpoints have updated the OpenAPI spec and docs;
- database changes have updated both `schema.sql` and `migrate.rs`.
