# syntax=docker/dockerfile:1
#
# Variáveis em runtime: defina no Coolify (ou compose / -e). Obrigatórias na prática:
#   DATABASE_URL, API_KEY_ADMIN
# Opcionais (o binário tem defaults se omitir): PORTA, DIRETORIO_ARMAZENAMENTO, BASE_URL

FROM rust:1.94-bookworm AS builder
WORKDIR /app

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --home-dir /app --shell /usr/sbin/nologin app \
    && mkdir -p /app/armazenamento \
    && chown -R app:app /app

WORKDIR /app

COPY --from=builder --chown=app:app /app/target/release/file_storage_service /app/file_storage_service

USER app

EXPOSE 3000

ENTRYPOINT ["/app/file_storage_service"]
