## 1. Kit-piece model and lane derivation

- [x] 1.1 Map General MIDI percussion numbers (35–81) to kit-piece roles: hi-hat, snare, toms (high→low), ride, crash and other accent cymbals, kick — 35 **and** 36 both resolve to the kick and both drive the bar; equivalent numbers (acoustic vs electric snare, the several crashes) collapse onto one role — build the table against real files, not from the GM list alone, and give an unmapped number its own generic piece in the terminal bucket rather than dropping it (no-silent-drop requirement; GM 44 included, as its own lane)
- [x] 1.2 Pure function: score → ordered lane layout, applying the sort rule (time-keeper, snare, toms high→low, remaining cymbals, then any other piece in stable ascending GM order) to the pieces **present**; the kick is excluded from the lanes by construction
- [x] 1.3 Encode the invariant as a test, not a comment: for any input containing a snare, position 1 is the continuously-struck piece and position 2 the snare, and adding pieces appends right without reordering positions 1–2; for a snare-less input the pieces close ranks leftward (empty buckets skipped, never reserved)
- [x] 1.4 Ride takes position 1 when the score has no hi-hat, and joins the cymbals when it does — the rule is keyed to function, not to the piece's name
- [x] 1.5 Expose the layout once on the player state, consumed by **both** the cascade and the pad strip; never derived twice
- [x] 1.6 Plumb the note's **voice** onto the Dart `TimedNote` in `notationToTimedNotes` (the parsed model already exposes `note.voice`; the timed model does not carry it) so the hands/feet split keys on voice — never on the `stemUp` proxy sitting right next to it — with the single-voice fallback by General MIDI number (35/36 and 44 → feet) stated in `hand-color-coding`

## 2. Cascade painter

- [x] 2.1 New percussion cascade painter beside `synthesia_painter.dart`; a percussion score never consults `keyboard_range.dart`
- [x] 2.2 Lanes at equal width over the available space, so a sparse kit gets wide lanes rather than a full kit with empty columns
- [x] 2.3 Kick as a full-width bar at its onset — **painted before the note layer**, thinner than a note, attenuated
- [x] 2.4 Open hi-hat drawn as a variant of the note inside the hi-hat lane (no bar, no extra lane)
- [x] 2.5 Hand notes take the hand colour, foot bars the foot colour, keyed to the **voice convention** stated in `hand-color-coding` (voice 1 hands / voice 2 feet; single-voice fallback by GM number) — never to staff, never to stem direction
- [x] 2.6 ~~Golden~~ **Pixel-probe** test on a **coinciding** kick + hand onset (platform-robust where a golden is pinned): the note reads as the hand colour over the bar, the bar stays visible in the clear and carries the foot colour. This is the one case that reveals an inverted paint order, and a score where feet and hands never align renders perfectly with the layers wrong
- [x] 2.7 ~~Golden~~ Derivation + probe coverage for sparse vs dense kits (3- and 7-piece layouts, equal-width lanes by construction, core leftmost); review screenshots generated for the feel pass

## 3. Pad strip

- [x] 3.1 Pad-strip painter replacing `piano_keyboard_painter.dart` / `piano_layout.dart` for percussion, one pad per lane **in the lane order**
- [x] 3.2 Kick as a single wide pedal beneath the pads, not one pad among them
- [x] 3.3 Height follows the existing viewport policy and is **independent of the piece count** — a sparse kit must not inflate the strip and steal height from the cascade
- [x] 3.4 Test that pads and lanes present the same pieces in the same order, including after an inverted-kit toggle
- [x] 3.5 Pads are display-only in this change: assert by test that a tap produces no note and no visual state change — input and pad feedback arrive with `add-drum-input-mapping`

## 4. Inverted-kit setting

- [x] 4.1 Persisted preference beside `keyboardRange` / `readingAid` / `metronome` in `player_preferences.dart`, seeded into `PlayerData`; defaults to the standard layout and is never inferred
- [x] 4.2 Reverses the lane order — applied to the single derived layout, so both consumers follow automatically
- [x] 4.3 Assert by test that the inversion is applied to the derived presentation layout **only** — no notation surface and no note-interpretation data is touched
- [x] 4.4 Offer it in the player settings only for a percussion score; label it by the **kit's layout**, never by handedness (a left-handed drummer may play a standard kit and must not be invited to enable it), localised fr/en

## 5. Wiring

- [x] 5.1 Route the player to the percussion cascade + pad strip when the loaded score is percussion, and to the existing keyboard path otherwise; the keyboard path stays untouched
- [x] 5.2 Confirm the range chooser is not offered for a percussion score and the stored range mode is left unchanged for the next keyboard score
- [x] 5.3 Kit-piece names in `l10n` (fr/en) — used by the pads and the lane labels; a generic piece carries its General MIDI name (the reading aid is tied to the Wait Mode gate, absent for percussion until `add-drum-scoring`, so it is not wired here)
- [x] 5.4 Follow the Riverpod layering rules: the screen reads state and calls notifier methods; no painter reads a service, no widget invalidates a sibling provider
- [x] 5.5 The render-mode toggle offers only the cascade for a percussion score — Staff and Partition omitted until `add-drum-notation-render`; keyboard scores keep all three modes, asserted by test
- [x] 5.6 Wait Mode is not offered for a percussion score — timed modes only until `add-drum-scoring` (an inert pad strip cannot satisfy the gate); keyboard scores keep it, asserted by test
- [x] 5.7 Hand selector for percussion: offered despite the single staff whenever the score has both hand and foot events, labelled **hands / feet / both** (fr/en), keyed to the voice convention; test that selecting hands hides the kick bar and every foot event, and that selecting feet keeps the bar and empties the hand lanes

## 6. Gates

- [x] 6.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 6.2 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 6.3 No golden files added (the paint-order proof is a pixel probe, which runs on every platform) — nothing to refresh
- [x] 6.4 `openspec validate add-drum-kit-view --strict`

## 7. Feel pass — the part tests cannot decide

- [ ] 7.1 Drive a real drum score at real tempo on a phone **and** a tablet: are the lanes wide enough, does the groove read without hunting?
- [ ] 7.2 Check the bar's attenuation **in motion** — a scrolling bar reads weaker than a static one, so the value tuned on a mockup may be too low
- [ ] 7.3 Check the foot colour on the **Paper** theme, where amber sits on ivory rather than on a dark ground and the contrast ratio is not the same
- [ ] 7.4 Confirm the open-hi-hat variant is distinguishable from a closed one at speed, not merely side by side
- [ ] 7.5 With the drummer tester: validate the tom ordering and the ride's placement on a score that alternates hi-hat and ride between sections — the one case the reasoning could not settle
- [ ] 7.6 Record what the feel pass changed, in `design.md`, so the next person knows which values were measured and which were guessed

## 8. Reversal — the drawn kit, the stage, the metre (2026-08-24)

- [x] 8.1 One kit object owning layout, painting and hit areas (`drum_kit_art.dart`); both play surfaces and the staff build their strike surface from it, and the Partition draws none
- [x] 8.2 The pieces straddle a central gap the bass drum's width while the row is sparse, and every name hangs under its own piece, clipped to its width
- [x] 8.3 The stage: a fourth render mode, percussion only, first in the row; true `1/z` projection; rails ending on the drawn piece they name
- [x] 8.4 The metre grid on both surfaces comes from `measureStartMs`/beat length, bar lines carry their written measure number, and it is painted after the kick bars
- [x] 8.5 Nothing lights on arrival: a note lights only when its piece is struck inside the tolerance window, and a passed note leaves the surface
- [x] 8.6 `flutter analyze`, `dart format`, `dart run custom_lint`, `flutter test --exclude-tags golden` clean; `openspec validate add-drum-kit-view --strict`
- [x] 8.7 Feel pass on the two surfaces at real tempo (iPhone, release build): stage-first, the tolerance window and the Paper theme all validated on device
- [x] 8.8 The guided player tour speaks percussion on a drum score — the kit, the electronic kit, and hands/feet — instead of the piano's vocabulary over a "Feet / Hands / Both" control
