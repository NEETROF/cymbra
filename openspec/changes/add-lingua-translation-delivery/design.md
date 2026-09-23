## Context

`add-lingua-translation-engine` (#532) built the Bergamot engine from `mozilla/translations` at a
pinned commit in our CI, put it behind the `TranslatorPort` seam, and hosted it in a classic
Worker — owned by an offscreen document on Chromium, by the event page on Firefox — so that it
never occupies a thread that paints. It reaches no reader: `build.mjs` builds it in only when a
developer sets `LINGUA_TRANSLATION_ENGINE`, the model is side-loaded by hand
(`apps/lingua-extension/TRANSLATION.md`), and `tool/check_variants.mjs` refuses any trace of it in
a shipped build.

What is known, measured on the artefact itself:

| | Value |
|---|---|
| Engine | `bergamot-translator.wasm` 4 963 402 B (1 082 904 brotli) + `.js` 92 034 B (18 647 brotli) |
| Model `en→fr` `base-memory` v2.0 | 25 752 472 B compressed (Mozilla's `.gz`), 36 749 127 B on disk; sha256 pinned in `TRANSLATION.md` |
| Working set | ~195 MiB whatever `INITIAL_MEMORY` |
| Chrome (Mac) | cold start 270 ms, warm 41 ms |
| Galaxy Tab S6 Lite (Firefox) | cold 4 783 ms, 377 ms once the heartbeat holds the event page |

Decisions already taken by the product owner and not reopened here: the setting is **one**
checkbox, **per device**, off by default; turning it on also covers single words the pack cannot
answer; the model is `base-memory`, never `tiny`; it is served by Cymbra, never Mozilla's CDN;
turning the setting off deletes it; a machine translation is never stored.

One decision **is** reopened, with a reason the product owner did not have: whether the engine is
downloaded too (D1).

## Goals / Non-Goals

**Goals:**
- A reader on Chromium or Firefox desktop can turn extended translation on, download the model
  once, and get the sentence engine's answer on every surface #532 wired.
- Nothing leaves the device and nothing is downloaded for a reader who does not turn it on.
- The setting's stored shape already admits `add-lingua-remote-translation`'s third state.
- The engine's memory is given back when reading stops.

**Non-Goals:**
- Firefox for Android (hidden at run time, D8) and Safari (no engine) — each its own change.
- Remote translation (`add-lingua-remote-translation`).
- Other language pairs. The model is `en→fr`; a second pair is a second model and a later change.
- Reworking the mark: `translate/reconcile.ts` stands as #532 left it; this change only gates the
  release on measuring it (task 1.2).

## Decisions

### D1 — The engine ships in the package; only the model is downloaded

The 2026-09-22 decision was to download the engine along with the model, so that "nothing leaves
unless you tick the box" stays literally true. It cannot stand:

- Chrome Web Store, Manifest V3: remotely hosted code is *"anything that is executed by the
  browser that is loaded from someplace other than the extension's own files. Things like
  JavaScript and WASM"*, and it is not allowed. The same page states it *"does not include data"*.
- AMO requires an add-on to be self-contained and not to load code from outside its package.
- App Store guideline 2.5.2 forbids downloading executable code (relevant once Safari takes the
  engine).

The model is data, so downloading it is allowed everywhere. The promise survives: shipping the
engine inside the package sends nothing and fetches nothing; before the box is ticked the engine
is inert bytes on disk. The cost is ~1.1 MB of brotli per Chromium and Firefox package — 4 % of
what the reader downloads when they tick the box.

*Alternative rejected:* downloading the wasm and instantiating it from bytes. It works
technically (`wasmBinary`) and is exactly what the policy names.

### D2 — The setting stores a host, the UI shows a checkbox

`storage.local` (per device, never synced — the reader-state IndexedDB is what syncs) holds
`translationHost: "none" | "local"`, absent meaning `"none"`. The settings view shows one
checkbox, **« Traduction étendue »**, with its cost stated before it is ticked:

> Traduit vos phrases sur cet appareil, sans rien envoyer. Télécharge 25,6 Mo une fois, puis
> utilise environ 200 Mo de mémoire pendant la traduction. Réglage propre à cet appareil.

`add-lingua-remote-translation` widens the union with `"remote"` and turns the checkbox into a
choice; no stored value has to migrate. `createTranslatorPort()` becomes a function of this value
and of the model's readiness instead of a build constant.

### D3 — The model's addresses and hashes are in the package; the host only serves bytes

A `model-manifest.json` bundled with the extension lists, per file: its address, its compressed
size, and the sha256 of its **decompressed** bytes (the hashes `TRANSLATION.md` already pins). The
reviewed package therefore decides what is accepted; the host cannot substitute a model.

Files are served as Mozilla publishes them (`.gz`), under a content-addressed path
(`…/en-fr/base-memory/2.0/<sha256>/model.bin.gz`), with `Cache-Control: immutable` and
`Access-Control-Allow-Origin: *`. CORS rather than a host permission is deliberate: adding a host
permission to an installed Chromium extension disables it until the reader accepts the new
permission. Each file is at most 23 045 432 B, under the 25 MiB per-file limit of a static Pages
deployment, should that be the host.

Verification is `DecompressionStream("gzip")` then sha256 of the result. It also guards a known
trap of Cymbra's static site: an unknown path answers **200 with the French home page**, not 404.

*Open:* which origin — see Open Questions.

### D4 — The download runs where the engine lives

The download runs in the engine's host — the offscreen document's worker on Chromium, the event
page's worker on Firefox — because that is where the bytes will be read, and because a Chromium
service worker can be stopped in the middle of a 25 MB transfer. Progress, completion and failure
travel on the existing engine channel to the settings view. While the settings view is open it
keeps the host awake with the existing keep-warm ping; if the host is torn down anyway, the
download is **interrupted**, the setting says so and offers to resume, and resuming restarts only
the files not yet verified. Nothing restarts on its own (`A model the browser removed is not
fetched again unasked` applies to an interrupted download too).

### D5 — The model lives in its own IndexedDB database

A `lingua-model` database, separate from the reader-state store, holds each file's decompressed
bytes keyed by its sha256, plus a record of which manifest version is complete. Never
`storage.local`: its quota is what filled up in the exposure-counter incident. Storing decompressed
bytes keeps the load at the measured ~24 ms file read instead of adding a decompression per cold
start. The host calls `navigator.storage.persist()`; when the database is found empty while the
setting is on, the setting reports the model removed (D-requirement) rather than downloading again.

Turning the setting off terminates the worker (Chromium: closes the offscreen document) and deletes
the `lingua-model` database.

### D6 — Released after ten idle minutes

#532 left open *when the Worker is torn down*. Answer: the host records the time of the last
translation asked; keep-warm pings do not update it. Ten minutes after it, the host terminates the
worker (and on Chromium closes the offscreen document), giving back ~195 MiB. The next translation
pays a cold start (270 ms on the Mac). Ten minutes is a starting value: long enough to cover pauses
in reading, short enough that a forgotten tab does not hold 200 MiB for the afternoon.

### D7 — A single word goes to the engine only when the pack is silent

In `selection-card.ts`, a single-word selection keeps its dictionary card when the pack has a gloss.
When it has none and the analysis does not class the word `ProperNounOutOfLexicon`, the card asks
the engine for the word's **sentence** with the word marked — the same request as a phrase. Never
the word alone: the 100-sentence evaluation showed fragments translated alone going wrong
(`put up with` → « mis en place avec », `looked it up` → « Il le regarda ») where the same fragments
are right in their sentence. The trigger case observed in dogfooding: `disambiguation` on Wikipedia,
absent from the pack. A proper noun is excluded because translating it produces noise.

### D8 — Firefox for Android hides the setting at run time

The Android variant is the Firefox package, so the engine code is in it. The setting is hidden when
`runtime.getPlatformInfo()` reports `android`, and nothing there can start a download. Reasons it
waits for its own change: a 4 783 ms cold start (beyond what the card waits), and restarts under
memory pressure seen on a 4 GB tablet even with the heartbeat.

### D9 — Build, CI and review

- `build.mjs`: Chromium and Firefox **release** builds require the engine artefact and fail without
  it; Safari keeps `translationHost` `none`. The `offscreen` permission joins the Chromium manifest.
  The model is never in a package.
- `tool/check_variants.mjs` inverts: engine present in Chromium and Firefox, absent in Safari,
  `offscreen` present in Chromium only, no model file anywhere, `model-manifest.json` present where
  the engine is.
- The release workflow downloads the `lingua-engine-build` artefact for the pinned commit and checks
  its hash before packaging.
- The AMO source archive gains the engine's reproducible build (pinned `mozilla/translations`
  commit, emsdk 3.1.8, the CI recipe), since AMO reviews the source of any WebAssembly it ships.
  `REVIEWERS.md` explains the engine, the model download and why no code is fetched.
- `TRANSLATION.md` becomes the developer page for the release path; side-loading remains only as a
  development override of the manifest's host.

## Risks / Trade-offs

- **[The mark lands on the wrong word]** — seen once in five in a Safari demo (`seldom`) before
  `reconcile.ts`. → Release gate: the corpus evaluation (task 1.2) is run on the shipping artefact
  and its result recorded; `reconcile.ts` withholds a mark it cannot confirm rather than show a
  wrong one.
- **[The model's licence]** → the licence of `mozilla/firefox-translations-models` and its
  attribution terms must be confirmed before Cymbra redistributes the files (task 1.1); the
  attribution goes where the setting is described.
- **[Adding `offscreen` disables the extension on update]** → it carries no install warning as far
  as is known; verified on an update from the published version before release (task 7.3).
- **[The browser evicts the model]** → reported, not silently re-fetched; `persist()` requested.
- **[Package size]** → +~1.1 MB brotli; Firefox for Android carries it unused until its change.
- **[Bandwidth]** → 25.75 MB per device that ticks the box, from a static host.
- **[Two changes edit the setting]** → `add-lingua-remote-translation` archives after this one and
  should rewrite its **The reader chooses where the engine runs** as a modification of **Extended
  translation is chosen per device, off by default**.

## Migration Plan

No reader state changes shape: an absent `translationHost` is `"none"`. Development builds that
side-loaded the model keep working through the override. Rollback is a release that hides the
setting and deletes the `lingua-model` database on start, so no reader keeps 37 MB they can no
longer remove.

## Open Questions

- **Which origin serves the model?** Candidates: the site's static deployment (`cymbra.app`,
  already on Cloudflare, but model files are never committed, so its build would have to fetch
  them), or a dedicated static bucket (for example `models.cymbra.app`) filled by a workflow from
  Mozilla's LFS sources. Recommendation: a dedicated origin, so the site's deploys and its
  200-for-everything fallback never touch the model.
- The mark evaluation's threshold for release — to be set by the product owner from the result.
- Whether `unlimitedStorage` is needed on either browser for 37 MB, or the default quota suffices.
- Ten idle minutes (D6) — to be checked against real reading.
