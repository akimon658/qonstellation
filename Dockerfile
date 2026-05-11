#syntax=docker/dockerfile:1
FROM ghcr.io/denoland/deno:debian-2.7.14 AS builder

WORKDIR /app

RUN --mount=type=bind,source=deno.jsonc,target=deno.jsonc \
    --mount=type=bind,source=deno.lock,target=deno.lock \
    --mount=type=cache,target=/deno-dir/ \
    deno install --frozen

COPY . .

RUN --mount=type=cache,target=/deno-dir/ \
    deno task build

FROM ghcr.io/denoland/deno:distroless-2.7.14

WORKDIR /app
COPY --from=builder /app/dist/ ./

EXPOSE 8000
ARG GIT_REVISION
ENV DENO_DEPLOYMENT_ID=${GIT_REVISION}

CMD ["serve", "--allow-env", "--allow-net", "--allow-read", "./server.js"]
