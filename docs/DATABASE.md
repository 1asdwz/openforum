# Database

[English](DATABASE.md) · [简体中文](DATABASE.zh-CN.md)

OpenForum uses MySQL 8.0+ with a default charset of `utf8mb4`. The complete baseline schema lives in [`backend-rs/sql/schema.sql`](../backend-rs/sql/schema.sql) and defines 21 tables.

## 1. Initialization

```bash
mysql -uroot -p -e "CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -uroot -p openforum < backend-rs/sql/schema.sql
```

Notes:

- `schema.sql` was exported from a production database and contains only `CREATE TABLE` statements, no business data.
- The script uses MySQL 8 collations such as `utf8mb4_0900_ai_ci`, and is **not supported on MySQL 5.7**.
- On startup the backend executes the idempotent `CREATE TABLE IF NOT EXISTS` statements in `migrate.rs`, filling in the profile, homepage, and OAuth tables; the application can therefore start even if only part of the schema was imported.
- When `sub_forums` is empty, `seeder.rs` injects demo forums and default accounts (`admin` / `alice` / `bob`).
- No manual SQL is needed after the first startup.

### Connection configuration

The backend connects using the following environment variables:

```bash
DB_URL=jdbc:mysql://127.0.0.1:3306/openforum
DB_USER=root
DB_PASSWORD=your-password
```

`DB_URL` accepts the `jdbc:mysql://` prefix; the host, port, and database are parsed from it, and the remaining connection parameters are set by `sqlx` (charset `utf8mb4`).

## 2. Table overview

| Group | Table | Purpose |
| --- | --- | --- |
| Accounts | `users` | Account, password hash, role, avatar, Google ID |
| Accounts | `user_profiles` | Legacy structured profile (gender, city, occupation, property, etc.) |
| Profiles | `info_fields` | Profile field catalog (group, type, verifiable or not, gate operators) |
| Profiles | `user_info` | User profile values, stored per field, with verification status and level |
| Verification | `user_authentications` | Verification requests, materials, review results, and levels |
| Verification | `auth_challenges` | Pre-submission challenges (nonce, action, expiration time) |
| Verification | `auth_media` | Verification material objects (ID documents, selfies, videos) |
| Verification | `auth_review_logs` | Review logs (action, method, confidence, reason) |
| Forums | `sub_forums` | Sub-forums, gate rules, templates, and statistics |
| Forums | `forum_members` | Members, moderators, owners, and permission bits |
| Content | `posts` | Posts, vote counts, comment counts, pinned / removed flags |
| Content | `post_media` | Post media objects |
| Content | `comments` | Threaded comments (`parent_id` self-reference) |
| Content | `votes` | Post / comment vote records |
| Homepages | `user_homepages` | User-defined homepage HTML and status |
| Homepages | `user_homepage_reports` | User homepage reports |
| Homepages | `forum_homepages` | Forum-defined homepage HTML and status |
| Homepages | `forum_homepage_reports` | Forum homepage reports |
| OAuth | `oauth_clients` | Third-party applications and secret hashes |
| OAuth | `oauth_authorization_codes` | Authorization code hashes and PKCE challenges |
| OAuth | `oauth_refresh_tokens` | Refresh token hashes and revocation status |

## 3. Key table structures

### 3.1 `users`

| Column | Type | Description |
| --- | --- | --- |
| `id` | `bigint` PK, AUTO_INCREMENT | User ID (JWT `sub`) |
| `username` | `varchar(50)` UNIQUE | Login name |
| `email` | `varchar(120)` UNIQUE | Email address, nullable |
| `password` | `varchar(120)` | bcrypt hash |
| `role` | `enum('ADMIN','USER')` | Global role |
| `avatar` | `varchar(500)` | Avatar URL / object key |
| `bio` | `text` | Bio |
| `google_id` | `varchar(64)` UNIQUE | Google account binding |
| `is_active` | `bit(1)` | Whether the account is enabled |
| `created_at` / `updated_at` | `datetime(6)` | Timestamps |

### 3.2 `sub_forums`

| Column | Description |
| --- | --- |
| `slug` | Unique URL identifier |
| `name` / `description` / `icon` / `theme_color` | Display information |
| `is_public` | Whether the sub-forum is public |
| `member_count` / `post_count` | Denormalized counters |
| `template_type` | `DEFAULT` / `CUSTOM` |
| `template_config` | Layout configuration JSON |
| `required_auth_types` | Legacy verification requirements (comma-separated) |
| `browse_rule` / `operate_rule` | Browse / operate gate rule JSON |
| `access_rules` | Combined rules (new version, `longtext`) |
| `gate_version` | Gate version number |
| `custom_page_html` / `custom_page_status` / `custom_page_review_note` | Legacy custom page |

### 3.3 `forum_members`

`UNIQUE(forum_id, user_id)`. `role` is `MEMBER` / `MODERATOR` / `OWNER`.

`permissions` is a permission bit string; the values are defined in `backend-rs/src/access.rs`:

| Permission | Meaning |
| --- | --- |
| `EVERYTHING` | All permissions |
| `USERS` | Member management |
| `CONFIG` | Forum settings |
| `FLAIR` | Flair / customization |
| `MAIL` | Internal mail |
| `POSTS_COMMENTS` | Content management |
| `WIKI` | Wiki / profile pages |

### 3.4 `posts` / `comments` / `votes`

- `posts.is_pinned`, `posts.is_removed`, and `comments.is_removed` use `bit(1)`.
- `posts.score`, `upvotes`, `downvotes`, and `comment_count` are denormalized statistics that are updated in the same write.
- An empty `comments.parent_id` denotes a top-level comment.
- `votes` enforces one vote per user via `UNIQUE(user_id, target_type, target_id)`; `value` is `1` / `-1`.

### 3.5 `user_authentications`

| Column | Description |
| --- | --- |
| `type` | `REAL_NAME`, `ID_CARD`, `AGE`, `NATIONALITY`, `DRIVER_LICENSE`, `MARRIAGE_CERT`, `PROPERTY_CERT`, `ASSETS`, `CITY`, `EDUCATION`, `GENDER`, `INTERESTS`, `OCCUPATION`, `CUSTOM` |
| `level` | `LOW` / `MEDIUM` / `HIGH` / `EXTREME` |
| `status` | `PENDING` / `APPROVED` / `REJECTED` |
| `method` | `AI` / `MANUAL` / `RULE` / `SYSTEM` |
| `confidence` | Review confidence (0–1) |
| `auth_data` | Submitted business data JSON |
| `media` | Material reference JSON |
| `liveness` | Liveness detection result |
| `attestation` | User commitment / declaration |
| `reviewed_by` / `reviewed_at` / `rejection_reason` | Manual review information |

`UNIQUE(user_id, type)`: only one record is kept per user and type.

### 3.6 Profile system

- `info_fields`: field catalog; `field_key` is the primary key, `group_key` is one of `IDENTITY` / `LOCATION` / `CAREER` / `PROPERTY` / `INTEREST`, and `gate_ops` declares the operators that may be used in access gates.
- `user_info`: `UNIQUE(user_id, field_key)`; `value_json` stores the actual value, while `auth_level` / `auth_status` / `auth_id` link to the verification result.

### 3.7 OAuth tables

| Table | Key columns |
| --- | --- |
| `oauth_clients` | `client_id` UNIQUE, `secret_hash`, `redirect_uris` (JSON array), `client_type`, `scopes`, `owner_id`, `status` |
| `oauth_authorization_codes` | Primary key `code_hash`, `client_id`, `user_id`, `redirect_uri`, `code_challenge`, `expires_at`, `used_at` |
| `oauth_refresh_tokens` | `token_hash` UNIQUE, `client_id`, `user_id`, `scope`, `expires_at`, `revoked_at` |

All tokens and authorization codes are stored as SHA-256 hashes only; plaintext is never persisted.

## 4. Indexes

| Table | Index | Purpose |
| --- | --- | --- |
| `users` | `username`, `email`, `google_id` | Login and account binding |
| `sub_forums` | `slug` UNIQUE | Route lookup |
| `forum_members` | `(forum_id, user_id)` UNIQUE | Membership checks |
| `user_info` | `(user_id, field_key)` UNIQUE, `field_key` | Profile reads and access gates |
| `user_authentications` | `(user_id, type)` UNIQUE | Verification lookups |
| `votes` | `(user_id, target_type, target_id)` UNIQUE | Duplicate vote prevention |
| `*_homepage_reports` | `(target_id, created_at)` | Reports queried by time |
| `oauth_clients` | `client_id` UNIQUE, `owner_id` | Client lookup |
| `oauth_authorization_codes` | `code_hash` PK, `client_id`, `expires_at` | Token exchange and expiry cleanup |
| `oauth_refresh_tokens` | `token_hash` UNIQUE, `client_id`, `user_id` | Refresh and revocation |

## 5. Conventions and caveats

- **No foreign key constraints**: relationships between tables are maintained by the application layer (inherited from the legacy JPA schema); deleting a user or forum requires cleaning up related data manually.
- **Booleans**: legacy tables use `bit(1)`, new tables use `tinyint` / `varchar`; reads normalize all of them to Rust `bool` / strings.
- **Time**: legacy tables use `datetime(6)`, new tables use `datetime`; neither carries a time zone, and the application side uses `NaiveDateTime`. OAuth logic uses UTC while some legacy code uses server local time; pin the server time zone to UTC in deployments to avoid cross-time-zone drift.
- **Enums**: some columns use MySQL `enum`, so changing the allowed values requires `ALTER TABLE`.
- **JSON**: rules, templates, and profile values are stored as JSON strings in `TEXT` / `LONGTEXT` and serialized on the Rust side.
- **Collations**: both `utf8mb4_unicode_ci` and `utf8mb4_0900_ai_ci` exist in the same database; be mindful of case and accent sensitivity differences when comparing across tables.

## 6. Backup and restore

```bash
# Backup
mysqldump -uroot -p --single-transaction --routines --triggers openforum > openforum-backup.sql

# Restore
mysql -uroot -p openforum < openforum-backup.sql
```

Media files are not stored in the database; back up the MinIO bucket separately.

## 7. Schema change process

There is currently no versioned migration tool, so when changing table structures, update the following in sync:

1. Update `backend-rs/sql/schema.sql` (baseline).
2. Add an idempotent migration in `backend-rs/src/migrate.rs` so that existing databases can be upgraded smoothly.
3. Update this document and the affected fields in `backend-rs/api/openapi.yaml`.
4. Describe compatibility and rollback in the PR description.
