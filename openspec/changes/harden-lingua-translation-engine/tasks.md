# Tasks

## 1. A trap, told from a refusal (apps/lingua-extension)

- [x] 1.1 `src/translate/host/engine.ts`: `WorkerResponse` gains `trap?: true`; a pure `isTrap(error)` (a `WebAssembly.RuntimeError`, or a message starting with `Aborted(`) in a host-testable module, with tests (D1).
- [x] 1.2 `engine-worker.ts`: the catch reports a trap with the flag, once, with the request's id, and closes the worker; `onAbort` stays as it is (D1).

## 2. The respawn (apps/lingua-extension)

- [x] 2.1 `channel.ts`: a reply flagged `trap` resets the worker and replays the request once on a fresh one — load, then the request; the requests in flight on the trapped worker are replayed once each; a second trap answers "the engine trapped twice"; an optional `log` names the pair and the markup's length (D2, D5).
- [x] 2.2 `test/translate-channel.spec.ts`, with `FakeWorker` answering a trap: each scenario of *The engine survives a trap* — a trap in a translation, while loading, a second trap, two requests in flight, and a refusal that is no trap (the existing test kept).

## 3. Residency (apps/lingua-extension)

- [x] 3.1 `src/translate/host/model-residency.ts`: loaded routes in use order, a load and a translation both uses; for a route to load, the model ids to delete among those it does not need, least recently used first, to a bound of two, and the routes dropped with them (D3). `test/model-residency.spec.ts`: each scenario of *The engine holds at most two models* (*Two loads at once* as a sequence of two decisions), the shared-model case, a route of two models, a route already held.
- [x] 3.2 `engine-worker.ts` keeps each model's aligned-memory handles, runs loads one at a time, applies the decision before building, deletes the evicted `TranslationModel`s and their memory, drops their routes, and records a translation as a use (D3).

## 4. The soak tool (apps/lingua-extension)

- [x] 4.1 `tool/marks/engine.mjs`: the engine loader moved out of `measure_marks.mjs` (which keeps its results byte for byte); `tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]`: the report (count, trapped ids, time per sentence, RSS high-water mark), the run stopping at the first trap unless `--isolate` (D4). `tool/marks/README.md` documents it.
- [ ] 4.2 Run by hand for en-fr and es-fr on the committed corpus; record the figures in the pull request (no trap; RSS with two models).

## 5. Docs, gates and spec

- [x] 5.1 `TRANSLATION.md`: what a trap does, the two-model bound, the soak tool; `docs/lingua/language-matrix-programme.md`: M25's recommendation carried by this change (the owner settles it), change 9 done.
- [x] 5.2 Gates, in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `openspec validate harden-lingua-translation-engine --strict` passes, and `python3 scripts/openspec_archive_order.py harden-lingua-translation-engine` exits 0.
