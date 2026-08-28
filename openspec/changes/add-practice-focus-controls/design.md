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

### D7 — A focus-restricted run is scored locally, submitted nowhere, and still counts as practice

*Settled during implementation (task 5.1).* Three separate questions were folded
into one in the Open Questions, and they get three different answers:

- **Scored locally: yes.** The run covers the whole piece and produces an honest
  verdict on what it asked for. A drummer working the hi-hat and snare wants to
  know how that went; withholding the number would make the control feel like a
  downgrade.
- **Submitted: no.** No leaderboard entry, no play reward. A 100 % on a groove
  with the crashes muted is not the same achievement as a 100 % on the groove,
  and the boards have no way to say so — the piece id is the same either way.
- **Practice: yes.** The session is captured as a practice record, so it holds
  the streak. This is the half that must not be forgotten: the tester whose
  feedback started this change was *also* being told he had not played. An
  isolation drill is practice by any definition, and a control that silently
  costs a player their streak would be worse than the control it replaces.

*Why not the measure-range posture (never arm the scorer at all):* a selective
run is unscored because it has no end — it stops mid-piece, so there is no
comparable whole. A focus-restricted run reaches the last bar; only its *content*
is narrower. Different defect, different remedy.

The decision lands on the submission seam (`captureSession`), not on a hidden
summary: the player still sees the result.

### D8 — Solo is a control in the settings list, not a pad gesture

*Settled during implementation (task 4.2).* The open question offered a pad
long-press as the faster, more discoverable gesture. It is the wrong surface: the
pad strip is an **instrument**, driven by a per-pointer `Listener` that treats
every pointer-down as a stroke and supports two-finger rolls on one pad. A hold
long enough to trigger a gesture is an ordinary thing to do while playing, and
paying for it by silently reconfiguring the exercise is the worst kind of
surprise. (The same collision bit the in-game measure selection, where an
`IconButton` tooltip stole the long-press.)

So the focus control is a list in the settings surface, one row per piece:
a checkbox for in/out of focus, and a **Solo** action on the row for "only
this one". The additive rule the spec pins then falls out of the two together —
solo the hi-hat, then check the snare, and both are asked for.

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
  against the notes it actually asked for, shown to the player, and delivered to
  nothing (D7). The streak is held through the practice record, so isolating part
  of a groove never reads to the app as not having played.

## Open Questions

- Whether the expected-notes mute should also silence a **preview** playback (the
  card audition), or only the player's own transport. Leaning: player only — the
  audition is a listening surface, not a practice one.

*Settled:* whether a focus-restricted run is scored and ranked (D7) and whether
solo is a pad gesture (D8).
