# syntax=docker/dockerfile:1
FROM rust:1.98.0-slim-trixie@sha256:17d1ba895198f9934c6314ec5346a0d5115372f3243390c3d731e242f35c2f27 AS builder

WORKDIR /app

ENV CARGO_TARGET_DIR=/tmp/target

RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    pkg-config

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=bind,source=.,target=. \
    cargo fetch --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/tmp/target,sharing=locked \
    --mount=type=bind,target=. \
    SQLX_OFFLINE=true cargo build  --release && \
    cp /tmp/target/release/qonstellation /tmp/qonstellation

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:c31ff9abcb1910f3ab25c7957bdaf0bfe12a01eb546e8df2282f1c8f682b606c

WORKDIR /app

COPY --from=builder /tmp/qonstellation /app/qonstellation

EXPOSE 8000

ENTRYPOINT ["/app/qonstellation"]
