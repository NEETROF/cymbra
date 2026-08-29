#!/usr/bin/env bash
# Cymbra — provision the community Discord server from a declarative manifest.
#
# What you MUST do by hand first (there is no API for it):
#   1. Create the server: Discord → "+" → "Create My Own". Name it anything; this
#      script renames it from the manifest.
#   2. Create the application + bot: https://discord.com/developers/applications
#      → New Application → Bot → Reset Token → copy it ONCE (it is shown once).
#   3. Invite the bot to the server with the right permissions. Run
#      `DISCORD_CLIENT_ID=<application id> bash provision.sh --invite-url`
#      to get the exact URL (the bitfield is computed from this manifest, because
#      a bot cannot GRANT a permission it does not itself hold).
#   4. Get the server id: Discord → Settings → Advanced → Developer Mode ON, then
#      right-click the server icon → "Copy Server ID".
#
# What this script then does, idempotently (safe to re-run; matches by NAME):
#   - patches the @everyone permissions, then creates/updates every role
#   - creates the categories and the text/voice channels, with their overwrites
#   - patches the guild (name, description, locale, verification, content filter)
#     and switches it to Community, which announcement + forum channels require
#   - creates the announcement and forum channels (Community-only types)
#   - creates/updates the AutoMod rules
#   - creates the per-channel webhooks the backend publishes through, and writes
#     their URLs to a 0600 file — they are SECRETS (anyone holding one can post
#     as Cymbra in that channel)
#
# Usage:
#   DISCORD_GUILD_ID=<server id> DISCORD_BOT_TOKEN=<bot token> \
#     bash scripts/discord/provision.sh [manifest.json]
#
#   DRY_RUN=1 …                     # read the server, print the writes, change nothing
#   WEBHOOKS_OUT=/path/secrets.env  # where to write webhook URLs (default below)
#   DISCORD_CLIENT_ID=… --invite-url
#
# The token is read from $DISCORD_BOT_TOKEN or prompted (never echoed, never
# logged, never written to the webhooks file).
#
# Deps: curl, jq.
set -euo pipefail

API="https://discord.com/api/v10"
UA="CymbraProvision/1.0 (+https://github.com/NEETROF/cymbra)"
HERE="$(cd "$(dirname "$0")" && pwd)"
MANIFEST="${HERE}/server.json"
DRY_RUN="${DRY_RUN:-0}"
WEBHOOKS_OUT="${WEBHOOKS_OUT:-${HERE}/.webhooks.env}"
MODE="apply"

for arg in "$@"; do
  case "$arg" in
    --invite-url) MODE="invite" ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *.json) MANIFEST="$arg" ;;
    *) echo "error: unknown argument '$arg'" >&2; exit 1 ;;
  esac
done

for bin in curl jq; do
  command -v "$bin" >/dev/null 2>&1 || { echo "error: '$bin' is required" >&2; exit 1; }
done
[[ -f "$MANIFEST" ]] || { echo "error: manifest not found: $MANIFEST" >&2; exit 1; }
jq empty "$MANIFEST" 2>/dev/null || { echo "error: manifest is not valid JSON: $MANIFEST" >&2; exit 1; }

log() { printf '  %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# --- Permissions -----------------------------------------------------------
# Discord permission bitfield. Kept as a case (not an associative array) so this
# runs on the bash 3.2 that ships with macOS.
perm_bit() {
  case "$1" in
    CREATE_INSTANT_INVITE)     echo $((1 << 0)) ;;
    KICK_MEMBERS)              echo $((1 << 1)) ;;
    BAN_MEMBERS)               echo $((1 << 2)) ;;
    ADMINISTRATOR)             echo $((1 << 3)) ;;
    MANAGE_CHANNELS)           echo $((1 << 4)) ;;
    MANAGE_GUILD)              echo $((1 << 5)) ;;
    ADD_REACTIONS)             echo $((1 << 6)) ;;
    VIEW_AUDIT_LOG)            echo $((1 << 7)) ;;
    PRIORITY_SPEAKER)          echo $((1 << 8)) ;;
    STREAM)                    echo $((1 << 9)) ;;
    VIEW_CHANNEL)              echo $((1 << 10)) ;;
    SEND_MESSAGES)             echo $((1 << 11)) ;;
    SEND_TTS_MESSAGES)         echo $((1 << 12)) ;;
    MANAGE_MESSAGES)           echo $((1 << 13)) ;;
    EMBED_LINKS)               echo $((1 << 14)) ;;
    ATTACH_FILES)              echo $((1 << 15)) ;;
    READ_MESSAGE_HISTORY)      echo $((1 << 16)) ;;
    MENTION_EVERYONE)          echo $((1 << 17)) ;;
    USE_EXTERNAL_EMOJIS)       echo $((1 << 18)) ;;
    VIEW_GUILD_INSIGHTS)       echo $((1 << 19)) ;;
    CONNECT)                   echo $((1 << 20)) ;;
    SPEAK)                     echo $((1 << 21)) ;;
    MUTE_MEMBERS)              echo $((1 << 22)) ;;
    DEAFEN_MEMBERS)            echo $((1 << 23)) ;;
    MOVE_MEMBERS)              echo $((1 << 24)) ;;
    USE_VAD)                   echo $((1 << 25)) ;;
    CHANGE_NICKNAME)           echo $((1 << 26)) ;;
    MANAGE_NICKNAMES)          echo $((1 << 27)) ;;
    MANAGE_ROLES)              echo $((1 << 28)) ;;
    MANAGE_WEBHOOKS)           echo $((1 << 29)) ;;
    MANAGE_GUILD_EXPRESSIONS)  echo $((1 << 30)) ;;
    USE_APPLICATION_COMMANDS)  echo $((1 << 31)) ;;
    REQUEST_TO_SPEAK)          echo $((1 << 32)) ;;
    MANAGE_EVENTS)             echo $((1 << 33)) ;;
    MANAGE_THREADS)            echo $((1 << 34)) ;;
    CREATE_PUBLIC_THREADS)     echo $((1 << 35)) ;;
    CREATE_PRIVATE_THREADS)    echo $((1 << 36)) ;;
    USE_EXTERNAL_STICKERS)     echo $((1 << 37)) ;;
    SEND_MESSAGES_IN_THREADS)  echo $((1 << 38)) ;;
    USE_EMBEDDED_ACTIVITIES)   echo $((1 << 39)) ;;
    MODERATE_MEMBERS)          echo $((1 << 40)) ;;
    *) return 1 ;;
  esac
}

perms_sum() { # perms_sum NAME NAME … → decimal bitfield
  local total=0 bit p
  for p in "$@"; do
    bit="$(perm_bit "$p")" || die "unknown permission '$p' in the manifest"
    total=$((total + bit))
  done
  printf '%s' "$total"
}

# --- Invite URL ------------------------------------------------------------
# The bot needs (a) what this script does, and (b) every permission the manifest
# hands to a role — Discord refuses to let a bot grant a permission it lacks.
if [[ "$MODE" == invite ]]; then
  CLIENT_ID="${DISCORD_CLIENT_ID:-}"
  [[ -n "$CLIENT_ID" ]] || die "DISCORD_CLIENT_ID (the application id) is required for --invite-url"
  script_perms=(MANAGE_GUILD MANAGE_ROLES MANAGE_CHANNELS MANAGE_WEBHOOKS
                VIEW_CHANNEL SEND_MESSAGES MANAGE_MESSAGES READ_MESSAGE_HISTORY)
  role_perms=()
  while IFS= read -r p; do [[ -n "$p" ]] && role_perms+=("$p"); done < <(
    jq -r '[.roles[].permissions[]?, .everyone_permissions[]?] | unique | .[]' "$MANIFEST"
  )
  # Deduplicate the union before summing (a bit added twice would double it).
  all="$(printf '%s\n' "${script_perms[@]}" "${role_perms[@]}" | sort -u)"
  # shellcheck disable=SC2086
  bits="$(perms_sum $all)"
  echo "Invite the bot with this URL, then re-run without --invite-url:"
  echo
  echo "https://discord.com/oauth2/authorize?client_id=${CLIENT_ID}&scope=bot%20applications.commands&permissions=${bits}"
  echo
  echo "Least-privilege bitfield: ${bits} (no Administrator)."
  exit 0
fi

# --- Credentials -----------------------------------------------------------
GUILD_ID="${DISCORD_GUILD_ID:-}"
if [[ -z "$GUILD_ID" ]]; then read -r -p "Discord server (guild) id: " GUILD_ID; fi
[[ -n "$GUILD_ID" ]] || die "a guild id is required"
TOKEN="${DISCORD_BOT_TOKEN:-}"
if [[ -z "$TOKEN" ]]; then read -r -s -p "Bot token (not echoed): " TOKEN; echo; fi
[[ -n "$TOKEN" ]] || die "a bot token is required"
EVERYONE_ID="$GUILD_ID" # the @everyone role id is always the guild id

# --- HTTP ------------------------------------------------------------------
api() { # api METHOD PATH [JSON_BODY] → response body on stdout
  local method="$1" path="$2" body="${3:-}" resp code wait attempts=0
  if [[ "$method" != GET && "$DRY_RUN" == 1 ]]; then
    printf '  [dry-run] %s %s %s\n' "$method" "$path" "${body:0:160}" >&2
    printf '{}'
    return 0
  fi
  while :; do
    if [[ -n "$body" ]]; then
      resp="$(curl -sS -w $'\n%{http_code}' -X "$method" "$API$path" \
        -H "Authorization: Bot $TOKEN" -H 'Content-Type: application/json' \
        -H "User-Agent: $UA" --data-binary "$body")"
    else
      resp="$(curl -sS -w $'\n%{http_code}' -X "$method" "$API$path" \
        -H "Authorization: Bot $TOKEN" -H "User-Agent: $UA")"
    fi
    code="${resp##*$'\n'}"
    resp="${resp%$'\n'*}"
    if [[ "$code" == 429 ]]; then
      attempts=$((attempts + 1))
      ((attempts <= 5)) || die "rate limited five times on $method $path — give it a minute"
      wait="$(jq -r '.retry_after // 2' <<<"$resp" 2>/dev/null || echo 2)"
      log "rate limited, waiting ${wait}s"
      sleep "$wait"
      continue
    fi
    if ((code >= 400)); then
      printf 'error: %s %s → HTTP %s\n%s\n' "$method" "$path" "$code" "$resp" >&2
      # 50013 = Missing Permissions: almost always the invite bitfield, not the code.
      grep -q '"code": *50013' <<<"$resp" && \
        printf 'hint: re-invite the bot with the URL from --invite-url\n' >&2
      return 1
    fi
    printf '%s' "$resp"
    return 0
  done
}

ROLES_JSON='[]'
CHANNELS_JSON='[]'
load_roles()    { ROLES_JSON="$(api GET "/guilds/$GUILD_ID/roles")"; }
load_channels() { CHANNELS_JSON="$(api GET "/guilds/$GUILD_ID/channels")"; }
role_id()    { jq -r --arg n "$1" 'map(select(.name == $n)) | .[0].id // empty' <<<"$ROLES_JSON"; }
channel_id() { jq -r --arg n "$1" 'map(select(.name == $n)) | .[0].id // empty' <<<"$CHANNELS_JSON"; }

CHANNEL_TYPE_text=0
CHANNEL_TYPE_voice=2
CHANNEL_TYPE_category=4
CHANNEL_TYPE_announcement=5
CHANNEL_TYPE_forum=15
channel_type() {
  case "$1" in
    text) echo $CHANNEL_TYPE_text ;;
    voice) echo $CHANNEL_TYPE_voice ;;
    category) echo $CHANNEL_TYPE_category ;;
    announcement) echo $CHANNEL_TYPE_announcement ;;
    forum) echo $CHANNEL_TYPE_forum ;;
    *) die "unknown channel type '$1'" ;;
  esac
}
# Announcement + forum channels only exist on a Community guild.
needs_community() { [[ "$1" == announcement || "$1" == forum ]]; }

echo "Cymbra Discord provisioning — guild $GUILD_ID"
[[ "$DRY_RUN" == 1 ]] && echo "DRY RUN: reads happen, writes are printed only"
load_roles
load_channels

# --- 1. @everyone ----------------------------------------------------------
echo "@everyone permissions"
everyone_perms=()
while IFS= read -r p; do [[ -n "$p" ]] && everyone_perms+=("$p"); done < <(
  jq -r '.everyone_permissions[]?' "$MANIFEST"
)
if ((${#everyone_perms[@]} > 0)); then
  bits="$(perms_sum "${everyone_perms[@]}")"
  api PATCH "/guilds/$GUILD_ID/roles/$EVERYONE_ID" \
    "$(jq -n --arg p "$bits" '{permissions: $p}')" >/dev/null
  log "set (${#everyone_perms[@]} permissions, no MENTION_EVERYONE)"
fi

# --- 2. Roles --------------------------------------------------------------
echo "Roles"
while IFS= read -r role; do
  name="$(jq -r '.name' <<<"$role")"
  perms=()
  while IFS= read -r p; do [[ -n "$p" ]] && perms+=("$p"); done < <(jq -r '.permissions[]?' <<<"$role")
  bits=0
  ((${#perms[@]} > 0)) && bits="$(perms_sum "${perms[@]}")"
  body="$(jq -n --arg n "$name" --arg p "$bits" \
    --argjson c "$(jq '.color // 0' <<<"$role")" \
    --argjson h "$(jq '.hoist // false' <<<"$role")" \
    --argjson m "$(jq '.mentionable // false' <<<"$role")" \
    '{name: $n, permissions: $p, color: $c, hoist: $h, mentionable: $m}')"
  id="$(role_id "$name")"
  if [[ -n "$id" ]]; then
    api PATCH "/guilds/$GUILD_ID/roles/$id" "$body" >/dev/null
    log "~ $name"
  else
    api POST "/guilds/$GUILD_ID/roles" "$body" >/dev/null
    log "+ $name"
    [[ "$DRY_RUN" == 1 ]] || load_roles
  fi
done < <(jq -c '.roles[]?' "$MANIFEST")

# --- Overwrites ------------------------------------------------------------
overwrites_for() { # overwrites_for <channel json> → JSON array
  local ch="$1" arr='[]' ro private role rid
  ro="$(jq -r '.readonly // false' <<<"$ch")"
  if [[ "$ro" == true ]]; then
    arr="$(jq -n --arg id "$EVERYONE_ID" \
      --arg deny "$(perms_sum SEND_MESSAGES SEND_MESSAGES_IN_THREADS CREATE_PUBLIC_THREADS)" \
      '[{id: $id, type: 0, deny: $deny, allow: "0"}]')"
    rid="$(role_id Mod)"
    if [[ -n "$rid" ]]; then
      arr="$(jq --arg id "$rid" --arg allow "$(perms_sum SEND_MESSAGES MANAGE_MESSAGES)" \
        '. + [{id: $id, type: 0, allow: $allow, deny: "0"}]' <<<"$arr")"
    fi
  fi
  private="$(jq -r '[.private_to[]?] | length' <<<"$ch")"
  if ((private > 0)); then
    arr="$(jq -n --arg id "$EVERYONE_ID" --arg deny "$(perms_sum VIEW_CHANNEL)" \
      '[{id: $id, type: 0, deny: $deny, allow: "0"}]')"
    while IFS= read -r role; do
      rid="$(role_id "$role")"
      [[ -n "$rid" ]] || { log "! role '$role' not found yet, skipping its overwrite"; continue; }
      arr="$(jq --arg id "$rid" \
        --arg allow "$(perms_sum VIEW_CHANNEL SEND_MESSAGES READ_MESSAGE_HISTORY)" \
        '. + [{id: $id, type: 0, allow: $allow, deny: "0"}]' <<<"$arr")"
    done < <(jq -r '.private_to[]?' <<<"$ch")
  fi
  printf '%s' "$arr"
}

ensure_channel() { # ensure_channel <channel json> <parent id|"">
  local ch="$1" parent="$2" name type tid body id ow
  name="$(jq -r '.name' <<<"$ch")"
  type="$(jq -r '.type // "text"' <<<"$ch")"
  tid="$(channel_type "$type")"
  ow="$(overwrites_for "$ch")"
  body="$(jq -n --arg n "$name" --argjson t "$tid" --argjson ow "$ow" \
    --arg topic "$(jq -r '.topic // ""' <<<"$ch")" \
    --argjson slow "$(jq '.slowmode // 0' <<<"$ch")" \
    --arg parent "$parent" \
    '{name: $n, type: $t, permission_overwrites: $ow}
     | if $topic != "" then . + {topic: $topic} else . end
     | if $slow > 0 then . + {rate_limit_per_user: $slow} else . end
     | if $parent != "" then . + {parent_id: $parent} else . end')"
  id="$(channel_id "$name")"
  if [[ -n "$id" ]]; then
    # PATCH rejects `type` on an existing channel; drop it.
    api PATCH "/channels/$id" "$(jq 'del(.type)' <<<"$body")" >/dev/null
    log "~ $name ($type)"
  else
    api POST "/guilds/$GUILD_ID/channels" "$body" >/dev/null
    log "+ $name ($type)"
    [[ "$DRY_RUN" == 1 ]] || load_channels
  fi
}

# --- 3. Categories + plain channels ---------------------------------------
# Announcement/forum channels wait for Community (step 4).
echo "Categories and channels"
while IFS= read -r cat; do
  cname="$(jq -r '.name' <<<"$cat")"
  # `.enabled // true` would be WRONG here: jq's `//` treats an explicit `false`
  # as absent, so a disabled category would be created anyway.
  if [[ "$(jq -r 'if has("enabled") then .enabled else true end' <<<"$cat")" != true ]]; then
    log "· $cname (declared, disabled — flip \"enabled\" to create it)"
    continue
  fi
  cid="$(channel_id "$cname")"
  if [[ -z "$cid" ]]; then
    api POST "/guilds/$GUILD_ID/channels" \
      "$(jq -n --arg n "$cname" --argjson t "$CHANNEL_TYPE_category" '{name: $n, type: $t}')" >/dev/null
    log "+ $cname (category)"
    [[ "$DRY_RUN" == 1 ]] || load_channels
    cid="$(channel_id "$cname")"
  else
    log "~ $cname (category)"
  fi
  while IFS= read -r ch; do
    ctype="$(jq -r '.type // "text"' <<<"$ch")"
    needs_community "$ctype" && continue
    ensure_channel "$ch" "$cid"
  done < <(jq -c '.channels[]?' <<<"$cat")
done < <(jq -c '.categories[]?' "$MANIFEST")

# --- 4. Guild settings + Community ----------------------------------------
echo "Guild settings"
rules_name="$(jq -r '[.categories[].channels[]? | select(.rules_channel == true) | .name] | .[0] // ""' "$MANIFEST")"
updates_name="$(jq -r '[.categories[].channels[]? | select(.mod_updates_channel == true) | .name] | .[0] // ""' "$MANIFEST")"
guild_body="$(jq -n \
  --arg n "$(jq -r '.guild.name // ""' "$MANIFEST")" \
  --arg d "$(jq -r '.guild.description // ""' "$MANIFEST")" \
  --arg l "$(jq -r '.guild.preferred_locale // "en-US"' "$MANIFEST")" \
  --argjson v "$(jq '.guild.verification_level // 2' "$MANIFEST")" \
  --argjson f "$(jq '.guild.explicit_content_filter // 2' "$MANIFEST")" \
  '{preferred_locale: $l, verification_level: $v, explicit_content_filter: $f}
   | if $n != "" then . + {name: $n} else . end
   | if $d != "" then . + {description: $d} else . end')"

if [[ "$(jq -r '.guild.enable_community // false' "$MANIFEST")" == true ]]; then
  rules_id="$(channel_id "$rules_name")"
  updates_id="$(channel_id "$updates_name")"
  if [[ -n "$rules_id" && -n "$updates_id" ]]; then
    features="$(api GET "/guilds/$GUILD_ID" | jq '[.features[]?] + ["COMMUNITY"] | unique')"
    guild_body="$(jq --argjson feat "${features:-[\"COMMUNITY\"]}" \
      --arg r "$rules_id" --arg u "$updates_id" \
      '. + {features: $feat, rules_channel_id: $r, public_updates_channel_id: $u}' <<<"$guild_body")"
  else
    log "! Community needs both '$rules_name' and '$updates_name' — skipping it this run"
  fi
fi
if api PATCH "/guilds/$GUILD_ID" "$guild_body" >/dev/null; then
  log "patched (Community enables the announcement + forum channel types)"
else
  log "! guild patch failed — Discord states its own requirements above; the rest continues"
fi

# --- 5. Community-only channels -------------------------------------------
echo "Announcement and forum channels"
while IFS= read -r cat; do
  [[ "$(jq -r 'if has("enabled") then .enabled else true end' <<<"$cat")" == true ]] || continue
  cid="$(channel_id "$(jq -r '.name' <<<"$cat")")"
  while IFS= read -r ch; do
    ctype="$(jq -r '.type // "text"' <<<"$ch")"
    needs_community "$ctype" || continue
    ensure_channel "$ch" "$cid"
  done < <(jq -c '.channels[]?' <<<"$cat")
done < <(jq -c '.categories[]?' "$MANIFEST")

# --- 6. AutoMod ------------------------------------------------------------
echo "AutoMod"
automod_alert="$(jq -r '.automod.alert_channel // ""' "$MANIFEST")"
alert_id="$(channel_id "$automod_alert")"
existing_rules="$(api GET "/guilds/$GUILD_ID/auto-moderation/rules" || echo '[]')"
upsert_rule() { # upsert_rule <name> <body>
  local name="$1" body="$2" id
  id="$(jq -r --arg n "$name" 'map(select(.name == $n)) | .[0].id // empty' <<<"$existing_rules")"
  if [[ -n "$id" ]]; then
    api PATCH "/guilds/$GUILD_ID/auto-moderation/rules/$id" "$body" >/dev/null && log "~ $name"
  else
    api POST "/guilds/$GUILD_ID/auto-moderation/rules" "$body" >/dev/null && log "+ $name"
  fi
}
alert_action() { # a SEND_ALERT_MESSAGE action, or nothing when no alert channel exists
  [[ -n "$alert_id" ]] || { printf '[]'; return; }
  jq -n --arg c "$alert_id" '[{type: 2, metadata: {channel_id: $c}}]'
}
block='[{"type":1,"metadata":{"custom_message":"Blocked by Cymbra AutoMod."}}]'
actions="$(jq -n --argjson b "$block" --argjson a "$(alert_action)" '$b + $a')"

upsert_rule "Cymbra — spam" \
  "$(jq -n --argjson a "$actions" '{name: "Cymbra — spam", event_type: 1, trigger_type: 3, actions: $a, enabled: true}')"
upsert_rule "Cymbra — mention spam" \
  "$(jq -n --argjson a "$actions" --argjson n "$(jq '.automod.mention_total_limit // 6' "$MANIFEST")" \
    '{name: "Cymbra — mention spam", event_type: 1, trigger_type: 5, trigger_metadata: {mention_total_limit: $n}, actions: $a, enabled: true}')"
presets="$(jq '[.automod.keyword_presets[]? | if . == "PROFANITY" then 1 elif . == "SEXUAL_CONTENT" then 2 elif . == "SLURS" then 3 else empty end]' "$MANIFEST")"
if [[ "$(jq 'length' <<<"$presets")" != 0 ]]; then
  upsert_rule "Cymbra — offensive language" \
    "$(jq -n --argjson a "$actions" --argjson p "$presets" \
      '{name: "Cymbra — offensive language", event_type: 1, trigger_type: 4, trigger_metadata: {presets: $p, allow_list: []}, actions: $a, enabled: true}')"
fi
keywords="$(jq '[.automod.blocked_keywords[]?]' "$MANIFEST")"
if [[ "$(jq 'length' <<<"$keywords")" != 0 ]]; then
  upsert_rule "Cymbra — scam links" \
    "$(jq -n --argjson a "$actions" --argjson k "$keywords" \
      '{name: "Cymbra — scam links", event_type: 1, trigger_type: 1, trigger_metadata: {keyword_filter: $k}, actions: $a, enabled: true}')"
fi

# --- 7. Webhooks -----------------------------------------------------------
# Each URL is a SECRET: whoever holds one can post as Cymbra in that channel.
# They go to a 0600 file, never to stdout.
echo "Webhooks"
if [[ "$DRY_RUN" == 1 ]]; then
  log "skipped in dry run (creating one would mint a real secret)"
else
  umask 077
  : >"$WEBHOOKS_OUT"
  {
    echo "# Cymbra Discord webhook URLs — SECRETS. Do not commit, do not paste in chat."
    echo "# Generated by scripts/discord/provision.sh"
  } >>"$WEBHOOKS_OUT"
  while IFS= read -r wh; do
    ch="$(jq -r '.channel' <<<"$wh")"
    wname="$(jq -r '.name' <<<"$wh")"
    cid="$(channel_id "$ch")"
    [[ -n "$cid" ]] || { log "! channel '$ch' not found, skipping its webhook"; continue; }
    existing="$(api GET "/channels/$cid/webhooks" || echo '[]')"
    hook="$(jq -c --arg n "$wname" 'map(select(.name == $n and .token != null)) | .[0] // empty' <<<"$existing")"
    if [[ -z "$hook" ]]; then
      hook="$(api POST "/channels/$cid/webhooks" "$(jq -n --arg n "$wname" '{name: $n}')")"
      log "+ $ch"
    else
      log "~ $ch (reused)"
    fi
    hid="$(jq -r '.id' <<<"$hook")"
    htoken="$(jq -r '.token' <<<"$hook")"
    var="DISCORD_WEBHOOK_$(printf '%s' "$ch" | tr '[:lower:]-' '[:upper:]_')"
    printf '%s=https://discord.com/api/webhooks/%s/%s\n' "$var" "$hid" "$htoken" >>"$WEBHOOKS_OUT"
  done < <(jq -c '.webhooks[]?' "$MANIFEST")
  log "written to $WEBHOOKS_OUT (mode 0600) — move these into the backend environment"
fi

echo
echo "Done. Left to do in the Discord UI (deliberately not scripted):"
echo "  - Onboarding (Server Settings → Onboarding): the wizard is 3 minutes and its"
echo "    API needs pre-existing prompt ids; attach the Beta/Curator roles there."
echo "  - Server icon and banner (upload your artwork)."
echo "  - Post and pin the rules in #welcome."
