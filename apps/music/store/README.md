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
- [ ] iOS screenshots — 6.7" iPhone (1290×2796→landscape) and 12.9" iPad, **landscape** (the app is landscape-only). Capture manually from a simulator/device.
- [ ] Android phone screenshots — ≥2, landscape.
- [ ] Listing copy under `copy/` — description, subtitle/short promo, iOS keywords, category.

## Categories (decided)

- App Store: primary **Education**, secondary **Music**.
- Google Play: **Education**.

The feature graphic tagline ("Learn piano by playing") is a placeholder — adjust
per final marketing and per-locale listings (app ships en/fr/it/es).
