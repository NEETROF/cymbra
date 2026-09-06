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

Each set shows the same seven surfaces, in this order:

| # | Surface | What it shows |
|---|---------|---------------|
| 01 | `library` | The score library: interactive courses on top, favorites by level |
| 02 | `synthesia` | The falling-notes player mid-performance, with the live accuracy gauge |
| 03 | `staff` | The scrolling staff view of the same performance |
| 04 | `courses` | The solfège learning path — units, lessons and progress |
| 05 | `measures` | Measure-range selection, for drilling one passage |
| 06 | `drums` | The animated kit, mid-groove |
| 07 | `drums_staff` | The same drum part read on the staff |

The drum pair only became publishable when `drums.enabled` went global: while
it was scoped to `beta:midi-drums`, shipping those two would have advertised a
feature an ordinary installer could not reach.

Shipped features **not** in the current sets — the audit trail this table
exists for: the score hub and its search, the seasonal leaderboard, badges and
the rewards shop, the rating deck, piano-sound selection, and the note reading
aid. Adding one is a `kCaptureSurfaces` entry in `tool/store_manifest.dart`,
a step in `capture_test.dart`, and a re-run of every target.

Keep that list short on purpose: 4 targets × 4 locales × 7 surfaces is ~22 MiB
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
- [x] Android tablet screenshots — `android/tablet_7/` (1920×1080) and
      `android/tablet_10/` (2560×1440), four locales. Play marks both
      **required** as soon as the bundle does not restrict screen sizes, which
      ours does not; they are 16:9 because Play demands exactly that on these
      slots, even though a real 7" tablet is 16:10.
- [x] Listing copy under `copy/` — en, fr, it, es: name, subtitle, short/promo,
      keywords, full description, categories, all within store char limits.

## Publishing

`release-build.yml` uploads the signed AAB to Play on a `music-v*` tag, as a
**draft** release on the production track (`status: draft` plus
`changesNotSentForReview`). CI therefore removes the part that actually hurts —
pushing a ~110 MB bundle through a browser — without shipping anything on its
own: a human still opens *Vue d'ensemble de la publication* and presses *Envoyer
pour examen*.

That split is deliberate. Managed publishing is **off**, so whatever is sent for
review goes live the moment Google approves it; if CI also sent it, merging a
release-please PR would publish to every country with nobody in the loop. Making
it fully automatic is one line (`status: completed`, drop
`changesNotSentForReview`) — a decision, not an oversight.

Release notes are not set from CI. Play makes them optional, and the person who
sends the release for review is already in the console; a `whatsnew/` directory
in the repo would go stale between releases.

The step needs a `PLAY_SERVICE_ACCOUNT_JSON` repo secret — the raw JSON key of a
Google Cloud service account granted **Release manager** on the app (Play Console
→ Configuration → Accès à l'API). Note that the `neetrof.fr` org inherits
`iam.disableServiceAccountKeyCreation`, so creating the key needs the same
project-scoped override already used for the RevenueCat service account. Without
the secret the upload is skipped with a warning and the AAB is still attached to
the GitHub Release, which is how it was published by hand until now.

## Categories (decided)

- App Store: primary **Education**, secondary **Music**. Same on macOS, and the
  primary must stay Education — it is hard-coded as
  `LSApplicationCategoryType = public.app-category.education` in the macOS
  `Info.plist`, and App Store Connect flags a mismatch between the two.
- Google Play: **Education**.

The feature graphic tagline now reads "Learn piano and drums", matching the
subtitle. It is the one listing asset whose text lives in **pixels**, so it has
to be re-rendered whenever the positioning changes — and it is easy to forget,
which is exactly how it kept saying "Learn piano by playing" after drums went
global. It is also **English-only**: Play serves the same graphic to every
locale, so it is a marketing artefact, not a translated string.

Re-rendered by rebuilding the tagline band from the clean rows above and below
it (linear interpolation preserves the radial glow; a copied strip from
elsewhere leaves a visible seam), then re-drawing the line in SF Pro 30px
`#8FA3D6` at the original left edge — the tagline is left-aligned with the
title, not centred under it.
