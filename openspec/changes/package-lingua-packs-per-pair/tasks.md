## 1. Language to pair

- [ ] 1.1 `src/analyzer/pairs.ts` exports `SHIPPED_PAIRS`, `packPath` and `pairFor` (D2). A vitest spec covers:
  - the first listed pair that studies a language, and `null` for a language none studies;
  - `packPath` equal to `tool/packs.mjs` `packFile` for en-fr and es-fr;
  - `SHIPPED_PAIRS` equal to `shippedPairs()`.

## 2. Packs loaded on first need

- [ ] 2.1 `WasmAnalyzerPort` takes `pairs` (default `SHIPPED_PAIRS`) and builds the engine with the first pair's pack, replacing `PACK_PATH`. The bound view reaches the engine through `engineFor(language)`, which:
  - refuses a language no listed pair studies (D4);
  - loads another listed pair's pack once, sharing a pending load and forgetting a failed one (D3).
- [ ] 2.2 `applyStatusChanges`, `applyCardOps` and `applyDeclaredLevelChanges` load the packs of the listed languages their records name before applying; unlisted languages load nothing (D5). No other whole-reader call loads a pack.
- [ ] 2.3 `test/engine.spec.ts`, on a fake glue that records fetched paths and `addPack` calls:
  - en-fr alone: the engine fetches only `assets/packs/en-fr.lingua`. A Spanish call rejects with a message naming "es" and "en-fr", and the engine receives nothing.
  - en-fr and es-fr: two Spanish calls fetch and add es-fr once, before the first call. Two concurrent first calls also add it once.
  - A failed es-fr fetch fails its call, and the next Spanish call fetches again.
  - Each of the three applies loads es-fr for a Spanish record and nothing for English-only records. A record in a language no pair studies is handed to the engine without a load.
  - Backup, restore, the counts and the exports fetch nothing beyond the default pack.
- [ ] 2.4 `test/messaging.spec.ts`: an RPC naming a language nothing ships, on a `WasmAnalyzerPort` over the fake glue, answers `{ ok: false }` with the refusal's message.

## 3. Docs

- [ ] 3.1 `apps/lingua-extension/README.md` ("The data pack"): the engine loads the default pair's pack, and another listed pair's the first time its language is needed. The `browser-extension-architecture` skill gets the same line where it describes the engine.

## 4. Gates

- [ ] 4.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate) and `yarn format:check`;
  - `yarn build` (three variants), then `yarn check:variants`; each bundle names `assets/packs/en-fr.lingua` and no other pack;
  - `cargo test -p lingua-wasm --test english_baseline` passes.
- [ ] 4.2 `openspec validate package-lingua-packs-per-pair --strict` passes. In `docs/lingua/spanish-programme.md`, change 9 is marked done and change 10 next.
