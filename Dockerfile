FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src backend/src
COPY backend/migrations backend/migrations
RUN cargo build --release --package ascendia-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 10001 ascendia
COPY --from=builder /app/target/release/ascendia-server /usr/local/bin/ascendia-server
USER ascendia
EXPOSE 8080
ENTRYPOINT ["ascendia-server"]
