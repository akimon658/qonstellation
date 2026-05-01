FROM ghcr.io/denoland/deno:debian-2.7.14 AS builder

ARG GIT_REVISION
ENV DENO_DEPLOYMENT_ID=${GIT_REVISION}

WORKDIR /app

COPY . .

RUN deno task build

FROM ghcr.io/denoland/deno:distroless-2.7.14

WORKDIR /app
COPY --from=builder /app/_fresh/ ./

EXPOSE 8000

CMD ["serve", "--allow-env", "--allow-net", "--allow-read", "./_fresh/server.js"]
