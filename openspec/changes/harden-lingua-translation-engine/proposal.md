# harden-lingua-translation-engine — an engine that survives a trap, and holds two models

## Why

Change 9 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the sixth of stage 1 and a silent release. The programme's risk 3: the en-es model traps the
engine. Measured in the study (2026-10-06), some inputs through the en-es model raise a
WebAssembly trap — "memory access out of bounds" — in the pinned engine. The trap poisons every
model built after it: the worker stays alive, and so do its models, and every translation after it
fails until the worker is put down. Today the worker's catch turns every error into a reply the
channel passes on without a reset (`channel.ts`, "passes on what the engine said when it could
not translate"), so the engine stays dead for up to ten minutes, the idle release. The study
measured the fix: put the worker down and ask once more.

The study measured a second thing. The worker never deletes a model: a reader who moves between
languages keeps three or four models in the worker, 463 MiB, where one pair needs at most two
(321.8 MiB measured with es-fr's pivot, #709). The device prunes its stored models when the
reader's languages change (model-state D3); the engine is never told.

Both are needed before en-es ships (changes 22, 25, 35) and are harmless for a reader of French:
en-fr and es-fr do not trap on the measured corpus, and a French reader of English and Spanish
holds two models, en-fr shared.

Decision M25 is settled here: the real engine does not run in CI; a per-route soak is a manual
tool, beside the marks harness.

## What Changes

- **A trap is recognised and reported as one.** The worker tells a WebAssembly trap (a
  `WebAssembly.RuntimeError`, or the glue's abort) from an ordinary refusal (no model, a route
  too long, not loaded), reports it as a trap and closes itself: a trapped instance's memory is
  not to be trusted.
- **The channel respawns and asks once more.** On a trap, the channel puts the worker down, starts
  a fresh one, loads the route again and sends the request a second time. A request that traps
  twice is unavailable. The requests in flight on the worker that trapped are asked again on the
  fresh one too, once. An ordinary refusal is passed on as today, with no reset.
- **The engine holds at most two models.** Loading a route whose models would make a third
  deletes first the models no loaded route needs, least recently used first; the decision is a
  pure module, tested on its own, and the worker applies it. A French reader of English and
  Spanish keeps en-fr and es-en, and nothing is deleted.
- **A soak, by hand** (M25): `tool/soak_engine.mjs --pair <pair>` runs the real engine through a
  pair's route over the committed corpus of its studied language, in Node, and reports the inputs
  that trap and the memory high-water mark. It is how en-es is tried before change 35.
- **Docs follow**: `TRANSLATION.md` says what a trap does and what the engine holds; the
  programme marks M25 settled.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: ADDED *The engine survives a trap*, *The engine holds at most two
  models*, *A route can be soaked by hand*. No requirement is modified. *The engine is not torn
  down between a reader's selections* stands: a trap is the one new reason to put it down, and
  the reader's request is answered by the fresh one.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`src/translate/host/channel.ts`,
  `engine-worker.ts`, `engine.ts`, a new `model-residency.ts`, `tool/soak_engine.mjs`,
  `TRANSLATION.md`). ID, Music, Live, the back office and the site are untouched.
- **No model, host, pack or byte moves.** The catalogue, the engine pin, the packs and the French
  copy are untouched. The three hosts keep their shapes: the respawn lives in the channel, which
  the offscreen document and the event page both own.
- **After change 8.** The channel and the worker are keyed by pair there; this change is written
  against that shape and is rebased after it.
- **Not here.** The en-es model and its deployment (change 25), its marks (change 26), its enable
  (change 35); a trap's telemetry beyond the console.
