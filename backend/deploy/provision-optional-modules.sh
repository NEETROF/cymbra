#!/usr/bin/env bash
# Provision an OPTIONAL module's Postgres role on a LIVE box and wire its .env.
#
#   provision-optional-modules.sh [--rotate] [module…]      default: flags plans
#
# A module is one role (`<module>_svc`), one schema and one password, described
# by the neighbouring `provision-<module>-role.sql` (flags, plans, music,
# analytics). Those files are already idempotent and targeted — they never touch
# another role's password. What this adds is the part that actually breaks: the
# password lives in TWO places, the role AND the URL in .env, and a divergence
# between them is not caught here, it is caught at the next boot.
#
# So the password is generated on this box, handed to psql over stdin (never in
# argv, never in the shell history), written to both .env lines, verified by
# connecting as the new role — and never printed.
#
# Re-runnable: a module already wired in .env is SKIPPED, not re-provisioned.
# `--rotate` is the deliberate opposite — it mints a new password for a module
# that already has one (the role and the URL move together, so the stack keeps
# working from the next roll).
#
# Then: ./deploy.sh <version> — the server's MIGRATOR creates the module's
# tables at boot. Order matters: `flags` before `plans`, because the plan module
# is driven by runtime flags and stays dark without a flag store.
set -euo pipefail

DIR="${CYMBRA_DEPLOY_DIR:-/opt/cymbra/backend/deploy}"
ENV_FILE="$DIR/.env"
PG_SUPERUSER="${PG_SUPERUSER:-cymbra}"
PG_DB="${PG_DB:-cymbra}"
# The host the SERVER will use, i.e. the one written into the URL. Verifying over
# it is not a detail: the image's own pg_hba trusts 127.0.0.1 unconditionally, so
# a loopback check would pass with any password at all.
PG_NET_HOST="${PG_NET_HOST:-postgres}"

usage() {
  echo "usage: $(basename "$0") [--rotate] [flags|plans|music|analytics …]" >&2
  exit 2
}

ROTATE=0
REQUESTED=()
for arg in ${@+"$@"}; do
  case "$arg" in
    --rotate) ROTATE=1 ;;
    -h|--help) usage ;;
    *)
      [[ "$arg" =~ ^[a-z][a-z0-9]*$ ]] || usage
      [[ -f "$DIR/provision-$arg-role.sql" ]] || {
        echo "[provision] refused: no provision-$arg-role.sql in $DIR" >&2; exit 2; }
      REQUESTED+=("$arg")
      ;;
  esac
done
[[ ${#REQUESTED[@]} -gt 0 ]] || REQUESTED=(flags plans)

# Canonical order, whatever order they were asked in: flags first (the plan
# module reads its own kill-switch from the flag store).
MODULES=()
for known in flags plans music analytics; do
  for want in "${REQUESTED[@]}"; do
    if [[ "$want" == "$known" ]]; then MODULES+=("$known"); break; fi
  done
done
for want in "${REQUESTED[@]}"; do
  case "$want" in flags|plans|music|analytics) ;; *) MODULES+=("$want") ;; esac
done

[[ -f "$ENV_FILE" ]] || { echo "[provision] refused: no .env in $DIR" >&2; exit 1; }

CID="${PG_CONTAINER:-$(docker compose -f "$DIR/docker-compose.prod.yml" ps -q postgres 2>/dev/null || true)}"
[[ -n "$CID" ]] || { echo "[provision] refused: the postgres container is not running" >&2; exit 1; }

# A check that cannot fail is not a check. The image's own pg_hba trusts
# 127.0.0.1 unconditionally, so before touching anything, prove that THIS path
# actually demands a password — otherwise every "verified" below is a fiction.
if PGPASSWORD="no-such-password-$$" docker exec -e PGPASSWORD -i "$CID" \
     psql -tAX -h "$PG_NET_HOST" -U "$PG_SUPERUSER" -d "$PG_DB" -c 'select 1' \
     </dev/null >/dev/null 2>&1; then
  echo "[provision] refused: this database accepts ANY password over '$PG_NET_HOST'," >&2
  echo "[provision]   so no role could be verified. Check pg_hba.conf — and note that" >&2
  echo "[provision]   a live box answering this way is itself the thing to fix." >&2
  exit 1
fi

# Alphanumeric on purpose: the value is embedded in a postgres:// URL and in a
# dotenv line, where a '/', '@', '#' or a quote would silently truncate it.
gen_pw() {
  local pool
  pool="$(head -c 512 /dev/urandom | base64 | LC_ALL=C tr -dc 'A-Za-z0-9')"
  [[ ${#pool} -ge 40 ]] || { echo "[provision] refused: no entropy" >&2; exit 1; }
  printf '%s' "${pool:0:40}"
}

env_get() { sed -n "s/^$1=//p" "$ENV_FILE" | tail -1; }

env_set() {
  local key="$1" value="$2" tmp
  tmp="$(mktemp "$ENV_FILE.XXXXXX")"
  cp -p "$ENV_FILE" "$tmp"        # keep the .env's own mode/owner…
  KEY="$key" VAL="$value" awk '
    BEGIN { k = ENVIRON["KEY"]; v = ENVIRON["VAL"]; done = 0 }
    index($0, k "=") == 1 { if (!done) { print k "=" v; done = 1 } ; next }
    { print }
    END { if (!done) print k "=" v }
  ' "$ENV_FILE" > "$tmp"          # …the redirect truncates, the mode survives.
  mv "$tmp" "$ENV_FILE"
}

for module in "${MODULES[@]}"; do
  upper="$(printf '%s' "$module" | tr '[:lower:]' '[:upper:]')"
  pw_var="CYMBRA_${upper}_DB_PASSWORD"
  url_var="CYMBRA_${upper}_DATABASE_URL"
  current="$(env_get "$pw_var")"

  if [[ -n "$current" && "$current" != CHANGE_ME* && "$current" != *_dev_pw && $ROTATE -eq 0 ]]; then
    echo "[provision] $module: already wired in .env — skipped (--rotate to mint a new password)"
    continue
  fi

  if [[ "$module" == plans ]]; then
    flags_pw="$(env_get CYMBRA_FLAGS_DB_PASSWORD)"
    if [[ -z "$flags_pw" || "$flags_pw" == CHANGE_ME* || "$flags_pw" == *_dev_pw ]]; then
      echo "[provision] $module: WARNING — no flag store wired yet, so plans.enabled" >&2
      echo "[provision]   can never be turned on and the module stays dark. Run flags too." >&2
    fi
  fi

  pw="$(gen_pw)"

  echo "[provision] $module: applying provision-$module-role.sql…"
  { printf "\\set %s_pw '%s'\n" "$module" "$pw"; cat "$DIR/provision-$module-role.sql"; } \
    | docker exec -i "$CID" psql -q -U "$PG_SUPERUSER" -d "$PG_DB" -f - >/dev/null

  echo "[provision] $module: verifying ${module}_svc can log in…"
  identity="$(PGPASSWORD="$pw" docker exec -e PGPASSWORD -i "$CID" \
    psql -tAX -h "$PG_NET_HOST" -U "${module}_svc" -d "$PG_DB" \
      -c "select current_user || ' on ' || current_schema" </dev/null 2>/dev/null)" || {
    echo "[provision] $module: FAILED — ${module}_svc cannot log in with the password" >&2
    echo "[provision]   just set. The role now holds a password that is NOT in .env:" >&2
    echo "[provision]   re-run this script to set both again." >&2
    exit 1
  }

  env_set "$pw_var" "$pw"
  env_set "$url_var" "postgres://${module}_svc:${pw}@postgres:5432/${PG_DB}"
  echo "[provision] $module: OK — $identity, $pw_var + $url_var written to .env"
done

echo "[provision] done. Roll the stack to pick it up:  ./deploy.sh <version>"
