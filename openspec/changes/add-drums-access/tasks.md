## 1. Flag declaration

- [x] 1.1 `backend/feature-flags/src/registry.rs`: declare **`drums.enabled`** (app `music`, type bool, code default **off**) — the key follows the registry's `<feature>.enabled` convention (`rating.enabled`, `onboarding.enabled`) — with a doc line naming the intended `beta:midi-drums` scope
- [x] 1.2 Confirm the registry's existing "declared defaults match declared type" and "every declared category is owned by a feature" tests still pass
- [x] 1.3 Do NOT create a stored override yet — it goes on only after the campaign exists (task 9.2)

## 2. Schema and backfill — additive first, re-derived second

- [x] 2.1 Migration: add `instrument TEXT NOT NULL DEFAULT 'unknown'` to `music.catalog_scores` and `music.user_scores`, with a CHECK constraining it to `keyboard` | `percussion` | `unknown`. NOT NULL by construction — the column never holds NULL, so the gate and the instrument filter test exactly one undeterminable value. The migration does nothing else: the tables hold only an `object_key`, so no SQL can classify a row
- [x] 2.2 Build the one-shot **application-level re-derivation pass** (an admin command or worker job in `backend/music`, idempotent so it can resume): stream each row's object from the store, parse it with the **new** classifier from `add-unpitched-notation`, update the row; a row whose bytes cannot be read or parsed stays `unknown`. Re-derive — do NOT translate `is_piano`. Verified on prod 2026-08-23: the corpus already holds ~252 scores containing unpitched notes, ~51 of them percussion-first and ~29 pure drum parts (`Drumset`, `Snare Drum`, `Schlagzeug`), so a flag translation would record real percussion rows as `unknown` and the gate would serve them to everyone
- [x] 2.3 Sequence it: the pass runs after the migration and **before the flag override goes on** (9.3) — the gate is not a boundary until every row has been re-derived (see 4.13)
- [x] 2.4 Index `instrument` on `catalog_scores` if the search predicate needs it (check the existing search index shape first — do not add a redundant one)
- [x] 2.5 Do NOT drop `is_piano` in this migration; it goes in 7.1 once no reader remains

## 3. Backend + crates — instrument replaces the proxy

- [x] 3.1 `repo.rs` / `ScoreMeta`: replace `is_piano: bool` with the instrument, sourced from the parser's classification
- [x] 3.2 `pg.rs`: update `META_COLS`, `bind_meta` and `meta_from_row` (the canonical bind order — all three must move together)
- [x] 3.3 `pg.rs:387`: replace the `is_piano` search predicate with the instrument predicate
- [x] 3.4 `catalog_search.rs`: replace `is_piano` across its 7 sites (params, row, builder helpers, the in-memory filter)
- [x] 3.5 `grpc.rs:1001` and `module.rs:457`: carry the instrument through to the wire types
- [x] 3.6 `meta.rs` in `musicxml-core`: `note_count` counts unpitched notes, and `validate()` therefore accepts a percussion score — this is the gate opening
- [x] 3.7 `meta.rs` in `musicxml-core`: remove `ScoreSummary.is_piano` (`staves >= 2`) and expose the instrument classification on the summary instead — `add-unpitched-notation` deliberately leaves this field untouched, so its retirement is owned **here**
- [x] 3.8 `crates/score-crawler`: switch every `is_piano` site — `metadata.rs:38`, `manifest.rs:60`, `catalog.rs:78`, `crawl.rs:265`, `output.rs:171` — to deriving and writing the instrument; the manifest stops carrying `is_piano` (see the `corpus-manifest` delta)
- [x] 3.9 Confirm the crawler now ingests a percussion score end to end, with **no** instrument condition added: its admission rule stays the redistributable-licence whitelist alone, and a crawled drum score is classified, stored and withheld by the same enforcement as any other

## 4. Backend — enforcement (the part that must not have a hole)

- [x] 4.1 Wire `FlagService` into `MusicModule`, following `notifications/src/dispatch.rs:38`; read from the hot in-memory store, default every unreadable value to disabled
- [x] 4.2 Build the `EvalContext` from `AuthIdentity.roles` (staff) and the existing `PlanSource` snapshot's `beta_keys()` (memberships); never from any request field
- [x] 4.3 One shared predicate `caller_may_see_percussion`, failing closed on an unreadable flag store, an unwired `PlanSource`, or unresolvable memberships
- [x] 4.4 Gate `SearchCatalog` — as a `WHERE` predicate, not a post-filter, so paging counts stay correct
- [x] 4.5 Gate the by-id reads: `GetCatalogScore` (full metadata — title, composer, facets), `GetCatalogScoreBytes`, and `GetScoreBytes` for a score the caller does not own — each answers as for an unknown id, never a distinguishable "forbidden"
- [x] 4.6 Gate `ListSavedCatalogScores`. `ListMyScores` is **not** gated — a caller's own uploads are always listed (see 4.7)
- [x] 4.7 Own-uploads carve-out: listing, fetching and deleting one's **own** user scores work regardless of eligibility (no disclosure to self; prevents quota lock and the invisible-but-deletable incoherence), while `ProposeScore` for a percussion score requires eligibility
- [x] 4.8 Gate `UploadScore` — refuse a percussion upload from an ineligible caller with a **typed** refusal the app maps to a localised reason (this is an accepting path: the caller holds the file, so not-found would be incoherent)
- [x] 4.9 `pg.rs:796`: the rating-deck predicate becomes `instrument <> 'percussion' OR eligible` — keyboard **and `unknown`** rows for every rater, percussion only for the eligible (the old proxy's exclusion of single-staff and unclassified rows was its bug, not intent) — and the same predicate gates `GetRatingPreviewBytes` (today it serves any pending/accepted score's bytes to any signed-in caller, bypassing the deck listing) and `SubmitScoreRating`
- [x] 4.10 Gate the existence oracles: `SaveCatalogScore`, `SubmitScoreRating` and `UnlockCatalogScoreForToday` answer as for an unknown id when the target is percussion and the caller ineligible — success-versus-not-found is itself a disclosure
- [x] 4.11 Gate the HTTP audio-preview route `GET /scores/{catalog_id}/preview`: for an ineligible caller a percussion score's preview answers not-found, indistinguishable from an absent preview — its 200-vs-404 is an existence oracle and its body an audible clip. And the preview **render** job skips percussion scores at acceptance (the server-side twin of the console Play guard): no piano-font clip of a drum part is ever baked, so the route has nothing wrong to serve; `add-drum-audio-channel` lifts the skip
- [x] 4.12 Interim scoring is off: the ingest sites for play rewards, per-piece and global leaderboards, streak accounting and daily-access consumption each **fail closed** on `instrument = 'percussion'` until `add-drum-scoring` — these artifacts are permanent (ledger rows, monotone bests, badges) and keyboard-shaped scoring of a drum part would bake wrong data
- [x] 4.13 A test per gated path for the ineligible case — search, metadata by id, catalog bytes, user bytes, saved listing, upload, deck sourcing, rating-preview bytes, rating submission, save, daily unlock, HTTP preview, and the scoring ingest sites; a predicate tested once in isolation does not prove the call sites use it. Plus the carve-out tests: own drum uploads listed/fetched/deleted by an ineligible owner, propose refused
- [x] 4.14 Test that an `unknown`-instrument score is still served to an ineligible caller (the no-regression invariant), and that a `percussion` one is not. Note the ordering constraint: the gate is only a boundary once the re-derivation (2.2) has run — before it, real percussion rows are still recorded `unknown`

## 5. Wire protocol

- [x] 5.1 `score.proto`: retire `SearchCatalogRequest.is_piano` — `reserved 6; reserved "is_piano";` — and add the instrument filter under a **fresh** field number (never reuse 6: an old client's boolean varint would misdecode under the new field). Shipped clients that still pin `isPiano: true` silently lose the filter and see unfiltered results, including `unknown` rows — accepted and stated in the design. Add the instrument to `CatalogHit` and `ScoreRecord`
- [x] 5.2 Regenerate the app client (`melos run gen-grpc`) and the console client (`yarn gen`)
- [x] 5.3 `apps/music/rust/src/api/musicxml.rs`: the bridged `ScoreSummary` swaps `is_piano` for the instrument classification; run `flutter_rust_bridge_codegen generate` (public API change) so the upload Verify summary (6.4) can show the detected instrument

## 6. Front ends

- [x] 6.1 App — `catalog_search_notifier.dart:270`: stop pinning `isPiano: true`; carry the instrument filter instead
- [x] 6.2 App — `catalog_service.dart:131`: replace `isPiano` with the instrument in the filter model and the request mapping
- [x] 6.3 App — Score Hub filter UI: offer the instrument, listing the drum option only when the drum feature is visible
- [x] 6.4 App — upload: show the detected instrument in the read-only Verify summary (`score_upload_notifier.dart:73` carries the bridged summary once 5.3 lands); add no control to change it
- [x] 6.5 App — upload: refuse a percussion file when the feature is not visible, with a localised reason (fr/en), read through the notifier and surfaced via state — never a raw technical string
- [x] 6.6 App — score card / library: show the instrument; an `unknown` instrument shows nothing rather than an "unknown" label
- [x] 6.7 App — opening a percussion score shows a localised "drums not playable yet" state (fr/en) instead of entering the player — the app-side mirror of the console guard, removed by `add-drum-kit-view`. Without it the waterfall would draw the GM key numbers as falling piano notes and the piano synth would sound them
- [x] 6.8 BO — `FiltersBar.vue:55`: replace the piano checkbox with an instrument filter
- [x] 6.9 BO — `stores/catalog.ts:26` and `CatalogView.vue:50`: carry the instrument through; keep async state as the existing `Async<T>` union
- [x] 6.10 BO — confirm moderators see percussion scores by construction (they are staff), in both the catalog and the review queue
- [x] 6.11 BO — badge the instrument on the review-queue row (`ReviewView.vue`): a percussion proposal must be identifiable as such before it is opened, since a moderator judging it needs to know why the preview is unavailable
- [x] 6.12 BO — `lib/notation/painter.ts` (the console's own TypeScript painter, independent of the app's Dart one): detect a percussion score and return an explicit "not previewable yet" state instead of drawing it with treble/bass assumptions
- [x] 6.13 BO — `ScorePreview.vue`: render that state as its own case in the existing `Async<T>` match, visually distinct from undecodable/unparseable so a moderator does not read a fine file as corrupt; localise the copy (fr/en)
- [x] 6.14 BO — same guard on the Play control: a percussion score must not be auditioned through the piano-channel renderer (`audio-wasm` still hardcodes `PIANO_CHANNEL = 0`), so it would play silence or nonsense; present a localised "not auditionable yet" affordance, not an error

## 7. Retire the proxy

- [x] 7.1 Migration: drop `is_piano` from both tables, once 3.x, 5.x and 6.x leave no reader
- [x] 7.2 Grep the whole repo for `is_piano` / `isPiano` and confirm only generated files remain — the crawler crate (3.8), the `musicxml-core` summary (3.7) and the frb bridge (5.3) are the sites this grep exists to catch

## 8. Gates

- [x] 8.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 8.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [x] 8.3 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 8.4 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 8.5 BO — `yarn test` and the Playwright e2e (pass `BO_E2E_PORT` to avoid colliding with another worktree's dev server)
- [x] 8.6 `openspec validate add-drums-access --strict`

## 9. Operations and manual verification

- [ ] 9.1 Apply the migration on staging, run the re-derivation pass (2.2), and confirm it: two-staff keyboard rows read `keyboard`, unreadable rows read `unknown`, and the known percussion rows read `percussion` — spot-check against the ~29 pure drum scores identified on prod (all `pending`, all `PublicDomain`)
- [x] 9.2 Create the `midi-drums` beta campaign (`CreateCampaign`, kind `feature`) in the admin plan console, then get testers into it — either `EnrolHandle` per account, or `MintCodes` and have each tester redeem — and confirm with `ListMembers` that they are actually members — VALIDÉ 2026-08-24 : campagne `midi-drums` créée + codes frappés + rachat sur le site, `ListMembers` confirme `mickyoun` (2026-08-24)
- [x] 9.3 Set the flag override to `beta:midi-drums` from the back-office flags panel — only after 9.1 confirms the re-derivation — spelling the key **exactly** as the campaign's: nothing validates today that a `beta:<key>` scope names an existing campaign (`RolloutScope::parse` checks the shape only, and the flags service has no knowledge of plans), so a typo stores cleanly and matches nobody, forever, silently. `add-flag-campaign-integrity` closes exactly this hole with write-time validation — this rollout is its first beneficiary — and once it lands this manual spelling care is superseded; until then it is the only guard — VALIDÉ 2026-08-24 : override `drums.enabled` = `beta:midi-drums` posé depuis le back-office ; l'intégrité d'écriture d'`add-flag-campaign-integrity` refuse désormais une clé inexistante
- [x] 9.4 Verify the wiring from a **non-staff** tester account. An admin or moderator session proves nothing: `Beta(key) => self.staff || self.betas.contains(key)`, so staff match every beta scope whether the campaign exists, is empty, or is misspelled — VALIDÉ 2026-08-24 : vérifié depuis `mickyoun` (Role.User, non-staff)
- [x] 9.5 As a **non**-member on a premium plan: confirm no drum option in the hub, no percussion score in search, a direct fetch by id answers as unknown, the rating deck never deals one, the audio-preview route 404s, and a drum upload is refused — VALIDÉ 2026-08-24 : non-membre : aucune option batterie, aucune partition percussive, deck sans carte batterie ni chips instrument (2026-08-24)
- [x] 9.6 As a campaign member: confirm the drum option appears, a drum score uploads, it is found in search, and opening it shows the "drums not playable yet" state — and that the run-less score engages no reward, streak or daily-access artifact — VALIDÉ 2026-08-24 : membre : option batterie visible, partition batterie ouverte et JOUÉE — l'état « pas encore jouable » ne s'applique plus, la chaîne audio/entrée/scoring ayant landé
- [ ] 9.7 As a moderator: confirm the console shows percussion scores, the instrument filter works, the review row is badged, and opening one shows the "not previewable yet" state rather than a broken rendering or a silent Play
- [x] 9.8 Confirm withdrawal works: close the campaign and check its members lose access at the next evaluation, with no flag edit — and that a former tester still sees, fetches and can delete their **own** drum uploads while proposing one is refused — then re-open/re-enrol for the rest of the beta — VALIDÉ 2026-08-24 : campagne fermée sans toucher au drapeau, les membres perdent l'accès ; un ancien testeur voit, télécharge et supprime encore SES propres uploads batterie tandis que la proposition au catalogue lui est refusée ; réouverture faite depuis la console
- [ ] 9.9 Record the general-availability sequence where the operator will read it (runbook or the flag's doc line): **widen the scope to `global` FIRST, close the campaign SECOND**. `beta:<key>` matches staff *or* member, so closing the campaign while the scope is still `beta:` drops every tester at once and leaves only staff seeing the feature — a broken rollout produced by an action that looks correct
- [ ] 9.10 **Prerequisite for general availability, not for the beta:** bundle drum scores in `assets/scores/` so the core loop is playable without an account, as `welcome-onboarding` requires for keyboard today. **Author them** rather than source them — that is what the existing bundled scores are (`assets/scores/CREDITS.md`: "authored for this project and released under the repository's licence"), and a basic groove is an idiom, not a copyrightable work, so the licence question that dogs the drum repertoire does not arise. Do **not** promote the ~29 public-domain drum scores found in the catalog: they are crawler-classified and `pending`, never human-reviewed, and the bar for shipping bytes inside the binary is higher than for holding them in a moderated catalog ("drop anything whose licence cannot be confirmed"). Cover the beginner/intermediate/advanced tiers by authoring at each level, which also sidesteps the unresolved drum-difficulty question. Between them they should exercise unpitched notes, the part-list instrument table, the percussion clef, two voices on one staff, and open/closed hi-hat; record each in `CREDITS.md`. Lands with `add-instrument-context`, which is what makes the drum path reachable from a first launch
- [x] 9.11 Confirm a keyboard score is unaffected throughout — upload, search, hub, rewards/leaderboards/streak/daily access, and console behave exactly as before — VALIDÉ 2026-08-25 en **production** : la voie clavier se comporte exactement comme avant l'ouverture de la bêta batterie, la fonctionnalité batterie n'ayant rien déplacé sur son chemin
