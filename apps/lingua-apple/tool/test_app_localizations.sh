#!/bin/bash
# Tests for app_localizations.sh, against the committed app Info.plists (run: bash tool/test_app_localizations.sh).
# Needs plutil, so a Mac — lingua-apple-build runs it before the Xcode builds.
#
# What the phase sees is Xcode's product, not the committed file: with GENERATE_INFOPLIST_FILE,
# Xcode writes $(DEVELOPMENT_LANGUAGE) — the project's developmentRegion, `en` — over the source
# plist's CFBundleDevelopmentRegion, and keeps the source's CFBundleLocalizations ([fr] on the apps,
# none on the extensions, whose generated plist holds no such key). A French-only build leaves that
# product as it is; a localised one declares the natives; a French-only build after it restores it.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
script="$here/app_localizations.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The project's development language, what Xcode writes as CFBundleDevelopmentRegion.
development_language=$(sed -n 's/^[[:space:]]*developmentRegion = \([A-Za-z-]*\);.*/\1/p' "$here/../Cymbra Lingua.xcodeproj/project.pbxproj")

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

app_source_plist() { # app_source_plist FILE — what the committed app plists declare
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

extension_source_plist() { # extension_source_plist FILE — what the committed extension plists declare: no language at all
  cat >"$1" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSExtension</key>
	<dict>
		<key>NSExtensionPointIdentifier</key>
		<string>com.apple.Safari.web-extension</string>
	</dict>
</dict>
</plist>
EOF
}

processed() { # processed SOURCE PROCESSED — Xcode's product: the source, CFBundleDevelopmentRegion written from the development language
  cp "$1" "$2"
  plutil -replace CFBundleDevelopmentRegion -string "$development_language" "$2"
}

french=$(packs french en-fr es-fr)
english=$(packs english en-fr es-fr es-en)
spanish=$(packs spanish en-fr en-es)
three=$(packs three en-fr es-en en-es)

if [ "$development_language" = "en" ]; then pass "the project's development language is en, what the generated plists are documented to hold"; else fail "the project's development language is '$development_language'; the generated plists are documented to hold en"; fi

# 1. French natives leave Xcode's product untouched: an app plist, XML or binary, and an extension's generated one.
app_source_plist "$work/app.src.plist"
processed "$work/app.src.plist" "$work/app.plist"
cp "$work/app.plist" "$work/app.bin.plist"
plutil -convert binary1 "$work/app.bin.plist"
for plist in "$work/app.plist" "$work/app.bin.plist"; do
  cp "$plist" "$plist.before"
  bash "$script" "$french" "$plist" "$work/app.src.plist" "$development_language" >/dev/null
  if cmp -s "$plist" "$plist.before"; then pass "French natives leave $(basename "$plist") untouched"; else fail "French natives rewrote $(basename "$plist")"; fi
done
extension_source_plist "$work/ext.src.plist"
processed "$work/ext.src.plist" "$work/ext.plist"
cp "$work/ext.plist" "$work/ext.plist.before"
bash "$script" "$french" "$work/ext.plist" "$work/ext.src.plist" "$development_language" >/dev/null
if cmp -s "$work/ext.plist" "$work/ext.plist.before"; then pass "French natives leave an extension's generated plist untouched"; else fail "French natives rewrote an extension's generated plist"; fi

# 2. … and what Xcode makes of the committed plists, as they are.
for committed in "$here/../iOS (App)/Info.plist" "$here/../macOS (App)/Info.plist" "$here/../iOS (Extension)/Info.plist" "$here/../macOS (Extension)/Info.plist"; do
  target=$(basename "$(dirname "$committed")")
  processed "$committed" "$work/committed.plist"
  cp "$work/committed.plist" "$work/committed.plist.before"
  bash "$script" "$french" "$work/committed.plist" "$committed" "$development_language" >/dev/null
  if cmp -s "$work/committed.plist" "$work/committed.plist.before"; then pass "French natives leave the committed $target plist, as Xcode processes it, untouched"; else fail "French natives rewrote the committed $target plist, as Xcode processes it"; fi
done

# 3. fr and en natives yield [fr, en] / en, on an app plist (binary stays binary) and an extension's.
for plist in "$work/app.plist" "$work/app.bin.plist"; do
  bash "$script" "$english" "$plist" "$work/app.src.plist" "$development_language" >/dev/null
  if [ "$(localizations "$plist")" = '["fr","en"]' ] && [ "$(region "$plist")" = "en" ]; then pass "fr + en natives declare [fr, en] / en in $(basename "$plist")"; else fail "fr + en natives declared $(localizations "$plist") / $(region "$plist") in $(basename "$plist")"; fi
done
if file "$work/app.bin.plist" | grep -q "binary property list"; then pass "a binary plist stays binary"; else fail "the binary plist was converted"; fi
bash "$script" "$english" "$work/ext.plist" "$work/ext.src.plist" "$development_language" >/dev/null
if [ "$(localizations "$work/ext.plist")" = '["fr","en"]' ] && [ "$(region "$work/ext.plist")" = "en" ]; then pass "fr + en natives declare [fr, en] / en in an extension's plist"; else fail "fr + en natives declared $(localizations "$work/ext.plist") / $(region "$work/ext.plist") in an extension's plist"; fi

# 4. A localised plist is left untouched when the list is unchanged.
cp "$work/app.plist" "$work/app.plist.before"
bash "$script" "$english" "$work/app.plist" "$work/app.src.plist" "$development_language" >/dev/null
if cmp -s "$work/app.plist" "$work/app.plist.before"; then pass "an unchanged list leaves a localised plist untouched"; else fail "an unchanged list rewrote a localised plist"; fi

# 5. A French-only build after a localised one restores Xcode's product: [fr] / the development language on an app plist, nothing / it on an extension's.
bash "$script" "$french" "$work/app.plist" "$work/app.src.plist" "$development_language" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr"]' ] && [ "$(region "$work/app.plist")" = "$development_language" ]; then pass "French natives restore an app plist to [fr] / $development_language"; else fail "French natives left $(localizations "$work/app.plist") / $(region "$work/app.plist") in an app plist"; fi
bash "$script" "$french" "$work/ext.plist" "$work/ext.src.plist" "$development_language" >/dev/null
if [ "$(localizations "$work/ext.plist")" = "absent" ] && [ "$(region "$work/ext.plist")" = "$development_language" ]; then pass "French natives restore an extension's plist to no CFBundleLocalizations / $development_language"; else fail "French natives left $(localizations "$work/ext.plist") / $(region "$work/ext.plist") in an extension's plist"; fi
if plutil -lint "$work/ext.plist" >/dev/null; then pass "the restored extension plist is well-formed"; else fail "the restored extension plist is not well-formed"; fi

# 6. fr and es natives, without en, keep fr as the development region; all three list fr, en, es.
bash "$script" "$spanish" "$work/app.plist" "$work/app.src.plist" "$development_language" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr","es"]' ] && [ "$(region "$work/app.plist")" = "fr" ]; then pass "fr + es natives declare [fr, es] / fr"; else fail "fr + es natives declared $(localizations "$work/app.plist") / $(region "$work/app.plist")"; fi
bash "$script" "$three" "$work/app.plist" "$work/app.src.plist" "$development_language" >/dev/null
if [ "$(localizations "$work/app.plist")" = '["fr","en","es"]' ] && [ "$(region "$work/app.plist")" = "en" ]; then pass "fr + en + es natives declare [fr, en, es] / en"; else fail "fr + en + es natives declared $(localizations "$work/app.plist") / $(region "$work/app.plist")"; fi

# 7. No pack is a build that forgot the extension; a missing plist is a wrong phase; the phase's arguments are all four.
mkdir -p "$work/empty"
if bash "$script" "$work/empty" "$work/app.plist" "$work/app.src.plist" "$development_language" 2>/dev/null; then fail "no pack passed"; else pass "no pack fails the build"; fi
if bash "$script" "$french" "$work/missing.plist" "$work/app.src.plist" "$development_language" 2>/dev/null; then fail "a missing plist passed"; else pass "a missing plist fails the build"; fi
if bash "$script" "$french" "$work/app.plist" "$work/missing.src.plist" "$development_language" 2>/dev/null; then fail "a missing source plist passed"; else pass "a missing source plist fails the build"; fi
if bash "$script" "$french" "$work/app.plist" "$work/app.src.plist" "" 2>/dev/null; then fail "an empty development language passed"; else pass "an empty development language fails the build"; fi
if bash "$script" "$french" "$work/app.plist" 2>/dev/null; then fail "two arguments passed"; else pass "two arguments fail"; fi

if [ "$failures" -ne 0 ]; then
  echo "$failures failure(s)" >&2
  exit 1
fi
echo "all passed"
