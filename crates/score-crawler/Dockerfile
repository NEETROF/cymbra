# Self-contained score-crawler image: the binary plus the converters it needs,
# so `docker run score-crawler --sources …` just works with the default `local`
# converter backend (the tools are on PATH inside the container).
#
# Build from the repository root (the crate has path deps on crates/ + backend/):
#   docker build -f crates/score-crawler/Dockerfile -t score-crawler .
# Run (mount a host dir to keep the corpus):
#   docker run --rm -v "$PWD/out:/work/output" score-crawler --sources musetrainer --limit 5

FROM rust:1-slim AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev git \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --release -p score-crawler

FROM python:3.12-slim
# Runtime converters: git (git-clone sources) + python-ly (LilyPond → MusicXML,
# for Mutopia). MuseScore (.mscx, OpenScore) is intentionally NOT installed:
# Debian's musescore3 (3.2.3) is too old to read OpenScore's 3.6.1 files, and a
# newer MuseScore only ships as an AppImage that is painful to run headless in a
# container. For .mscx conversion, point the crawler at a purpose-built MuseScore
# image via the `docker` converter backend (converters.backend: docker) instead
# of bloating this image. See the README.
RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates \
    && pip install --no-cache-dir python-ly \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/score-crawler /usr/local/bin/score-crawler
WORKDIR /work
ENTRYPOINT ["score-crawler"]
CMD ["--help"]
