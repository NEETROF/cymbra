## 1. Kit-piece model and lane derivation

- [ ] 1.1 Map General MIDI percussion numbers (35–81) to kit-piece roles: hi-hat, snare, toms (high→low), ride, crash and other accent cymbals, kick; equivalent numbers (acoustic vs electric snare, the several crashes) collapse onto one role — build the table against real files, not from the GM list alone, and leave an unmapped number as its own generic piece rather than dropping it
- [ ] 1.2 Pure function: score → ordered lane layout, applying the sort rule (time-keeper, snare, toms high→low, remaining cymbals) to the pieces **present**; the kick is excluded from the lanes by construction
- [ ] 1.3 Encode the invariant as a test, not a comment: for any input, position 1 is the continuously-struck piece and position 2 the snare; adding pieces appends right and never reorders positions 1–2
- [ ] 1.4 Ride takes position 1 when the score has no hi-hat, and joins the cymbals when it does — the rule is keyed to function, not to the piece's name
- [ ] 1.5 Expose the layout once on the player state, consumed by **both** the cascade and the pad strip; never derived twice

## 2. Cascade painter

- [ ] 2.1 New percussion cascade painter beside `synthesia_painter.dart`; a percussion score never consults `keyboard_range.dart`
- [ ] 2.2 Lanes at equal width over the available space, so a sparse kit gets wide lanes rather than a full kit with empty columns
- [ ] 2.3 Kick as a full-width bar at its onset — **painted before the note layer**, thinner than a note, attenuated
- [ ] 2.4 Open hi-hat drawn as a variant of the note inside the hi-hat lane (no bar, no extra lane)
- [ ] 2.5 Hand notes take the hand colour, foot bars the foot colour, keyed to **voice** — not to staff (a drum part is one staff, two voices)
- [ ] 2.6 Golden test on a **coinciding** kick + hand onset: the note stays legible over the bar. This is the one case that reveals an inverted paint order, and a score where feet and hands never align renders perfectly with the layers wrong
- [ ] 2.7 Golden tests for a 3-piece and a 7-piece score, confirming lane widths adapt and the core stays leftmost

## 3. Pad strip

- [ ] 3.1 Pad-strip painter replacing `piano_keyboard_painter.dart` / `piano_layout.dart` for percussion, one pad per lane **in the lane order**
- [ ] 3.2 Kick as a single wide pedal beneath the pads, not one pad among them
- [ ] 3.3 Height follows the existing viewport policy and is **independent of the piece count** — a sparse kit must not inflate the strip and steal height from the cascade
- [ ] 3.4 Test that pads and lanes present the same pieces in the same order, including after an inverted-kit toggle

## 4. Inverted-kit setting

- [ ] 4.1 Persisted preference beside `keyboardRange` / `readingAid` / `metronomeEnabled` in `player_preferences.dart`, seeded into `PlayerData`; defaults to the standard layout and is never inferred
- [ ] 4.2 Reverses the lane order — applied to the single derived layout, so both consumers follow automatically
- [ ] 4.3 Assert by test that it does **not** touch the notation modes nor the interpretation of incoming notes
- [ ] 4.4 Offer it in the player settings only for a percussion score; label it by the **kit's layout**, never by handedness (a left-handed drummer may play a standard kit and must not be invited to enable it), localised fr/en

## 5. Wiring

- [ ] 5.1 Route the player to the percussion cascade + pad strip when the loaded score is percussion, and to the existing keyboard path otherwise; the keyboard path stays untouched
- [ ] 5.2 Confirm the range chooser is not offered for a percussion score and the stored range mode is left unchanged for the next keyboard score
- [ ] 5.3 Kit-piece names in `l10n` (fr/en) — used by the pads, the lane labels and the reading aid
- [ ] 5.4 Follow the Riverpod layering rules: the screen reads state and calls notifier methods; no painter reads a service, no widget invalidates a sibling provider

## 6. Gates

- [ ] 6.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 6.2 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [ ] 6.3 Refresh goldens on the pinned platform (`flutter test --tags golden --update-goldens`) and review each diff rather than accepting wholesale
- [ ] 6.4 `openspec validate add-drum-kit-view --strict`

## 7. Feel pass — the part tests cannot decide

- [ ] 7.1 Drive a real drum score at real tempo on a phone **and** a tablet: are the lanes wide enough, does the groove read without hunting?
- [ ] 7.2 Check the bar's attenuation **in motion** — a scrolling bar reads weaker than a static one, so the value tuned on a mockup may be too low
- [ ] 7.3 Check the foot colour on the **Paper** theme, where amber sits on ivory rather than on a dark ground and the contrast ratio is not the same
- [ ] 7.4 Confirm the open-hi-hat variant is distinguishable from a closed one at speed, not merely side by side
- [ ] 7.5 With the drummer tester: validate the tom ordering and the ride's placement on a score that alternates hi-hat and ride between sections — the one case the reasoning could not settle
- [ ] 7.6 Record what the feel pass changed, in `design.md`, so the next person knows which values were measured and which were guessed
