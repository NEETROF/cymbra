## 1. Flag

- [x] 1.1 `backend/feature-flags/src/registry.rs`: `drums.enabled` defaults **on**; rewrite the `DRUMS_ENABLED` doc comment (general availability reached, kept as the kill-switch) and the flag's English `doc`
- [x] 1.2 Move `DRUMS_ENABLED` out of `feature_flags_default_off_and_are_safe` and assert it defaults on; update the service/grpc/context tests that assumed the default was off so each keeps testing what it meant (a `beta:` scope still restricts a flag; a global off override still hides it)
- [x] 1.3 `apps/back-office/src/i18n/flag-descriptions.ts`: French description of `drums.enabled` as a kill-switch

## 2. Backend gate

- [x] 2.1 `backend/music/src/module.rs` `caller_may_see_percussion`: evaluate with a `DRUMS_DEFAULT = true` pinned to the registry default by a test — `FlagService::bool` applies the **caller's** default, not the registry's, so flipping the registry alone would have opened the drums in the app but not on the server; update the doc comment (audience = every player while the flag is on); keep the unwired-flag-service case closed; tests: a non-member free player now reaches the feature under the default, and a global off override hides it for everyone including staff

## 3. App

- [x] 3.1 `apps/music/lib/main.dart`: the `drums.enabled` read falls back to **true**; rewrite the `drumsEnabledProvider` doc comment (the provider's own test default stays `false`)
- [x] 3.2 A test proving the app shows the drums before its first flag fetch and hides them when the fetched flag is off

## 4. Docs

- [x] 4.1 `backend/deploy/DEPLOY.md`: the feature-beta section's live example becomes a completed one (drums opened to all, flag kept as kill-switch, campaign closed), and the backfill warning no longer reads as a pre-launch step
- [x] 4.2 `backend/music/README.md` / any runbook text naming the drums a beta — none found beyond `DEPLOY.md` (the app's remaining "beta" mentions are historical code comments, no user-facing copy)

## 5. Rollout

- [ ] 5.1 Archive `add-drum-audio-channel` **before** archiving this change (same requirement, renamed here)
- [ ] 5.2 After deploy: in the back office, delete the `drums.enabled` override (or keep it global — both mean on) and check a non-member account still sees a percussion score
- [ ] 5.3 Close the `midi-drums` campaign in the back office

## 6. Verification

- [x] 6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, tests and coverage green
- [x] 6.2 `melos run analyze`, `dart run custom_lint`, `dart format`, `flutter test` green
- [x] 6.3 `openspec validate graduate-drums-from-beta --strict`
