#!/usr/bin/env bash
# Generate the Dart gRPC stubs for the shared feature-flag client (change:
# add-runtime-feature-flags). Reads backend/feature-flags/proto/flags.proto and
# writes into lib/src/grpc/ (gitignored, like the app's stubs). Self-contained:
# flags.proto has no imports, so the package owns its own flag stubs.
#
# Requires: `protoc` on PATH + the Dart plugin `protoc-gen-dart` (pinned via the
# same protoc_plugin version the app uses so the generated code matches the
# `protobuf` runtime).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$PKG_DIR/../.." && pwd)"

OUT_DIR="$PKG_DIR/lib/src/grpc"
FLAGS_PROTO_DIR="$REPO_ROOT/backend/feature-flags/proto"

command -v protoc >/dev/null 2>&1 || {
  echo "error: protoc not found on PATH (brew install protobuf)" >&2
  exit 1
}

PROTOC_PLUGIN_VERSION="22.5.0"
dart pub global activate protoc_plugin "$PROTOC_PLUGIN_VERSION" >/dev/null
export PATH="$PATH:$HOME/.pub-cache/bin:${LOCALAPPDATA:-}/Pub/Cache/bin:${APPDATA:-}/Pub/Cache/bin"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

to_native() {
  if command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else printf '%s' "$1"; fi
}

protoc \
  --proto_path="$(to_native "$FLAGS_PROTO_DIR")" \
  --dart_out=grpc:"$(to_native "$OUT_DIR")" \
  flags.proto

echo "Generated feature-flag gRPC Dart stubs into $OUT_DIR"
