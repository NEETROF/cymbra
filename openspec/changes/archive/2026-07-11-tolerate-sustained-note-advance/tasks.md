## 1. Gate logic — seed unconsumed held pitches

- [x] 1.1 Add a `consumedHeld` set to `PlayerData` ([player_data.dart](apps/music/lib/state/player_data.dart)): currently-held pitches whose current press has already satisfied an onset.
- [x] 1.2 In `advance()` ([player_notifier.dart](apps/music/lib/state/player_notifier.dart)), detect when the playhead first reaches a new onset and seed `gateSatisfied` with `onsetPitchesAt(onset) ∩ activeNotes ∩ ¬consumedHeld` before evaluating the freeze condition; add each seeded pitch to `consumedHeld`.
- [x] 1.3 In `noteOn()`, when a fresh press latches a pitch into `gateSatisfied`, also add it to `consumedHeld`; and always clear the pitch from `consumedHeld` on a new note-on (a fresh attack starts a new, uncounted hold).
- [x] 1.4 In `noteOff()`, remove the pitch from both `activeNotes` and `consumedHeld` so the ended hold can be re-attacked cleanly.
- [x] 1.5 Ensure seeding happens exactly once per onset arrival and reuses the same `onsetPitchesAt` definition the freeze check uses, so held-tolerance and freezing agree on "the active onset".
- [x] 1.6 Verify the existing latch semantics still hold: a seeded/held pitch stays in `gateSatisfied` if released after the playhead arrives but before the next tick (no re-block).

## 2. Tests

- [x] 2.1 Add a wait-mode test: pitch pressed **before** the onset and **still held** when the playhead arrives (not a repeat) → gate releases without a re-press (maps to "Pitch held through the onset satisfies").
- [x] 2.2 Add a test: pitch pressed **and released** before the onset → gate still freezes at the onset until the pitch is down again (maps to "Early press-and-release does not pre-satisfy").
- [x] 2.3 Add a chord test: one pitch held from before the onset (not consumed) + the remaining pitches pressed while active → gate releases (maps to "Mix of held and freshly pressed pitches"); and a partial-chord case stays frozen.
- [x] 2.4 Keep/adapt the existing "repeated pitches require fresh attack" test so it still passes: the same pitch required at two consecutive onsets, held continuously, must NOT satisfy the second onset without a re-press (maps to "Repeated pitch held across onsets requires a fresh attack").
- [x] 2.5 Add a test for the non-adjacent repeat: pitch required at onsets N and N+2 (not N+1), held continuously → N is satisfied by the hold, N+2 stays frozen until re-attack.
- [x] 2.6 Re-run and keep green the existing wait-mode tests in `test/player_notifier_test.dart` and `test/state/wait_onset_test.dart` (press-releases-gate, early-press-not-pre-satisfy for the release case, chord onsets).

## 3. Validation & gates

- [x] 3.1 `openspec validate tolerate-sustained-note-advance --strict` passes.
- [x] 3.2 `cd apps/music && dart run build_runner build --delete-conflicting-outputs` then `melos run analyze` + `dart format` clean; `dart run custom_lint` passes.
- [x] 3.3 `flutter test --coverage --exclude-tags golden` passes with line coverage ≥ 80%.
- [x] 3.4 Manually confirm on device/simulator (Wait Mode on): pressing a note slightly early and holding it advances the score when the playhead reaches that onset.
