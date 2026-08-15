## 1. Pure naming module

- [x] 1.1 Create `apps/music/lib/state/note_label.dart` with the degree/alteration
      model: `naturalPitchOf(diatonic)` (written degree `octave*7 + step` → MIDI
      natural) and an `Alteration` value derived as `pitch − naturalPitchOf(diatonic)`,
      clamped to the ♭♭…♯♯ range.
- [x] 1.2 Implement `noteLabel(TimedNote, {required bool solfege, required bool
      frenchRe, int keyFifths})` returning the degree name plus its alteration
      symbol, with no octave index.
- [x] 1.3 Implement the no-spelling fallback: when `diatonic == null`, name from
      `pitch` choosing sharp or flat spelling by the sign of the key signature in
      force (flat keys → flats, otherwise sharps).
- [x] 1.4 Move `_keyName` from `apps/music/lib/screens/pre_play_setup_modal.dart`
      into this module as a public function, delete the private copy, and update
      the modal to call it.
- [x] 1.5 Implement the figure token: `figureFor({String? noteType, required int
      dots, required int beatType, required double durationMs, double? beatMs})`
      returning figure kind + dots + an optional beat count, inferring the kind
      from duration when `noteType` is null and omitting the beat count when it
      does not land on a clean (half-or-better) value.
- [x] 1.6 Map figure kinds to their Bravura glyphs using the existing constants in
      `apps/music/lib/painters/smufl.dart` (`noteheadWhole`/`noteheadHalf`/
      `noteheadBlack`, `augmentationDot`).

## 2. Player state

- [x] 2.1 Extract the expected-set reference instant (onset under the playhead, or
      the upcoming onset while travelling) into one private helper in
      `apps/music/lib/state/player_data.dart`, and make `expectedKeys` and
      `expectedKeysForHand` both call it.
- [x] 2.2 Add `PlayerData.expectedNotes` → `List<TimedNote>` built from
      `visibleNotes` at that same instant, so the aid, the keyboard highlight and
      the gate can never disagree.
- [x] 2.3 Add the `NoteReadingAid` enum (`off`, `name`, `nameAndRhythm`) and mirror
      the selected level on `PlayerData`, seeded from preferences the way
      `metronomeEnabled` already is.
- [x] 2.4 Add the setter on the `Player` notifier that updates the mirrored value
      and writes through to the persisted preference.
- [x] 2.5 Run `dart run build_runner build --delete-conflicting-outputs` so the
      Freezed/Riverpod output for the new field and setter regenerates.

## 3. Persisted preference

- [x] 3.1 Add the reading-aid level to `PlayerPrefs`
      (`apps/music/lib/state/player_preferences.dart`) with `off` as the default.
- [x] 3.2 Extend `_encode`/`_decode` so the level round-trips, and a missing or
      unrecognized stored value falls back to `off` without discarding the other
      restored settings.
- [x] 3.3 Add the `setNoteReadingAid` mutator following the existing
      `setMetronome` shape (update + best-effort persist).

## 4. Localization

- [x] 4.1 Add ARB keys for the setting (label + the three level names) and for the
      figure prose (whole/half/quarter/eighth/sixteenth, dotted variants) plus the
      "hold N beats" string with a count placeholder, in `app_en.arb`.
- [x] 4.2 Translate the new keys in `app_fr.arb`, `app_es.arb` and `app_it.arb`.
- [x] 4.3 Add the bounded-chord summary string ("chord of N notes") in the four
      locales.
- [x] 4.4 Regenerate the localization output and confirm no missing-translation
      warnings.

## 5. Display (no layout space of its own)

- [x] 5.1 Create `apps/music/lib/widgets/reading_aid.dart`: the pure
      `readingAidViewOf` (a value type with structural equality so `select` does
      not rebuild every frame) plus the figure-to-prose mapping.
- [x] 5.2 Draw the awaited names on the keys themselves — extend
      `PianoKeyboardPainter` with a pitch→label map, and add `fitKeyLabel` so a
      label is laid out strictly inside its key, turning a quarter turn on a
      narrow key rather than overflowing onto its neighbours.
- [x] 5.3 Show the figure as a floating card over the bottom of the render area
      (a `Stack` sibling, so it takes no height), only at the `nameAndRhythm`
      level and only when every awaited note agrees on the figure.
- [x] 5.4 Gate both on Wait Mode actively blocking, so they appear only while the
      gate holds and are withdrawn as soon as the playhead resumes.
- [x] 5.5 Route the keyboard's octave anchors through the same naming module, and
      have an anchor yield to a reading-aid label on the same key.

## 6. Setup modal and in-game settings

- [x] 6.1 Add the three-level reading-aid control to the pre-play setup modal
      (`apps/music/lib/screens/pre_play_setup_modal.dart`), preselected from the
      persisted level and following the existing setting-row style.
- [x] 6.2 Confirm the same control is reachable from the in-game settings drawer
      (the same modal) and that a change made there updates the shared persisted
      value.
- [x] 6.3 Confirm Validate applies the chosen level and close (X) leaves it
      unchanged, matching the modal's existing apply/dismiss contract.

## 7. Tests

- [x] 7.1 Unit-test the naming module: key-signature alteration with no engraved
      accidental (F under one sharp → F♯), engraved accidentals, a natural
      cancelling the key signature, double alterations, and enharmonic preservation
      (written D♭ never named C♯).
- [x] 7.2 Unit-test the locale conventions: letters in English, solfège elsewhere,
      `Ré` in French vs `Re` in Spanish/Italian; and that no name carries an octave
      index.
- [x] 7.3 Unit-test the fallback path (`diatonic == null`): flat key → flat
      spelling, sharp key → sharp spelling, never an empty label.
- [x] 7.4 Unit-test the figure token: dotted half in 4/4 → three beats, inference
      from duration when `noteType` is null, and beat count suppressed when it does
      not resolve cleanly.
- [x] 7.5 Unit-test `expectedNotes`: agrees with `expectedKeys` on pitches, follows
      the selected hand, and returns the upcoming onset while the playhead travels.
- [x] 7.6 Unit-test preference round-trip: level persists, unknown/missing value
      falls back to `off` while other settings still restore.
- [x] 7.7 Widget-test the aid: no labels at `off`, names on the keys at `name`,
      figure card added at `nameAndRhythm`, both appearing only while blocked and
      withdrawn on resume, every note of a chord named, and mixed figures showing
      no figure.
- [x] 7.8 Widget-test that the render area and the keyboard keep exactly the same
      geometry with the aid off, enabled, blocked and released.
- [x] 7.9 Unit-test `fitKeyLabel`: upright on a wide key, turned on a narrow one,
      never exceeding the key box across a grid of sizes, dropped rather than
      drawn illegibly, and a real 88-key phone keyboard still labelled.

## 8. Gates

- [x] 8.1 `melos run analyze` and `dart format` clean.
- [x] 8.2 `dart run custom_lint` clean (Riverpod layering rules).
- [x] 8.3 `flutter test --coverage --exclude-tags golden` passing with line
      coverage ≥ 80 %.
- [x] 8.4 `openspec validate add-note-reading-aid --strict` passing.
- [x] 8.5 Manual check on a real score with a non-empty key signature: verify a
      key-signature-altered note is named with its alteration, and verify the aid
      on a phone in landscape.
