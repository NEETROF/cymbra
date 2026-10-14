## Why

« Traduction étendue » shipped on Chromium and Firefox desktop (`add-lingua-translation-delivery`,
#546) and stayed hidden on Firefox for Android (its D8), on a measurement of 2026-09-23: a 4.8 s
cold start the card could not wait for, and the event page torn down every 8–24 s. Since then the
card stopped waiting (it shows the pack's answer at once and the translation replaces it), and the
reading page's heartbeat holds the event page.

Measured again on 2026-09-26, on the same Galaxy Tab S6 Lite (SM-P610, 4 GB, Firefox release), with
`main`'s build and only the platform check lifted: **no failure in any session**. The 25.8 MB
download finished on the first attempt (3.8 s on Wi-Fi, 53 s at a mobile 500 KB/s); the event page
did not restart once while a reading page was open; a warm translation took 0.4–1 s; the extension
process grew by 180 MB and Android reclaimed memory elsewhere without killing anything. What is left
is cost, not instability: **4.1–4.7 s of cold start** on the first selection of a visit and after
every return from another app (Android freezes the tab, its heartbeat stops, Firefox tears the event
page down and the engine with it). And adjusting the selection handles translated the same sentence
five times in six seconds, 0.4 s of CPU each.

The product owner chose to offer it on Android with the engine on the device. Translating remotely
(`add-lingua-remote-translation`, parked) remains the later option for a device where this fails.

## What Changes

- **The setting is offered on Firefox for Android.** The run-time platform check goes; the setting,
  its download, its deletion and its states are the desktop ones, unchanged.
- **The engine starts loading when a selection begins**, not when it settles. The long-press and
  the handle adjustments that follow take one to several seconds on a tablet; the cold start runs
  during them instead of after them. Only when a model is ready; nothing is loaded otherwise.
- **The engine is restored when a page that was translating comes back.** A page that asked for a
  translation within the idle period asks for the engine again when it becomes visible, so the
  engine Android took away while the tab was frozen is loading while the reader finds their place.
- **A selection already translated on the page is not translated again.** The page keeps the
  answers it received; the same sentence with the same selection is answered from them.
- **Copy**: the store listings say where extended translation is offered (now Firefox for Android
  too). The privacy annex names no platform and does not change. The setting's text already states
  the download size and the memory used, which is what a tablet reader needs to know.
- Unchanged: the heartbeat, the ten-minute idle release, the download flow (on Android the settings
  page stays alive and its heartbeat carries the download; a dismissed page leaves a download the
  setting offers to resume), the model and its host, the manifest and its permissions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: **modified** — **Extended translation is offered where it has been
  measured** (Firefox for Android joins Chromium and Firefox desktop) and **The engine is loaded
  only while it is used** (a selection that begins, and a translating page that comes back, load the
  engine before a translation is asked; the idle release is unchanged). **Added** — a selection
  already translated on the page is answered without running the engine again. Both modified
  requirements are added by `add-lingua-translation-delivery`, still open: this change archives
  after it (`scripts/openspec_archive_order.py`).

## Impact

Products: **Cymbra Lingua only** (the browser extension). New: extended translation on Firefox for
Android. Consumed as they are: the model host `models.cymbra.app` and its manifest, the engine
artefact, the settings row. **ID, Music, Live, back office, site: not affected** — no backend, no
account, no privacy text changes.

- `apps/lingua-extension/src/background.ts` — the platform check goes; a `warm` message is answered
  when a model is ready.
- `src/translate/host/` — `EngineAccess` gains `warm()` (`channel.ts` on Firefox,
  `offscreen-engine.ts` + `offscreen.ts` on Chromium); warming arms the idle release like a
  translation does.
- `src/translate/` — the warm wire message; the page's answer memory and its return-to-page warm
  around the messaging port (`create-port.ts`, `keepalive.ts`); the idle period moves where a
  surface may read it.
- `src/reading/session.ts` — a selection that begins asks for the engine.
- `STORE-LISTING.md`, `TRANSLATION.md`, `README.md`, `REVIEWERS.md`.
- No new permission, no manifest change, no new request: the model download is the existing one.
- Memory: a Firefox for Android reader who turns the setting on pays about 180 MB while reading
  with it, released ten minutes after the last translation, or earlier when Android freezes the tab.
