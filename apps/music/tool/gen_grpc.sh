#!/usr/bin/env bash
# Generate the Dart gRPC client stubs for the Cymbra ID backend (task 2.2).
#
# Reads the backend's public protos (auth.proto, user.proto) and writes the
# generated Dart into lib/src/grpc/ (gitignored, like lib/src/rust/). The output
# is consumed only by the production gRPC service adapters, which are excluded
# from the coverage gate (task 2.4). Run via `melos run gen-grpc`.
#
# Requires: `protoc` on PATH (brew: `protobuf`, apt: `protobuf-compiler`) and the
# Dart plugin `protoc-gen-dart` (installed below from the pinned `protoc_plugin`
# dev dependency).
set -euo pipefail

# Resolve repo paths relative to this script (apps/music/tool/).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

OUT_DIR="$APP_DIR/lib/src/grpc"
AUTH_PROTO_DIR="$REPO_ROOT/backend/auth-port/proto"
USER_PROTO_DIR="$REPO_ROOT/backend/user-port/proto"

command -v protoc >/dev/null 2>&1 || {
  echo "error: protoc not found on PATH (brew install protobuf)" >&2
  exit 1
}

# Install the Dart codegen plugin and expose it on PATH. Pin the version so the
# generated code matches the `protobuf` runtime the app resolves (protoc_plugin
# 22.x ↔ protobuf 4.x); the latest plugin emits code for a newer runtime.
PROTOC_PLUGIN_VERSION="22.5.0"
dart pub global activate protoc_plugin "$PROTOC_PLUGIN_VERSION" >/dev/null
# pub global installs `protoc-gen-dart` under a per-OS cache bin dir; add the
# known locations (Unix, plus Windows Git Bash) so protoc can find the plugin.
export PATH="$PATH:$HOME/.pub-cache/bin:${LOCALAPPDATA:-}/Pub/Cache/bin:${APPDATA:-}/Pub/Cache/bin"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

protoc \
  --proto_path="$AUTH_PROTO_DIR" \
  --proto_path="$USER_PROTO_DIR" \
  --dart_out=grpc:"$OUT_DIR" \
  auth.proto user.proto

echo "Generated gRPC Dart stubs into $OUT_DIR"
