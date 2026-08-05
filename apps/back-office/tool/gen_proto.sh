#!/usr/bin/env bash
# Generate the TypeScript protobuf + service descriptors for the back office from
# the backend's protos, into src/gen/ (gitignored, like the Flutter app's stubs).
# Uses protoc (brew: protobuf) + the local protoc-gen-es plugin (Connect ES v2:
# the *_pb.ts files carry both messages and service descriptors, consumed by
# @connectrpc/connect's createClient). Run via `yarn gen`.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

OUT_DIR="$APP_DIR/src/gen"
AUTH_PROTO_DIR="$REPO_ROOT/backend/auth-port/proto"
USER_PROTO_DIR="$REPO_ROOT/backend/user-port/proto"
MUSIC_PROTO_DIR="$REPO_ROOT/backend/music/proto"
# Shared, app-agnostic feature-flag service (change: add-runtime-feature-flags).
FLAGS_PROTO_DIR="$REPO_ROOT/backend/feature-flags/proto"
# Feature-usage analytics reporting (change: add-feature-usage-analytics).
ANALYTICS_PROTO_DIR="$REPO_ROOT/backend/analytics/proto"

command -v protoc >/dev/null 2>&1 || {
  echo "error: protoc not found on PATH (brew install protobuf)" >&2
  exit 1
}

PLUGIN="$APP_DIR/node_modules/.bin/protoc-gen-es"
[ -x "$PLUGIN" ] || {
  echo "error: protoc-gen-es not found — run 'yarn install' first" >&2
  exit 1
}

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

protoc \
  --proto_path="$AUTH_PROTO_DIR" \
  --proto_path="$USER_PROTO_DIR" \
  --proto_path="$MUSIC_PROTO_DIR" \
  --proto_path="$FLAGS_PROTO_DIR" \
  --proto_path="$ANALYTICS_PROTO_DIR" \
  --plugin=protoc-gen-es="$PLUGIN" \
  --es_out="$OUT_DIR" \
  --es_opt=target=ts \
  auth.proto user.proto score.proto flags.proto usage.proto

echo "Generated TS gRPC stubs into $OUT_DIR"
