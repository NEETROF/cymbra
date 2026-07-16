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
└── copy/                              # Listing text (description, keywords, …)
```

## Status

- [x] Play hi-res icon (512×512)
- [x] Play feature graphic (1024×500)
- [x] iOS screenshots — **iPhone** and **12.9" iPad** (`ios/ipad_12.9/`, 2732×2048), landscape. Captured from the iPhone 15 Pro Max and iPad Pro 12.9" simulators: score library, Synthesia falling-notes, staff, and (iPad) full-score "Partition" view.
      - iPhone comes in two sizes: `ios/iphone_6.7/` (2796×1290, for the App Store Connect **6.7"/6.9"** slot) and `ios/iphone_6.5/` (2778×1284, for the **6.5"** slot — App Store Connect rejects 2796×1290 there). Upload whichever matches the slot the record shows.
- [x] Android phone screenshots — `android/phone/` (2160×1080, landscape): score library, Synthesia, staff. Captured on the Pixel 3a emulator, cropped to exactly 2:1 (Play caps the long:short side ratio at 2:1; the raw 2220×1080 was 2.06:1) with a demo-mode clean status bar.
- [x] Listing copy under `copy/` — **en, fr, it, es** (all four shipping locales): name, subtitle, short/promo, keywords, full description, categories, all within store char limits.

## Categories (decided)

- App Store: primary **Education**, secondary **Music**.
- Google Play: **Education**.

The feature graphic tagline ("Learn piano by playing") is a placeholder — adjust
per final marketing and per-locale listings (app ships en/fr/it/es).
