FROM ghcr.io/denoland/deno:distroless-2.7.14

ARG GIT_REVISION
ENV DENO_DEPLOYMENT_ID=${GIT_REVISION}

WORKDIR /app

COPY . .

RUN deno task build

EXPOSE 8000

CMD ["deno", "serve", "--allow-env", "--allow-net", "_fresh/server/server_entry.mjs"]
