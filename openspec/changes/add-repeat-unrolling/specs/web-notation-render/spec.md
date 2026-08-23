# web-notation-render — delta for add-repeat-unrolling

## ADDED Requirements

### Requirement: Repeat notation engraved in the browser render

The browser notation renderer SHALL engrave the score's repeat structure in
written order: repeat barlines (thick/thin lines plus dots on the repeated
side), volta brackets with their numbers, the `%` sign on measure-repeat
measures, and the segno, coda and D.C./D.S./Fine markers where the score
places them — so a moderator sees the same notation the app's Partition view
shows and can review repeat-carrying scores faithfully.

#### Scenario: Back-office render shows the repeat vocabulary

- **WHEN** the back office renders a score containing a repeated section, two
  voltas and a `%` measure
- **THEN** the rendered notation shows the repeat barlines, both volta
  brackets and the `%` sign at their written positions

#### Scenario: Scores without repeats are unchanged

- **WHEN** the back office renders a score with no repeat structure
- **THEN** the rendered output is unchanged from the pre-change renderer

### Requirement: Browser playback schedule follows the played order

The browser playback schedule (back-office Play preview) SHALL derive from
the engine's unrolled playback order: repeated sections
sound as many times as the score prescribes, the correct volta is selected
per pass, `%` measures replay the referenced measure's content, and the
schedule's total duration reflects the unrolled length. The unroll SHALL come
from the shared engine crate — the browser side SHALL NOT re-implement
repeat resolution.

#### Scenario: Play preview sounds the repeat

- **WHEN** a moderator plays a score whose measures 1–4 are repeated
- **THEN** the preview sounds measures 1–4 twice before continuing, and the
  reported duration covers the unrolled timeline

#### Scenario: Measure-repeat is audible in the preview

- **WHEN** a moderator plays a score containing a `%` measure
- **THEN** the preview sounds the referenced measure's notes during that
  measure instead of a bar of silence
