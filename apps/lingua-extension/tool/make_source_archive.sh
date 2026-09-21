#!/usr/bin/env bash
# Assemble the human-readable source Mozilla reviews into <out-dir>: everything the build in
# REVIEWERS.md reads, and nothing else of the monorepo.
#
# The add-on's crates inherit their edition and dependency versions from the workspace root
# (`edition.workspace = true`), so they do not build without a root Cargo.toml — and the real
# one globs `backend/*` and `apps/*/rust`, which are not in the archive. The archive carries a
# reduced root manifest instead: the same [workspace.package] and [workspace.dependencies]
# tables, with only the three Lingua crates as members. `yarn gen:proto` reads the .proto
# files of three backend crates, and `yarn gen:pack:real` runs the lingua-pack builder, so
# those come along too.
#
# The sign-in client ids are inlined into the bundle at build time from repository variables,
# not from git: the archive's README states the ones this package was built with, or a
# reviewer's rebuild differs from the submitted files by those strings. They are public
# identifiers — they ship in the package.
#
# Usage: tool/make_source_archive.sh <out-dir>   (run from anywhere inside the repository)
set -euo pipefail

OUT="${1:?usage: make_source_archive.sh <out-dir>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

PATHS=(
  apps/lingua-extension
  crates/lingua-core
  crates/lingua-wasm
  crates/lingua-pack
  scripts/lingua-data
  backend/auth-port/proto
  backend/lingua/proto
  backend/user-port/proto
  Cargo.lock
  rust-toolchain.toml
)
MEMBERS='"crates/lingua-core", "crates/lingua-wasm", "crates/lingua-pack"'

mkdir -p "$OUT"
git -C "$REPO_ROOT" archive HEAD "${PATHS[@]}" | tar -x -C "$OUT"

# The reduced workspace root: the shared tables verbatim, the members cut down to what is here.
{
  echo "# Reduced from the Cymbra monorepo's root manifest by make_source_archive.sh: the same"
  echo "# shared package and dependency tables, with only the crates in this archive as members."
  echo "[workspace]"
  echo 'resolver = "2"'
  echo "members = [$MEMBERS]"
  echo
  # A comment block belongs to the table header that follows it, so it is held back until
  # that header says whether it is kept.
  awk '
    /^[[:space:]]*(#|$)/ { held = held $0 "\n"; next }
    /^\[/ { keep = ($0 ~ /^\[workspace\.(package|dependencies)/) }
    { if (keep) printf "%s%s\n", held, $0; held = "" }
  ' "$REPO_ROOT/Cargo.toml"
} >"$OUT/Cargo.toml"

cp "$REPO_ROOT/apps/lingua-extension/REVIEWERS.md" "$OUT/README.md"
cat >>"$OUT/README.md" <<EOF

## What this package was built with

The submitted package was built from this archive with these variables set; the two client
ids are public identifiers the bundle carries, and an empty one hides that sign-in button.

\`\`\`sh
LINGUA_GRPC_WEB_URL=${LINGUA_GRPC_WEB_URL:-https://api.cymbra.app}
LINGUA_GOOGLE_CLIENT_ID=${LINGUA_GOOGLE_CLIENT_ID:-}
LINGUA_APPLE_CLIENT_ID=${LINGUA_APPLE_CLIENT_ID:-}
LINGUA_EXT_KEY=
\`\`\`
EOF

echo "Source archive assembled in $OUT"
