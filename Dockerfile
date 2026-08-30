# syntax=docker/dockerfile:1

FROM rust:1.95-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install --yes --no-install-recommends clang cmake \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --no-create-home app

COPY --from=builder /app/target/release/nomina /usr/local/bin/nomina

USER app

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["nomina"]
