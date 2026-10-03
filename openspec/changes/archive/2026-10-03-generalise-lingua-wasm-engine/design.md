# Design — generalise-lingua-wasm-engine

## Context

See proposal.md (Why). `crates/lingua-wasm/src/lib.rs` is a thin wasm-bindgen surface over
`lingua-core`:

```
LinguaEngine { pack: Pack, state: LinguaState, session: Option<ReviewSession>,
               level_vocabularies: OnceCell<[usize; 6]> }
```

`LinguaState` (knowledge, exposure, deck) already keys everything by `StudiedLanguage`, and the
core's exports already carry the language of each record (`StatusRecord.language`,
`DeclaredLevelRecord`, and `(language, card)` from the deck). The engine throws it away:
- its exports write `"language": "en"`;
- its applies `continue` past any record whose language is not `"en"`;
- `restore` prunes the English exposures against the single pack.

The extension constructs one engine per context with the embedded en-fr pack
(`new LinguaEngine(bytes)`) and calls its 40 bindings without any language.

wasm-bindgen is 0.2.126. A trailing `Option<String>` parameter is generated as optional in
TypeScript (checked: `analyse(blocks: string[], language?: string | null): string`). A
`Result<T, JsError>` return keeps the TypeScript return type `T`, and throws on error. Creating a
`JsError` off wasm panics, so native tests cannot reach an error path of a binding.

The English invariance baseline (`crates/lingua-wasm/tests/english_baseline.rs`) drives these
bindings natively with the real en-fr pack and freezes 142 outputs.

## Goals / Non-Goals

**Goals:**
- One engine instance that holds a pack per language and answers each call in the language it
  names, defaulting to the first pack's.
- Sync records that say which language they belong to, in and out.
- Byte-identical English answers, and no change to the extension's source.

**Non-Goals:**
- The TypeScript port bound to a language, and the language each surface passes
  (`generalise-lingua-extension-port`).
- Shipping a second pack, or choosing which packs to load (`package-lingua-packs-per-pair`,
  `add-lingua-studied-language-profile`).
- A language field in the review card, and a language filter on the review queue
  (`add-lingua-language-stats-review`). The queue is already mixed, since the deck is keyed by
  language.
- Backup v2, and the agent plugin's engine.

## Decisions

### D1 — A `PackSet` in `lingua-core`, held by the engine

`lingua_core::packs::set::PackSet` holds one `Pack` per `StudiedLanguage`, plus the language of
the first pack, which is the default. Its API:
- `new(pack)`;
- `add(pack) -> Result<StudiedLanguage, PackSetError>`, which refuses a language already held;
- `resolve(Option<&str>) -> Result<(StudiedLanguage, &Pack), PackSetError>`: `None` gives the
  default; an unknown tag, or a tag the set holds no pack for, gives an error naming the tag;
- `default_language()` and `languages()`.

Logic lives in the core because `lingua-wasm` is excluded from the coverage gate and its error
paths cannot run natively. The engine maps `PackSetError` to `JsError` in one place.

*Rejected — one engine per language.* The study's reason: each engine would carry, and back up,
its own whole state, and Firefox and Safari share one engine across tabs.

*Rejected — replacing a pack in place.* No surface needs it: a dictionary update ships with a new
extension build, which constructs new engines.

### D2 — A trailing optional `language` on the 21 language-bound bindings

These bindings gain `language: Option<String>` as their last parameter, and return
`Result<_, JsError>`:
- `setCalibration`, `calibration`;
- `setDeclaredLevel`, `declaredLevel`, `setDeclaredLevelAt`;
- `hasLevels`, `levelLadder`, `vocabularyEstimate`;
- `recordExposures`, `promoteByExposure`;
- `setStatus`, `setStatusAt`;
- `analyse`, `gloss`, `phraseGloss`, `wordGrammar`;
- `addCard`, `retireCard`, `seedLevel`;
- `notice`, `licences`.

Each one resolves the language through the pack set before touching state, so a refused call
changes nothing.

*Rejected — a mutable "current language" on the engine.* Calls from several tabs interleave on
Firefox and Safari, so a switch made for one tab would leak into another.

*Rejected — a second, language-taking method per binding (`analyseIn`).* That doubles a surface
the next change has to bind anyway.

*Rejected — a handle per language (`engine.forLanguage("es")`).* wasm-bindgen handles cannot
borrow the engine, so this would put the state behind `Rc<RefCell<…>>` for no gain over a
parameter.

### D3 — Root bindings keep their signatures and cover every language

These are unchanged, and already act on the whole state:
- `exportStatusOps`, `applyStatusChanges`, `exportCardOps`, `applyCardOps`;
- `exportDeclaredLevels`, `applyDeclaredLevelChanges`;
- `trackedCount`, `deckCount`, `dueCount`;
- the review session (`startReview` … `reviewMarkKnown`);
- `backup`, `restore`, `reset`, `resetStatuses`.

Two bindings are added: `addPack(bytes)` and `languages()` (the held tags as JSON, default
first). `reviewCurrent` keeps its JSON shape. A language field there would move the English
baseline, and belongs to the change that shows it.

### D4 — Records name their language; applies keep what the engine studies

Exports write `record.language.tag()`. Applies read each record's `language`, missing meaning
`en` (the wire's rule since #609), and resolve it through the pack set. A record whose language
the engine holds no pack for is skipped, as non-English records are today, and is not counted as
a change. With the en-fr pack alone, every output and every skip is the same as before.

### D5 — Per-language caches and pruning

- The level vocabularies become a map from language to `OnceCell`, filled from that language's
  pack on its first ladder.
- `promoteByExposure` and `recordExposures` work in the call's language.
- `restore` prunes each language that has a pack against that pack, using that language's latest
  observation. A language without a pack keeps its counters untouched: they cannot be judged, and
  the backup is lossless.

### D6 — How English is shown not to move

- The baseline harness unwraps the new `Result`s. Its golden is not re-blessed, and its diff
  against `main` is empty.
- The extension regenerates its bindings. `lingua_wasm.d.ts` shows each new parameter as
  optional, and `yarn typecheck`, `yarn test` and `yarn build` pass with no change under `src/`.
- Native tests cover a two-pack engine. A wasm-bindgen test covers the thrown errors.

### D7 — OpenSpec

Two ADDED requirements in `lingua-analysis`, which already holds the WASM module's requirements.
`lingua-sync` exists only in open changes, so it is not touched, and no requirement an open change
holds is rewritten.

## Risks / Trade-offs

- [A binding is missed and stays English-only] → `const EN` is deleted from the engine, so every
  use has to choose: resolve the call's language, or iterate the state. Tests keep their own
  constant.
- [A surface starts passing a language before packs exist for it] → It gets an explicit error,
  never English data. No surface passes one before `generalise-lingua-extension-port`.
- [Root exports now include records of languages without a pack (from a restored backup)] → They
  are exported with their own language, which is correct. The server filters by announced
  languages (#609), and released builds never hold such records.
- [Trailing optional parameters are fragile if a binding later gains a parameter] → The language
  stays last, by rule. The next change passes it explicitly everywhere.

## Migration Plan

None: no stored format, wire field or pack changes, and the extension's code is untouched.
Rollback is a revert.

## Open Questions

None.
