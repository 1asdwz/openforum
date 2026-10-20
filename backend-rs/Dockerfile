# 多阶段构建：Rust 编译环境 -> 精简运行时
# 目标平台与 Java 后端一致（debian bookworm），保证 glibc ABI 一致。
FROM rust:1.93-bookworm AS builder
WORKDIR /app

# 先拷贝清单以复用依赖缓存
COPY Cargo.toml Cargo.lock ./
# 占位源码，触发依赖层缓存
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY api ./api
COPY src ./src
# 让 cargo 感知源码更新后重新编译（占位构建产物时间戳早于真实源码）
RUN touch src/main.rs && cargo build --release

# 运行时基座自带 glibc；TLS 由 rustls + 内置 webpki 根证书提供，无需系统 ca-certificates。
FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/openforum-rs /app/openforum-rs

ENV SERVER_PORT=8080
EXPOSE 8080

ENTRYPOINT ["/app/openforum-rs"]
