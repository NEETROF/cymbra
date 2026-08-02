#!/usr/bin/env bash
# Cymbra — seed the downloadable SoundFont catalog from a curated manifest.
#
# Sourcing (NOT a crawler): for each licence-cleared entry in soundfonts.json it
# fetches the font, validates it, and pushes it through the SAME admin upload route
# the back office uses (POST /soundfonts/{id}); the server stores the object in the
# private bucket and records the music.soundfonts row (id, licence, attribution, …).
# No DB or bucket access needed — just an admin account.
#
# Licence validation stays a HUMAN step: the manifest only ever holds fonts whose
# licence you have verified allows in-app redistribution (CC0 / CC-BY). This script
# refuses to invent that judgement.
#
# Usage:
#   BASE_URL=http://localhost:8081 \
#   CYMBRA_EMAIL=admin@example.com \
#     bash backend/scripts/seed_soundfonts.sh [manifest.json]
#
# Auth: signs in with email+password (local sign-in) to obtain a short-lived access
# token, then uploads with it. The account must be moderator/admin in the `music`
# scope (see seed_admin.sh; in prod nobody is yet — target local/staging). The
# password is read from $CYMBRA_PASSWORD, or prompted (never echoed, never logged).
#
# Env:
#   BASE_URL          server origin (default http://localhost:8081)
#   CYMBRA_EMAIL      admin email (prompted if unset)
#   CYMBRA_PASSWORD   admin password (prompted with -s if unset)
#   AUDIENCE          token audience / scope (default music)
#
# SF2 only: rustysynth rejects compressed SF3 (the `OggS` sample format), so the
# catalog holds uncompressed SF2 exclusively. A source that turns out to be SF3 is
# skipped with a clear message (curate the manifest with SF2 sources instead).
#
# Deps: curl, jq.
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8081}"
BASE_URL="${BASE_URL%/}"
AUDIENCE="${AUDIENCE:-music}"
MANIFEST="${1:-$(dirname "$0")/soundfonts.json}"
CSRF_HEADER="x-cymbra-web"
MAX_BYTES=$((400 * 1024 * 1024)) # mirrors the server's upload cap

for bin in curl jq; do
  command -v "$bin" >/dev/null 2>&1 || { echo "error: '$bin' is required" >&2; exit 1; }
done
[[ -f "$MANIFEST" ]] || { echo "error: manifest not found: $MANIFEST" >&2; exit 1; }

# --- Credentials -----------------------------------------------------------
EMAIL="${CYMBRA_EMAIL:-}"
if [[ -z "$EMAIL" ]]; then read -r -p "Cymbra admin email: " EMAIL; fi
PASSWORD="${CYMBRA_PASSWORD:-}"
if [[ -z "$PASSWORD" ]]; then read -r -s -p "Password for $EMAIL: " PASSWORD; echo; fi
[[ -n "$EMAIL" && -n "$PASSWORD" ]] || { echo "error: email and password are required" >&2; exit 1; }

# --- Sign in → access token ------------------------------------------------
# POST /web/auth/signin  {kind:local,email,password,audience} → {accessToken}
signin_body="$(jq -n --arg e "$EMAIL" --arg p "$PASSWORD" --arg a "$AUDIENCE" \
  '{kind:"local",email:$e,password:$p,audience:$a}')"
signin_resp="$(curl -sS -w $'\n%{http_code}' \
  -X POST "$BASE_URL/web/auth/signin" \
  -H "$CSRF_HEADER: 1" -H 'Content-Type: application/json' \
  --data "$signin_body")"
signin_code="${signin_resp##*$'\n'}"
signin_json="${signin_resp%$'\n'*}"
if [[ "$signin_code" != "200" ]]; then
  echo "error: sign-in failed (HTTP $signin_code): $(jq -r '.error // .' <<<"$signin_json" 2>/dev/null)" >&2
  exit 1
fi
TOKEN="$(jq -r '.accessToken' <<<"$signin_json")"
[[ -n "$TOKEN" && "$TOKEN" != "null" ]] || { echo "error: no access token in sign-in response" >&2; exit 1; }
unset PASSWORD signin_body signin_resp signin_json # don't keep secrets around
echo "signed in as $EMAIL (audience=$AUDIENCE)"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# First 12 bytes are 'RIFF' <size> 'sfbk' for any SoundFont container.
is_sf2() { [[ "$(head -c4 "$1" 2>/dev/null)" == "RIFF" && "$(dd if="$1" bs=1 skip=8 count=4 2>/dev/null)" == "sfbk" ]]; }
# Compressed SF3 stores Ogg-Vorbis samples → the 'OggS' magic appears in the pool.
# Uncompressed SF2 (what rustysynth needs) does not. This mirrors rustysynth's own
# `four_cc == b"OggS"` reject check, so we catch an unplayable font before upload.
looks_sf3() { LC_ALL=C grep -qa 'OggS' "$1"; }

# Resolve the entry's payload to a single .sf2 path, extracting an archive if needed.
resolve_sf2() {
  local src="$1" dir; dir="$(dirname "$src")"
  if is_sf2 "$src"; then printf '%s' "$src"; return 0; fi
  case "$(file -b --mime-type "$src" 2>/dev/null)" in
    application/zip)                unzip -qo "$src" -d "$dir/x" ;;
    application/x-tar|application/gzip|application/x-bzip2|application/x-xz)
                                    mkdir -p "$dir/x" && tar -xf "$src" -C "$dir/x" ;;
    *) return 1 ;;
  esac
  local found; found="$(find "$dir/x" -type f -iname '*.sf2' | head -1)"
  [[ -n "$found" ]] && is_sf2 "$found" && { printf '%s' "$found"; return 0; }
  return 1
}

ok=0; skipped=0; failed=0
count="$(jq '.fonts | length' "$MANIFEST")"
for i in $(seq 0 $((count - 1))); do
  id="$(jq -r ".fonts[$i].id" "$MANIFEST")"
  label="$(jq -r ".fonts[$i].label" "$MANIFEST")"
  license="$(jq -r ".fonts[$i].license" "$MANIFEST")"
  attribution="$(jq -r ".fonts[$i].attribution // \"\"" "$MANIFEST")"
  instrument="$(jq -r ".fonts[$i].instrument // \"piano\"" "$MANIFEST")"
  url="$(jq -r ".fonts[$i].url" "$MANIFEST")"

  echo "── [$((i + 1))/$count] $id — $label ($license)"
  if [[ -z "$label" || -z "$license" || "$license" == "null" ]]; then
    echo "   skip: label and license are required in the manifest"; skipped=$((skipped + 1)); continue
  fi
  if [[ "$url" != http*://* ]]; then
    echo "   skip: url is not a fetchable http(s) link (got: $url)"; skipped=$((skipped + 1)); continue
  fi

  raw="$WORK/$id.bin"
  if ! curl -fSL --max-filesize "$MAX_BYTES" -o "$raw" "$url"; then
    echo "   fail: download error ($url)"; failed=$((failed + 1)); continue
  fi
  sf2="$(resolve_sf2 "$raw")" || { echo "   fail: no valid .sf2 found in payload"; failed=$((failed + 1)); continue; }
  if looks_sf3 "$sf2"; then
    echo "   skip: compressed SF3 — rustysynth can't play it. Use an uncompressed SF2 source instead."
    skipped=$((skipped + 1)); continue
  fi

  # POST /soundfonts/{id}?label&license&attribution&instrument  (raw .sf2 body)
  q="$(jq -rn --arg l "$label" --arg lic "$license" --arg a "$attribution" --arg ins "$instrument" \
    '@uri "label=\($l)&license=\($lic)&attribution=\($a)&instrument=\($ins)"')"
  code="$(curl -sS -o /dev/null -w '%{http_code}' \
    -X POST "$BASE_URL/soundfonts/$id?$q" \
    -H "Authorization: Bearer $TOKEN" \
    --data-binary "@$sf2")"
  case "$code" in
    201) echo "   ok: uploaded ($(wc -c <"$sf2" | tr -d ' ') bytes)"; ok=$((ok + 1)) ;;
    409) echo "   skip: already in catalog (id exists)"; skipped=$((skipped + 1)) ;;
    401|403) echo "   fail: not authorised — is $EMAIL moderator/admin in scope '$AUDIENCE'? (HTTP $code)"; failed=$((failed + 1)) ;;
    *)   echo "   fail: upload rejected (HTTP $code)"; failed=$((failed + 1)) ;;
  esac
done

echo "──────────────────────────────────────"
echo "done: $ok uploaded, $skipped skipped, $failed failed"
[[ "$failed" -eq 0 ]]
