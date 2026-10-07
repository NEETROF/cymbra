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
starts with `Aborted(`) — and two more throws after which the wasm stack was unwound by nothing:
a C++ exception the glue rethrows as the bare pointer it is (`throw ptr`, a number), and the wasm
stack blown (a `RangeError` whose message names the call stack in V8 and JavaScriptCore, an
`InternalError` in SpiderMonkey). The worker's catch reports it as `{ok: false, error, trap: true}`
(`WorkerResponse` gains the optional flag) and then calls `self.close()`: a trapped instance's
linear memory is not to be trusted for a next request, and a worker that closes itself cannot be
asked by mistake. It reports once — a `closed` flag — and frees none of the poisoned instance's
handles on the way out, so that a second error cannot replace the trap's report. Every other error
— `NO_MODEL`, `LONG_ROUTE`, `NOT_LOADED`, a missing file, a `RangeError` over an array's length —
stays an ordinary refusal, reported as today.

`onAbort` stays wired as it is, for the initialisation: an abort raised during a translation
throws a `RuntimeError` whose message starts with `Aborted(` right after the callback, and the
catch reports that throw as the trap — once, with the request's id.

Alternative: catch nothing new and rely on the translate bound. A trap answers at once, so the
bound never fires; the worker stays poisoned until idle.

### D2 — The channel respawns and asks once more

On a reply flagged `trap`, the channel resets — terminates the worker, clears `loads` — and
replays the request: a fresh worker, `load` for the pair, then the request again. A request is
replayed once; a second trap answers `{ok: false, reason: "the engine trapped twice"}`. The
requests in flight on the worker that trapped are not failed as a crash is: each is replayed
once on the fresh worker as well, so a marked selection's two requests (relay.ts) both get their
answer from the fresh instance. A `warm` that traps while loading is replayed the same way.

The channel takes an optional `log` (as `relayTranslation` does) and writes "trapped, asked
again" or "trapped twice", with the pair and the markup's length, never its text (D5). The relay
is unchanged: a replay that succeeds is an ordinary answer to it.

The existing behaviour for an ordinary refusal is kept, and so is its test ("passes on what the
engine said when it could not translate").

Alternative: replay in `relay.ts`. The relay does not know what a trap is, and the two hosts
would each need it; the channel is the one place both own.

### D3 — Residency: at most two models, least recently used first, one load at a time

A pure module, `src/translate/host/model-residency.ts`, keeps the loaded routes in the order
they were last used — a load and a translation are both uses — and answers, for a route about
to be loaded, which model ids to delete: among the models the route does not need, the least
recently used first, until the models held plus the route's make two at most; the routes that
used an evicted model are dropped with it. A route whose models are already held costs nothing.
A route of two models evicts everything else. A route dropped with a deleted model leaves its
other model with no route: that model is deleted too, unless the new route needs it — kept, it
would be held and counted by nothing, and a third would fit beside it. The decision is told the
models the worker actually holds, not the routes', so a model outside every route is found
whatever left it there, and what the worker holds is the union of its routes after every load
and after every failed one. The worker applies the answer before building: it deletes each
evicted `TranslationModel` and the aligned memory it was built from (the handles are kept beside
the model for that), and drops the routes.

The worker runs loads one at a time: a load waits for the previous one to finish before it asks
the residency module, so two loads cannot each decide against a bound the other is about to
move, and no route is registered over a model the other load deleted. A translation waits for
the load of its route, and only for it: over a route the worker holds, it goes to the engine at
once, without the chain (nothing yields between the test and the engine's call, so the route
cannot go in between); over a route the worker deleted since the channel loaded it, the worker
answers `reload`, and the channel forgets the pair, loads the route again under the start bound
— a pivot's rebuild on a tablet can pass the translate bound — and asks once more, once.

This is the Architecture's "evicts models no current pair needs", with the current pair being
the route about to be loaded and the bound being how it is counted: a model is deleted only when
keeping it would hold three.

Why two: a route chains two models at most (`LONG_ROUTE`), so two is the bound one pair needs,
and it is what a reader of French with English and Spanish holds (en-fr, es-en). A larger bound
buys nothing for the matrix's natives: an English reader of Spanish and French holds es-en and
fr-en; a Spanish reader of English and French holds en-es and fr-en, en-es shared by the pivot.

Alternative: delete the models of a pair when the device prunes its files. The worker would
need a new op and the controller a new call; and the common case — a reader who alternates two
languages — is a bound, not a prune.

### D4 — The soak is a tool, not a test (M25)

`tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]` loads the pinned
engine and the pair's route in Node through a loader shared with `measure_marks.mjs` (its private
`engine()` moves to `tool/marks/engine.mjs`), runs every selection's sentence of the committed
corpus of the pair's studied language, tagged as the extension tags it, and reports: the count
translated, each input that trapped (its corpus id, never its text in the summary line), the wall
time per sentence, and the process's resident high-water mark (`process.resourceUsage().maxRSS`
— Node's, not the worker's: the ≈ 322 MiB of es-fr was measured in the browser, and the soak says
nothing of the two-model bound, since a run loads one route and deletes nothing). In Node a trap
throws a `RuntimeError` the tool catches; the instance is poisoned from then on, so without
`--isolate` the run stops at the first trap and says so; with `--isolate`, each sentence runs in
a child process and the run goes on to the end — a child that ends without its report (a trap
while the route is built, a crash, a signal) counts as trapped for its id, one past 120 s is
counted apart as timed out. `--pair en-es` needs the en-es route pinned in the catalogue (change
25). Its README is `tool/marks/README.md`'s neighbour.

Not in CI: a run costs ≈ 100 MB of models and about a minute; the programme's M25 says a manual
tool.

### D5 — The log says what happened, and no more

A trap and a replay are logged by the channel (`console.warn`, as the relay logs a refusal): the
pair, "trapped, asked again" or "trapped twice", and the markup's length. The sentence is not
logged.

## Risks / Trade-offs

- **A replay loop** → one replay per request, counted on the request, not on the worker; a
  second trap is final.
- **Two requests replayed on one fresh worker, both trapping** → each is final after its own
  replay; the worker is put down once per trap, never left poisoned.
- **Two loads deciding at once** → loads are serialised in the worker (D3).
- **Eviction thrash for a reader who alternates three languages** → the bound is two models; a
  third language's route costs its load (≈ 200 ms on a Mac, 3–4 s on the measured tablet) each
  time it comes back. Accepted: the study found no reader with three studied languages, and the
  memory is the measured problem.
- **A `delete()` that the glue does not free** → not something RSS can show: a wasm instance's
  linear memory never shrinks, and the soak tool loads one route and evicts nothing. The bound
  caps growth — a freed block is reused by the next model built — and whether it is reused is
  read in the worker, by the working set once a route has been loaded over an evicted one; if it
  does not hold, the fallback is a reset when the bound is exceeded (the worker put down, as the
  idle release does), which the residency module can answer with "all".
- **The worker's own test** → `engine-worker.ts` stays excluded from unit coverage (it needs the
  real wasm); the trap classification and the residency decision are pure functions, tested.

## Migration Plan

One release, silent. No dispatch, no model, no stored state. `WorkerResponse.trap` is optional:
a channel built with it reads a worker built without it as before.
