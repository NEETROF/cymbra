#!/bin/bash
# Tests for app_localizations.sh, against the committed app Info.plists (run: bash tool/test_app_localizations.sh).
# Needs plutil, so a Mac — lingua-apple-build runs it before the Xcode builds.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
script="$here/app_localizations.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

failures=0
fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}
pass() {
  echo "ok: $1"
}

packs() { # packs NAME PAIR... — a packs directory holding one empty pack per pair
  local dir="$work/packs-$1"
  shift
  rm -rf "$dir"
  mkdir -p "$dir"
  for pair in "$@"; do : >"$dir/$pair.lingua"; done
  echo "$dir"
}

localizations() { plutil -extract CFBundleLocalizations json -o - "$1" 2>/dev/null || echo "absent"; }
region() { plutil -extract CFBundleDevelopmentRegion raw -o - "$1"; }

app_plist() { # app_plist FILE — what the committed app plists declare
  cat >"$1" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>fr</string>
	<key>CFBundleLocalizations</key>
	<array>
		<string>fr</string>
	</array>
	<key>CFBundleIdentifier</key>
	<string>com.cymbra.lingua</string>
</dict>
</plist>
EOF
}

generated_plist() { # generated_plist FILE — what Xcode generates for the extension targets
  cat >"$1" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleIdentifier</key>
	<string>com.cymbra.lingua.Extension</string>
</dict>
</plist>
EOF
}

french=$(packs french en-fr es-fr)
english=$(packs english en-fr es-fr es-en)
spanish=$(packs spanish en-fr en-es)
three=$(packs three en-fr es-en en-es)

# 1. French natives leave an app plist untouched, XML or binary.
app_plist "$work/app.plist"
cp "$work/app.plist" "$work/app.bin.plist"
plutil -convert binary1 "$work/app.bin.plist"
for plist in "$work/app.plist" "$work/app.bin.plist"; do
  cp "$plist" "$plist.before"
  bash "$script" "$french" "$plist" >/dev/null
  if cmp -s "$plist" "$plist.before"; then pass "French natives leave $(basename "$plist") untouched"; else fail "French natives rewrote $(basename "$plist")"; fi
done

# 2. … and the committed app plists, as they are.
for committed in "$here/../iOS (App)/Info.plist" "$here/../macOS (App)/Info.plist"; do
  cp "$committed" "$work/committed.plist"
  bash "$script" "$french" "$work/committed.plist" >/dev/null
  if cmp -s "$committed" "$work/committed.plist"; then pass "French natives leave the committed $(basename "$(dirname "$committed")") plist untouched"; else fail "French natives rewrote the committed $(basename "$(dirname "$committed")") plist"; fi
done

# 3. French natives leave a generated plist untouched.
generated_plist "$work/gen.plist"
cp "$work/gen.plist" "$work/gen.plist.before"
bash "$script" "$french" "$work/gen.plist" generated >/dev/null
if cmp -s "$work/gen.plist" "$work/gen.plist.before"; then pass "French natives leave a generated plist untouched"; else fail "French natives rewrote a generated plist"; fi

# 4. fr and en natives yield [fr, en] / en, on an app plist (binary stays binary) and a generated one.
for plist in "$work/app.plist" "$work/app.bin.plist"; do
  bash "$script" "$english" "$plist" >/dev/null
  if [ "$(localizations "$plist")" = '["fr","en"]' ] && [ "$(region "$plist")" = "en" ]; then pass "fr + en natives declare [fr, en] / en in $(basename "$plist")"; else fail "fr + en natives declared $(localizations "$plist") / $(region "$plist") in $(basename "$plist")"; fi
done
if file "$work/app.bin.plist" | grep -q "binary property list"; then pass "a binary plist stays binary"; else fail "the binary plist was converted"; fi
bash "$script" "$english" "$work/gen.plist" generated >/dev/null
if [ "$(localizations "$work/gen.plist")" = '["fr","en"]' ] && [ "$(region "$work/gen.plist")" = "en" ]; then pass "fr + en natives declare [fr, en] / en in a generated plist"; else fail "fr + en natives declared $(localizations "$work/gen.plist") / $(region "$work/gen.plist") in a generated plist"; fi

# 5. A localised plist is left untouched when the list is unchanged.
cp "$work/app.plist" "$work/app.plist.before"
bash "$script" "$english" "$work/app.plist" >/dev/null
if cmp -s "$work/app.plist" "$work/app.plist.before"; then pass "an unchanged list leaves a localised plist untouched"; else fail "an unchanged list rewrote a localised plist"; fi

# 6. A French-only build after a localised one is reset: [fr] / fr on an app plist, nothing / en on a generated one.
bash "$script" "$french" "$work/app.plist" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr"]' ] && [ "$(region "$work/app.plist")" = "fr" ]; then pass "French natives reset an app plist to [fr] / fr"; else fail "French natives left $(localizations "$work/app.plist") / $(region "$work/app.plist") in an app plist"; fi
bash "$script" "$french" "$work/gen.plist" generated >/dev/null
if [ "$(localizations "$work/gen.plist")" = "absent" ] && [ "$(region "$work/gen.plist")" = "en" ]; then pass "French natives reset a generated plist to no CFBundleLocalizations / en"; else fail "French natives left $(localizations "$work/gen.plist") / $(region "$work/gen.plist") in a generated plist"; fi
if plutil -lint "$work/gen.plist" >/dev/null; then pass "the reset generated plist is well-formed"; else fail "the reset generated plist is not well-formed"; fi

# 7. fr and es natives, without en, keep fr as the development region; all three list fr, en, es.
bash "$script" "$spanish" "$work/app.plist" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr","es"]' ] && [ "$(region "$work/app.plist")" = "fr" ]; then pass "fr + es natives declare [fr, es] / fr"; else fail "fr + es natives declared $(localizations "$work/app.plist") / $(region "$work/app.plist")"; fi
bash "$script" "$three" "$work/app.plist" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr","en","es"]' ] && [ "$(region "$work/app.plist")" = "en" ]; then pass "fr + en + es natives declare [fr, en, es] / en"; else fail "fr + en + es natives declared $(localizations "$work/app.plist") / $(region "$work/app.plist")"; fi

# 8. No pack is a build that forgot the extension; a missing plist is a wrong phase.
mkdir -p "$work/empty"
if bash "$script" "$work/empty" "$work/app.plist" 2>/dev/null; then fail "no pack passed"; else pass "no pack fails the build"; fi
if bash "$script" "$french" "$work/missing.plist" 2>/dev/null; then fail "a missing plist passed"; else pass "a missing plist fails the build"; fi
if bash "$script" "$french" "$work/app.plist" extension 2>/dev/null; then fail "an unknown third argument passed"; else pass "an unknown third argument fails"; fi

if [ "$failures" -ne 0 ]; then
  echo "$failures failure(s)" >&2
  exit 1
fi
echo "all passed"
