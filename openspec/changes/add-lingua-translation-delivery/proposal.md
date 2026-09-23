## Why

The sentence engine exists (`add-lingua-translation-engine`, #532): it answers a reader's
selection with their whole sentence translated and the selection marked in its French place, off
every thread that paints, measured on Chrome, Firefox desktop and a Galaxy Tab S6 Lite. It reaches
no reader. No shipped build carries it, nothing downloads its model, and no setting turns it on —
by design, that change proved the engine and stopped there.

This change is the one that puts it in readers' hands. It is also the precondition of two others
already proposed: `add-lingua-remote-translation` (its task 0.2 waits for the setting this change
introduces) and the device mode of `add-lingua-youtube-captions`.

## What Changes

- **A per-device "Traduction étendue" setting**, off by default, in the extension's settings. It
  is stored on the device and never synced: turning it on on the Mac does not start a 25.6 MiB
  download on the tablet. Its stored value is a choice of host (`none` / `local`), shown today as
  one checkbox, so that `add-lingua-remote-translation` adds its third state without migrating it.
- **The model is downloaded only when the reader turns the setting on**, once, from a Cymbra static
  host, and verified against pinned hashes before it is used. The download says its size before it
  starts, shows its progress, can be cancelled, and reports a failure in the reader's language.
  Turning the setting off deletes the model.
- **The engine ships inside the extension; only the model is downloaded.** This reverses the
  2026-09-22 decision to download the engine too: the Chrome Web Store counts WebAssembly as
  remotely hosted code, which a Manifest V3 extension may not load, and AMO and the App Store have
  the same rule. The model is data, which all three allow. Cost: about 1.1 MB (brotli) per
  variant that carries the engine.
- **Single words gain the engine where the pack has nothing.** A single word is translated — in
  its sentence, never alone — only when the pack has no gloss for it and it is not a proper noun
  outside the lexicon. The pack's gloss always wins when it exists.
- **The engine is released when reading stops.** It stays loaded while a reading tab uses it
  (the existing keep-warm), and is torn down after an idle period, giving its ~195 MiB back.
- **Offered on Chromium and Firefox desktop.** Firefox Android shares the Firefox build and keeps
  the setting hidden at run time until its own change; Safari carries no engine.
- **The public promise is rewritten where it becomes false**: "without an account, no network
  request" becomes "without an account and without extended translation, no network request".
  "The text of the pages you read never leaves your device" stays unconditional.

## Capabilities

### New Capabilities

None — the setting, the download and the engine's lifetime belong to `lingua-translation`.

### Modified Capabilities

- `lingua-translation`: **added** — the reader turns extended translation on per device; the model
  is downloaded only then, from Cymbra, verified; turning it off deletes it; the engine is part of
  the extension and only the model is fetched; a model the browser removed is not fetched again
  unasked; the engine is released when idle; where the setting is offered. **Modified** —
  **Without a model the extension behaves as it does today** (now also covers a setting that is on
  while its model is not ready) and **The translation is shown as a machine translation, in the
  reader's sentence** (a single word the pack cannot answer is now translated).
- `lingua-browser-extension`: **modified** — **No network requests**, which still says "in v1 …
  no network request at all" although signed-in sync already contradicts it; it is rewritten to
  name every request the extension makes and when.
- `lingua-privacy`: **added** — the model download carries nothing of the reader's, and the Lingua
  annex says what it is. Added rather than modified because `add-lingua-remote-translation`
  already modifies **Lingua's privacy disclosures match what it collects**.

## Impact

**Products.** Cymbra Lingua only: the extension (Chromium and Firefox variants), its store
listings, the site's privacy page, and CI. No backend, no `.proto`, no database, no sync op: the
setting is per device. Music, Live and the back office are untouched.

**Consumed, not redeclared.** The engine, its seam (`TranslatorPort`), its worker hosts, the mark
and its check (`translate/reconcile.ts`) and the keep-warm heartbeat, all from #532.

**Code.** `apps/lingua-extension`: the engine becomes part of every Chromium and Firefox build
(`build.mjs`, `tool/check_variants.mjs`, which today refuses any trace of it, is inverted per
variant); the model moves from files beside the bundle to storage owned by the background; a
settings row, a download manager, and the single-word trigger in `selection-card.ts`.

**CI and stores.** The release build takes the engine from the `lingua-engine-build` artefact at a
pinned commit. AMO requires the source and build steps of any WebAssembly it ships, so the AMO
source archive gains the engine's reproducible build. The Chromium variant gains the `offscreen`
permission, which #532 already uses in development builds.

**Hosting.** The model (25.75 MB compressed, 36.7 MB on disk) is served from a Cymbra static
origin, never from Mozilla's CDN (no CORS header, and it answers 406 to a Chrome user agent).

**Ordering.** Archives before `add-lingua-remote-translation`, whose **The reader chooses where the
engine runs** should then be rewritten as a modification of this change's setting requirement.
