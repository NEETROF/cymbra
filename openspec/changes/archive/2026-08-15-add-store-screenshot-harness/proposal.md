## Why

The checked-in store screenshots no longer show the app we ship, and two of the three
sets are no longer even the right size. The iOS and Android captures date from
2026-07-16 (#93); 49 commits have touched `apps/music/lib/screens|widgets` since,
including the #202 game-mode readability rework that replaced the HUD they show.
Features shipped since then — the 42-lesson solfège curriculum, the seasonal
leaderboard, badges and the rewards shop, measure-range selection, the note reading
aid — appear in no capture at all, so the listing advertises a materially poorer app
than the one in TestFlight.

Independently, the required sizes moved. App Store Connect no longer lists a 6.7"
iPhone class (it wants 6.9" — 2868×1320 landscape — or 6.5" as fallback), and the
mandatory iPad class is now 13" (2752×2064), with 12.9" merely scaled from it. Our
iPhone set is 2796×1290 and our iPad set is 2732×2048. The macOS set is recent but
was captured with no MIDI keyboard connected, so the store page reads "No MIDI
device" and "0%" on an app whose pitch is playing along on a real keyboard.

Every set is English-only while `store/copy/` already ships en/fr/it/es, and each set
was produced by hand — the macOS one requires a temporary, never-committed edit to
`macos/Runner/MainFlutterWindow.swift` to pin the window. That is why the assets
rotted within a month and why nobody noticed: re-capturing costs a manual afternoon,
so it does not happen.

## What Changes

- Add a **scripted capture harness**: a dedicated `integration_test` scenario that
  drives the app through the listing screens and writes PNGs via
  `integrationDriver(onScreenshot:)` on the existing `test_driver` seam, plus a
  `melos run screenshots` entry point parameterized by locale and target.
- Drive the **app locale from the harness** rather than the device, so one run per
  locale produces a full set without touching the account language sync.
- Seed a **deterministic capture state** — fixture score, a simulated MIDI device so
  no "No MIDI device" chip, a plausible non-zero accuracy, stable progress and
  rewards data — so captures are reproducible and never advertise an empty account.
- Retarget the **size classes** to what the stores require today: iPhone 6.9"
  (2868×1320 landscape), iPad 13" (2752×2064 landscape), macOS 1440×900 (unchanged),
  Android phone at 16:9 ≥ 1920×1080 rather than the current 2:1 2160×1080.
- Regenerate **all four locales** (en/fr/it/es) for every platform, replacing the
  English-only sets, and cover the features that currently have no shot.
- Remove the manual macOS window-pinning step by sizing the window from the harness.
- Update `apps/music/store/README.md`: the layout gains a locale level, the "known
  caveat" section goes away, and the reproduction instructions become the command.

Not in scope: the Play feature graphic and hi-res icon (unchanged, not screen
captures), the listing copy (already complete in four locales), and any change to how
the app itself localizes — the harness consumes `app-localization`, it does not
extend it.

## Capabilities

### New Capabilities

- `music-store-listing-capture`: store listing screenshots are produced by a
  reproducible, locale-parameterized harness from a seeded app state, at the size
  classes each store currently requires, for every shipping locale — rather than by
  hand.

### Modified Capabilities

- `store-distribution`: its "Store-listing assets" requirement names the 6.7" iPhone
  and 12.9" iPad classes, which App Store Connect no longer requires, and says
  nothing about locale coverage. The scenario must move to the current classes and
  defer screenshot provenance to `music-store-listing-capture`. **Sequencing**: that
  capability lives in the still-in-flight `prepare-store-distribution` change, so it
  is not in `openspec/specs/` yet — this delta only applies once that change is
  archived. If it is still open at implementation time, edit its spec in place
  instead of shipping a MODIFIED delta here.

## Impact

**Product affected: Cymbra Music only.** Consumes the existing `app-localization`
and `state-management` capabilities; introduces nothing for Cymbra ID, Live, or the
back office.

- **New**: a capture scenario under `apps/music/integration_test/`, its seeded
  fixtures, and a `melos.yaml` script.
- **Modified**: `apps/music/test_driver/integration_test.dart` gains the
  `onScreenshot` callback (today a bare `integrationDriver()`); `apps/music/store/`
  is restructured by locale and fully regenerated; `apps/music/store/README.md`.
- **Deleted**: the current `store/ios/iphone_6.7/`, `store/ios/ipad_12.9/`,
  `store/android/phone/` and `store/macos/` PNGs, superseded by the generated sets.
- **Risk**: the harness runs on simulators/emulators, whose device frames and status
  bars differ from the real hardware Apple's classes describe — output dimensions
  must be asserted, not assumed. Android capture also needs a clean status bar
  (demo mode), which the July pass already had to arrange by hand.
- **CI**: capture stays a local, on-demand command. Running it in CI would need
  booted simulators for three platforms and is deliberately out of scope; the gate
  only checks that the committed sets match the declared dimensions.
