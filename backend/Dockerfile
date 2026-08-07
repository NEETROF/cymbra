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
          target/release/backfill-titles /usr/local/bin/ \
    # Strip debug symbols here (Dockerfile-scoped) rather than via a workspace-wide
    # [profile.release] strip=true, which would also touch the Flutter FFI crates'
    # release builds (apps/*/rust) — out of scope for this image.
    && strip --strip-all /usr/local/bin/cymbra-server /usr/local/bin/cymbra-worker \
             /usr/local/bin/backfill-titles

# Distroless "cc" runtime: glibc + libgcc + libstdc++ + ca-certificates, no shell,
# no package manager. `ldd` on the built binaries shows only libc/libgcc/libm (the
# stack is all-rustls — no OpenSSL, no libpq), so this covers everything they need
# while cutting the base layer from ~136MB (debian:bookworm-slim) to ~48MB. Base
# (not :nonroot) so USER 1000:1000 below matches the host-side `chown 1000:1000`
# the compose file already documents for the score/soundfont bind mounts.
# Pinned by digest (not :latest) so the base can't drift between CI builds without
# a deliberate bump — refresh with `docker pull gcr.io/distroless/cc-debian12 &&
# docker inspect gcr.io/distroless/cc-debian12 --format '{{index .RepoDigests 0}}'`.
FROM gcr.io/distroless/cc-debian12@sha256:e8e7ee4b8b106d4c5fde9e422a321b2b8a2d5cca546c97adcce927f3e1d36e36 AS runtime
COPY --from=builder /usr/local/bin/cymbra-server /usr/local/bin/cymbra-server
COPY --from=builder /usr/local/bin/cymbra-worker /usr/local/bin/cymbra-worker
# One-off maintenance bin, run via `docker compose run --rm server backfill-titles`.
COPY --from=builder /usr/local/bin/backfill-titles /usr/local/bin/backfill-titles
USER 1000:1000
# Overridden per service in docker-compose (server vs worker).
CMD ["cymbra-server"]
