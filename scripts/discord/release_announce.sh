#!/usr/bin/env bash
# Cymbra — announce a GitHub Release on Discord.
#
# A release is a CI event, not a product event: release-please writes the notes
# from the Conventional Commits, so this script only has to reshape them for
# Discord and post them. The backend is not involved and never learns about
# releases.
#
# Called from two places:
#   - .github/workflows/release-build.yml, as a final job that waits for every
#     platform to attach its artifacts — announcing earlier would link a release
#     page with no downloads on it;
#   - .github/workflows/discord-release.yml, for the components that have no
#     artifacts to wait for (backend, back-office).
#
# Environment:
#   TAG                   release tag, e.g. music-v1.2.0            (required)
#   DISCORD_WEBHOOK_URL   target channel webhook                    (absent ⇒ skipped)
#   GH_TOKEN              token for `gh release view`               (required)
#   GITHUB_REPOSITORY     owner/repo (defaults to NEETROF/cymbra)
#   DRY_RUN=1             print the payload instead of posting it
#
# The webhook URL is a secret: it is never printed, not even on failure.
set -euo pipefail

TAG="${TAG:-}"
WEBHOOK="${DISCORD_WEBHOOK_URL:-}"
REPO="${GITHUB_REPOSITORY:-NEETROF/cymbra}"
DRY_RUN="${DRY_RUN:-0}"

# Discord caps an embed description at 4096 characters; keep room for the
# truncation marker and the changelog link.
MAX_DESC=3800
# A field value is capped at 1024 characters. Budget by LENGTH, not by asset
# count: a release's asset URLs run ~130 characters each, so a fixed count is one
# long filename away from a 400 from Discord. 950 leaves room for the "+N more".
MAX_DOWNLOADS_LEN=950
COLOR=5793266

[[ -n "$TAG" ]] || { echo "error: TAG is required (e.g. music-v1.2.0)" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "error: jq is required" >&2; exit 1; }
command -v gh >/dev/null 2>&1 || { echo "error: gh is required" >&2; exit 1; }

# An unconfigured webhook must not fail the release pipeline — same convention as
# the optional signing/TestFlight steps in release-build.yml.
if [[ -z "$WEBHOOK" && "$DRY_RUN" != 1 ]]; then
  echo "::warning::DISCORD_WEBHOOK_URL is not set — skipping the Discord announcement."
  exit 0
fi

# --- Product name from the release-please component prefix -----------------
case "$TAG" in
  music-v*)       PRODUCT="Cymbra Music" ;;
  backend-v*)     PRODUCT="Cymbra Backend" ;;
  back-office-v*) PRODUCT="Cymbra Back Office" ;;
  *)              PRODUCT="Cymbra" ;;
esac
VERSION="${TAG##*-v}"

echo "Announcing $PRODUCT $VERSION ($TAG)"

release="$(gh release view "$TAG" --repo "$REPO" --json body,url,assets)"
url="$(jq -r '.url' <<<"$release")"
body="$(jq -r '.body // ""' <<<"$release")"

# --- Reshape the release notes for Discord ---------------------------------
# 1. drop release-please's own "## [1.2.0](compare) (date)" heading — the embed
#    title already carries the product and version;
# 2. drop the trailing "([a1b2c3d](commit-url))" links, which are noise in chat;
# 3. squeeze the blank lines those removals leave behind.
description="$(
  printf '%s\n' "$body" \
    | sed -E -e '1{/^#{1,3} \[/d;}' -e 's/ ?\(\[[0-9a-f]{6,12}\]\([^)]+\)\)//g' \
    | cat -s \
    | awk 'found || NF {found = 1; print}' \
    | awk -v max="$MAX_DESC" '
        { if (len + length($0) + 1 > max) { print "…"; exit }
          print; len += length($0) + 1 }'
)"
[[ -n "${description//[[:space:]]/}" ]] || description="_No release notes._"
description="${description}"$'\n\n'"[Full changelog and downloads](${url})"

# --- Download links --------------------------------------------------------
downloads="$(jq -r --argjson max "$MAX_DOWNLOADS_LEN" '
  ((.assets // []) | map("[\(.name)](\(.url))")) as $links
  | ($links | length) as $n
  | (reduce range(0; $n) as $i ({kept: [], len: 0, stop: false};
       if .stop then .
       else (($links[$i] | length) + 3) as $add
         | if .len + $add <= $max
           then {kept: (.kept + [$links[$i]]), len: (.len + $add), stop: false}
           else .stop = true end
       end)) as $r
  | ($r.kept | join(" · "))
    + (if ($r.kept | length) < $n then " · +\($n - ($r.kept | length)) more" else "" end)
' <<<"$release")"

payload="$(jq -n \
  --arg title "$PRODUCT $VERSION" \
  --arg url "$url" \
  --arg desc "$description" \
  --arg downloads "$downloads" \
  --arg tag "$TAG" \
  --argjson color "$COLOR" '
  {
    username: "Cymbra",
    # Release notes can contain @handles; never let one become a real ping.
    allowed_mentions: {parse: []},
    embeds: [
      ({title: $title, url: $url, description: $desc, color: $color,
        footer: {text: $tag}}
       + (if ($downloads | length) > 0
          then {fields: [{name: "Downloads", value: $downloads}]}
          else {} end))
    ]
  }')"

if [[ "$DRY_RUN" == 1 ]]; then
  echo "--- payload (dry run, not posted) ---"
  jq . <<<"$payload"
  exit 0
fi

# ?wait=true makes Discord validate the payload and answer with the created
# message instead of an empty 204, so a malformed embed fails the job loudly.
code="$(curl -sS -o /tmp/discord_response.json -w '%{http_code}' \
  -X POST "${WEBHOOK}?wait=true" \
  -H 'Content-Type: application/json' \
  --data-binary "$payload")"

if [[ "$code" != 200 && "$code" != 204 ]]; then
  # Print Discord's error, never the URL that carries the secret token.
  echo "error: Discord rejected the announcement (HTTP $code)" >&2
  jq . /tmp/discord_response.json >&2 2>/dev/null || cat /tmp/discord_response.json >&2
  exit 1
fi

echo "Announced $PRODUCT $VERSION on Discord."
echo "::notice::Announcement posted. In an announcement channel, hit \"Publish\" on the message to crosspost it to following servers."
