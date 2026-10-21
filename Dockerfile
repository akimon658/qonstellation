# syntax=docker/dockerfile:1
FROM rust:1.98.1-slim-trixie@sha256:ce84a5edd80c5f91e05c5533b1e53eb1da54028f33734dc06aa6b49fa190462d AS builder

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

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:83b9b50586253821cf63ffcec87d3d94265a093ad30da232039af962dd8f4edf

WORKDIR /app

COPY --from=builder /tmp/qonstellation /app/qonstellation

EXPOSE 8000

ENTRYPOINT ["/app/qonstellation"]
