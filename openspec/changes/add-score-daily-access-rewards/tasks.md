## 1. Daily-access core (backend)

- [ ] 1.1 Add a `free_quota` (opens/day) + `day_slot_cost` (points) config via `cymbra-feature-flags`, with a kill-switch (quota 0 / gate off).
- [ ] 1.2 Add per-user/day state: an `opened_today` set of catalog piece ids (with a paid/free marker) keyed by user + client-offset date; reuse the `play_core` date helper for the day key.
- [ ] 1.3 Create a host-testable `score_access` core: `decide_open(piece_id, opened_today, free_quota, free_used, is_subscriber) -> Open` (`Serve | ServeFree | NeedsPoints{cost} | NeedsSubscription`), with unit tests for every branch incl. re-open-is-free and subscriber-bypass.
- [ ] 1.4 Add `has_active_subscription(user) -> bool` as a stub seam returning `false` (documented for future billing).

## 2. Gate the player open (backend)

- [ ] 2.1 Wire `decide_open` into `GetCatalogScoreBytes` ([backend/music/src/grpc.rs]): keep the `add-catalog-access-limits` (abuse) cap first, then the freemium quota; on a served open, record the piece into `opened_today` (idempotent).
- [ ] 2.2 On `NeedsPoints`, refuse the MusicXML and return the locked state + day-slot cost + upsell placeholder signal (extend the response/proto).
- [ ] 2.3 Verify `ListRatingDeck` / `GetRatingPreviewBytes` (moderation) and `GetCatalogScore` (metadata) stay ungated.

## 3. Points day-slot unlock (backend)

- [ ] 3.1 Add an unlock RPC (e.g. `UnlockScoreForToday(catalogId)`): host-testable debit decision (enough spendable balance? already in today's set?), then an atomic tx = ledger debit (`reason=score_day_slot`, ref piece+day) + record piece into `opened_today` as paid. NO `curation_grants` row.
- [ ] 3.2 Unit-test the debit core: sufficient/insufficient balance, already-paid-today idempotency, next-day re-charge.

## 4. Score audio preview (backend, reuses soundfont render engine)

- [ ] 4.1 Add a host-testable helper: MusicXML → bounded (~N s) render sequence for a piece; reuse `render_preview_pcm`/`encode_preview` from `add-soundfont-entitlement-previews` with a default soundfont.
- [ ] 4.2 Add a **public** score-preview object namespace (`score-preview/{catalogId}.wav`).
- [ ] 4.3 Hook render into the **accept** transition: after accept, render + store the public preview; failure logs and is non-fatal.
- [ ] 4.4 Add `GetScorePreviewBytes(catalogId)`: serve the public clip, moderation-visibility only, no quota/points gate; not-found when absent. Add an admin-gated regenerate RPC/action.
- [ ] 4.5 Unit-test the sequence bounding + determinism; encoder output validity.

## 5. App (Flutter)

- [ ] 5.1 Surface each catalog item's access state (free-opens-left / opened-today / locked-needs-points / subscriber) from the gate response.
- [ ] 5.2 Locked-piece flow: show the strong unlock affordance; play button auditions `GetScorePreviewBytes` via the audio playback seam; grey it when no preview.
- [ ] 5.3 Unlock confirmation ("débloquer pour X points aujourd'hui ?") → call `UnlockScoreForToday`; on success open the piece; react to state (no await-and-branch).
- [ ] 5.4 Leave a placeholder upsell hook at the quota-reached / unlock moment (wired to the real offer once subscriptions exist).
- [ ] 5.5 Widget/state tests: quota-remaining display; locked piece auditions the clip and never fetches MusicXML; unlock spends points and opens; re-open same day is free.

## 6. Back office (Vue)

- [ ] 6.1 Add a `regenerateScorePreview(id)` call behind the injectable client seam (`lib/api.ts` + `setClientsForTest`).
- [ ] 6.2 Add a **"Generate sample"** action to the score admin screen, state as an `Async<T>` union matched with `match(...).exhaustive()`.
- [ ] 6.3 Expose a `has_preview` flag on the admin score listing (derivable without downloading clips) and add a **"no sample" filter** to the score screen to list only pieces missing a preview (backfill workflow).
- [ ] 6.4 Vitest coverage (success + error; filter) via the client seam.

## 7. Coverage, gates & verification

- [ ] 7.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (access + debit + sequence cores covered; gRPC/route/synth glue excluded).
- [ ] 7.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [ ] 7.3 Back office: `yarn lint` + `yarn test` green.
- [ ] 7.4 Manual: exhaust the daily quota → (N+1)th piece locked, audio preview plays, MusicXML refused; spend points → opens + re-open free same day; next day re-charges; back-office "Generate sample" backfills a preview.
- [ ] 7.5 `openspec validate add-score-daily-access-rewards --strict` passes.
