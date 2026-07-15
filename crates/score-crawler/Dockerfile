# Self-contained score-crawler image: the binary plus the converters it needs,
# so `docker run score-crawler --sources …` just works with the default `local`
# converter backend (the tools are on PATH inside the container).
#
# Build from the repository root (the crate has path deps on crates/ + backend/):
#   docker build -f crates/score-crawler/Dockerfile -t score-crawler .
# Run (mount a host dir to keep the corpus):
#   docker run --rm -v "$PWD/out:/work/output" score-crawler --sources musetrainer --limit 5

FROM rust:1-slim AS build
# protobuf-compiler (protoc) is needed because the crate pulls in `cymbra-music`,
# whose build.rs compiles the ScoreService proto with tonic_build (same dep as
# backend/Dockerfile). Without it the build fails: "Could not find protoc".
RUN apt-get update \
    && apt-get install -y --no-install-recommends git libssl-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
# --locked: build strictly from the committed Cargo.lock (reproducible).
RUN cargo build --locked --release -p score-crawler

FROM python:3.12-slim
# Runtime converters: git (git-clone sources) + python-ly (LilyPond → MusicXML,
# for Mutopia). MuseScore (.mscx, OpenScore) is intentionally NOT installed here:
# Debian's musescore3 (3.2.3) is too old to read OpenScore's 3.6.1 files, and
# MuseScore 4 only ships as a 1.3 GB AppImage. For .mscx conversion, point the
# crawler at the purpose-built `cymbra/musescore` image via the `docker`
# converter backend (CYMBRA_SCORE_CONVERTER_BACKEND=docker) — see musescore.Dockerfile
# and the README. That spawns sibling containers, so the docker CLI is included.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && pip install --no-cache-dir --only-binary=:all: python-ly==0.9.10 \
    && rm -rf /var/lib/apt/lists/*
# The docker CLI (static, from the official image) so the `docker` converter
# backend can `docker run cymbra/musescore …` against a mounted host socket.
COPY --from=docker:cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=build /src/target/release/score-crawler /usr/local/bin/score-crawler
WORKDIR /work
ENTRYPOINT ["score-crawler"]
CMD ["--help"]
