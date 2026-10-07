# Tasks

## 1. A trap, told from a refusal (apps/lingua-extension)

- [ ] 1.1 `src/translate/host/engine.ts`: `WorkerResponse` gains `trap?: true`; a pure `isTrap(error)` (a `WebAssembly.RuntimeError`, or a message starting with `Aborted(`) in a host-testable module, with tests (D1).
- [ ] 1.2 `engine-worker.ts`: the catch reports a trap with the flag and closes the worker; `onAbort` reports for the instance's whole life (D1).

## 2. The respawn (apps/lingua-extension)

- [ ] 2.1 `channel.ts`: a reply flagged `trap` resets the worker and replays the request once on a fresh one — load, then the request; the requests in flight on the trapped worker are replayed once each; a second trap answers "the engine trapped twice" (D2). The log names the pair and the markup's length.
- [ ] 2.2 `test/translate-channel.spec.ts`, with `FakeWorker` answering a trap: each scenario of *The engine survives a trap* — a trap in a translation, while loading, a second trap, two requests in flight, and a refusal that is no trap (the existing test kept).

## 3. Residency (apps/lingua-extension)

- [ ] 3.1 `src/translate/host/model-residency.ts`: loaded routes in use order; for a route to load, the model ids to delete, least recently used first, to a bound of two (D3). `test/model-residency.spec.ts`: each scenario of *The engine holds at most two models*, the shared-model case, a route of two models, a route already held.
- [ ] 3.2 `engine-worker.ts` applies it before building: deletes the evicted `TranslationModel`s and their aligned memory, drops their routes.

## 4. The soak tool (apps/lingua-extension)

- [ ] 4.1 `tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]`: the loader shared with `measure_marks.mjs`; the report (count, trapped ids, time per sentence, RSS high-water mark) (D4). `tool/marks/README.md` or a sibling README documents it.
- [ ] 4.2 Run by hand for en-fr and es-fr on the committed corpus; record the figures in the pull request (no trap; RSS with two models).

## 5. Docs, gates and spec

- [ ] 5.1 `TRANSLATION.md`: what a trap does, the two-model bound, the soak tool; `docs/lingua/language-matrix-programme.md`: M25 settled, change 9 done.
- [ ] 5.2 Gates, in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `openspec validate harden-lingua-translation-engine --strict` passes, and `python3 scripts/openspec_archive_order.py harden-lingua-translation-engine` exits 0.
