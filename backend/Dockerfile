# Cymbra backend image — builds the server + worker binaries (plus the one-off
# `backfill-titles` maintenance bin, run via `docker compose run --rm server
# backfill-titles …`) into one small runtime image; the compose file picks the
# binary per service via `command:`. Build context is the REPO ROOT (the workspace
# Cargo.lock lives there):
#   docker build -f backend/Dockerfile -t cymbra-backend .
#
# The workspace also contains the Flutter FFI crates under apps/*/rust; they are
# NOT compiled here because `cargo build -p …` only builds the selected packages.
# Everything is rustls (no OpenSSL), so the only extra build dep is protoc.

# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
# Migrations are embedded at compile time via sqlx::migrate!, so the runtime image
# needs no SQL files. cache mounts keep rebuilds fast in CI.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p cymbra-server -p cymbra-worker \
    && cp target/release/cymbra-server target/release/cymbra-worker \
          target/release/backfill-titles /usr/local/bin/

FROM debian:bookworm-slim AS runtime
# ca-certificates: outbound TLS to Google/Apple JWKS, OIDC discovery, SMTP.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/cymbra-server /usr/local/bin/cymbra-server
COPY --from=builder /usr/local/bin/cymbra-worker /usr/local/bin/cymbra-worker
USER 1000:1000
# Overridden per service in docker-compose (server vs worker).
CMD ["cymbra-server"]
