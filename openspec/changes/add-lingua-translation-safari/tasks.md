## 1. The engine in the Safari package

- [x] 1.1 `build.mjs`: `translationHost("safari")` is `"event-page"`; a Safari build copies the pinned engine and bundles `model-manifest.json`, and refuses to run without the pinned engine, as Chromium's and Firefox's do
- [x] 1.2 `tool/check_variants.mjs`: engine (pinned bytes) and model manifest required in Safari too; no model file and no remote code anywhere; `offscreen` in Chromium only; its tests
- [x] 1.3 `test/lint-translation-platform.spec.ts` and the variant tests no longer expect Safari to fold the translation paths away

## 2. The Apple release lane

- [x] 2.1 `lingua-apple-release`: `yarn fetch:engine` before `yarn build:safari`
- [x] 2.2 `lingua-apple-release`: before `deliver`, `node tool/check_model_host.mjs`; a delivery whose model host does not answer fails

## 3. Listing and documentation

- [x] 3.1 `apps/lingua-apple/STORE-LISTING.md`: « sans compte ni traduction étendue, aucune requête réseau »; the setting described as in the Chrome and AMO listings; the review notes say the engine is in the app and only a data file is downloaded
- [x] 3.2 `apps/lingua-apple/README.md`: the copy phase keeps files removed from `dist-safari` — clean DerivedData after switching variants locally; `apps/lingua-extension/TRANSLATION.md` and `README.md`: Safari offers the setting

## 4. Archive order

- [x] 4.1 `scripts/openspec_archive_order.py` honours `archiveAfter` in a change's `.openspec.yaml`: a listed change still in `openspec/changes/` makes it wait (exit 10); tests in `scripts/test_openspec_archive_order.py`

## 5. Measured on the devices

- [x] 5.1 A build of this change on the iPhone, the iPad and the Mac (Safari): the setting, a download from `models.cymbra.app`, a phrase and an unknown single word translated, a return from another app; no termination
  - Passed on all three on 2026-09-26 (product owner), with the release build of this change — no probes; the measured times are the proposal's.
- [x] 5.2 The headless Chrome and Firefox pass (`e2e546` harness): nothing regressed

## 6. Gates

- [x] 6.1 `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (coverage ≥ 80 %), `yarn check:variants`; the Apple app builds for iOS and macOS
- [x] 6.2 `openspec validate add-lingua-translation-safari --strict`
