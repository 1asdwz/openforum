# backend-rs

The production backend of OpenForum: Rust + axum + sqlx + MySQL. See the root documentation:

- [README](../README.md) / [简体中文](../README.zh-CN.md)
- [Architecture](../docs/ARCHITECTURE.md)
- [API](../docs/API.md)
- [OAuth](../docs/OAUTH.md)
- [Database](../docs/DATABASE.md)
- [Local development](../docs/DEVELOPMENT.md)

```bash
cargo run          # start (requires the DB_* and JWT_SECRET environment variables)
cargo test         # unit tests
cargo build --release
```

`backend/` is the retired Java implementation, kept for historical reference only.
