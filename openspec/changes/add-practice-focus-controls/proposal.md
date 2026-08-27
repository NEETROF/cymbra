## Why

Two reports from the drum beta's first tester, from opposite ends of the same
question — *what is being asked of me, and what am I hearing while it is asked?*

**The hands/feet selector does not make sense to him.** It is the piano's
left/right/both control reused: `Hand.right` means the hands and `Hand.left` the
feet (`player_data.dart:769`). The split is read from the MusicXML voice, with a
General MIDI fallback for the single-voice case — and most drum exports *are*
single-voice, so the fallback is what almost always applies. It classifies the
kick as a foot event, which is correct and useless: choosing "hands only" removes
the one thing a drummer never stops playing. Meanwhile the hi-hat pedal (44) is
classified as a foot event for filtering but sits in the lane model's terminal
bucket, so the two halves of the app already disagree about it. A drummer
isolating part of a groove thinks in **pieces of the kit** — "just the hi-hat and
the snare", "everything but the crashes" — not in limbs.

**There is no way to silence the notes the app is asking for.** The app can be
told to stop doubling what the *player* plays ("Mon instrument produit son propre
son", `sound_output_section.dart:370`), but not to stop playing the *score*. On a
kit that matters more than on a piano: in Wait Mode the metronome, the written
part and the player's own strokes all sound at once, on percussion timbres that
mask each other, and the exercise becomes harder to hear than to play.

## What Changes

- **Per-piece focus replaces the limb selector on percussion.** Each lane of the
  kit (and the kick) can be muted or soloed. Muting a piece removes it from what
  is drawn, from what the Wait Mode gate waits for, and from what the scorer
  judges — exactly the three things the limb filter did, at the grain a drummer
  reasons in. Solo is the same rule from the other side, for the common case of
  isolating one or two pieces.
- **BREAKING (percussion only): the hands / feet / both choice is removed.** Its
  useful cases are expressible as piece selections and its confusing case (a
  kick-less "hands only") disappears. Keyboard scores keep left / right / both
  exactly as they are.
- **Input is never filtered.** A muted piece still sounds when struck and still
  flashes its pad — the same rule the limb filter already had
  (`add-drum-input-mapping` task 4.5). Focus is about what is *asked*, never
  about what is *heard from the player*.
- **A mute for the expected notes**, instrument-agnostic: the app stops sounding
  the score while still drawing it, gating on it and scoring against it. Distinct
  from the existing "my instrument sounds itself", which is about the player's
  own notes; both can be on, off, or either way round.
- **Both controls are reachable during play**, from the settings the top bar
  already opens, and the expected-notes mute also as a direct toggle in the
  transport rail — it is a thing a player reaches for mid-exercise, not a setting
  they configure once.
- The expected-notes mute is **persisted**; the per-piece focus is
  **session-only**, like the hand selection it replaces (it describes a passage
  being worked on, not a preference).

## Capabilities

### New Capabilities
- `music-kit-piece-focus`: muting and soloing individual pieces of the drum kit,
  and what that does to drawing, gating and scoring.
- `music-score-playback-mute`: silencing the app's playback of the written score
  without changing anything else about the session.

### Modified Capabilities
- `hand-selection`: the percussion reading of the three-valued state (hands /
  feet / both) is removed; the capability returns to the keyboard staff mapping
  it started as.

**Ordering constraint:** the percussion branch of `hand-selection` is introduced
by the in-flight `add-drum-kit-view` delta and is not yet in
`openspec/specs/hand-selection/`. That change must archive **before** this one,
or this change's delta will describe a removal of text the base spec never
received.

## Impact

**Product: Cymbra Music only.** No backend, no back office, no site, no
identity, no gRPC surface, no engine API change.

Consumed, unchanged:
- the kit model (`drum_kit.dart`) — lanes, roles, `laneIndexOf`, the canonical
  emission order — which already gives every piece a stable identity to key focus
  on;
- the single score-audio choke point `_applyScoreAudio`
  (`player_notifier.dart:299`), where the expected-notes mute lands;
- `local-preferences` for the persisted mute (one more key, no new requirement).

Modified in the app:
- `player_data.dart`: the percussion branches of `_showsNote` / `_showsRest` /
  `playableDrumSurfaces` / `hasHandsAndFeet` / the summary label, replaced by a
  focus set; `selectedHands` becomes keyboard-only;
- `player_notifier.dart`: the focus actions, and the mute honoured in
  `_applyScoreAudio`;
- `pre_play_setup_modal.dart`: the percussion limb section replaced by the focus
  control; the mute toggle added;
- `player_screen.dart`: the mute toggle in the transport rail;
- the drum painters, which already receive the visible-note set and need no
  change beyond what the filter feeds them;
- fr/en/es/it strings; the removed `handHands` / `handFeet` /
  `coachPlayerLimbs*` entries and the summary's `hands` / `feet` /
  `handsAndFeet` select branches.
