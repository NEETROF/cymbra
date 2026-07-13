# Headless MuseScore 4 (MuseScore Studio) for the score-crawler's `docker`
# converter backend. Exposes `mscore` on PATH so the crawler's generic call
#
#   docker run --rm -e QT_QPA_PLATFORM=offscreen -v <tmp>:/work <image> \
#       mscore -o /work/output.mxl /work/input.mscx
#
# converts OpenScore's CC0 `.mscx`/`.mscz` to `.mxl` with no display attached.
#
# Build (context is the crate dir; nothing is COPYed in):
#   docker build -f crates/score-crawler/musescore.Dockerfile \
#       -t cymbra/musescore crates/score-crawler
#
# Ubuntu 24.04 (glibc 2.39, libstdc++ from gcc 14) satisfies MuseScore 4.7's
# bundled Qt6, which needs GLIBC_2.38 / GLIBCXX_3.4.32 — newer than Debian
# bookworm ships. MuseScore only ships an AppImage for Linux; we extract it at
# build time (`--appimage-extract`, no FUSE mount) and run its AppRun directly,
# so no FUSE is needed at container run time.
FROM ubuntu:24.04

# Pin a concrete MuseScore Studio release; override with --build-arg to bump.
# The AppImage is arch-specific — pick the one matching the build platform so the
# image builds natively on both arm64 (Apple Silicon) and amd64 (servers).
ARG MSCORE_VERSION=4.7.4.260706075
ARG MSCORE_TAG=v4.7.4
# TARGETARCH is provided by BuildKit: "amd64" | "arm64".
ARG TARGETARCH

# --- Layer 1: fetch + extract the AppImage (curl + libfuse2 only). Isolated so
# tweaking the runtime libs below never re-downloads the ~190 MB AppImage. ---
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && (apt-get install -y --no-install-recommends libfuse2t64 \
        || apt-get install -y --no-install-recommends libfuse2) \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/musescore
RUN case "${TARGETARCH:-amd64}" in \
      arm64) MSARCH=aarch64 ;; \
      amd64) MSARCH=x86_64 ;; \
      *) echo "unsupported arch: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && URL="https://github.com/musescore/MuseScore/releases/download/${MSCORE_TAG}/MuseScore-Studio-${MSCORE_VERSION}-${MSARCH}.AppImage" \
    && echo "fetching $URL" \
    && curl -fsSL "$URL" -o mscore.AppImage \
    && chmod +x mscore.AppImage \
    && ./mscore.AppImage --appimage-extract >/dev/null \
    && rm mscore.AppImage

# `mscore` shim: rewrites the crawler's `mscore -o OUT IN` into MuseScore's
# offscreen batch-job mode (the one headless combination that converts and exits
# cleanly). See the script header for the why. Separate layer so shim tweaks
# don't re-download the AppImage.
COPY musescore-mscore.sh /usr/local/bin/mscore
RUN chmod +x /usr/local/bin/mscore

# --- Layer 2: system libraries the bundled Qt6 needs at run time (GL/EGL, xcb,
# fontconfig, dbus, nss, ALSA/Pulse). Qt6 and glib are bundled in the AppImage;
# these are the ones it dlopens from the system. ALSA/Pulse got the `t64` rename
# on 24.04, so try that first and fall back. ---
RUN apt-get update && apt-get install -y --no-install-recommends \
      fontconfig libfontconfig1 libfreetype6 \
      libdbus-1-3 libnss3 libgssapi-krb5-2 \
      libgl1 libegl1 libglx0 libopengl0 libxkbcommon0 \
      libx11-6 libxext6 libxrender1 libxcomposite1 libxdamage1 \
      libxrandr2 libxi6 libxtst6 libxfixes3 libxcb1 \
      libxcb-cursor0 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 \
      libxcb-randr0 libxcb-render-util0 libxcb-shape0 libxcb-xinerama0 \
    && (apt-get install -y --no-install-recommends libglib2.0-0t64 \
        || apt-get install -y --no-install-recommends libglib2.0-0) \
    && (apt-get install -y --no-install-recommends libasound2t64 \
        || apt-get install -y --no-install-recommends libasound2) \
    && (apt-get install -y --no-install-recommends libpulse0 || true) \
    && rm -rf /var/lib/apt/lists/*

# MuseScore writes config/cache/session state under $HOME; give it a writable
# one (the default root HOME=/root works, but be explicit for non-root runs).
ENV HOME=/tmp \
    QT_QPA_PLATFORM=offscreen \
    XDG_RUNTIME_DIR=/tmp
ENTRYPOINT []
CMD ["mscore", "--version"]
