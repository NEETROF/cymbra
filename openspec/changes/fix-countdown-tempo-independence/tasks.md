## 1. Make the countdown wall-clock

- [x] 1.1 In `apps/music/lib/screens/player_screen.dart`, `_onTick` passes the **raw** frame delta to `Player.advance` instead of `dt * speed`.
- [x] 1.2 In `apps/music/lib/state/player_notifier.dart`, `advance(dtMs)` now documents `dtMs` as real wall-clock frame time: the countdown branch subtracts it unscaled, and a local `musicalDtMs = dtMs * s.speed` drives the playhead. Everything downstream of the playhead reads positions, not deltas, so nothing else changes.
- [x] 1.3 Correct the stale `5…4…3…2…1…GO` doc comments in `player_data.dart`, `player_notifier.dart` and `countdown_overlay.dart` to the shipping `3…2…1…GO`.

## 2. Tests

- [x] 2.1 Notifier-level: the countdown consumes real ms at 0.25x and 2x, and the playhead advances at exactly `speed` (`apps/music/test/player_notifier_test.dart`).
- [x] 2.2 Screen-level (the seam the bug actually lived at): a widget test that drives the **real** Ticker at 0.25x in sub-100 ms frames and asserts the countdown drains on wall-clock time — verified to FAIL against the pre-fix code.
- [x] 2.3 Screen-level guard against double-scaling: pumping the real Ticker at 0.25x moves the playhead at `speed`, not `speed²` — verified to FAIL when the caller re-applies the speed.
- [x] 2.4 `flutter test --exclude-tags golden`, `flutter analyze`, `dart run custom_lint`, `dart format` clean.

## 3. Spec

- [x] 3.1 Delta spec: `gamified-feedback` / Pre-Start Countdown corrected to 3…2…1…GO and given the wall-clock requirement + scenario.
- [ ] 3.2 Archive after review/merge (`/opsx:archive fix-countdown-tempo-independence`).
