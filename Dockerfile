# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.94
ARG DEBIAN_VERSION=bookworm

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}-${DEBIAN_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
RUN cargo build --release --locked --bin rs && \
    cp target/release/rs /usr/local/bin/rs && \
    install -d -o 65532 -g 65532 /data

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder /usr/local/bin/rs /usr/local/bin/rs
COPY --chown=nonroot:nonroot migrations /app/migrations
COPY --from=builder /data /app/data

ENV RUST_LOG=info \
    DATABASE_URL=sqlite:///app/data/app.db?mode=rwc

VOLUME ["/app/data"]
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/rs"]
