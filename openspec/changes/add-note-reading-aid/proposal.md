## Why

A beginner using the player knows *which key to press* — the on-screen keyboard
highlights the expected keys and the waterfall shows them coming — but not *what
that note is called* nor *how long to hold it*. The app therefore teaches finger
placement without ever teaching note reading, which is the skill the player
actually came for.

The opportunity is that Wait Mode already stops time at every onset it gates. At
that instant nothing is moving and nothing is being judged, so a large, legible
reading aid can be shown **without competing with play at all** — a property no
other moment in the player offers.

## What Changes

- Add an opt-in **note reading aid**: while Wait Mode is blocking at an onset, the
  player shows the awaited note(s) by name with the correct alteration, and
  optionally the rhythmic figure to hold.
- Name notes from their **written spelling** (the note's staff degree), not from
  the MIDI number, and derive the **effective** alteration (key signature included)
  rather than only the accidental the score engraves. A note that sounds F♯ under
  a one-sharp key signature is named "F♯"/"Fa♯", never "F"/"Fa".
- Localize the naming convention: letter names (C, D, E…) in English, solfège
  (Do, Ré/Re, Mi…) elsewhere, with the French `Ré` and the Spanish/Italian `Re`.
  The existing key-signature namer in the pre-play modal moves into the new shared
  module so there is exactly one naming implementation.
- Never show octave numbers — the aid says "Do♯", not "Do♯4". The keyboard already
  shows *which* key; an octave index is a second vocabulary to learn.
- Add a persisted play setting with three levels — **off / note name / note name +
  rhythm** — editable from the pre-play setup modal and the in-game settings
  drawer (the same modal), defaulting to **off**.
- Scope the aid to Wait Mode for this change. Outside Wait Mode the expected set
  refers to notes already sounding under the playhead, so an aid would always
  arrive too late; a look-ahead variant is deliberately deferred.

## Capabilities

### New Capabilities

- `note-reading-aid`: naming an awaited note for a learner — written-spelling based
  note names with effective alterations, localized letter/solfège conventions,
  rhythmic figure labelling, and the rules governing when and where the aid is
  displayed so it never competes with play.

### Modified Capabilities

- `pre-play-setup`: the setup modal's contents and the set of play settings
  persisted across scores and restarts both gain the reading-aid level.

## Impact

- **Flutter app (`apps/music`)** only. No backend, no Rust engine, no gRPC, no
  migration — every input the aid needs is already carried by the parsed notation.
- New pure, host-testable naming module; the private `_keyName` helper in the
  pre-play modal is removed in favour of it.
- `PlayerData` gains a way to expose the *awaited notes* (today only their MIDI
  pitches are exposed, which loses the spelling the aid depends on).
- `PlayerPrefs` gains one persisted field, with tolerant decoding so an unknown or
  missing stored value falls back to the default.
- New display widget in the player's existing overlay stack; its footprint is
  reserved in the layout so the render area never shifts when the aid appears.
- New localized strings in the four locales (en/fr/es/it).
- Test coverage stays ≥ 80 % (pure naming logic, the awaited-notes selection, and
  widget-level display/visibility rules).
