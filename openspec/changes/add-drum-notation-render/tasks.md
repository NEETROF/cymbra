## 1. Shared crate — the one classification

- [x] 1.1 Verify the Bravura assets **first**: `unpitchedPercussionClef1`
      (U+E069) and the x-head series (`noteheadXWhole`/`noteheadXHalf`/
      `noteheadXBlack`, U+E0A7–U+E0A9) exist in the app's bundled font and the
      console's same-origin subset — a missing glyph renders as tofu, and
      re-subsetting is cheap now and disruptive after the painters land
- [x] 1.2 Derive the engraved head class in `crates/musicxml-core` beside the
      resolved General MIDI number on `Unpitched` — one table, 0-based GM keys:
      cymbals (42, 44, 46, 49, 51, 52, 53, 55, 57, 59) → x form (44, the pedal
      hi-hat "chick", is a cymbal sound and engraves as x by convention — its
      kit-view `other` role is a lane question, not a head question), 46
      additionally the open variant, everything else (unresolved included) →
      ordinary oval. Serde carries it to the console, frb to the app; neither
      painter owns GM ranges
- [x] 1.3 Crate tests: the cymbal set classifies as x, the drums as oval, 46 as
      open, unresolved as oval; the field serializes under the `serde` feature
      (the wasm JSON contract)
- [x] 1.4 Regenerate the app bridge (`flutter_rust_bridge_codegen generate` —
      crate public API changed) and rebuild the console wasm (`yarn gen:wasm`);
      update `apps/back-office/src/lib/notation/geometry.ts` with the new field's
      type. If anything later "looks unchanged" in the console, check the wasm
      mtime before debugging the painter — a stale local wasm renders against
      the old parser silently

## 2. Console painter (`apps/back-office`)

- [x] 2.1 `smufl.ts`: percussion clef and x-head glyphs; give the percussion
      sign an explicit `clefGlyph` mapping — the current default-to-treble
      fallback would silently draw a G clef and no "a clef is drawn" assertion
      would catch it
- [x] 2.2 `painter.ts`: retire the `percussion_unsupported` result and the
      `RenderResult` union arm; keep `isPercussionScore` and repurpose it to
      route percussion documents to the percussion paint path (a clef-less drum
      export still needs the routing)
- [x] 2.3 Percussion paint path: single staff, percussion clef, **no** armature
      and no key-change naturals (even when `fifths ≠ 0`), time signature as
      usual; unpitched heads placed from `display_step`/`display_octave`
      through the existing `yForPitch` diatonic machinery with the treble
      reference; ledger lines unchanged; unresolved-GM notes still engraved
- [x] 2.4 Heads from the crate's head class — x forms following the duration
      class like ordinary heads, the open mark above GM-46 heads; stems from
      the file's `<stem>`, else voice 1 up / voice 2 down; beam groups stay
      keyed on `staff_voice`; rests displaced by voice in two-voice measures
      (voice 1 above the middle line, voice 2 below), midline in single-voice
      measures; same-position shared-onset heads offset so neither hides
- [x] 2.5 Retire the unpreviewable state's consumers: `ScorePreview.vue`,
      `ScoreDetailView.vue`, `ReviewView.vue` — the state disappears; the
      decode/parse/oversize failure states and the **Play guard stay exactly
      as they are** (the guard is `add-drum-audio-channel`'s to lift)
- [x] 2.6 Vitest over the real fixtures (`groove_ouvert` at minimum): SVG
      asserts the percussion clef glyph (not merely "a clef"), snare C5 on the
      third space and kick F4 in the bottom space, x heads on hi-hat/crash and
      oval on snare/kick, no accidental glyphs from a `fifths ≠ 0` synthetic
      variant, voice-2 rest below the middle line, both heads present on a
      coinciding kick + hi-hat onset

## 3. App — schedule mirror and Staff mode

- [x] 3.1 `notation_playback.dart`: plumb `voice` onto `TimedRest` (TimedNote
      has it since `add-drum-kit-view` 1.6; rests do not yet), so the Staff
      painter can displace rests by voice
- [x] 3.2 `staff_painter.dart`: percussion clef glyph in `smufl.dart` (same
      default-to-treble trap as the console); place unpitched notes by
      `TimedNote.diatonic` (already filled from the display position) and
      **never** fall back to the MIDI slot for a percussion note — the GM
      number is a sound, not a position; no armature on a percussion staff
      (skip the armature and modulation redraws when the clef is percussion)
- [x] 3.3 Heads from the bridged head class (x forms by duration class, the
      open mark on GM 46); stems file-first then voice; rests displaced by
      voice in two-voice measures; hands/feet colours per `hand-color-coding`
      (voice convention, single-voice GM fallback) with the playhead emphasis
      unchanged
- [x] 3.4 Widget tests on the fixture scores: percussion clef drawn, snare/kick
      positions, x vs oval heads, open-hi-hat mark present for 46 and absent
      for 42, voice-2 rest below the midline, hands blue / feet amber, hand
      filter (hands) hides foot events without touching the staff furniture
- [x] 3.5 Confirm the StaffPainter reuse surfaces inherit the path — the
      in-card rating preview and the upload preview render a percussion score
      correctly at their smaller `noteScale` (widget test on one), since the
      rating deck already deals percussion to eligible raters

## 4. App — Partition mode

- [x] 4.1 `partition_painter.dart`: the percussion paint path — single
      percussion-clef staff, no armature/key-change naturals, written-position
      placement, crate head classes, open mark, file-first stems then voice,
      per-voice rests, shared-onset offsetting — mirroring §2.3/2.4 so the two
      painters stay rule-identical
- [x] 4.2 Scope the staff-collapse rule to keyboard scores: for percussion the
      hand filter hides **events** by the hands/feet voice convention and the
      single staff (lines, clef, time signature) is always drawn — assert by
      test that selecting **hands** never blanks the canvas
- [x] 4.3 Hands/feet colours on engraved heads, playhead emphasis and the
      Paper theme's darkened palette applying unchanged
- [x] 4.4 Both `PartitionPainter` instantiation sites of the Partition screen
      (the height measurer and the paint instance, `player_screen.dart:1516`
      and `:1572`) and the measure-select screen work on a percussion score —
      `heightFor` reflects the single staff, range selection taps still resolve
      written measures
- [x] 4.5 Tests: fixture-driven paint assertions as in §3.4, plus a
      `fifths ≠ 0` synthetic percussion score drawing no armature, and a
      measure-select smoke test on a percussion score

## 5. Mode re-offer

- [x] 5.1 Lift the toggle restriction (`player_screen.dart:940`): a percussion
      score offers the same mode set a keyboard score gets on the same device
      (the phone's Partition→Staff remap included); the cascade remains the
      **default** on load
- [x] 5.2 Wait Mode stays not-offered for a percussion score — untouched here,
      asserted by an existing-behaviour test so this change cannot regress the
      `add-drum-scoring` interim
- [x] 5.3 Tests: toggle offers cascade/Staff/Partition for percussion and the
      default stays the cascade; keyboard scores unchanged; switching to Staff
      or Partition and back preserves playback state like a keyboard score

## 6. Drift pinning

- [x] 6.1 App test pinning the two tables' overlap: every GM number
      `drum_kit.dart` assigns a cymbal role (hiHat/ride/crash) classifies as an
      x head in the bridged head class, and no drum-role number does — the
      gameplay and engraving tables cannot disagree silently
- [x] 6.2 Parity check as a test artifact: the same fixture rendered by the
      console painter (SVG) and asserted in the app painters covers the same
      facts (clef glyph, positions, head classes), so a rule change that lands
      on one side only breaks a named test on the other

## 7. Gates

- [x] 7.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [x] 7.3 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 7.4 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 7.5 BO — `yarn test` and the Playwright e2e (pass `BO_E2E_PORT` to avoid
      colliding with another worktree's dev server)
- [x] 7.6 `flutter_rust_bridge_codegen generate` ran (crate public API changed)
      and the regenerated bridge is committed; `yarn gen:wasm` documented as a
      local step (the wasm itself is not committed)
- [x] 7.7 `openspec validate add-drum-notation-render --strict`

## 8. Manual verification

- [ ] 8.1 Drive all four drum fixtures through Staff and Partition on a phone
      and a tablet: clef, positions, x heads, two voices, rests — and the
      cascade still the default on load
- [ ] 8.2 Open the same fixtures in the console preview and compare
      side-by-side with the app's Partition: same content and layout (the
      `web-notation-render` faithfulness contract), no unpreviewable state
      anywhere, Play still refusing with its localised state
- [x] 8.3 Paper theme: x heads, the open mark and the hands/feet colours hold
      their contrast on ivory (the amber-on-ivory risk flagged by
      `add-drum-kit-view` applies to engraved heads too) — VALIDÉ 2026-08-24 : thème Papier validé sur appareil : têtes en croix, marque d'ouvert et couleurs mains/pieds tiennent
- [ ] 8.4 Legibility judgement calls recorded back into `design.md`: the open
      mark at in-card preview scale, and the open-x half/whole forms at staff
      scale (fallbacks are recorded in Open Questions)
- [ ] 8.5 With the drummer tester: read a two-voice groove from the Partition
      view and confirm the voices and the open/closed hi-hat marks read
      correctly at practice distance
