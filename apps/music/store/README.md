# Store listing assets

Version-controlled marketing assets for the App Store and Google Play listings.
Source branding lives in `../assets/branding/`; these are the store-facing
deliverables generated/curated from it.

## Layout

```
store/
├── play/
│   ├── play_icon_512.png              # Google Play hi-res icon, 512×512, opaque
│   └── play_feature_graphic_1024x500.png  # Google Play feature graphic
├── ios/                               # App Store screenshots (see below)
├── macos/                             # Mac App Store screenshots, 1440×900
└── copy/                              # Listing text (description, keywords, …)
```

## Status

- [x] Play hi-res icon (512×512)
- [x] Play feature graphic (1024×500)
- [x] iOS screenshots — **6.7" iPhone** (`ios/iphone_6.7/`, 2796×1290) and **12.9" iPad** (`ios/ipad_12.9/`, 2732×2048), landscape. Captured from the iPhone 15 Pro Max and iPad Pro 12.9" simulators: score library, Synthesia falling-notes, staff, and (iPad) full-score "Partition" view.
- [x] Android phone screenshots — `android/phone/` (2160×1080, landscape): score library, Synthesia, staff. Captured on the Pixel 3a emulator, cropped to exactly 2:1 (Play caps the long:short side ratio at 2:1; the raw 2220×1080 was 2.06:1) with a demo-mode clean status bar.
- [x] macOS screenshots — `macos/` (1440×900, one of the four sizes the Mac App Store accepts): score library, Synthesia falling-notes, staff, full-score "Partition". **English UI**, captured from a release build of the real app against a local backend, window pinned to 1440×900. Only the `en` locale is covered — fr/it/es listings reuse these unless localized sets are captured later.
- [x] Listing copy under `copy/` — **en, fr, it, es** (all four shipping locales): name, subtitle, short/promo, keywords, full description, categories, all within store char limits.

## Categories (decided)

- App Store: primary **Education**, secondary **Music**. Same on macOS, and the
  primary must stay Education — it is hard-coded as
  `LSApplicationCategoryType = public.app-category.education` in the macOS
  `Info.plist`, and App Store Connect flags a mismatch between the two.
- Google Play: **Education**.

## Known caveat on the macOS captures

The three player screenshots show **"No MIDI device"** and **"0%"** in the top
bar, because they were taken without a MIDI keyboard connected. Both are truthful
UI, but they read as a warning and a bad score on a store page. Re-take them with
a keyboard plugged in (and a few correct notes played) before the listing goes
live.

Two things that silently ruin a capture, both hit during the first pass:
- Take the shot while the window is **focused**, or the traffic lights render grey
  and the window reads as inactive. Run `open <path>.app` immediately before
  `screencapture`.
- Don't leave playback running between shots. The piece reaches its end and the
  session summary ("0% · 268 missed") covers the view.

## Reproducing the macOS captures

`screencapture` needs Screen Recording permission for whatever runs it. The window
size is pinned by a **temporary** edit to `macos/Runner/MainFlutterWindow.swift`
(`isRestorable = false` + an explicit 1440×900 frame, set *after* `super.awakeFromNib()`
— macOS restores the saved geometry and silently overrides an earlier `setFrame`).
That edit must be reverted, never committed. Then:

```bash
screencapture -x -R 100,600,1440,900 apps/music/store/macos/01_library.png
```

The feature graphic tagline ("Learn piano by playing") is a placeholder — adjust
per final marketing and per-locale listings (app ships en/fr/it/es).
