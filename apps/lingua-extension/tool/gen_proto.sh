#!/usr/bin/env bash
# Generate the TypeScript protobuf + Connect-ES service descriptors the extension
# calls over gRPC-web, into src/gen/ (gitignored, like the back office's and the
# Flutter app's stubs). Uses protoc (brew: protobuf) + the local protoc-gen-es plugin
# (Connect ES v2: the *_pb.ts files carry both messages and service descriptors,
# consumed by @connectrpc/connect's createClient). Run via `yarn gen:proto`.
#
# The auth service (sign in / refresh / logout), the user service (the account's handle —
# add-lingua-account-parity) plus the cymbra.lingua.v1 client services: the three sync
# services (known words, deck, stats) of add-lingua-connected-clients and the reader's
# data controls (add-lingua-privacy-controls), and « Traduction en ligne », the one RPC that carries
# page text (add-lingua-online-translation), and the anonymous usage counts, sent with no
# account at all (add-lingua-usage-counts). The lingua admin
# service is NOT here: that is the back office's, not a client's.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

OUT_DIR="$APP_DIR/src/gen"
AUTH_PROTO_DIR="$REPO_ROOT/backend/auth-port/proto"
LINGUA_PROTO_DIR="$REPO_ROOT/backend/lingua/proto"
USER_PROTO_DIR="$REPO_ROOT/backend/user-port/proto"
# The caller's effective flags (change: add-lingua-feature-betas).
FLAGS_PROTO_DIR="$REPO_ROOT/backend/feature-flags/proto"

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
  --proto_path="$LINGUA_PROTO_DIR" \
  --proto_path="$USER_PROTO_DIR" \
  --proto_path="$FLAGS_PROTO_DIR" \
  --plugin=protoc-gen-es="$PLUGIN" \
  --es_out="$OUT_DIR" \
  --es_opt=target=ts \
  auth.proto user.proto known_words.proto deck.proto stats.proto lingua_data.proto online_translation.proto flags.proto \
  usage_counts.proto

echo "Generated TS gRPC stubs into $OUT_DIR"
