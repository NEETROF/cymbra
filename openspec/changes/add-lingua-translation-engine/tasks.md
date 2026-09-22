## 1. Build the engine in CI

- [ ] 1.1 Add a `lingua-engine-build` workflow that checks out `mozilla/translations` at the pinned commit with submodules, runs `inference/scripts/build-wasm.py` with `ALLOW_RUN_ON_HOST=1`, and uploads `bergamot-translator.js` and `bergamot-translator.wasm` as an artefact
- [ ] 1.2 State the pinned commit in the workflow itself, next to a line saying why it is pinned, so a reader is never left inferring which engine a build holds
- [ ] 1.3 Fail the job if either artefact is missing or its size leaves the range the design records, so a silently-empty build cannot pass
- [ ] 1.4 Add the workflow to `scripts/check_ci_units.py` expectations if it introduces a unit no workflow watches, and run `python3 scripts/check_ci_units.py --list` to confirm
- [ ] 1.5 Record in the workflow that this build must not run on macOS AArch64, with the upstream warning as the reason

## 2. The seam

- [ ] 2.1 Define `TranslatorPort` in `apps/lingua-extension/src/translate/port.ts`: one call taking the sentence, the selection's offsets within it, and the language pair, returning the translated sentence plus the marked span
- [ ] 2.2 Define the answer type so the marked span is addressable without re-reading the source, and so the caller can render the sentence alone
- [ ] 2.3 Write the factory so that no implementation running in the caller's own context exists — a content script or extension page can only obtain the messaging implementation
- [ ] 2.4 Unit-test the factory per target, asserting that the in-context implementation is unreachable from a content script and from an extension page

## 3. The engine host

- [ ] 3.1 Write `engine-worker.js` as a **classic** worker that loads the pristine glue with `importScripts`, keeping sloppy mode so no patch to Mozilla's artefact is needed
- [ ] 3.2 Implement load and translate in that worker, mirroring `translations-engine.worker.mjs`: alignments model 256 / lex 64 / vocab 64, the marian config the design records, `INITIAL_MEMORY` at Mozilla's value, and the model bytes handed over as `wasmBinary`
- [ ] 3.3 Bound the init: if the engine has not signalled readiness within the bound, settle the request as failed — an unsettled init is the failure mode that looks like slowness
- [ ] 3.4 On Chromium, create the offscreen document with `reasons: ["WORKERS"]` and a justification naming the thread rule; give it no work beyond owning the worker and relaying
- [ ] 3.5 On Firefox, construct the worker from the event page and build no offscreen document into that variant; assert the split in `check:variants`
- [ ] 3.6 Add the `offscreen` permission to the Chromium manifest only, and confirm `check:variants` fails if it reaches another variant

## 4. The request

- [ ] 4.1 Take the sentence and the selection's offsets from `sentenceForRange`, which already resolves both by position
- [ ] 4.2 Escape the sentence before placing the selection's markup in it, and unit-test a sentence containing markup characters end to end
- [ ] 4.3 Request the translation with `html: true` and read the marked span back out of the answer
- [ ] 4.4 Unit-test the mapping on the measured cases: a fragment conjugated by its context, several source words collapsing to one target word, and a sentence the engine reorders

## 5. Development-only model loading

- [ ] 5.1 Add a development path that reads the model, lexical shortlist and vocabulary from a local directory, behind a build flag that is off in every shipped variant
- [ ] 5.2 Document in `apps/lingua-extension/REVIEWERS.md` or the development notes where to place those files and where they come from, without adding any fetch
- [ ] 5.3 Assert in `check:variants` that no shipped bundle references the model paths or the engine artefact

## 6. Prove the requirements

- [ ] 6.1 Measure, with the real model in a loaded extension, that the page's frame pacing during a translation is indistinguishable from idle, with a control that blocks the page so the instrument is known to detect a freeze
- [ ] 6.2 Measure that the background context keeps answering during a translation, and record the worst latency
- [ ] 6.3 Verify with no model present that every surface answers exactly as before, and that nothing reports a missing engine to the reader
- [ ] 6.4 Verify that a card created from a translated selection holds no machine translation
- [ ] 6.5 Record all measurements in the change, replacing any estimate carried from the roadmap

## 7. Gates

- [ ] 7.1 `yarn lint`, `yarn format:check`, `yarn typecheck` clean from `apps/lingua-extension`
- [ ] 7.2 `yarn test` passes and keeps line coverage at or above the extension's gate
- [ ] 7.3 `yarn build` and `yarn check:variants` pass for all three variants
- [ ] 7.4 `openspec validate add-lingua-translation-engine --strict` passes
