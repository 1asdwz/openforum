# backend (legacy)

This directory contains the early Java 25 / Spring Boot 4.1.1 backend. It **no longer runs in production** and is kept only to compare historical API semantics, table structures and the migration path.

Use [`backend-rs/`](../backend-rs/) for new work. Documentation entry points:

- [README](../README.md) / [简体中文](../README.zh-CN.md)
- [Architecture](../docs/ARCHITECTURE.md)
- [API](../docs/API.md)
- [Database](../docs/DATABASE.md)

Historical run command (for reference only):

```bash
mvn spring-boot:run
```

Requires Java 25, Maven and MySQL 8; the old configuration also depended on Redis.
