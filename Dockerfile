FROM ghcr.io/denoland/deno:distroless-2.7.14

ARG GIT_REVISION
ENV DENO_DEPLOYMENT_ID=${GIT_REVISION}

WORKDIR /app

COPY . .

EXPOSE 8000

CMD ["run", "--allow-env", "--allow-net", "--allow-read", "./_fresh/server.js"]
