## 1. Theme

- [x] 1.1 Add `handRight` (blue) and `handLeft` (amber) to `CymbraColors`, distinct from `tertiary` (green) and `primaryContainer` (purple).

## 2. Keyboard

- [x] 2.1 `PianoKeyboardPainter`: add a `leftHandNotes` subset of `requiredNotes`; colour expected keys by hand (right→handRight, left→handLeft); keep correct=green, pressed=purple. Update `shouldRepaint`.
- [x] 2.2 `PlayerData.expectedKeysForHand({required bool rightHand})` splits the gate by staff; wire `leftHandNotes` from it in `player_screen.dart`.

## 3. Render modes

- [x] 3.1 Synthesia: colour falling notes by `staff` (brighter at the playhead, green when held).
- [x] 3.2 Staff: colour scrolling note heads by `staff` (brighter at the playhead, green when held).
- [x] 3.3 Partition: colour every note head by `staff`; playhead note green when held, else a brighter hand tint.

## 4. Tests & goldens

- [x] 4.1 Unit-test `expectedKeysForHand` (right→staff-1, left→staff-2, union = expectedKeys).
- [x] 4.2 Refresh the keyboard, Synthesia, Staff and Partition goldens.

## 5. Verify

- [x] 5.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean; `flutter test --exclude-tags golden` green; coverage ≥ 80%.
- [x] 5.2 `openspec validate hand-color-coding --strict` passes.
