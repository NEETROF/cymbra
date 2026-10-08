#!/bin/bash
# The host app's languages are the extension's shipped natives (localise-lingua-apple-host, D1).
#
# The Safari build carries one pack per shipped pair, assets/packs/<studied>-<native>.lingua, and
# the extension's interface speaks each pair's native language. This writes those natives to the
# processed Info.plist of a target — CFBundleLocalizations, which the App Store's "Languages" line
# reads and Foundation's preferredLocalizations picks from, and CFBundleDevelopmentRegion, `en`
# once English ships and `fr` otherwise — only when the plist holds something else. A French-only
# build therefore never touches the plist (an iOS plist is binary; a rewrite would change its
# bytes), and a French-only build after a localised one is reset.
#
# Run as a Run Script phase, last on each app target and after the copy phase on each extension
# target, with the processed plist as its input (never as an output, which would collide with
# ProcessInfoPlistFile) and ENABLE_USER_SCRIPT_SANDBOXING off (the packs are outside the project).
#
#   app_localizations.sh PACKS_DIR PLIST [generated]
#
# `generated` says the plist is one Xcode generates (GENERATE_INFOPLIST_FILE, the extension
# targets): it holds no CFBundleLocalizations and the project's development region, `en`, and that
# is what a French-only list means there. Without it, French-only means [fr]/fr, what the committed
# app Info.plists hold. Tested by test_app_localizations.sh (lingua-apple-build).
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "usage: $0 PACKS_DIR PLIST [generated]" >&2
  exit 2
fi
packs_dir=$1
plist=$2
generated=${3:-}
if [ -n "$generated" ] && [ "$generated" != "generated" ]; then
  echo "error: the third argument is \`generated\` or nothing, not \`$generated\`" >&2
  exit 2
fi
if [ ! -f "$plist" ]; then
  echo "error: $plist is not a processed Info.plist" >&2
  exit 1
fi

# The natives of the shipped packs, each once, in the order the interface lists its languages
# (src/i18n/language.ts: fr, en, es); a native the app has no copy for is declared after them,
# so the plist says what ships, and Swift shows the development region for it.
found=""
for pack in "$packs_dir"/*.lingua; do
  [ -f "$pack" ] || continue
  pair=$(basename "$pack" .lingua)
  native=${pair##*-}
  if [ "$native" = "$pair" ] || [ -z "$native" ]; then
    echo "error: $pack is not a <studied>-<native>.lingua pack" >&2
    exit 1
  fi
  found="$found $native"
done
if [ -z "$found" ]; then
  echo "error: no pack in $packs_dir. Build the extension first: cd apps/lingua-extension && yarn build:safari" >&2
  exit 1
fi
natives=""
for native in fr en es $(printf '%s\n' $found | sort -u); do
  case " $found " in *" $native "*) ;; *) continue ;; esac
  case " $natives " in *" $native "*) continue ;; esac
  case "$native" in fr | en | es) ;; *) echo "warning: $native-native packs ship, but the app has no copy in $native (copy.js, SignInCopy)" >&2 ;; esac
  natives="$natives $native"
done
natives=${natives# }

# What the plist should hold.
if [ "$natives" = "fr" ] && [ -n "$generated" ]; then
  wanted_json=""
  wanted_region="en"
else
  wanted_json="["
  for native in $natives; do
    wanted_json="$wanted_json\"$native\","
  done
  wanted_json="${wanted_json%,}]"
  case " $natives " in *" en "*) wanted_region="en" ;; *) wanted_region="fr" ;; esac
fi

# What it holds: `plutil -extract … json` prints an array compact, as wanted_json is written.
current_json=$(plutil -extract CFBundleLocalizations json -o - "$plist" 2>/dev/null || true)
current_region=$(plutil -extract CFBundleDevelopmentRegion raw -o - "$plist" 2>/dev/null || true)

if [ "$current_json" = "$wanted_json" ] && [ "$current_region" = "$wanted_region" ]; then
  echo "note: $(basename "$plist") already declares the shipped natives ($natives); left untouched"
  exit 0
fi

if [ -z "$wanted_json" ]; then
  if [ -n "$current_json" ]; then
    plutil -remove CFBundleLocalizations "$plist"
  fi
else
  plutil -replace CFBundleLocalizations -json "$wanted_json" "$plist"
fi
plutil -replace CFBundleDevelopmentRegion -string "$wanted_region" "$plist"
echo "note: $(basename "$plist") now declares ${wanted_json:-no CFBundleLocalizations} with CFBundleDevelopmentRegion $wanted_region (shipped natives: $natives)"
