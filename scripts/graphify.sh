#!/usr/bin/env bash
#
# graphify.sh — build the three per-stack code knowledge graphs for this monorepo.
#
# Opt-in dev tooling (like rtk/caveman): it helps an AI assistant answer
# "what calls X" / "blast radius of changing Y" across the codebase from a local
# knowledge graph instead of grepping blindly. Nothing here runs in CI or is
# required to build the app. See the "Code knowledge graphs" section in CLAUDE.md.
#
# Prerequisite (once per machine):
#   uv tool install graphifyy && graphify install
#
# Usage:
#   scripts/graphify.sh          # build/refresh all three graphs
#   scripts/graphify.sh rust     # only the Rust workspace graph
#   scripts/graphify.sh flutter  # only apps/music
#   scripts/graphify.sh vue      # only apps/back-office
#   scripts/graphify.sh install-hook    # opt-in: refresh graphs in the background after every commit
#   scripts/graphify.sh uninstall-hook  # remove that hook
#
# The hook is per-machine (written to the shared .git/hooks, NOT committed) so it
# never forces a rebuild on teammates who don't use the graph. It rebuilds in the
# background, so commits return instantly. One install covers every worktree.
#
# Graphs are written to (all git-ignored):
#   graphify-out/graph.json                    <- Rust workspace (backend + crates + apps/music/rust)
#   apps/music/graphify-out/graph.json         <- Flutter app
#   apps/back-office/graphify-out/graph.json   <- Vue back office
#
# Query them (from repo root), e.g.:
#   graphify god-nodes --top 10
#   graphify explain "EnqueueRequest"
#   graphify affected "EnqueueRequest" --depth 2
#   graphify god-nodes --graph apps/music/graphify-out/graph.json
#
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if ! command -v graphify >/dev/null 2>&1; then
  echo "error: 'graphify' not found. Install once with: uv tool install graphifyy && graphify install" >&2
  exit 1
fi

TARGET="${1:-all}"
TMP="$(mktemp -d)"
# Always clean up temp dir and any .graphifyignore files we drop in the tree.
cleanup() {
  rm -rf "$TMP"
  rm -f "$ROOT/apps/music/rust/.graphifyignore" "$ROOT/apps/music/lib/.graphifyignore"
}
trap cleanup EXIT

# extract <src-path> <out-dir>  — local AST only, no LLM, no API key, no tokens.
extract() { graphify extract "$1" --code-only --out "$2" 2>&1 | grep -E 'wrote .*graph.json' | sed 's#.*graphify-out#  graphify-out#' || true; }

build_rust() {
  echo "== Rust workspace (backend + crates + apps/music/rust) =="
  # frb_generated.rs is the generated FFI bridge — exclude it from the FFI crate.
  printf 'src/frb_generated.rs\n' > apps/music/rust/.graphifyignore
  extract backend           "$TMP/rust-backend"
  extract crates            "$TMP/rust-crates"
  extract apps/music/rust   "$TMP/rust-ffi"
  rm -f apps/music/rust/.graphifyignore
  # Merge the three Rust roots into one workspace graph at the repo root.
  graphify merge-graphs \
    "$TMP/rust-backend/graphify-out/graph.json" \
    "$TMP/rust-crates/graphify-out/graph.json" \
    "$TMP/rust-ffi/graphify-out/graph.json" \
    --out "$ROOT/graphify-out/graph.json" 2>&1 | grep -E 'Merged|Written' | sed 's#^#  #'
}

build_flutter() {
  echo "== Flutter app (apps/music/lib) =="
  # Exclude the generated frb Dart bindings and codegen outputs.
  printf 'src/rust/\n*.g.dart\n*.freezed.dart\n' > apps/music/lib/.graphifyignore
  extract apps/music/lib "$ROOT/apps/music"
  rm -f apps/music/lib/.graphifyignore
}

build_vue() {
  echo "== Vue back office (apps/back-office/src) =="
  extract apps/back-office/src "$ROOT/apps/back-office"
}

hook_path() { echo "$(git rev-parse --git-common-dir)/hooks/post-commit"; }

install_hook() {
  local hook; hook="$(hook_path)"
  if [ -e "$hook" ] && ! grep -q 'graphify auto-refresh' "$hook" 2>/dev/null; then
    echo "error: a post-commit hook already exists at $hook — not overwriting." >&2
    echo "       Add this line to it yourself: (\"\$(git rev-parse --show-toplevel)/scripts/graphify.sh\" all >/dev/null 2>&1 &)" >&2
    exit 1
  fi
  mkdir -p "$(dirname "$hook")"
  cat > "$hook" <<'HOOK'
#!/usr/bin/env sh
# graphify auto-refresh — rebuild code knowledge graphs in the background.
# Installed by scripts/graphify.sh install-hook. Local only, not committed.
ROOT="$(git rev-parse --show-toplevel)"
[ -x "$ROOT/scripts/graphify.sh" ] || exit 0   # no-op where the script is absent
( "$ROOT/scripts/graphify.sh" all >"${TMPDIR:-/tmp}/graphify-refresh.log" 2>&1 & ) >/dev/null 2>&1
exit 0
HOOK
  chmod +x "$hook"
  echo "installed background post-commit hook at $hook (covers all worktrees; log: \${TMPDIR:-/tmp}/graphify-refresh.log)"
}

uninstall_hook() {
  local hook; hook="$(hook_path)"
  if [ -e "$hook" ] && grep -q 'graphify auto-refresh' "$hook" 2>/dev/null; then
    rm -f "$hook"; echo "removed $hook"
  else
    echo "no graphify post-commit hook to remove."
  fi
}

case "$TARGET" in
  all)     build_rust; build_flutter; build_vue ;;
  rust)    build_rust ;;
  flutter) build_flutter ;;
  vue)     build_vue ;;
  install-hook)   install_hook; trap - EXIT; exit 0 ;;
  uninstall-hook) uninstall_hook; trap - EXIT; exit 0 ;;
  *) echo "usage: scripts/graphify.sh [all|rust|flutter|vue|install-hook|uninstall-hook]" >&2; exit 2 ;;
esac

echo "done."
