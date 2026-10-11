# Tasks

## 0. Prerequisite

- [ ] 0.1 `add-lingua-feature-betas` is merged and deployed (Lingua campaigns, Lingua betas in flag evaluation, effective flags read by the extension). Verify: its own integration task passes on production.

## 1. The promise first

- [ ] 1.1 Rewrite the Lingua annex of `apps/site/src/pages/confidentialite.md`, `apps/site/src/pages/en/privacy.md` and `apps/site/src/pages/es/privacidad.md`: page text stays on the device except the sentence sent with « Traduction en ligne »; sent only on a press, off by default, per device; translated by Anthropic, or Google when Anthropic is unavailable; kept by no one, not used for training. Verify: `yarn build` in `apps/site` and the rendered annex read against the `lingua-privacy` delta.
- [ ] 1.2 Correct every "never leaves your device" claim in `apps/lingua-extension/STORE-LISTING.md` (en, fr, es) to name « Traduction en ligne » as the one exception, off by default. Verify: `grep -n` for the old claims returns only the corrected sentences.
- [ ] 1.3 Re-read the recorded App Store privacy answers for Cymbra Lingua against the new annex and record the result with the Apple app (no new data type is expected: the sentence is "other user content", already declared). Verify: the recorded answers file states the review date and outcome.
- [ ] 1.4 Name the new request in `apps/lingua-extension/REVIEWERS.md` ("Where the add-on reaches the network") and in the `data_collection_permissions` comment of `tool/manifests.mjs`. Verify: `yarn check:variants` passes.

## 2. The service

- [ ] 2.1 Add `backend/lingua/proto/online_translation.proto` (`OnlineTranslationService.Translate`, request `sentence`, `selection_start`, `selection_end`, `source_language`, `target_language`; response `oneof translated | unavailable | limit_reached`), list it in `backend/lingua/build.rs`, and add it to `backend/lingua/tests/privacy_allow_list.rs`. Verify: `cargo test -p cymbra-lingua privacy_allow_list` and `buf lint`.
- [ ] 2.2 Write the pure core: prompt building (selection marked, three fixed examples, target language), validation of the structured answer, and placing marks from the `highlighted` segments in order, with no marks when a segment is missing. Verify: unit tests for a contiguous rendering, a split rendering ("remettait … à plus tard"), a missing segment, a repeated word, non-ASCII offsets (UTF-16), and a pair outside the six (refused).
- [ ] 2.3 Declare the provider port and the fallback chain (primary, then secondary, then unavailable; an unreadable answer counts as a failure). Verify: mockall tests for primary success, primary error then secondary success, both failing, and an unreadable primary answer.
- [ ] 2.4 Write the two thin adapters (Anthropic Messages API with structured output and adaptive thinking; Gemini `generateContent` with a response schema), each with a timeout, errors mapped without their body, model ids, keys and the provider order read from the environment (`CYMBRA_LINGUA_TRANSLATION_PRIMARY` = `gemini` by default, `CYMBRA_LINGUA_TRANSLATION_GEMINI_API_KEY`, `…_GEMINI_MODEL`, `CYMBRA_LINGUA_TRANSLATION_ANTHROPIC_API_KEY`, `…_ANTHROPIC_MODEL`); document them in `backend/.env.example` and `backend/deploy/.env.prod.example`. Add the adapters to `.github/coverage-ignore-regex.txt` as thin HTTP glue. Verify: a manual call against each provider from a local server, recorded in the PR.
- [ ] 2.5 Declare `lingua.online_translation.enabled` (default false, app `lingua`, evaluated per caller with their Lingua betas through `add-lingua-feature-betas`) and `lingua.online_translation.daily_limit` (default 100) in `feature-flags/src/registry.rs`, with a flag adapter in `server/src/flags.rs`. Add the read-then-increment limiter variant (count read before asking a provider, incremented after an answer). Verify: tests for a caller without access (unavailable, no provider asked), a beta member (answered), the limit reached (`limit_reached` with `resets_at`, no provider asked), and a failed request not counted.
- [ ] 2.6 Implement the gRPC service with the caller from the interceptor only, the sentence bound (1 000 characters → `INVALID_ARGUMENT`), and mount it in `backend/server/src/main.rs` behind the strict interceptor. Amend the doc comment of `backend/lingua/src/lib.rs` to name this RPC as the one that carries page text. Verify: a test that a request without a token is `UNAUTHENTICATED`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 2.7 Prove the service keeps nothing: a test that captures tracing output across a successful request and a provider error and asserts the sentence and the translation appear nowhere. Verify: the test, and `cargo llvm-cov --workspace --fail-under-lines 80`.

## 3. The extension

- [ ] 3.1 Add the setting: storage key `cymbra-lingua-online-translation` (boolean, `chrome.storage.local`, default false), a toggle in `reading/settings-view.ts` shown only when the extension's effective flags say `lingua.online_translation.enabled` is true, a note with a link to the annex, strings in `i18n/{fr,en,es}/settings.ts`. Verify: settings tests for signed out (hidden), signed in without access (hidden), with access (unticked by default), access withdrawn (hidden), and persistence; `test/i18n.spec.ts`.
- [ ] 3.2 Regenerate the gRPC-web stubs (`tool/gen_proto.sh`), add the client to `net/transport.ts` `Clients`, and answer a new runtime message in the background that calls it and maps the answer to `MarkedTranslation`, `unavailable` or `limit_reached`. The page never calls the network. Verify: background tests with a mocked client for the three answers and for a transport error (→ unavailable).
- [ ] 3.3 Add the button to the card in `reading/selection-card.ts`: shown for several words when the setting is ticked and the reader has access, independent of the engine; on press, a "translating online" line, then the answer through the same generation guard as the engine's; disabled once pressed; strings in `i18n/{fr,en,es}/card.ts` ("Traduction en ligne", the label, the unavailable and limit messages). Verify: card tests for every scenario of the `lingua-translation` delta (ticked/unticked, no model, single word, answer lands, reader moved on, markup as text, unavailable, limit reached).
- [ ] 3.4 Refuse to send a sentence over the bound on the client side and show the unavailable message. Verify: a card test with a 1 001-character sentence that asserts no message reaches the background.
- [ ] 3.5 Build every variant. Verify: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` with coverage, `yarn build` and `yarn check:variants`.

## 4. Integration

- [ ] 4.1 With the flag off on a local backend, press the button in Chrome, Firefox and Safari. Verify: each shows "unavailable for now", and the server log shows no provider call.
- [ ] 4.2 With the flag on and the setting unticked, read a chapter. Verify: the browser's network log shows no request to the translation RPC.
- [ ] 4.3 With the flag on, press the button on the 22-sentence corpus of design.md (six pairs, including a split rendering), once with each provider forced primary, in each interface language (fr, en, es). Verify: every answer is shown with its marks, the card's copy follows the interface language, and the server log for these requests holds no sentence.
- [ ] 4.4 Run `openspec validate add-lingua-online-translation --strict`. Verify: it passes.

## 5. Turning it on

- [ ] 5.1 Set the provider keys in the production environment and a spend cap on each provider account. Verify: `docker inspect` on the server container shows the variables are set.
- [ ] 5.2 Publish the policy (group 1) before the first extension release that contains the button. Verify: the live annex at `https://cymbra.app/confidentialite/` names « Traduction en ligne ».
- [ ] 5.3 After the release reaches readers, create the Lingua feature campaign `online-translation` in the back office, set `lingua.online_translation.enabled` to scope `beta:online-translation`, and enrol the first readers by handle (those who asked, e.g. on Discord). Watch the first day's provider failures and token counts. Verify: from a NON-staff account not enrolled, the setting is absent; from an enrolled one, it appears and answers; the server log shows answered requests with no sentence.
- [ ] 5.4 Rehearse the kill switch: set the flag off. Verify: an enrolled non-staff reader's button answers "unavailable" with no provider call, and the setting disappears at the next flag read.
