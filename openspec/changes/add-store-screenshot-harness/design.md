## Context

`apps/music/store/` holds the listing images for three stores. They were produced by
hand: the iOS and Android sets on simulators in July, the macOS set in August with a
temporary edit to `macos/Runner/MainFlutterWindow.swift` to pin the window to
1440×900 — an edit the README explicitly says must never be committed. Both passes
ran against a **local backend**, so their content depended on whatever that database
held at the time. Nothing about either pass is reproducible.

The consequences are already visible: the sets show a HUD replaced in #202, omit
every feature shipped since July, display "No MIDI device" and "0%" on the macOS
player shots, cover only English while `store/copy/` ships four locales, and sit at
size classes App Store Connect no longer requires (6.7" iPhone, 12.9" iPad).

Two seams already exist and shape this design. `apps/music/test_driver/integration_test.dart`
is a bare `integrationDriver()` used by `melos run integration-watch`, which drives
the app through `flutter drive` on a visible simulator. And
`apps/music/integration_test/app_test.dart` already boots the real app inside a
`ProviderScope(overrides: [...])` with a fixture score, in-memory preferences and a
fake connectivity service — the proven way to put this app in a chosen state.

## Goals / Non-Goals

**Goals:**

- One command per (platform, locale) regenerates a complete, correctly-sized set.
- Capture content is seeded and deterministic — no live backend, no real keyboard.
- All four shipping locales covered, driven from the harness.
- Committed images verified against declared dimensions, not trusted.
- The macOS window-pinning source edit disappears.

**Non-Goals:**

- Running capture in CI. Three booted simulators and a working native build make it a
  poor gate; CI only checks committed dimensions.
- Uploading to App Store Connect / Play. Out of scope; the deliverable is the files.
- Feature graphic, hi-res icon, listing copy — not screen captures, already done.
- Changing how the app localizes. The harness consumes `app-localization`.
- Pixel-identical determinism. Falling notes and animations move; "same content on
  every surface" is the bar, not byte equality.

## Decisions

### D1 — Extend the existing `integration_test` seam rather than adopt a screenshot tool

Wire `integrationDriver(onScreenshot: ...)` in `test_driver/integration_test.dart` and
add a capture scenario beside `app_test.dart`, run through `flutter drive` exactly as
`integration-watch` already does.

*Alternative rejected*: fastlane `snapshot` (iOS-only, needs UI tests in Swift) or the
`screenshots` package (unmaintained, wraps the same driver). Both add a dependency and
a second navigation language for a job the existing seam already does. The decisive
factor is that `app_test.dart` has already solved "boot the real app in a chosen
state" for this app — the capture scenario reuses that override list rather than
re-deriving it.

### D2 — Force locale by overriding `deviceLocaleProvider`

`AppLocale.build()` watches `deviceLocaleProvider`, itself a one-liner over
`PlatformDispatcher.instance.locale` ([app_locale.dart:33](../../../apps/music/lib/state/app_locale.dart)).
Overriding it in the harness `ProviderScope` selects the run's language.

*Alternative rejected*: a `--dart-define` read in `main.dart`. That puts a test-only
branch in production startup. Overriding the provider is the pattern the repo already
mandates for dependencies, and it keeps `main.dart` untouched.

*Consequence to handle*: `AppLocale` pushes `SetLocale` to the backend when it
restores. The capture run must be signed out, or `accountServiceProvider` overridden,
so a capture never mutates a real account's language.

### D3 — Seed state through provider overrides, never a live backend

The capture scenario overrides the catalog, session, preferences, connectivity and
rewards seams with fixtures — extending the set `app_test.dart` already uses — so the
library, courses, leaderboard, badges and shop surfaces render chosen content.

*Alternative rejected*: capture against a seeded local backend. That is what the July
and August passes did, and it is precisely why they cannot be reproduced: the state
lived in someone's Postgres, not in the repo.

### D4 — Simulate the MIDI device and the performance

Override `midiServiceProvider` with a fake that reports a connected device named
plausibly and emits a scripted note stream timed against the fixture score, so the
player shots show a connected instrument and a healthy accuracy.

*Alternative rejected*: plug in a real keyboard, as the README currently asks. That
reintroduces the manual dependency this change exists to remove, and it is unavailable
on CI-class machines and on the Android emulator.

### D5 — Target the classes each store requires today, and assert the output

| Target | Class | Landscape px | Capture device |
|---|---|---|---|
| iPhone | 6.9" | 2868 × 1320 | iPhone 17 Pro Max simulator |
| iPad | 13" | 2752 × 2064 | iPad Pro 13" (M4/M5) simulator |
| macOS | accepted size | 1440 × 900 | window sized by the harness |
| Android phone | 16:9 | 1920 × 1080 | emulator profile at that resolution |

iPhone 6.9" and iPad 13" are what App Store Connect currently requires; the existing
2796×1290 / 2732×2048 sets are stale classes, not merely stale content. macOS 1440×900
remains accepted and is kept. Android moves from 2160×1080 (exactly 2:1 — legal under
Play's "long side ≤ 2× short side" cap, which is why the July pass cropped to it) to
16:9, which is what Play asks for to appear in recommendation surfaces.

The harness asserts every produced image against the declared dimensions and fails the
run on a mismatch, rather than emitting an off-size file that App Store Connect
rejects days later.

### D6 — Layout: `store/<platform>/<class>/<locale>/NN_<surface>.png`

Platform-major, locale-innermost. Diffing one locale's regeneration touches one
directory, and the per-image locale is unambiguous from the path — which the spec
requires.

### D7 — CI checks dimensions, not capture

A cheap gate reads the PNG headers of committed assets and compares them to a declared
manifest. It catches an off-size or hand-edited image without booting anything.

### D8 — macOS window sizing moves into the harness

The scenario sets the window size at runtime through the existing window seam, so the
`MainFlutterWindow.swift` edit — and the README warning never to commit it —
disappear. `isRestorable` must stay off for the run, since macOS restores saved
geometry over an earlier `setFrame`; that lesson is already recorded in the README and
carries over.

## Risks / Trade-offs

- **Alpha channel** → Apple rejects screenshots with transparency, and Flutter's
  engine capture can carry an alpha channel. The harness must flatten and the
  dimension gate must also assert no alpha.
- **Repository weight** → 4 locales × 4 targets × ~5 surfaces ≈ 80 images, fully
  replaced on every regeneration, so each pass adds its full weight to git history.
  Mitigation: keep the surface list tight (5, not 10), and commit JPEG — both stores
  accept it — if PNG proves heavy. This is the main cost of the four-locale decision
  and it is accepted knowingly.
- **Simulator drift** → Apple renames and retires device classes (that is how the
  current sets rotted). The declared manifest holds the required dimensions, so a
  future class change is a one-line manifest edit plus a re-run, not an
  archaeology exercise.
- **Android status bar** → the July pass needed emulator demo mode for a clean status
  bar; the harness must set it, or the shots show a real battery and clock.
- **Animated surfaces** → falling notes are in motion, so two runs differ pixel-wise.
  The spec's repeatability scenario is deliberately about content, not bytes.
- **Locale coverage is a maintenance multiplier** → adding a fifth shipping locale
  now costs a capture run per platform. Acceptable while the harness is one command;
  it would not be if capture stayed manual.

## Migration Plan

1. Land the harness and the manifest with the current sets still in place.
2. Regenerate all four targets in `en`, compare against the existing English sets to
   confirm the harness reproduces known-good framing.
3. Regenerate the remaining locales, delete the old `iphone_6.7/`, `ipad_12.9/`,
   `android/phone/` and flat `macos/` directories in the same commit.
4. Rewrite `store/README.md`: layout gains the locale level, the "known caveat"
   section and the manual reproduction instructions are replaced by the command.
5. Update the `store-distribution` spec's "Store-listing assets" requirement, which
   names the retired 6.7"/12.9" classes — in place if `prepare-store-distribution` is
   still in flight, as a MODIFIED delta once it is archived.

Rollback is `git revert`: the assets are files, nothing is deployed.

## Open Questions

- Which five surfaces earn a slot? The library, falling-notes player and staff are
  settled; the remaining slots compete between courses, leaderboard, badges/shop and
  measure selection, and that is a marketing call rather than a technical one.
- Do the it/es listings want their own captures, or is the four-locale decision worth
  revisiting if repository weight becomes a problem after the first full pass?
- Does Play's tablet listing warrant its own set? Not required for a phone app, but
  the app ships a distinct iPad layout that Android tablets would share.
