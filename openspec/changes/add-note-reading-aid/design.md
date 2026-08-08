## Context

The player already tells a beginner *which key to press*: `PlayerData.expectedKeys`
drives the keyboard's expected-key highlight and the waterfall shows the note
approaching. What it never says is what that note is **called** or how long to
**hold** it, so the app trains finger placement without teaching note reading.

Everything the aid needs is already parsed and carried on `TimedNote`
(`apps/music/lib/state/player_data.dart`): `pitch` (the sounding MIDI pitch),
`diatonic` (the *written* staff degree, `octave*7 + step`), `accidental`,
`noteType`, `dots`, `staff`. `PlayerData` also carries `keyFifths`,
`measureKeyFifths`, `beats`/`beatType` and the `blocked` flag. No engine, backend
or bridge work is implied — this change is confined to `apps/music`.

Two existing facts shape the design:

- **Wait Mode stops time.** When the gate blocks, the playhead is frozen and
  nothing is being judged. That is the only moment in the player where a large,
  legible panel costs the player nothing.
- **A naming helper already exists but is private.** `_keyName` in
  `apps/music/lib/screens/pre_play_setup_modal.dart` already picks letters vs
  solfège per locale for the key signature. It must not be duplicated.

## Goals / Non-Goals

**Goals:**

- Name the awaited note(s) correctly — written spelling, effective alteration,
  localized letter/solfège convention — at the moment the player is stuck on them.
- Optionally name and quantify the rhythmic figure, so "half note" comes with
  "hold 2 beats".
- Make it an ordinary persisted play setting, off by default, editable from the
  same modal that already serves as both the pre-play setup and the in-game
  settings drawer.
- Keep the naming logic pure and host-testable, with a single implementation in
  the app.

**Non-Goals:**

- No aid outside Wait Mode, and no look-ahead. Outside Wait Mode the expected set
  refers to notes already sounding, so the aid would always be late; a look-ahead
  variant is a separate change.
- No octave index in the name.
- No labels drawn on the keyboard keys or on the waterfall tiles. Those are
  plausible sibling designs and are deliberately deferred so this change ships one
  surface, not three.
- No re-implementation of MusicXML accidental resolution; the parser already did
  it.
- No refactor of `StaffPainter`'s internal pitch→staff-degree fallback (see
  Decision 4).
- No analytics on aid usage.

## Decisions

### 1. Trigger on `blocked`, not on the playhead

The aid is bound to `PlayerData.blocked` (Wait Mode actively gating) rather than
to elapsed time. This is what makes "large and legible" compatible with "does not
disturb play": while blocked, there is no motion to compete with and no timing to
disrupt. It also gives the withdrawal rule for free — the gate resolving *is* the
signal to hide.

*Alternatives considered.* A permanently visible HUD: rejected, it adds a fourth
fixation point (score, hands, keyboard, HUD) and forces an eye saccade per note.
A look-ahead HUD driven by `nextOnsetAfter`: rejected for v1 — it is the right
answer for continuous play but needs a lead-time policy and its own flicker
control at fast note rates, which would dominate this change.

### 2. Effective alteration by arithmetic, not by replaying the key signature

The alteration is computed as `pitch − naturalPitchOf(diatonic)`, where
`naturalPitchOf` maps a written degree (`octave*7 + step`) to the MIDI pitch of its
natural. The difference lands in −2…+2 and yields ♭♭ / ♭ / ♮ / ♯ / ♯♯ directly.

This is exact and cheap because `pitch` is *already* the fully-resolved sounding
pitch — the parser applied the key signature and every in-measure accidental to
produce it — while `diatonic` is the unresolved written degree. Their difference
is, by construction, the alteration in force. It solves the trap that motivated
this design: `TimedNote.accidental` is non-null only when the score *engraves* a
symbol, so an F♯ carried by the key signature alone would otherwise be named "F".

*Alternatives considered.* Re-deriving the alteration from `measureKeyFifths` plus
an accidental state machine over each measure: rejected — it duplicates parser
logic that has already run, and would drift from it. Trusting
`TimedNote.accidental`: rejected — it is the actual bug, not a shortcut.

### 3. Fallback naming when the written spelling is absent

The demo score and the MIDI-only replay journal carry `diatonic == null`. There the
module names from `pitch` alone and picks the spelling by the sign of the key
signature in force at that point (`measureKeyFifths` when populated, else
`keyFifths`): sharps for sharp keys, flats for flat keys, with `0` treated as
sharps. Enharmonic accuracy is unobtainable without a spelling — the rule only
guarantees a plausible name that never contradicts the sounding pitch.

### 4. The shared module owns *naming*; `StaffPainter` keeps its own placement helper

`StaffPainter._diatonic` deliberately **collapses** enharmonics (A♭→G) because it
needs a staff line, and any line is better than none. Naming needs the opposite —
a spelling choice. The two are not the same function despite the similar shape, so
the painter is left untouched and the shared module gets its own fallback. What
*is* de-duplicated is the locale naming: `_keyName` moves out of the pre-play modal
into the shared module and the modal calls it there.

### 5. Pure module returns tokens; the widget resolves l10n

The naming module stays free of `BuildContext` and `AppLocalizations`:

- Degree names are generated in-module from `solfege` / `frenchRe` flags (the
  convention already used by `_keyName`), not from seven ARB keys per locale.
- Figure naming returns a **token** (figure kind + dots + resolved beat count),
  and the widget maps that token to the localized prose ("dotted half note",
  "hold 3 beats"). Prose belongs in ARB; the arithmetic does not.

This keeps the whole naming/figure layer testable as plain Dart functions, which
is what carries the coverage requirement.

### 6. Beat count from the notated figure, with a clean-value guard

Duration in beats is `wholeNoteFraction(noteType, dots) × beatType` — a half note
in 4/4 is 2 beats, a dotted half is 3. When `noteType` is absent it is inferred
from `durationMs` against the measure's beat duration. When the result does not
land on a clean value (halves or better), the aid shows the figure name **without**
a beat count rather than printing a misleading number. This also contains the
compound-meter case (see Risks).

### 7. The level is mirrored into `PlayerData`, following the metronome precedent

`PlayerPrefs` (`apps/music/lib/state/player_preferences.dart`) gains the persisted
level; `PlayerData` gains the session-mirrored value, seeded from prefs and set
through the `Player` notifier — exactly how `metronome`/`metronomeEnabled`,
hands and speed already work. The widget watches `playerProvider`; it does not
read the preferences provider directly, and no provider imperatively invalidates
another. JSON decoding is tolerant: a missing or unrecognized level falls back to
*off* without discarding the rest of the stored record.

### 8. `expectedNotes` on `PlayerData`, sharing one reference-time helper

`expectedKeys` returns `Set<int>` and therefore loses the spelling the aid needs.
A new `expectedNotes` getter returns the awaited `TimedNote`s. The reference
instant ("the onset under the playhead, or the upcoming one while travelling") is
currently computed twice — inside `expectedKeys` and again inside
`expectedKeysForHand` — so it is extracted into one private helper that all three
call, preventing the aid from ever disagreeing with the keyboard highlight.

### 9. Reserved footprint, above the keyboard

The aid renders in a band of fixed height directly above the on-screen keyboard,
inside the existing player `Column`/`Stack`. The band is reserved whenever the
level is not *off*, visible or not, so the score render area never resizes when
the gate blocks. When the level is *off*, no band is allocated at all and the
render area is unchanged from today's layout.

## Risks / Trade-offs

- **The aid becomes a crutch — the player reads the panel and never the staff.**
  → Default *off*; shown only while the gate has already stopped play (so it can
  never carry a moving performance); no octave index to lean on; look-ahead
  explicitly deferred. Whether to fade it out over sessions is a product question
  left open, not designed in.

- **A wrong name is worse than no name.** → Decision 2 makes the alteration
  arithmetic exact rather than heuristic, and the pure module is tested across
  sharp keys, flat keys, engraved accidentals, naturals cancelling the key
  signature, and enharmonic spellings (D♭ must never surface as C♯).

- **Compound meters report notated beats, not felt beats.** In 6/8 a dotted
  quarter is 3 beat-type units but musicians count it as one beat. → The aid states
  duration in units of the notated beat-type and the clean-value guard (Decision 6)
  suppresses the count when it would mislead. Accepted as a known limitation for
  v1 rather than modelled.

- **Reserved height is expensive on a phone in landscape**, where vertical space is
  already scarce. → Single compact line, height scaled by device class, and nothing
  reserved when the aid is off — the default player layout is untouched for anyone
  who has not enabled it.

- **A dense chord overflows the line.** → Bounded at four names with a summary
  beyond that, and the figure is shown once only when every awaited note agrees on
  it.

- **`diatonic`-less sources get a plausible rather than a correct spelling.**
  → Accepted: it affects the demo score and replay only, both of which are
  MIDI-only by nature, and the fallback can never contradict the sounding pitch.

## Migration Plan

No data migration. The persisted settings record gains one optional field and
decoding tolerates its absence, so an existing install restores its other settings
and starts with the aid off. Rollback is either turning the setting off or
reverting the widget; nothing outside `apps/music` is touched and no stored state
becomes unreadable.

## Open Questions

- Should the aid also be shown during the pre-start countdown, naming the first
  note before the run begins? It is the same "time is stopped" argument, but it
  overlaps `CountdownOverlay`.
- Should the beginner onboarding actively propose enabling it, or is discovery
  through the setup modal enough? Promoting it belongs to the `welcome-onboarding`
  capability and is not specified here.
- Is `off` the right default for a player who self-identifies as a beginner during
  onboarding, or should that path default it to *note name*?
