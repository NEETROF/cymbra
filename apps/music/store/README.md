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
- [x] iOS screenshots — **6.7" iPhone** (`ios/iphone_6.7/`, 2796×1290) and **12.9" iPad** (`ios/ipad_12.9/`, 2732×2048), landscape. Captured from the iPhone 15 Pro Max and iPad Pro 12.9" simulators: score library, Synthesia falling-notes, staff, and (iPad) full-score "Partition" view.
- [ ] Android phone screenshots — ≥2, landscape.
- [x] Listing copy under `copy/` — **en** and **fr** done (`copy/en.md`, `copy/fr.md`): name, subtitle, short/promo, keywords, full description, categories, all within store char limits. **it** and **es** still to translate (app ships 4 locales).

## Categories (decided)

- App Store: primary **Education**, secondary **Music**.
- Google Play: **Education**.

The feature graphic tagline ("Learn piano by playing") is a placeholder — adjust
per final marketing and per-locale listings (app ships en/fr/it/es).
