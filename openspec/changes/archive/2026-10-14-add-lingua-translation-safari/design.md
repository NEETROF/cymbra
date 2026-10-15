## Context

Safari's variant is derived from Firefox's (`add-lingua-apple`): an event page — non-persistent,
as iOS requires — a static content script, the in-page drawer. The engine's host on Firefox is that
event page (`__TRANSLATION_HOST__ = "event-page"`), which owns the engine's worker and the model
download's. `build.mjs` sets Safari to `"none"`, so the Safari bundle folds every translation path
away and `check_variants.mjs` asserts the engine is absent.

The 2026-09-26 measurement (proposal) flipped only that: `translationHost("safari")` returned
`"event-page"`, the engine and `model-manifest.json` were copied into the bundle, and the rest of
`main` ran unchanged. On the three devices the worker started in the event page, the model
downloaded and verified (`DecompressionStream`, sha256), IndexedDB held it, and the engine loaded in
117–165 ms. iOS restarted the event page once on each device, on the return from another app — the
same freeze #553 answers with its return-to-page warm — and the engine was back in under 170 ms.

Memory could not be read on iOS without device tooling; what was observed is the absence of any
termination while the engine was held, through a reading session with its heartbeat.

## Goals / Non-Goals

**Goals:**
- Offer extended translation in Safari on iPhone, iPad and Mac, identical to Firefox's.
- Keep the Apple release as safe as the extension release: the pinned engine, and no delivery of a
  build whose model host does not answer.

**Non-Goals:**
- Apple's own Translation framework: the local engine answers in under 250 ms on the measured
  devices and marks the selection; Apple's would need the native relay and marks nothing.
- A per-device gate inside Safari: no measured device needs one (D2).
- The delay between lifting a finger and the card opening on iOS (about two seconds as felt, the
  engine answering in 0.1–0.2 s of it): it is the selection gesture's, with or without the engine.

## Decisions

### D1 — Safari hosts the engine in its event page, like Firefox

`translationHost("safari")` returns `"event-page"`. Everything downstream is the Firefox path
already built and tested: `EngineChannel` and `DownloadHost` in the background, the engine worker
loading its model from the `lingua-model` database, the setting and its states, the warm signals and
the answer memory of `add-lingua-translation-android`. No Safari-specific code is added.

*Alternative:* a native engine in the host app over `nativeMessaging`. Rejected: the WASM engine
runs in Safari's event page (measured), and a second engine would be a second thing to pin, review
and keep in step.

### D2 — Offered on every Safari device, with no run-time gate

The one package serves iPhone, iPad and Mac, so a gate could only be decided at run time. None is
added: the smallest-memory measured device, the iPhone, held the engine without being cut, and the
setting states the memory it uses before it is ticked. A device where iOS does cut it is what would
bring a gate — or `add-lingua-remote-translation` — back.

### D3 — Build checks and the Apple release lane

- `check_variants.mjs`: the engine, pinned, and `model-manifest.json` in Chromium, Firefox **and
  Safari**; no model file, no remote code, anywhere; `offscreen` in Chromium only.
- `lingua-apple-release`: `yarn fetch:engine` before `yarn build:safari` (the build now refuses to
  run without the pinned engine, as Chromium's and Firefox's do); and before `deliver`, the same
  `check_model_host.mjs` the extension release runs — a build that ships the setting must not reach
  TestFlight or the App Store while its model host does not answer.

### D4 — Listing and review notes

`apps/lingua-apple/STORE-LISTING.md`: the description's « Sans compte, l'extension ne fait aucune
requête réseau » becomes « Sans compte ni traduction étendue, l'extension ne fait aucune requête
réseau », and a sentence describes the setting as the Chrome and AMO listings do. The review notes
add that the translation engine is part of the app, off by default, and that turning it on
downloads a data file (the model) once — no code. The App Store privacy labels do not change: the
download is associated with no account and no device, and carries nothing of the reader's
(`lingua-privacy`, *The model download carries nothing of the reader's*).

## Risks / Trade-offs

- [iOS memory limit on an older or smaller device] → Not measured below the iPhone 15 Pro Max. A
  cut extension restarts and the next selection reloads the engine (the card answers from the pack
  meanwhile); a device that fails reopens the gate question (D2).
- [App Review reads the WASM as downloaded code] → It is in the bundle; the review notes say so, and
  what is downloaded is a model file with a pinned hash.
- [Package size] → about 4.8 MB more in the app, uncompressed.
- [A stale bundle in a local build] → Xcode's « Copy Lingua extension » phase does not delete files
  that disappeared from `dist-safari`; a local build switched between variants keeps old files until
  DerivedData is cleaned. Release builds start clean. Noted in `apps/lingua-apple/README.md`.

## Migration Plan

Readers who update the Safari extension see the setting, off; nothing downloads until they tick it.
Rollback: return `"none"` for Safari; a downloaded model stays until the setting is turned off.

## D5 — Archive order, declared

The requirement this change modifies is added by `add-lingua-translation-delivery` and modified by
`add-lingua-translation-android`, both still in `openspec/changes/`. `scripts/openspec_archive_order.py`
holds a change until the one that ADDS a requirement it modifies is archived, but it cannot tell
which of two changes modifying the same requirement comes first — and the workflow walks changes
newest first, so this one would archive before `…-android`, whose text would then replace it and
take Safari out again.

The order is declared, not guessed: `.openspec.yaml` gains `archiveAfter`, a list of changes this
one must follow (`openspec` ignores the key). The script treats a listed change still in
`openspec/changes/` as one to wait for (exit 10), exactly as an ADDED dependency. This delta states
the final text for all four platforms.

*Alternative:* order by creation date. Rejected: both changes were created the same day, and a date
says when a change was written, not which text must win.

## Open Questions

None.
