# Design — package-lingua-packs-per-pair

## Context

See proposal.md (Why). Since the three changes before this one:

| Where | What it does today |
|---|---|
| `crates/lingua-wasm` | `new LinguaEngine(bytes)` loads a first pack, whose language becomes the default. `addPack(bytes)` loads another language's pack and refuses a language already held (#626). A language-bound binding refuses a language with no pack. `applyStatusChanges`, `applyCardOps` and `applyDeclaredLevelChanges` skip a record whose language has no pack. `restore` keeps such a language's state as it is. |
| `src/analyzer/engine.ts` | `WasmAnalyzerPort` builds the engine on first use with `PACK_PATH`, the first pair of `__LINGUA_PACKS__`. `for(language)` returns a `WasmLanguagePort` that hands the language to every call (#628). |
| `build.mjs` | Every listed pack is copied, checked against its own language and analyser, and made web-accessible. `__LINGUA_PACKS__` is the list, comma-separated (#630). |
| `src/background.ts` | Two `WasmAnalyzerPort`s on the static glue. One is the reading engine behind the RPC (always on Firefox and Safari; on Chromium, for pages whose policy blocks WASM). The other is the sync engine. |
| `src/sync/sync.ts` | A sync restores the latest backup and applies the pulled records. Then it saves the backup, and only then moves its cursors. |

The pack list holds en-fr alone, and `check_variants` refuses any other list until
`enable-lingua-spanish`.

## Goals / Non-Goals

**Goals:**
- An engine holds the packs of the languages it is asked about, each loaded once, on first need.
- A pulled record of a language the package ships is never skipped for want of its pack.
- An explicit refusal for a language the package does not ship.
- English untouched: same pack, same moment, nothing else fetched.

**Non-Goals:**
- Unloading a pack. The engine has no `removePack`, and whether a second pack's memory matters on a
  phone is measured in R3 before anything is built.
- Choosing which pack an engine starts with. It stays the default pair's; starting with the
  reader's language belongs with the profile that records it (`add-lingua-studied-language-profile`).
- A sync engine with no pack, lazy pack decoding, and the translation engine's mobile policy: the
  study's other memory levers.
- Listing Spanish. That is `enable-lingua-spanish`, which widens the gate.

## Decisions

### D1 — The port that holds the engine loads its packs

`WasmAnalyzerPort` is the only code that fetches a pack today, and the only one that can hand
bytes to its engine. Loading happens there:
- The messaging port already sends each call's language (#628).
- The background's reading engine is a `WasmAnalyzerPort`, so Firefox, Safari and the Chromium
  fallback load the same way as Chromium's in-page engine.
- `handleRpc` and `MessagingLinguaPort` do not change.

Alternatives:
- **Load every listed pack at start.** Simplest, but each engine would pay about 16 MiB per
  listed language for a language the reader may never open: on Chromium, every page; in the
  background, twice.
- **Keep one pack and swap it when the language changes.** That needs `removePack`, and a mixed
  page or a sync batch would reload packs back and forth. Deferred until R3 measures a need.

### D2 — One mapping from a language to its pair

`src/analyzer/pairs.ts` holds three things:
- `SHIPPED_PAIRS`: `__LINGUA_PACKS__` split on commas.
- `packPath(pair)`: `assets/packs/<pair>.lingua`.
- `pairFor(language, pairs)`: the first listed pair whose studied side is the language, or
  `null`.

The engine holds one pack per studied language, so the first matching pair is the only one that
could be loaded. `tool/packs.mjs` cannot be imported into a bundle (it reads files with
`node:fs`), so a spec holds `packPath` equal to its `packFile`, and `SHIPPED_PAIRS` equal to
`shippedPairs()`.

The port takes the list as a constructor parameter, defaulting to `SHIPPED_PAIRS`
(`new WasmAnalyzerPort(glue, pairs?)`). Its first pair's pack builds the engine, as `PACK_PATH`
did. Specs pass `["en-fr", "es-fr"]` without touching the bundle's list.

### D3 — Once per engine, shared while pending, forgotten on failure

The port keeps a `Map<StudiedLanguage, Promise<void>>` of pack loads. The first call in a
language starts the fetch and the `addPack`, and concurrent calls wait on the same promise. A
rejected load leaves the map, so the next call retries: the pattern `initialise` uses for the glue.
The default language never enters the map, since its pack built the engine.

`for(language)` stays synchronous and never throws: surfaces call it freely, some from getters.
The bound view's calls reach the engine through `engineFor(language)`, which resolves the pair,
waits for the pack, then returns the engine.

### D4 — A language nothing ships fails before the engine

`engineFor` rejects when `pairFor` finds no pair, with a message naming the language and the
shipped pairs: `no shipped pack studies "es" (shipped pairs: en-fr)`. The engine receives
nothing, and nothing is fetched. Over the RPC the error returns as `{ ok: false, error }`, and
the messaging port rejects with it, as for any engine error.

In R2 this is reached only by a call naming Spanish, which no surface makes yet (they all pass
`STUDIED_LANGUAGE`, English).

### D5 — Whole-reader applies load the packs their records name

`applyStatusChanges`, `applyCardOps` and `applyDeclaredLevelChanges` first collect the
distinct `language` of their records. They load the packs of those a listed pair studies, then
apply.

Without this, the sync engine would apply a pulled Spanish record before anything had loaded
es-fr. The engine would skip the record, and the sync would save and move its cursor past it,
losing the record on this device until a full re-pull. A failed load rejects the apply, so the
sync fails before saving and its cursors stay where they were (`sync.ts` moves them only after
the save).

A language no listed pair studies is not loaded, and the engine keeps skipping its records. A
build that does not ship a language must not fail every sync on it.

The other whole-reader calls load nothing:
- **Backup, restore, reset:** a restore keeps an unheld language's state as it is.
- **Counts, the review session, the exports:** they read the state, which holds every language
  whatever is loaded.
- **`languages()`:** it keeps answering with the languages the engine holds a pack for. Its one
  caller, the Chromium probe in `resolveContentPort`, only forces the build.

## Risks / Trade-offs

- **Two packs held for a reader of two languages** → about 32 MiB in that engine until it ends.
  On Chromium a page's engine ends with the page; the background engines end when the worker or
  event page is suspended. R3 measures this before any unloading is built (D1).
- **The first call in a new language waits for its pack** → the same fetch and decode an
  engine's first call already pays for the default pack. Every later call does not wait.
- **The sync engine always loads the default pack**, even for a reader who studies only another
  language. That is the "sync engine with no pack" lever, out of scope here.
- **Exposure counters restored while their language had no pack** are bounded at that language's
  next recording, not when its pack arrives. The engine is unchanged. They were bounded when they
  were written, since only an engine holding the pack records exposures.
- **A pack listed but missing from the package** cannot happen silently: `build.mjs` refuses to
  build without every listed pack, and `check_variants` refuses a bundle whose packs differ from
  the list (#630).
