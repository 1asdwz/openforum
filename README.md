# OpenForum

**English** · [简体中文](README.zh-CN.md)

An open-source modern community forum: sub-forums, rich-text posts, threaded comments, voting, an identity verification center, fine-grained access gates, custom homepages, and an OAuth 2.0 platform for third-party applications.

Live instance: <https://openforum.yingrest.com>

[![Rust](https://img.shields.io/badge/backend-Rust%20%2B%20axum-000000)](https://github.com/1asdwz/openforum)
![Vue](https://img.shields.io/badge/frontend-Vue%203%20%2B%20TypeScript-42b883)
![MySQL](https://img.shields.io/badge/database-MySQL%208-4479A1)

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Repository Layout](#repository-layout)
- [Getting Started](#getting-started)
- [Environment Variables](#environment-variables)
- [Default Accounts](#default-accounts)
- [API & Developer Platform](#api--developer-platform)
- [Deployment](#deployment)
- [Documentation](#documentation)
- [Security](#security)
- [Contributing](#contributing)

## Overview

OpenForum is a deployable forum system built as extensible community infrastructure rather than just a posting page:

- **Content**: sub-forums, rich-text posts, Markdown, images/videos, threaded comments, voting and hot ranking.
- **Identity**: accounts plus an identity verification center whose results are graded Low / Medium / High / Extreme and can be referenced directly by forum access rules.
- **Permissions**: each forum defines browse and operate rules (combinations of verification level, age, region, profile fields), alongside granular owner/moderator permissions.
- **Expression**: users and forums can publish custom homepages rendered from sanitized HTML inside a sandboxed frame, with templates and authoring guides.
- **Extension**: a plugin registry (content review, recommendation ranking) and an OAuth 2.0 developer platform that lets third-party apps call the full public API on behalf of a user.

The frontend is a single-page application. The backend is implemented in Rust (axum), stores data in MySQL 8, and keeps media in MinIO (S3-compatible).

## Features

| Area | Capabilities |
| --- | --- |
| Accounts | Registration, login, Google sign-in, JWT auth, profiles |
| Sub-forums | Create/edit, join/leave, members and moderators, public/private, theme color and icon |
| Posts | Create, edit, delete, pin, vote, images and videos, pagination and sorting |
| Comments | Nested tree, voting, deletion, aggregated by post |
| Search | Query by title, content and forum |
| Verification center | Real name, ID card, age, nationality, driver license, marriage, property, assets; document upload, liveness check, AI/rule/manual review, level system |
| Access gates | Visual browse/operate rule editor; evaluated against verification levels, profile fields and logical combinations |
| Custom homepages | User and forum homepages; constrained HTML, templates, reporting, admin blocking |
| Plugins | `ai-review`, `ai-vision-review`, `heuristic-document`, `hot-score-recommendation`, `rule-based-review` |
| Admin | Verification review, user list, homepage moderation and blocking |
| Developer platform | Create OAuth apps, reset secrets, browse the public API reference |
| OAuth 2.0 | Authorization code + PKCE (S256), access tokens, refresh-token rotation, userinfo, revoke |

## Tech Stack

### Backend (`backend-rs/`, production)

| Component | Choice |
| --- | --- |
| Language / runtime | Rust 2021, Tokio |
| Web framework | axum 0.7 (multipart, macros) |
| Data access | sqlx 0.8 (MySQL, chrono, json) |
| Auth | jsonwebtoken (HS256) + bcrypt |
| Object storage | MinIO / S3 via `rust-s3` (path-style) |
| HTML sanitization | ammonia |
| Image processing | `image` (JPEG / PNG / WebP / GIF) |
| Observability | tracing + tracing-subscriber |
| Middleware | tower / tower-http (CORS, trace, body limit) |

### Frontend (`frontend/`)

Vue 3 + TypeScript + Vite 5 + Pinia + Vue Router + Element Plus + Axios + Tiptap.

### Data & hosting

MySQL 8 and MinIO; the frontend is hosted on Cloudflare Pages and the backend runs as a Docker container.

### Legacy implementation (`backend/`)

`backend/` is the earlier Java 25 / Spring Boot 4.1.1 implementation. Production now runs the Rust backend; the Java code is kept for reference on legacy schema, API semantics and migration history. **New work should target `backend-rs/` only.**

## Repository Layout

```text
openforum/
├── backend-rs/              # Rust backend (production)
│   ├── api/openapi.yaml     # OpenAPI 3 specification (served publicly)
│   ├── deploy/              # Environment variable example
│   ├── sql/schema.sql       # Full schema (21 tables)
│   └── src/
│       ├── main.rs          # Startup, routing, security gate middleware
│       ├── routes/          # HTTP handlers per module
│       ├── repo.rs          # Data access
│       ├── oauth.rs         # OAuth primitives (PKCE, tokens, clients)
│       ├── access.rs        # Forum owner / moderator permissions
│       ├── gate.rs          # Access rule engine
│       ├── info.rs          # Profile field definitions
│       ├── plugins.rs       # Plugin registry
│       ├── storage.rs       # MinIO storage
│       └── ...
├── frontend/                # Vue 3 SPA
│   ├── public/_worker.js    # Same-origin API proxy for Cloudflare Pages
│   └── src/{views,components,api,router,stores}
├── backend/                 # Legacy Java implementation (reference only)
├── docs/                    # Technical documentation
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── OAUTH.md
│   ├── DATABASE.md
│   ├── DEPLOYMENT.md
│   └── DEVELOPMENT.md
├── docker-compose.yml       # Local stack for the legacy Java backend
└── CONTRIBUTING.md
```

## Getting Started

### Prerequisites

- Rust (stable; the Docker image builds with 1.93)
- MySQL 8.0+
- Node.js 18+ and npm
- Optional: MinIO (required for media uploads and reads)

### 1. Prepare the database

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

`backend-rs/sql/schema.sql` contains the full 21-table schema. On first start the backend also runs idempotent migrations (profile, homepage and OAuth tables) and inserts seed data.

### 2. Start the backend

```bash
cd backend-rs

export SERVER_PORT=8080
export DB_URL='jdbc:mysql://127.0.0.1:3306/openforum'
export DB_USER=root
export DB_PASSWORD='your-password'
export JWT_SECRET='replace-with-a-random-secret-at-least-32-bytes'

# Optional: enable media storage
export MINIO_ENABLED=true
export MINIO_ENDPOINT='http://127.0.0.1:9000'
export MINIO_ACCESS_KEY=minioadmin
export MINIO_SECRET_KEY=minioadmin
export MINIO_BUCKET=openforum-auth

cargo run
```

The backend listens on `http://localhost:8080` and mounts all routes under `/api`:

```bash
curl http://localhost:8080/api/plugins
```

> The process refuses to start when `JWT_SECRET` is shorter than 32 bytes. Always replace the built-in defaults in production.

### 3. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

The dev server runs at `http://localhost:5173` and proxies `/forum/api` to `http://localhost:8080` (see [vite.config.ts](frontend/vite.config.ts)).

Production build:

```bash
npm run type-check
npm run build
```

### 4. Run the backend with Docker

Copy the example env file, adjust the database URL, then build and run:

```bash
cp backend-rs/deploy/openforum-rs.env.example backend-rs/.env.local
# Edit backend-rs/.env.local: at minimum DB_URL, DB_PASSWORD and JWT_SECRET

docker build -t openforum-rs:latest backend-rs
docker run -d --name openforum-rs \
  -p 8080:8080 \
  --env-file backend-rs/.env.local \
  openforum-rs:latest
```

`docker-compose.yml` describes the legacy Java backend + Redis + MySQL + frontend stack. The Rust backend does not use Redis.

## Environment Variables

All backend configuration is read from the environment; see [openforum-rs.env.example](backend-rs/deploy/openforum-rs.env.example).

| Variable | Default | Description |
| --- | --- | --- |
| `SERVER_PORT` | `8080` | Listen port |
| `DB_URL` | `jdbc:mysql://localhost:3306/openforum` | MySQL URL; the `jdbc:mysql://` prefix is accepted |
| `DB_USER` | `root` | Database user |
| `DB_PASSWORD` | `root123456` | Database password |
| `JWT_SECRET` | built-in dev key | JWT signing key, **must be ≥ 32 bytes** |
| `JWT_EXPIRATION_HOURS` | `168` | Login token lifetime (hours) |
| `MINIO_ENABLED` | `false` | Enable object storage |
| `MINIO_ENDPOINT` | `http://localhost:9000` | MinIO endpoint |
| `MINIO_ACCESS_KEY` | `minioadmin` | MinIO access key |
| `MINIO_SECRET_KEY` | `minioadmin` | MinIO secret key |
| `MINIO_BUCKET` | `openforum-auth` | Bucket name |
| `AI_ENABLED` | `false` | Enable AI review plugins |
| `OPENAI_API_KEY` | empty | AI service key |
| `OPENAI_API_BASE` | `https://api.openai.com/v1` | AI service base URL |
| `OPENAI_API_MODEL` | `gpt-4o-mini` | Model name |
| `AI_TIMEOUT_SECONDS` | `20` | AI request timeout |
| `GOOGLE_CLIENT_ID` | empty | Google sign-in client ID |
| `AUTH_CHALLENGE_TTL_MINUTES` | `10` | Verification challenge TTL (minutes) |
| `AUTH_MAX_DOCUMENT_COUNT` | `5` | Max documents per submission |
| `AUTH_MAX_IMAGE_BYTES` | `10485760` | Image size limit (10 MB) |
| `AUTH_MAX_VIDEO_BYTES` | `52428800` | Video size limit (50 MB) |
| `AUTH_AUTO_APPROVE_THRESHOLD` | `0.85` | Auto-approval confidence threshold |

Frontend variables (`frontend/.env.*`):

| Variable | Description |
| --- | --- |
| `VITE_API_BASE_URL` | API base URL; defaults to `/forum/api` in development |
| `VITE_APP_NAME` | Site name |
| `VITE_GOOGLE_CLIENT_ID` | Google client ID; leave empty to hide Google sign-in |
| `VITE_BASE_PATH` | Build-time base path; defaults to `/forum/`, use `/` for a standalone domain |

## Default Accounts

When the database is empty, the backend seeds the following accounts (development only — change the passwords before deploying):

| Username | Password | Role |
| --- | --- | --- |
| `admin` | `admin123` | ADMIN |
| `alice` | `alice123` | USER |
| `bob` | `bob123` | USER |

## API & Developer Platform

- **Base URL**: `https://openforum.yingrest.com/api` (local: `http://localhost:8080/api`)
- **Response envelope**: `{"code":200,"message":"ok","data":...}`
- **Auth**: `Authorization: Bearer <JWT>`
- **Reference UI**: `/developers/docs`; the OpenAPI 3 YAML is also served publicly at `GET /api/developers/docs`
- **Spec file**: [backend-rs/api/openapi.yaml](backend-rs/api/openapi.yaml)

```bash
# Log in
curl -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"admin","password":"admin123"}'

# Call an authenticated endpoint
curl http://localhost:8080/api/auth/me \
  -H 'Authorization: Bearer <access_token>'
```

### Third-party apps (OAuth 2.0)

1. Sign in, open `/developers`, and create an app with redirect URIs (HTTPS; `http://localhost` and `http://127.0.0.1` are allowed for development).
2. Choose a client type:
   - `CONFIDENTIAL`: server-side app using `client_secret` (HTTP Basic).
   - `PUBLIC`: browser/mobile app without a secret; **PKCE is mandatory**.
3. After the user grants access, exchange the authorization code for tokens and call any public API with `Authorization: Bearer`.

| Endpoint | Purpose |
| --- | --- |
| `GET /api/oauth/authorize/info` | App details for the consent screen |
| `POST /api/oauth/authorize` | Approve/deny and receive the redirect URL |
| `POST /api/oauth/token` | `authorization_code` / `refresh_token` grants |
| `GET /api/oauth/userinfo` | User profile for the access token |
| `POST /api/oauth/revoke` | Revoke a refresh token |

Access tokens live for 2 hours; refresh tokens live for 30 days and rotate on every use. See [docs/OAUTH.md](docs/OAUTH.md) for the full flow, parameters and error codes.

## Deployment

### Frontend (Cloudflare Pages)

```bash
cd frontend
VITE_BASE_PATH=/ VITE_API_BASE_URL=/api npm run build
npx wrangler pages deploy dist --project-name openforum --branch main
```

`frontend/public/_worker.js` proxies same-origin `/api/*` requests to `BACKEND_BASE` (configure it in the Pages project), serves static assets, and falls back to `index.html`.

### Backend (GitHub Actions + Docker)

[.github/workflows/deploy-rs.yml](.github/workflows/deploy-rs.yml) runs when `backend-rs/**` changes: build image → compress → SCP to the server → start the container with `--env-file` (published on `8091:8080` by default).

Configure the `SSH_PRIVATE_KEY` repository secret and prepare the environment file on the server. Manual deployment also works:

```bash
docker build -t openforum-rs:latest backend-rs
docker run -d --name openforum-rs --restart unless-stopped \
  -p 8091:8080 --env-file /path/to/openforum-rs.env openforum-rs:latest
```

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for reverse proxy, Nginx and legacy pipeline details.

## Documentation

| Document | Contents |
| --- | --- |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture, request lifecycle, modules, design decisions |
| [API.md](docs/API.md) | Conventions, route index, errors, pagination |
| [OAUTH.md](docs/OAUTH.md) | Authorization code + PKCE flow with examples |
| [DATABASE.md](docs/DATABASE.md) | The 21-table schema and initialization |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md) | Cloudflare Pages, Docker, Nginx, CI/CD |
| [DEVELOPMENT.md](docs/DEVELOPMENT.md) | Local development, builds, tests, troubleshooting |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution workflow and code style |

Chinese versions of every document are kept next to the English ones with a `.zh-CN.md` suffix: [README.zh-CN.md](README.zh-CN.md), [docs/](docs/), [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md).

## Security

- Never commit real secrets. `JWT_SECRET`, database passwords, `MINIO_SECRET_KEY`, `OPENAI_API_KEY` and `GOOGLE_CLIENT_ID` are injected through environment variables.
- `JWT_SECRET` must be at least 32 bytes; use a high-entropy random value in production.
- Custom homepage HTML is sanitized and rendered in a sandboxed frame.
- Verification documents are stored in a private MinIO bucket and only served through authenticated media endpoints.
- Please report vulnerabilities privately to the maintainers instead of opening a public issue.

## Contributing

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first:

```bash
# Backend
cd backend-rs && cargo fmt && cargo test && cargo build --release

# Frontend
cd frontend && npm run type-check && npm run build
```

