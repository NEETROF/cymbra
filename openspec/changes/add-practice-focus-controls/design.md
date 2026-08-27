## Context

The player already has one mechanism for "show me less than the whole piece": the
three-valued `Hand` state, filtering by staff. `add-drum-kit-view` extended it to
percussion by reinterpreting the same three values as hands / feet / both, keyed
to the MusicXML voice with a General MIDI fallback for single-voice parts.

The reinterpretation is where it goes wrong, and the tester found it immediately.
Most drum exports are single-voice, so the fallback — kick 35/36 and pedal hi-hat
44 are feet, everything else hands — is what almost always runs. It is a correct
classification and the wrong *grain*: choosing "hands only" removes the kick,
which nobody practising a groove wants. The two halves of the app already
disagree about GM 44, which the filter treats as a foot event while the lane
model puts it in the terminal bucket. And a drummer isolating part of a groove
names pieces, not limbs.

Separately, the app can be told not to double the notes the player plays
(`instrumentSoundsItself`) but not to stop playing the score. All score audio
already passes through a single function, `_applyScoreAudio`
(`player_notifier.dart:299`), which is the whole of the second half of this
change.

## Goals / Non-Goals

**Goals:**
- Let a drummer isolate a groove at the grain they think in — pieces of the kit.
- Keep the invariant the limb filter established: whatever is not drawn is not
  gated and not judged, from one source, so those three can never disagree.
- Keep the other invariant it established: focus never touches input. A muted
  piece still sounds and still flashes when struck.
- Let a player silence the written part without silencing anything else.

**Non-Goals:**
- Per-piece *volume*. Mute/solo is a practice control, not a mixer; a partial
  level would raise "is it in focus or not" for the gate, which has no partial
  answer.
- Any change to keyboard hand selection. It stays exactly as it is.
- Muting the metronome. It has its own control, and the point of silencing the
  score is often to hear the click better.

## Decisions

### D1 — Focus is a set of pieces, not a filter over notes

The state is the set of **kit pieces** in focus, and a note is drawn if the piece
its number belongs to is in that set — resolved through the kit model's existing
`laneIndexOf` / piece identity, which already collapses the numbers that mean one
aim point (closed and open hi-hat, acoustic and electric snare).

*Why not a set of GM numbers:* the player would then have to mute the open and
closed hi-hat separately, which is one pad on the instrument. The kit model
already decided what one piece is; focus reuses that decision rather than making
a second one.

*Why session-only:* it describes the bar being worked on. The hand selection it
replaces is session-only for the same reason, and persisting it would silently
hand a later score a kit with holes in it.

### D2 — Solo is expressed in the same set, not a second state

Solo is "focus = exactly these", mute is "focus = everything except these". One
set, two ways of editing it, so there is no mode where both are set and the
answer depends on precedence.

The corollary is the empty-set rule: muting the last piece would leave a session
that asks for nothing, draws nothing and judges nothing — indistinguishable from
a broken score. It restores everything instead.

### D3 — The kick is a piece, not a special case

The kick has no lane (it is the full-width bar — `add-drum-kit-view`'s "the foot
does not aim"), but for focus it is a piece like any other. Hiding it must hide
the bar: the existing spec already says as much for the limb filter, and the
reason carries over unchanged — the bar is a note in a different shape.

### D4 — The expected-notes mute lands at the one existing choke point

`_applyScoreAudio` is where every scheduled note becomes sound, on both the
percussion and the keyboard branch. The mute is read there and nowhere else.

The subtlety is `_sounding`, the set of notes whose release is owed. Skipping the
attack must skip the bookkeeping too, or a release is issued for a voice that was
never started — the exact shape of the bug `add-drum-audio-channel` 10.3 fixed
for the Wait-Mode double-strike. Toggling the mute *on* mid-playback must
therefore release what is currently sounding and clear the set, and toggling it
*off* must not try to release notes it never started.

*Why not mute at the audio service:* it would silence the metronome and the
player's own notes too, which are explicitly not what this control is about.

### D5 — The mute gets a one-tap toggle in the top bar, the focus control does not

The mute is reached for mid-exercise — "let me hear myself for this pass" — so it
needs to be one tap from where the player already is. The focus selection is a
several-taps decision made between passes and belongs in the settings surface
with the other multi-choice controls.

*Revised during implementation.* The transport rail was the obvious home, beside
Wait Mode. It does not fit: its seven controls fill a phone-landscape viewport to
the pixel, and an eighth also overflowed a 820×460 desktop window by 34 px. The
toggle sits in the **top-bar trailing cluster** instead, next to the metronome
chip — which is arguably where it belonged anyway, since the two answer the same
question ("what am I hearing while I play") and a drummer reaches for both in the
same breath. That cluster is a scale-down `FittedBox`, so it absorbs a narrow
window as one block rather than overflowing. It is still the right-hand bar the
tester asked for.

One detail worth pinning: the sounding-state icon is `audiotrack`, not
`music_note`, because `music_note` is already the Staff mode's segment icon a few
widgets away in the same bar.

### D6 — Deleting the limb selector rather than keeping it alongside

Two overlapping filters over the same notes is one more thing that can disagree,
and every use of the limb filter is expressible as a piece selection ("feet only"
is solo the kick and the pedal hi-hat). The strings and the coach step go with
it.

## Risks / Trade-offs

- **[Removing a shipped control]** → It shipped to a closed beta of one, who
  reported it as confusing. The removal is percussion-only and the keyboard
  control — the one with actual users — is untouched.
- **[Focus and the scorer must agree exactly]** → Both read `visibleNotes`,
  which is already the single source the limb filter established. The change
  swaps what that source filters *on*; the property that gate, drawing and
  judgment share it is unchanged and stays asserted.
- **[Toggling the mute mid-note]** → Covered by D4. Tested in both directions,
  including a toggle in the middle of a sustained keyboard note, where a missing
  release would leave a voice hanging.
- **[A soloed kit and a scored run]** → A run with pieces muted is still scored
  against the notes it actually asked for. That is consistent with how a selective
  measure-range run behaves today (`add-measure-range-practice`) — except that one
  is deliberately *not* scored. Whether a focus-restricted run should count toward
  leaderboards needs deciding, not assuming; see Open Questions.

## Open Questions

- **Should a focus-restricted run be scored and ranked?** A measure-range
  practice run is deliberately never scored (D2 of `add-measure-range-practice`),
  on the grounds that a partial run is not comparable to a whole one. Muting the
  crashes is arguably the same kind of partial. Leaning: score it locally, but
  do not submit it to leaderboards or count it toward play rewards — the same
  posture practice sessions already have. Settle before implementing §5.
- Whether solo should be a distinct gesture (long-press a pad) or a second column
  in the settings list. A pad long-press is faster and discoverable by drummers,
  but the pad strip's long-press is currently unclaimed only by accident.
- Whether the expected-notes mute should also silence a **preview** playback (the
  card audition), or only the player's own transport. Leaning: player only — the
  audition is a listening surface, not a practice one.
