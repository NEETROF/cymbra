# Tasks

## 1. The catalogue (apps/lingua-extension)

- [ ] 1.1 `model-manifest.json`: `routes` keyed `en-fr` and `es-fr`, the same models in the same order (D1).
- [ ] 1.2 `src/translate/host/model-manifest.ts`: `parseCatalogue` reads the studied and the native language from each key, refuses a key that is no pair, a route that does not start from the studied language or does not end in the native one; `TARGET_LANGUAGE` removed; `routeOf` and `modelsFor` take pairs. `test/model-manifest.spec.ts` covers each scenario of *Translation models are listed in one catalogue, with a route per pair*, and `tool/model-catalogue.mjs` and `check_variants` still pass the routes through as JSON.

## 2. The pair, from the background inward (apps/lingua-extension)

- [ ] 2.1 `relay.ts` `pairOf(language, native)`; `src/background.ts` forms the pair with it from the document's language and the reader's native language (read as `languages()` reads it), gates on `model.ready(pair)`, and passes it to `relayTranslation` and `relayWarm` (D2). `test/translate-relay.spec.ts`: *Every reader today*, *The same page for another native language*, *A pair without a route*. The warm and translate wire messages are unchanged.
- [ ] 2.2 `engine.ts`, `offscreen-engine.ts`, `offscreen.ts`, `channel.ts`, `engine-worker.ts`: `translate`, `warm`, `load` and the loaded routes keyed by pair (D2). `test/translate-channel.spec.ts` and `test/translate-offscreen.spec.ts` carry the pair; `lint-translation-platform.spec.ts` and `lint-translator-placement.spec.ts` pass.
- [ ] 2.3 `markup.ts` `MARKED_PAIRS = ["en-fr", "es-fr"]`; `relay.ts` checks the pair it is given (D4). `test/translate-relay.spec.ts`: en-fr marked, es-fr marked, es-en unmarked (*A pair measured in another native language*, *A language whose marks are not measured*, *A Spanish sentence*).

## 3. The model state and the controller (apps/lingua-extension)

- [ ] 3.1 `src/translate/setting.ts`: `ready` and `missing` name `pairs`; a stored `languages` is read as `<language>-<DEFAULT_NATIVE>`, a bare `ready` as `["en-fr"]`; `pairReady(host, state, pair)` for the background, `languageReady(host, state, language)` for a page, true when a ready pair studies it (D3). Tests: the three stored shapes; *A state recorded before pairs*.
- [ ] 3.2 `src/translate/host/model-controller.ts`: `deps.pairs()`, `Needs.pairs`, `pivots` and `translatable` per pair, `modelsFor(catalogue, pairs)`; the background supplies the accepted languages mapped through `pairFor(language, native)` (D5). Tests: each scenario of *The models follow the reader's pairs*, including *The native language changes* with a catalogue listing en-es.
- [ ] 3.3 `create-port.ts` and `session.ts` unchanged in shape: the `TranslatorSource` that `translatorSource(deps)` returns gates `language` through `languageReady`; a test shows a page asking in `en` with en-fr ready gets a port, and with only es-fr ready gets none.

## 4. The harness and the docs (apps/lingua-extension)

- [ ] 4.1 `tool/measure_marks.mjs --pair <pair>`: the route from `catalogue.routes[pair]`, the studied language from the pair, the tables from `tables/<studied>/` and `tables/<pair>/`; `tool/marks/results-*.jsonl` and `judged-*.tsv` renamed by pair (`git mv`, bytes unchanged); `select_corpus.mjs` and `corpus.json` unchanged; `tool/marks/README.md` names the files and the command (D6).
- [ ] 4.2 `TRANSLATION.md` (routes per pair, the pair formed in the background), `README.md` (the packs list), `env.d.ts` (the shipped hosts), and the comments that still say "into French".

## 5. Gates and docs

- [ ] 5.1 Gates, in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; the real engine is not run in CI (the programme's recommendation for M25, which the owner settles); `node --experimental-strip-types tool/measure_marks.mjs --pair en-fr --models <dir>` runs by hand and reproduces `results-en-fr.jsonl`.
- [ ] 5.2 Docs and spec:
  - `openspec validate generalise-lingua-translation-routes-by-pair --strict` passes, and `python3 scripts/openspec_archive_order.py generalise-lingua-translation-routes-by-pair` exits 0;
  - change 8 is marked done in `docs/lingua/language-matrix-programme.md`.
