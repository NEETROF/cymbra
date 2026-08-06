## 1. Entitlement gate (backend)

- [x] 1.1 Extend `SoundFontRepo`'s font lookup row to surface `point_cost`, `redeemable`, and `uploaded_by` (add to the row struct + the runtime `SELECT`).
- [x] 1.2 Add a `has_grant(user_id, soundfont_id) -> bool` read to the rewards/soundfont repo (`SELECT 1 FROM music.curation_grants WHERE user_id = $1 AND soundfont_id = $2`).
- [x] 1.3 Create a host-testable `soundfont_access` core with `entitlement(caller, font, has_grant, is_music_mod_admin) -> Access` (`Allow` iff free ∨ own-import ∨ has_grant ∨ mod/admin), plus unit tests for every branch incl. the locked-deny case.
- [x] 1.4 Wire `entitlement(...)` into `serve` (`backend/server/src/soundfont.rs`): keep the moderation-visibility gate first, then compute `has_grant` + `is_music_mod_admin` and refuse a non-entitled caller with the **same not-found response as a missing font** (no existence oracle).
- [x] 1.5 Verify the in-app instrument load (`GET /soundfonts/{id}`) is now refused for a locked font (same gate; no separate path).

## 2. Server-side preview render (backend)

- [x] 2.1 Add `rustysynth` as a backend dependency (match the client-vendored version).
- [x] 2.2 Define the fixed `SampleSequence` (a short ~2–3s phrase, same for every font) as a covered constant/helper.
- [x] 2.3 Add a host-testable render core: `render_preview_pcm(font_bytes, &SampleSequence) -> Vec<i16>` (headless synth → PCM) and `encode_preview(pcm, rate) -> Vec<u8>` (WAV/PCM). Cover the sequence + encoder; leave the raw synth call as excluded glue.
- [x] 2.4 Unit-test render determinism (same font+sequence → equivalent clip) and encoder output (valid WAV header, expected length).

## 3. Preview storage & lifecycle (backend)

- [x] 3.1 Add a **public** preview object namespace/key (`{id}.preview.wav`), distinct from the private font bytes.
- [x] 3.2 Hook the render into `upload` (`POST /soundfonts/{id}`): after storing the font, render + store the public preview; a render failure logs and is non-fatal (upload still succeeds).
- [x] 3.3 Add the admin-gated regenerate endpoint `POST /soundfonts/{id}/preview`: re-read stored font bytes → render → overwrite the public preview; return success/failure. Register the route.
- [x] 3.4 Add `GET /soundfonts/{id}/preview`: serve the public clip with **no entitlement gate**, moderation-visibility gate only; not-found when no preview exists. Register the route.
- [x] 3.5 Gate acceptance on a preview: `SetSoundFontModerationStatus` refuses the `accepted` transition (FailedPrecondition) unless the font's preview object exists; `pending`/`rejected` unaffected; unknown id still NotFound. Unit-test refuse/allow/reject.

## 4. Back office (Vue)

- [x] 4.1 Add a `regeneratePreview(id)` call behind the injectable client seam (`lib/api.ts` + `setClientsForTest`).
- [x] 4.2 Add a **"Generate sample"** action to the SoundFonts admin screen, its state modeled as an `Async<T>` union matched with `match(...).exhaustive()`; show in-flight/success/error.
- [x] 4.3 Vitest coverage for the store/composable action (success + error) via the client seam.
- [x] 4.4 Map an accept-time FailedPrecondition to a clear "generate a preview sample first" hint (i18n en/fr) instead of the generic error; vitest coverage.

## 5. App audition via preview clip (Flutter)

- [x] 5.1 Add a preview seam: fetch `GET /soundfonts/{id}/preview` and play the returned clip through a simple audio playback service (injectable, faked in tests).
- [x] 5.2 Rewire the catalog **play** button (`soundfonts_screen.dart::_togglePreview`) to audition via the preview clip instead of `soundFontSourceProvider.resolve` + local synth; keep start/stop semantics.
- [x] 5.3 Expose `pointCost`/`redeemable` (already on the reward view) so a locked font's play uses the preview and shows the unlock CTA; disable/grey play when no preview exists.
- [x] 5.4 Widget/state tests: locked font auditions via the preview clip (font bytes never resolved); absent-preview greys the control; owned/free still selectable & usable.

## 6. Coverage, gates & verification

- [x] 6.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (entitlement + render/sample/encoder cores covered; route/synth glue excluded via the usual regex).
- [x] 6.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [x] 6.3 Back office: `yarn lint` + `yarn test` (vitest) green.
- [ ] 6.4 Manual: upload a costed font → preview object created; as a non-entitled user, download is 404 but play (preview) works; redeem → download works; back-office "Generate sample" backfills a seeded font's preview.
- [x] 6.5 `openspec validate add-soundfont-entitlement-previews --strict` passes.
