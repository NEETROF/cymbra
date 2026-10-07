# Design — harden-lingua-translation-engine

## Context

See proposal.md (Why). What the code does today, from the study's reading (2026-10-06):

| Where | What |
|---|---|
| `engine-worker.ts` `scope.onmessage` | every throw becomes `{ok: false, error}`; the worker and its `models` survive |
| `engine-worker.ts` `instance()` | `onAbort` is wired for initialisation only |
| `channel.ts` `translateNow` | an `ok: false` reply is passed on, with no reset; a reset happens on a translate timeout (10 s), `onerror`, idle, `shutDown`, or a failed load leaving no route |
| `channel.ts` `loads` | one promise per route, forgotten on failure |
| `engine-worker.ts` `models`, `routes` | only grow; no unload op (`engine.ts` `WorkerRequest`) |
| `relay.ts` | a marked selection is two requests in parallel on the one worker |
| `model-db.ts` `prune`, `model-controller.ts` | the device's stored models are pruned; the engine is never told |
| `tool/measure_marks.mjs` | the real engine in Node `vm`, the worker's constants copied; no soak |

Measured: the working set is 195.4 MiB with one model and 321.8 MiB with es-fr's two (#709); with
models of a previous language lingering, 463 MiB (study). A trap through en-es poisons every
model built after it until the worker is put down; putting it down and asking once more answers
the request (study).

## Goals / Non-Goals

**Goals:**
- A trap costs the reader one respawn, not ten minutes.
- The worker holds at most two models, the bound one pair needs.
- en-es can be tried, by hand, before it ships (M25).
- Nothing a reader of French sees moves.

**Non-Goals:**
- Finding why en-es traps, or a fix in the engine or the model: it is Mozilla's pinned engine; a
  respawn is the measured mitigation.
- Running the real engine in CI (M25: no).
- Telemetry of traps beyond the console log.
- Changing when the idle release happens, or the start and translate bounds.

## Decisions

### D1 — The worker tells a trap from a refusal, and closes itself

A trap is a `WebAssembly.RuntimeError`, or an error the glue raises from `abort()` (its message
starts with `Aborted(`). The worker's catch reports it as `{ok: false, error, trap: true}`
(`WorkerResponse` gains the optional flag) and then calls `self.close()`: a trapped instance's
linear memory is not to be trusted for a next request, and a worker that closes itself cannot be
asked by mistake. Every other error — `NO_MODEL`, `LONG_ROUTE`, "the engine is not loaded", a
missing file — stays an ordinary refusal, reported as today.

`onAbort` is wired to the same reporting for the whole life of the instance, not only its
initialisation: an abort raised during a translation is a trap too.

Alternative: catch nothing new and rely on the translate bound. A trap answers at once, so the
bound never fires; the worker stays poisoned until idle.

### D2 — The channel respawns and asks once more

On a reply flagged `trap`, the channel resets — terminates the worker, clears `loads` — and
replays the request: a fresh worker, `load` for the pair, then the request again. A request is
replayed once; a second trap answers `{ok: false, reason: "the engine trapped twice"}`, logged with
the pair and the markup's length (never its text). The requests in flight on the worker that
trapped are not failed as a crash is: each is replayed once on the fresh worker as well, so a
marked selection's two requests (relay.ts) both get their answer from the fresh instance. A
`warm` that traps while loading is replayed the same way.

The existing behaviour for an ordinary refusal is kept, and so is its test ("passes on what the
engine said when it could not translate").

Alternative: replay in `relay.ts`. The relay does not know what a trap is, and the two hosts
would each need it; the channel is the one place both own.

### D3 — Residency: at most two models, least recently used first

A pure module, `src/translate/host/model-residency.ts`, keeps the loaded routes in use order and
answers, for a route about to be loaded, which model ids to delete: those the route does not need,
least recently used first, until the models held plus the route's make two at most. A route
whose models are already held costs nothing. A route of two models evicts everything else. The
worker applies the answer before building: it deletes each evicted `TranslationModel` and the
aligned memory it was built from, and drops the routes that used it.

Why two: a route chains two models at most (`LONG_ROUTE`), so two is the bound one pair needs,
and it is what a reader of French with English and Spanish holds (en-fr, es-en). A larger bound
buys nothing for the matrix's natives: an English reader of Spanish and French holds es-en and
fr-en; a Spanish reader of English and French holds en-es and fr-en, en-es shared by the pivot.

Alternative: delete the models of a pair when the device prunes its files. The worker would
need a new op and the controller a new call; and the common case — a reader who alternates two
languages — is a bound, not a prune.

### D4 — The soak is a tool, not a test (M25)

`tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N]` loads the pinned engine and the
pair's route in Node as `measure_marks.mjs` does (the loader is shared), runs every selection's
sentence of the committed corpus of the pair's studied language, tagged as the extension tags it,
and reports: the count translated, each input that trapped (its corpus id, never its text in the
summary line), the wall time per sentence, and `process.memoryUsage().rss` at its highest. A
trap in Node is fatal to the process, as it is to the worker: the tool runs each sentence in a
child process when `--isolate` is given, so a trapping input does not end the run. Its README is
`tool/marks/README.md`'s neighbour.

Not in CI: a run costs ≈ 100 MB of models and about a minute; the programme's M25 says a manual
tool.

### D5 — The log says what happened, and no more

A trap and a replay are logged through the relay's log (`console.warn`, as every other refusal):
the pair, "trapped, asked again" or "trapped twice", and the markup's length. The sentence is not
logged.

## Risks / Trade-offs

- **A replay loop** → one replay per request, counted on the request, not on the worker; a
  second trap is final.
- **Two requests replayed on one fresh worker, both trapping** → each is final after its own
  replay; the worker is put down once per trap, never left poisoned.
- **Eviction thrash for a reader who alternates three languages** → the bound is two models; a
  third language's route costs its load (≈ 200 ms on a Mac, 3–4 s on the measured tablet) each
  time it comes back. Accepted: the study found no reader with three studied languages, and the
  memory is the measured problem.
- **A `delete()` that the glue does not free** → measured by the soak tool's RSS before and after
  an eviction; if the memory does not come back, the fallback is a reset when the bound is
  exceeded (the worker put down, as the idle release does), which the residency module can
  answer with "all".
- **The worker's own test** → `engine-worker.ts` stays excluded from unit coverage (it needs the
  real wasm); the trap classification and the residency decision are pure functions, tested.

## Migration Plan

One release, silent. No dispatch, no model, no stored state. `WorkerResponse.trap` is optional:
a channel built with it reads a worker built without it as before.
