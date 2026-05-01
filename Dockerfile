# syntax=docker/dockerfile:1.7

FROM rust:1.94-bookworm AS builder
WORKDIR /app
ENV SQLX_OFFLINE=true
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --bin rs && \
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
