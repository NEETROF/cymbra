# Tasks

## 0. Before the pull request

- [ ] 0.1 [owner] D9's checklist: the changes on main, the releases and deploys live (change 32's extension and host-app releases, the model host with en-it, the backend, change 41's site, change 43), change 44's pass done, and the owner's decisions of *Open Questions* taken.

## 1. The extension

- [ ] 1.1 `packs.json` appends en-it; `src/i18n/language.ts`'s `INTERFACE_LANGUAGES` gains `it`, its comment with it; `tool/check_variants.mjs`'s `SHIPPED_PAIRS` and comment (D1).
- [ ] 1.2 The tests reading the list, their earlier lists kept as explicit arguments: `packs.spec.ts`, `pairs.spec.ts`, `manifests.spec.ts`, `model-manifest.spec.ts`, `studied-languages-view.spec.ts`, `native-language.spec.ts`, `native-language-view.spec.ts`; `engine.spec.ts`, `model-residency.spec.ts`, `translation-setting.spec.ts`, `model-controller.spec.ts` for an Italian reader (D1).
- [ ] 1.3 The links (D2): `LEAVING_PAGES.it`, `PRIVACY_POLICY_URLS.it`, `deleteAccountUrl`'s Italian and Spanish arms, their comments; `account-locale.spec.ts` and `account-page.spec.ts` (five cases).
- [ ] 1.4 The « not yet » tests flipped: change 33's stored `it`, change 40's teller; change 32's « preset to Italian » added (D3).
- [ ] 1.5 Models (D4): en-it's route as change 2 writes it; `MARKED_PAIRS` if change 38 puts en-it on the first tier; the route dropped if change 38 withholds its translation.
- [ ] 1.6 The capture harness's one-language mode (D5): `tool/capture/plan.mjs` (`it`, `it-IT`, `STUDIED.it`), `capture.mjs`, `test/capture-tool.spec.ts`; the Italian run, its « second » shot on a second English page.
- [ ] 1.7 `README.md`, `REVIEWERS.md`, `TRANSLATION.md`, `tool/marks/README.md` name Italian, as the precedents did.

## 2. The site and the data

- [ ] 2.1 `gloss_coverage.py --write` (en-it's row in `lingua-coverage.json`); `test_gloss_coverage.py`'s listed floors; `crates/lingua-pack/tests/committed_tables.rs` « listed »; the Lingua fixtures refreshed (D6).
- [ ] 2.2 `/it/lingua/`'s media from task 1.6; `LINGUA_TEXT.it.showcase` rewritten against them, the capture tests with Italian (D5, D6).
- [ ] 2.3 Change 42's leftovers (D6): *Nine pairs on a phone* at 375 px; `alternates.spec`'s Lingua test generalised; `test/astro/lingua-page.spec.ts` and `lingua.spec.ts` for `/it/lingua/`; `head.spec`'s share preview; `lingua-text.spec`'s card buttons.
- [ ] 2.4 `pinned-routes.ts`: `/it/lingua/` pinned; `/en/delete-account/`'s `pinnedBy` and source (D2, D6).

## 3. The Apple host

- [ ] 3.1 `apps/lingua-apple/README.md`'s languages line names Italian, so that the squashed commit releases the host app; no `Release-As` (D8).

## 4. Gates and owner

- [ ] 4.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; in `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; the Python tests; `cargo test --workspace`.
- [ ] 4.2 `openspec validate enable-lingua-italian-speakers --strict`; the archive order; the eight MODIFIED blocks re-copied as their holders then stand; `docs/lingua/italian-programme.md` row 45's status.
- [ ] 4.3 [owner] The dogfood on the five targets in Italian, the Italian checks of changes 39 and 40 included (D9).
- [ ] 4.4 [owner] The releases, the site deploy and the submissions in D9's order, change 46's listings pasted.
