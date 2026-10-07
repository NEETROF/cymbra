# Design — generalise-lingua-translation-routes-by-pair

## Context

See proposal.md (Why). Where the studied language is the key today, from the page inward:

| Where | What |
|---|---|
| `model-manifest.json` `routes` | `{"en": [en-fr], "es": [es-en, en-fr]}` |
| `src/translate/host/model-manifest.ts` | `TARGET_LANGUAGE = "fr"`; `parseCatalogue` refuses a route not ending in French; `routeOf(catalogue, language)`, `modelsFor(catalogue, languages)` |
| `src/translate/host/engine.ts` | `EngineAccess.translate(markup, language)`, `warm(language)`; `WorkerRequest` `load`/`translate` carry `language` |
| `src/translate/host/offscreen-engine.ts`, `offscreen.ts` | the offscreen protocol carries `language` |
| `src/translate/host/engine-worker.ts` | `routes: Map<language, ids>`; `load(language)` reads `catalogue.routes[language]` |
| `src/translate/host/channel.ts` | `loads: Map<language, Promise<boolean>>` |
| `src/translate/host/relay.ts`, `src/translate/markup.ts` | `MARKED_LANGUAGES = ["en", "es"]` |
| `src/translate/setting.ts` | `ModelState` `ready`/`missing` name `languages`; `READY_BEFORE_LANGUAGES = ["en"]`; `languageReady` |
| `src/translate/host/model-controller.ts` | `deps.languages()`, `pivots`, `translatable` per language |
| `src/background.ts` | `model.ready(message.request.language)`; `languages()` filters the studied languages by the native and drops the native |
| `tool/measure_marks.mjs`, `tool/marks/` | `PAIRS = {en: "en-fr", es: "es-fr"}`; `results-en.jsonl`, `judged-en.tsv` |

The page's side — `port.ts`, `wire.ts`, `messaging-port.ts`, `create-port.ts`, `keepalive.ts`,
`answer-memory.ts`, `session.ts` — asks in the document's language and stays so (D2, D7).

## Goals / Non-Goals

**Goals:**
- A pair is a catalogue entry: `es-en` can be added by a route and a model, with no host code.
- Nothing a reader of French sees or stores moves.
- The three hosts keep their shapes and their placement lint; no platform branch is added.

**Non-Goals:**
- A worker that respawns after a trap, and models evicted when no pair needs them:
  `harden-lingua-translation-engine` (change 9).
- es-en, en-es and their models: changes 21, 22 and 25; their marks: change 26.
- The native language's choice in Réglages, and what an open page does when it changes: change
  20. The answer memory and the keep-warm stay keyed by the document's language (D7).
- A second model host or a change to `lingua-model-deploy`.

## Decisions

### D1 — The catalogue keys its routes by pair, and the parser reads both languages from the key

`routes` is `{"en-fr": ["en-fr/base-memory/2.0"], "es-fr": ["es-en/base-memory/2.0",
"en-fr/base-memory/2.0"]}`. `parseCatalogue` splits each key on its first `-`, refuses a key that
is not `<studied>-<native>`, checks that the first model translates from the studied language,
chains each model's target to the next model's source, and checks that the last model translates
into the native language. `TARGET_LANGUAGE` is removed. `routeOf(catalogue, pair)` and
`modelsFor(catalogue, pairs)` take pairs.

`tool/model-catalogue.mjs` copies `routes` through unchanged, and `check_variants` compares the
bundled routes with the committed ones as JSON: both change together, and the check stays.

Alternative: keep the studied key and add a `native` field to each route. Two things would have to
agree for a route to be found, and a pair named once — as the packs name it — is the shape the
programme settled.

### D2 — The pair is formed in the background, from the document's language and the reader's native language

The page keeps asking in the document's language (*A translation is asked in its document's
language*): `TranslationRequest.language` and `WarmMessage.language` are unchanged. The
background forms the pair with `pairOf(language, native)` — in `src/analyzer/pairs.ts`, beside
`studiedOf` and `nativeOf`, which read it back (a pair's name splits at its first `-`, and every
reader of a name agrees) — with the native language read from the stored profile exactly as
`ModelController`'s `languages()` reads it today (`storedNativeLanguage`: `nativeLanguageOf`, or
`DEFAULT_NATIVE` without a backup), then asks `model.ready(pair)`, `relayTranslation(engine,
request, pair)` and `relayWarm(…, pair)`. `background.ts` itself is excluded from the coverage gate,
so the forming, the gate and the two answers (*The same page for another native language*, *A pair
without a route*) live in `relay.ts`'s `answerTranslation` and `answerWarm`, tested, with the
background as thin glue. From there inward everything carries the pair:
`EngineAccess.translate(markup, pair)` and `warm(pair)`; the offscreen `translate` and `warm`
requests; `WorkerRequest` `load` and `translate`; the channel's `loads`; the worker's `routes`.

Alternative: the page sends the pair. A content script would read the native language from its
engine on every selection, and a page could ask for a pair its reader is not. The background owns
the profile and is the one place that reads it without an engine (model-state D2); it forms the
pair once.

### D3 — The device records the ready pairs; a state that names languages is read as pairs

`ModelState` `ready` and `missing` name `pairs: string[]` in place of `languages`. `parseModelState`
reads a stored `languages` — written by every release so far — as `${language}-${DEFAULT_NATIVE}`,
because every pair shipped so far is glossed in French (M22); a `ready` that names neither is read
as `["en-fr"]`, the one pair the release before `generalise-lingua-translation-model-state` could
have. The next reconciliation rewrites the state with pairs.

The background gates on the exact pair (`pairReady(host, state, pair)`). The reading page's gate,
`translatorSource(language)`, stays on the document's language: `languageReady` answers whether a
ready pair studies it. The device holds one native language, so the two gates agree; when the
native language changes (change 20), the background's reconciliation rewrites the pairs and the
page's gate follows through `storage.onChanged`, as it does today.

Alternative: record both `languages` and `pairs`. Two lists to keep equal, for a gate that reads
one.

### D4 — Marks are measured per pair

`MARKED_PAIRS: readonly string[] = ["en-fr", "es-fr"]` replaces `MARKED_LANGUAGES`, and
`relayTranslation` checks the pair it was given. es-fr's marks were measured through its pivot
(89 of 90 shown marks right, 10 % withheld), which says nothing of es-en's one model; a pair not in
the list is translated without a mark, as a language outside the list is today
(add-lingua-spanish-translation-pivot D3).

### D5 — The controller's needs are pairs

`ModelControllerDeps.languages()` becomes `pairs()`: in the background, the reader's accepted
languages (change 4's `acceptedLanguages`) mapped through `pairFor(language, native)` — the shipped
pairs of the reader's native language that study each accepted language. `Needs` carries `pairs`;
`pivots` is whether a pair's route has more than one model; `translatable` is the pairs whose
whole route is complete; `modelsFor(catalogue, pairs)` is the union of their routes. The setting's
cost text is unchanged: it reads `pivot` and the sums.

### D6 — The harness measures a pair; the corpus stays per studied language

`measure_marks.mjs --pair <pair>` reads `catalogue.routes[pair]`, the studied language from the
pair's name, and the gloss tables from `tables/<studied>/` and `tables/<pair>/`. Results and
judgments are filed by pair: `results-en.jsonl` → `results-en-fr.jsonl`, `judged-en.tsv` →
`judged-en-fr.tsv`, and the Spanish ones likewise (`git mv`, bytes unchanged). `corpus.json` and
`select_corpus.mjs` stay keyed by studied language: a selection is of the text it was made in, and
es-en will be measured on the same Spanish selections as es-fr (change 26). The harness's
stop-word list and its `french` result key are French-native today; change 26 generalises them
with the first pair of another native. The harness README says which file is which.

### D7 — The page's memory and keep-warm stay keyed by the document's language

`answer-memory.ts` keys a page's answers by sentence, span and language; `keepalive.ts` re-warms
the last language. Both live in the page, which has one reader with one native language, and they
go with the page. Keying them by pair would mean reading the native language in the content
script on every request, which D2 avoids. Change 20, which lets the native language change while a
page is open, decides what the page does then.

## Risks / Trade-offs

- **A stored state misread as the wrong pairs** → `parseModelState` tests cover `languages` from
  the two previous shapes, with and without `models`; the mapping uses `DEFAULT_NATIVE`, which is
  the native of every pair shipped so far.
- **A route key the parser accepts that a pack never has** → the key is split on its first `-`
  into a studied and a native language; `pairs.ts` reads a pack's name with `split("-")`, which
  agrees on every real pair; a key with no `-` or an empty side (`en`, `en-`) is refused, and so
  is a key whose native side is no language the catalogue's models reach (`en-fr-x` ends in
  `fr-x`, which no model translates into), and the catalogue with it (nothing is fetched).
- **The three hosts drift** → the host-independence lint (`lint-translation-platform.spec.ts`)
  and the offscreen and channel tests carry the pair; `check_variants` still requires the bundled
  routes to equal the committed ones.
- **Marks shown for a pair never measured** → `MARKED_PAIRS` is an explicit list; a pair must be
  added by hand, after its measurement (change 26).

## Migration Plan

One release, silent. No dispatch, no model, no host change. A device updating from any earlier
release reads its stored state as pairs and continues; nothing is downloaded or deleted.
