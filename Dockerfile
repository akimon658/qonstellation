#syntax=docker/dockerfile:1
FROM ghcr.io/denoland/deno:debian-2.8.1@sha256:ddaad47cbbbbd856d73bd0d50074a0e308c51671d83442eebb15f1039dd4a822 AS builder

WORKDIR /app

RUN --mount=type=bind,source=deno.jsonc,target=deno.jsonc \
    --mount=type=bind,source=deno.lock,target=deno.lock \
    --mount=type=cache,target=/deno-dir/ \
    deno install --frozen

COPY . .

RUN --mount=type=cache,target=/deno-dir/ \
    deno task build

FROM ghcr.io/denoland/deno:distroless-2.8.1@sha256:2005d7c2aed55c198dcf97df5a3d4d1926a87a80b5bb8b5175a607b18b319f7b

WORKDIR /app
COPY --from=builder /app/_fresh/ ./

EXPOSE 8000
ARG GIT_REVISION
ENV DENO_DEPLOYMENT_ID=${GIT_REVISION}

CMD ["serve", "--allow-env", "--allow-net", "--allow-read", "--v8-flags=--expose-gc", "/app/server.js"]
