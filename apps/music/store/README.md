# Store listing assets

Version-controlled marketing assets for the App Store and Google Play listings.
Source branding lives in `../assets/branding/`; these are the store-facing
deliverables generated/curated from it.

## Layout

```
store/
├── manifest.json                      # generated: the declared targets, locales and assets
├── play/
│   ├── play_icon_512.png              # Google Play hi-res icon, 512×512, opaque
│   └── play_feature_graphic_1024x500.png  # Google Play feature graphic
├── ios/
│   ├── iphone_6.9/<locale>/NN_<surface>.png   # 2868×1320 landscape
│   └── ipad_13/<locale>/NN_<surface>.png      # 2752×2064 landscape
├── macos/
│   └── desktop_1440x900/<locale>/NN_<surface>.png
├── android/
│   └── phone_16x9/<locale>/NN_<surface>.png   # 1920×1080 landscape
└── copy/                              # listing text (description, keywords, …)
```

`<locale>` is one of `en`, `fr`, `it`, `es` — the four locales the listing copy
ships in — so the language of any image is unambiguous from its path.

## Regenerating the screenshots

Screenshots are **generated**, never taken by hand: one command per target, with
every locale by default.

```bash
cd apps/music && tool/capture_store_screenshots.sh macos/desktop_1440x900
```

```bash
cd apps/music && tool/capture_store_screenshots.sh ios/iphone_6.9 fr
```

Equivalently, from the repo root: `melos run screenshots -- ios/iphone_6.9`.

The command drives the real app through the listing surfaces
(`integration_test/capture_test.dart`) and writes the set for that target. Boot
the matching simulator/emulator first; override the device with
`CAPTURE_DEVICE="iPhone 17 Pro Max"` when yours is named differently.

What makes a run reproducible — and what the old, hand-made sets got wrong:

- **No backend.** The library, the courses and the progress are seeded from
  fixtures in `integration_test/support/`; the scores themselves are the real
  bundled ones. Nothing depends on what a database happened to hold.
- **A connected instrument.** The harness fakes a MIDI keyboard and *plays the
  piece* through the app's Wait Mode, so the player shots show a connected
  device and an earned, non-zero accuracy — not the "No MIDI device" and "0%"
  the previous macOS set shipped.
- **A driven locale.** The run forces the app's language, so a full set exists
  per locale without touching the device language or an account preference.
- **No source edit.** The window size comes from the manifest at runtime;
  nothing under `macos/Runner/` is touched, and playback is stopped before the
  non-player surfaces so no end-of-session summary can cover a capture.

Sizes are **declared**, not inherited from the capture device: see
`../tool/store_manifest.dart` (mirrored to `manifest.json`). That file is the
single place to edit when a store changes its required class — then re-run the
capture. Both the capture run and the CI gate refuse an image that does not
match it, or that carries the alpha channel Apple rejects:

```bash
cd apps/music && dart run tool/check_store_assets.dart
```

## Surfaces covered

Each set shows the same five surfaces, in this order:

| # | Surface | What it shows |
|---|---------|---------------|
| 01 | `library` | The score library: interactive courses on top, favorites by level |
| 02 | `synthesia` | The falling-notes player mid-performance, with the live accuracy gauge |
| 03 | `staff` | The scrolling staff view of the same performance |
| 04 | `courses` | The solfège learning path — units, lessons and progress |
| 05 | `measures` | Measure-range selection, for drilling one passage |

Shipped features **not** in the current sets — the audit trail this table
exists for: the score hub and its search, the seasonal leaderboard, badges and
the rewards shop, the rating deck, piano-sound selection, and the note reading
aid. Adding one is a `kCaptureSurfaces` entry in `tool/store_manifest.dart`,
a step in `capture_test.dart`, and a re-run of every target.

Keep that list short on purpose: 4 targets × 4 locales × 5 surfaces is ~16 MiB
of PNG, replaced wholesale on every regeneration. JPEG is not the way out — on
these images (dark, flat fields plus sharp text and glows) quality 92 comes out
slightly *larger* than PNG and quality 85 saves 15% for visible artifacts. The
surface count is the only real lever.

## Status

- [x] Play hi-res icon (512×512)
- [x] Play feature graphic (1024×500)
- [x] iOS screenshots — **6.9" iPhone** (`ios/iphone_6.9/`, 2868×1320) and
      **13" iPad** (`ios/ipad_13/`, 2752×2064), landscape, four locales. These
      are the classes App Store Connect requires today; the 6.7"/12.9" sets they
      replace were retired classes, not merely stale content.
- [x] macOS screenshots — `macos/desktop_1440x900/` (one of the four sizes the
      Mac App Store accepts), four locales.
- [x] Android phone screenshots — `android/phone_16x9/` (1920×1080, 16:9, within
      Play's 2:1 long-side cap), four locales.
- [x] Listing copy under `copy/` — en, fr, it, es: name, subtitle, short/promo,
      keywords, full description, categories, all within store char limits.

## Categories (decided)

- App Store: primary **Education**, secondary **Music**. Same on macOS, and the
  primary must stay Education — it is hard-coded as
  `LSApplicationCategoryType = public.app-category.education` in the macOS
  `Info.plist`, and App Store Connect flags a mismatch between the two.
- Google Play: **Education**.

The feature graphic tagline ("Learn piano by playing") is a placeholder — adjust
per final marketing and per-locale listings (app ships en/fr/it/es).
