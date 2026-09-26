## Why

« Traduction étendue » runs the Bergamot engine on the device on Chromium, Firefox desktop and,
since #553, Firefox for Android. Safari was left out by `add-lingua-translation-delivery` because
nothing had been measured there: iOS caps the memory of an extension's process at a limit Apple
does not publish, the iOS simulator does not enforce it, and the one Safari package serves iPhone,
iPad and Mac alike.

Measured on 2026-09-26 on the three, with `main`'s Safari package carrying the engine the way the
Firefox package does (a throwaway build, timestamped probes to a Mac over the local network,
nothing committed), the model downloaded from `models.cymbra.app`:

| Device | Download (25.8 MB) | Engine load | Translation | Cut by the system |
|---|---|---|---|---|
| iPhone 15 Pro Max (iOS 27) | 0.8 s | 125–127 ms | 94–217 ms | never; one restart on return from another app, reloaded in 127 ms |
| iPad Air 13" M2 (iPadOS 26) | 0.9 s | 158–165 ms | 53–240 ms | never; one restart on return, reloaded in 165 ms |
| Mac (Safari) | < 1 s | 117 ms | 200 ms | never |

The engine loads twenty times faster than on the Android tablet that now offers it, and iOS never
stopped the extension while it held the engine. The product owner saw the translations as
instantaneous on all three and asked for the change.

## What Changes

- **The Safari package carries the engine**, as the Firefox package does: the engine's worker in
  the event page, the model downloaded on demand from the same host, the same setting with the same
  copy and states, on iPhone, iPad and Mac.
- **The build and its checks follow.** `build.mjs` gives Safari the event-page host;
  `check_variants.mjs` requires the engine and the model manifest in Safari (and still no model
  file, no remote code, the `offscreen` permission only on Chromium). The Apple release lane
  (`lingua-apple-release`) fetches the pinned engine before building, and refuses to deliver a
  build whose model cannot be downloaded, as the extension release already does.
- **The App Store listing and review notes say it.** « Sans compte, l'extension ne fait aucune
  requête réseau » becomes « sans compte ni traduction étendue »; the page text still never leaves
  the device. The review notes explain that the engine ships in the app and only a data file is
  downloaded (guideline 2.5.2 forbids downloading code, not data).
- Unchanged: the privacy annex (it names no platform), the App Store privacy labels (the download
  carries nothing of the reader's and is associated with no one), the host app, the model and its
  host, #553's warm signals and answer memory — which Safari gets as they are.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: **modified** — **Extended translation is offered where it has been
  measured**: Safari joins, on iPhone, iPad and Mac, and no variant is left out. The requirement is
  added by `add-lingua-translation-delivery` and modified by `add-lingua-translation-android`, both
  still in `openspec/changes/`: this change archives after both, an order it declares
  (`archiveAfter` in `.openspec.yaml`, read by `scripts/openspec_archive_order.py`).

## Impact

Products: **Cymbra Lingua only** — the Safari variant of the extension and the Apple host app's
release lane and listing. Consumed as they are: the engine artefact and its pin, the model host
and manifest, the setting, the warm signals. **ID, Music, Live, back office, site: not affected.**

- `apps/lingua-extension/build.mjs`, `tool/check_variants.mjs`, their tests.
- `.github/workflows/lingua-apple-release.yml` — fetch the pinned engine; check the model host
  before `deliver`.
- `scripts/openspec_archive_order.py` (+ test) — honours `archiveAfter`.
- `apps/lingua-apple/STORE-LISTING.md`, `README.md`; `apps/lingua-extension/TRANSLATION.md`,
  `README.md`.
- Package size: the Apple app grows by the engine, about 4.8 MB uncompressed (1.1 MB brotli).
- No new permission or entitlement: the extension's requests are made by Safari's web content,
  as the model download measured on the three devices shows.
