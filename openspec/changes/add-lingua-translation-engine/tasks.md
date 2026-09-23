## 1. Build the engine in CI

- [x] 1.1 Add a `lingua-engine-build` workflow that checks out `mozilla/translations` at the pinned commit with submodules, runs `inference/scripts/build-wasm.py` with `ALLOW_RUN_ON_HOST=1`, and uploads `bergamot-translator.js` and `bergamot-translator.wasm` as an artefact
- [x] 1.2 State the pinned commit in the workflow itself, next to a line saying why it is pinned, so a reader is never left inferring which engine a build holds
- [x] 1.3 Fail the job if either artefact is missing or its size leaves the range the design records, so a silently-empty build cannot pass
- [x] 1.4 Add the workflow to `scripts/check_ci_units.py` expectations if it introduces a unit no workflow watches, and run `python3 scripts/check_ci_units.py --list` to confirm — it introduces no unit; the guard passes unchanged
- [x] 1.5 Record in the workflow that this build must not run on macOS AArch64, with the upstream warning as the reason

## 2. The seam

- [x] 2.1 Define `TranslatorPort` in `apps/lingua-extension/src/translate/port.ts`: one call taking the sentence and the selection's offsets within it, returning the translated sentence plus the marked spans — no language pair: one engine loads one model, and a pair the model cannot serve is not a request anyone can make
- [x] 2.2 Define the answer type so the marked span is addressable without re-reading the source, and so the caller can render the sentence alone
- [x] 2.3 Write the factory so that no implementation running in the caller's own context exists — a content script or extension page can only obtain the messaging implementation
- [x] 2.4 Prove the in-context implementation is unreachable: `test/lint-translator-placement.spec.ts` walks the import graph from every entry point that paints and fails if one reaches `translate/host/`, constructs a worker, or loads the glue — with a positive control that the walk does reach the host from the background, and verified by mutation

## 3. The engine host

- [x] 3.1 Write `engine-worker.ts` as a **classic** worker that loads the pristine glue with `importScripts`, keeping sloppy mode so no patch to Mozilla's artefact is needed
- [x] 3.2 Implement load and translate in that worker, mirroring `translations-engine.worker.mjs`: alignments model 256 / lex 64 / vocab 64, the marian config the design records, `INITIAL_MEMORY` at Mozilla's value, and the model bytes handed over as `wasmBinary`
- [x] 3.3 Bound the init: if the engine has not signalled readiness within the bound, settle the request as failed — an unsettled init is the failure mode that looks like slowness
- [x] 3.4 On Chromium, create the offscreen document with `reasons: ["WORKERS"]` and a justification naming the thread rule; give it no work beyond owning the worker and relaying
- [x] 3.5 On Firefox, construct the worker from the event page and build no offscreen document into that variant; assert the split in `check:variants`
- [x] 3.6 Add the `offscreen` permission to the Chromium manifest only, and confirm `check:variants` fails if it reaches another variant

## 4. The request

- [x] 4.1 Resolve the sentence AND the selection's offsets in it by position — `sentenceAndSelection`, which `sentenceForRange` now delegates to. (The design assumed `sentenceForRange` already returned the offsets; it computed them and threw them away.)
- [x] 4.2 Escape the sentence before placing the selection's markup in it, and unit-test a sentence containing markup characters end to end
- [x] 4.3 Request the translation with `html: true` and read the marked span back out of the answer
- [x] 4.4 Unit-test the mapping on the measured cases: a fragment conjugated by its context, several source words collapsing to one target word, and a sentence the engine reorders

## 5. Development-only model loading

- [x] 5.1 Add a development path that reads the model, lexical shortlist and vocabulary from a local directory, behind a build flag that is off in every shipped variant — `LINGUA_TRANSLATION_ENGINE=<dir>`, which sets `__TRANSLATION_HOST__`
- [x] 5.2 Document where to place those files and where they come from, without adding any fetch — `apps/lingua-extension/TRANSLATION.md`, whose model download was run exactly as written and verifies each file's sha256
- [x] 5.3 Assert in `check:variants` that no shipped bundle references the model paths or the engine artefact

## 6. The card shows the translation (added during implementation)

The proposal had no task that displayed anything, which left the development build showing
nothing and 7.4 without an object.

- [x] 6.1 Carry the selection's offsets from the capture to the card (`Capture.selection`, `SelectionInput.selection`)
- [x] 6.2 Ask the pack and the engine together for a selection of several words, neither able to take the other down; bound the engine's wait (`TRANSLATION_WAIT_MS`, 2.5 s) below the card's own timeout, so a slow or cold engine never costs the reader the pack's answer
- [x] 6.3 Render « Dans votre phrase — traduction automatique », the translated sentence with the selection's place in the answer colour, from text nodes only — the sentence came from the page, so as markup it could carry anything
- [x] 6.4 With a translation, show no word-by-word rows and no "the pack has no translation" note above it; keep an expression's dictionary gloss
- [x] 6.5 Keep a single word on its dictionary card: only a selection of several words is translated
- [x] 6.6 Carry the translation in no gesture, so no path can store it on a card

## 7. Prove the requirements

- [x] 7.1 Measure, with the real model in a loaded extension, that the page's frame pacing during a translation is indistinguishable from idle, with a control that blocks the page so the instrument is known to detect a freeze
- [x] 7.2 Measure that the background context keeps answering during a translation, and record the worst latency
- [x] 7.3 Verify with no model present that every surface answers exactly as before, and that nothing reports a missing engine to the reader
- [x] 7.4 Verify that a card created from a translated selection holds no machine translation
- [x] 7.5 Record all measurements in the change, replacing any estimate carried from the roadmap

## 7b. The mark is checked (added during device testing)

The tag's position is the engine's alignment, and on a real page it put `seldom` on the verb.

- [x] 7b.1 Ask the engine for the selection alone next to the tagged sentence, from the relay, as a check that is never a dependency: when it fails, the tag's marks stand
- [x] 7b.2 Check the marks against it in a pure module, `translate/reconcile.ts`: move a mark the lone translation places elsewhere, split or trim it word by word, never invent one and never remove every one
- [x] 7b.3 Unit-test it on the engine's real answers — the misplaced tag, the separated words, the inflected form, the elided clitic, the grammar's short words, a synonym, the stray article
- [x] 7b.4 Measure it on 100 sentences with the real engine, judged by hand, and record the result in the design

## 7c. The engine is kept loaded between selections (found in device testing)

A Galaxy Tab S6 Lite pass showed the engine reloading between selections and losing the race.

- [x] 7c.1 Hold a port open from the reader for as long as it is on the page, so the context hosting the engine is not torn down (`translate/keepalive.ts`); accept it in the background, and send nothing over it
- [x] 7c.2 Warm nothing eagerly: the engine still loads on the first translation, so a page where nothing is selected never pays for it
- [x] 7c.3 Reopen the port if the host goes away anyway, and stop when the extension context is gone — an orphaned page has nothing to hold
- [x] 7c.4 Gate it exactly as the port is gated, as a folded expression, and confirm `check:variants` finds no trace of it in any shipped bundle
- [x] 7c.5 Unit-test it, and re-measure on the device that showed the problem

## 8. Gates

- [x] 8.1 `yarn lint`, `yarn format:check`, `yarn typecheck` clean from `apps/lingua-extension`
- [x] 8.2 `yarn test` passes and keeps line coverage at or above the extension's gate
- [x] 8.3 `yarn build` and `yarn check:variants` pass for all three variants, and `check:variants --engine` passes on a build made with the engine
- [x] 8.4 `openspec validate add-lingua-translation-engine --strict` passes
