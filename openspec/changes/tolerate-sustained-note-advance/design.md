## Context

Wait Mode's onset gate lives entirely in the Flutter layer, not the Rust engine:

- `Player` notifier ([player_notifier.dart](apps/music/lib/state/player_notifier.dart)):
  - `noteOn()` (~L232-246): on a MIDI/keyboard note-on it adds the pitch to
    `activeNotes` and, **if** the onset is currently active, latches the pitch into
    `gateSatisfied`.
  - `noteOff()` (~L248-255): removes the pitch from `activeNotes`.
  - `advance()` (~L310-377): freezes while the current onset's pitches aren't all
    in `gateSatisfied`; resets `gateSatisfied` when leaving an onset (~L373).
- `PlayerData` ([player_data.dart](apps/music/lib/state/player_data.dart)):
  - `activeNotes` (Set<int>, ~L76): pitches **currently held down**. Already
    maintained, but only used for visual feedback today.
  - `gateSatisfied` (Set<int>, ~L125): pitches counted for the current onset.
  - `onsetPitchesAt(t)` (~L196-202): required pitches whose onset coincides with
    `t` (±1ms), restricted to `visibleNotes`.

The defect: `gateSatisfied` is populated **only** by a note-on that arrives while
the onset is already active. A pitch pressed slightly early — and still held — is
in `activeNotes` but never generates a new note-on once the playhead arrives, so
the gate never sees it and freezes forever until the player releases and
re-presses. `activeNotes` already gives us exactly the "is this pitch down right
now?" signal we need.

## Goals / Non-Goals

**Goals:**
- A required onset pitch that is **currently held** when the playhead reaches the
  onset satisfies the gate without a re-press.
- Preserve the anti-cheat guarantee: a pitch pressed **and released** before the
  onset does not pre-satisfy it (it is no longer in `activeNotes`).
- Preserve the existing latch semantics: once an onset pitch is satisfied,
  releasing the key before the next tick does not re-block.
- **Preserve mandatory re-attack for repeated notes**: a single held key must not
  auto-advance through consecutive onsets of the same pitch.
- Keep all gate logic host-testable in Dart (unit/widget tests, no native lib).

**Non-Goals:**
- No sustain-duration or timing/synchronization scoring (still out of scope per
  the existing "No Sustain Or Synchronization Scoring" requirement).
- No change to the Rust MIDI parsing/stream or the public FFI surface.
- No change to free-play (non-wait) mode.

## Decisions

**Decision 1 — Seed `gateSatisfied` from *unconsumed* held pitches when an onset
becomes the active gate, rather than checking a live union at freeze time.**

When the playhead first arrives at a new onset, add `onsetPitchesAt(onset) ∩
activeNotes ∩ ¬consumedHeld` into `gateSatisfied` (once), then evaluate the freeze
as today. `consumedHeld` (see Decision 3) is the set of currently-held pitches
whose current press has already satisfied an onset.

- *Why seed instead of `gateSatisfied ∪ activeNotes` in the freeze check?* A live
  union would re-block a held-through pitch if the player releases it in the brief
  moment after the playhead arrives but before the gate advances — violating the
  "release after press does not re-block" scenario. Seeding latches held pitches
  identically to fresh presses, so release-after-arrival behaves consistently.
- *Alternative considered:* have `noteOn()` retro-satisfy — rejected, because the
  triggering case has **no** note-on at the onset (the key was already down).

**Decision 2 — Use `activeNotes` membership as the "is it down now?" signal; a
pressed-then-released early key is naturally excluded.**

A note-off removes the pitch from `activeNotes`, so an early press that was
released is absent at onset arrival — the anti-cheat property comes for free.

**Decision 3 — Preserve re-attack for repeated notes: each press satisfies at most
one onset.** A single sustained hold must NOT auto-advance through consecutive
onsets of the same pitch. To enforce this without breaking the tied/legato
carry-in case, track a `consumedHeld` set:

- A pitch is added to `consumedHeld` at the instant its current hold satisfies an
  onset — both when seeded at onset arrival (Decision 1) and when the existing
  `noteOn()` latch counts a fresh press.
- A **fresh attack resets consumption**: `noteOn()` removes the pitch from
  `consumedHeld` (a new press starts a new, uncounted hold); `noteOff()` also
  removes it (the hold ended).
- Seeding only considers pitches **not** in `consumedHeld`, so a hold that already
  satisfied onset N is ignored at onset N+1 — the repeat stays frozen until the
  player releases and re-presses.

This satisfies both requirements at once: a genuinely tied/sustained note (whose
press has not yet counted anywhere) is tolerated the first time it is required,
while a held key cannot walk through repeated notes. It also handles the
non-adjacent repeat correctly (P required at N and N+2 but not N+1): the hold is
consumed at N and stays consumed at N+2, so N+2 requires a fresh attack.

*Alternative considered:* compare each onset's pitch set against the previous
onset's to detect repeats — rejected as more fragile (only handles adjacent
repeats, mishandles the N→N+2 gap case) than the per-press "consumed" flag.

## Risks / Trade-offs

- **[`consumedHeld` desync could either block a legit hold or leak a repeat]** →
  Keep its lifecycle trivial and colocated: mutate it only in `noteOn()` (clear),
  `noteOff()` (clear), and at the seed/latch point (set). Cover both directions
  with tests (2.4, 2.6) so a genuine tied carry-in is never blocked and a repeat is
  never auto-satisfied.
- **[Seeding at the wrong moment could pre-satisfy an onset the playhead hasn't
  reached]** → Seed strictly when the gate's active onset changes to this onset,
  reusing the same `onsetPitchesAt` used by the freeze check, so held-tolerance
  and freezing share one definition of "the active onset."
- **[Stale hold from a long-past onset]** → Not possible: `activeNotes` reflects
  keys that are physically down *now*; a note-off clears it. Only genuinely
  sustained, not-yet-consumed keys are seeded.

## Migration Plan

Pure behavior change, no data or API migration. Ships behind no flag (Wait Mode
already opt-in). Rollback is a straight revert of the gate change.

## Open Questions

- Should there eventually be a "strict" Wait Mode that always requires a fresh
  attack (disabling this tolerance)? Out of scope here; captured for the future
  scoring capability.
