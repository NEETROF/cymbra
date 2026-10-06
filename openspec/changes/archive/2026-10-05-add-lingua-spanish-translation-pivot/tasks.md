# Tasks

## 1. The catalogue

- [x] 1.1 `model-manifest.json`: `es-en/base-memory/2.0` with its three files pinned (design D1), and Spanish's route es-en then en-fr. Specs:
  - the catalogue is valid;
  - Spanish's route ends in French;
  - a reader of English and Spanish needs both models, once each.

## 2. The engine

- [x] 2.1 `engine-worker.ts`: a route of two models loads both and translates through `translateViaPivoting`; a longer route is refused; `NO_PIVOT` goes (design D2).

## 3. The marks

- [x] 3.1 `MARKED_LANGUAGES` (`["en"]`) in `translate/markup.ts`. `relay.ts` sends a language outside it untagged, in one request, and answers without a mark (design D3). Specs:
  - a Spanish selection makes one untagged request and gets no mark;
  - an English one is marked and checked as before.

## 4. The setting

- [x] 4.1 `translation-setting.ts`: the memory figure is the pivot's when a needed route has two models (design D4). Specs:
  - a reader of Spanish reads 52,0 Mo and about 340 Mo;
  - a reader of English alone reads 25,8 Mo and about 200 Mo.

## 5. The copy

- [x] 5.1 Both `STORE-LISTING.md` files say Spanish is translated through English, with its download (design D5).
- [x] 5.2 `apps/site/src/pages/lingua.astro` and `en/lingua.astro` say extended translation serves English and Spanish (design D5).

## 6. Checks

- [x] 6.1 In `apps/lingua-extension`, the steps of `lingua-extension-check`, the variant check included; the English baseline does not move.
- [x] 6.2 In `apps/site`, the steps of `site-check`.
- [x] 6.3 `openspec validate add-lingua-spanish-translation-pivot --strict` passes.
- [x] 6.4 `docs/lingua/spanish-programme.md`:
  - change 26 is done;
  - D3 notes the owner's decision of 2026-10-05;
  - change 27 measures the marks.

## 7. Owner

- [x] 7.1 `lingua-model-deploy` is dispatched before the release: host assembled with es-en, mirror release created, deployed, `check_model_host` passes (design D6). Done on 2026-10-05, run #3: mirrored and deployed. Its check ran 20 s after the deployment and met 404s; by hand, minutes later, it passed. #712 now gives the check five minutes.
- [x] 7.2 Dogfood Spanish translation on Chrome, Firefox for desktop, Safari on macOS, and Safari on the iPhone and the iPad. Watch for memory on iOS. Done on 2026-10-05, marks shown since `release-lingua-spanish-translation`, on Chrome, Firefox and Safari on macOS, Firefox for Android on a Boox, and Safari on the iPhone; the iPad was not connected.
