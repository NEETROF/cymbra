## 1. Repo write (backend)

- [ ] 1.1 Add `set_pricing(id, point_cost: i64, redeemable: bool) -> Result<bool>` to `SoundFontRepo` (`backend/music/src/soundfont.rs`): `UPDATE music.soundfonts SET point_cost = $2, redeemable = $3 WHERE id = $1`, returning whether a row matched. Implement on `PgSoundFontRepo` **and** `FakeSoundFontRepo`.
- [ ] 1.2 Unit-test the fake: set pricing flips `point_cost`/`redeemable` on `lookup`, leaves other fields untouched, and returns `false` for an unknown id.

## 2. Pricing RPC (backend)

- [ ] 2.1 Proto: add `SetSoundFontPricing(SetSoundFontPricingRequest{id, int32 point_cost, bool redeemable}) -> SetSoundFontPricingResponse` to `ScoreService`; add `point_cost` + `redeemable` to `AdminSoundFont` (admin listing only — NOT to the public `SoundFont`).
- [ ] 2.2 Host-testable decision: authenticate → require music-scope **admin** (`guard::require_admin`) → reject `point_cost < 0` (`InvalidArgument`). Cover every branch (admin+valid → ok; moderator → forbidden; negative → invalid).
- [ ] 2.3 Handler `set_sound_font_pricing`: run the decision, then `repo.set_pricing(...)`; unknown id → `NotFound`. Music-scope moderator/admin gate mirrors the other soundfont admin RPCs.
- [ ] 2.4 Populate `point_cost`/`redeemable` on the `AdminSoundFont` rows in `admin_list_sound_fonts` (from the font row); assert in the existing admin-listing test.
- [ ] 2.5 Regenerate gRPC stubs (Rust build.rs; back office `yarn gen`; app `melos run gen-grpc`).

## 3. Back office (Vue)

- [ ] 3.1 Add a `setSoundFontPricing(id, pointCost, redeemable)` action to the soundfonts store behind the injectable `api()` client seam; model its result as an `Async<T>` union and surface success/error via the toasts.
- [ ] 3.2 On the Sound fonts admin screen, show each font's current price (`point_cost`/`redeemable`) and, **for an admin**, a control to change them (cost input + redeemable toggle). Hidden/disabled for a non-admin.
- [ ] 3.3 When pricing a font that has `has_preview = false`, show a non-blocking hint that the app will grey its play until a sample is generated.
- [ ] 3.4 Vitest: the store action (success + error) via the client seam; the admin gate (control absent for a moderator).

## 4. Verification & gates

- [ ] 4.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (pricing decision + repo fake covered; RPC/pg glue excluded as usual).
- [ ] 4.2 App: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test` green (no app behavior change, but stubs regenerated).
- [ ] 4.3 Back office: `yarn lint` + `yarn test` (vitest) + `yarn format:check` green; `yarn dart format`/`prettier` clean.
- [ ] 4.4 Manual: as an admin, price a free accepted font (cost > 0) → a non-entitled user's download is 404, the app locks it and auditions the preview, and it appears redeemable in the shop; redeem → download works; set cost back to 0 → free again. A moderator sees no pricing control.
- [ ] 4.5 `openspec validate add-soundfont-reward-pricing --strict` passes.
