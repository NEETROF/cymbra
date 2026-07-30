## Context

The player already trims **leading** silence: `effectiveStartMs(visibleNotes)` in
[player_data.dart](apps/music/lib/state/player_data.dart:108) finds the smallest note onset,
subtracts a bounded lead-in, and clamps to zero; `PlayerData.startMs` scopes it to the
selected hand(s). Every fresh-start transport seeds `elapsedMs: startMs`.

The **end**, however, is still the raw `songEndMs`. In
[notation_playback.dart](apps/music/lib/state/notation_playback.dart:122) both notes *and
rests* push `songEndMs` outward, so trailing rests / empty measures inflate it. The
end-of-song block in [player_notifier.dart](apps/music/lib/state/player_notifier.dart:491)
keys off that raw value:

```dart
if (s.songEndMs > 0 && next >= s.songEndMs) {
  if (ref.read(performanceScorerProvider).active) { next = s.songEndMs; finishScoredRun = true; }
  else { next = s.startMs; loop = true; }
}
```

So the run drags through the trailing silence before it finishes or loops. This change adds
the symmetric trim: an `effectiveEndMs` derived from the last sounding note, used by that
block. `TimedRest` already lives in a separate `rests` channel from `notes`, so "sounding
notes only" is just `visibleNotes`.

## Goals / Non-Goals

**Goals:**
- A pure `effectiveEndMs(visibleNotes, {songEndMs})` mirroring `effectiveStartMs`, plus a
  selection-scoped `PlayerData.endMs` getter.
- End-of-song transport (scored finish, unscored loop, Wait-Mode completion) keys off `endMs`.
- Fall back to raw `songEndMs` when the selection has no sounding notes; never exceed it.

**Non-Goals:**
- No change to `songEndMs` itself, to the parser, or to the data model (`songEndMs` stays as
  the raw fallback and is still used for progress-bar extent / display).
- No change to which notes are played or judged; no UI change.
- No trailing **lead-out** padding (the mirror of the start lead-in). The run ends when the
  last note *resolves* (onset + duration), which already gives the note its full ring time.

## Decisions

- **Derive end from `startMs + durationMs`, not `startMs`.** Unlike the start (onset of the
  first note), the end must include the last note's duration so the final note isn't cut off.
  `effectiveEndMs` = `max(n.startMs + n.durationMs)` over `visibleNotes`.
  _Alternative — last onset only:_ rejected, it would clip the final note's sustain.

- **Clamp to `songEndMs` and fall back to it when empty.** `effectiveEndMs` returns
  `songEndMs` when `visibleNotes` is empty, and never returns more than `songEndMs`. This
  keeps behaviour identical for note-free selections and guarantees `endMs <= songEndMs`.
  _Alternative — always compute from notes with no fallback:_ rejected, a no-note selection
  would collapse `endMs` to 0 and end instantly.

- **Compute lazily as a getter, no new stored state.** `PlayerData.endMs => effectiveEndMs(
  visibleNotes, songEndMs: songEndMs)`, exactly parallel to `startMs`. The `advance()` block
  swaps `s.songEndMs` for `s.endMs` in the threshold and scored-finish clamp; the unscored
  branch keeps wrapping to `s.startMs`. Because it derives from `visibleNotes`, a hand change
  recomputes it for free — no extra invalidation, satisfying the sibling-provider rule.
  _Alternative — store `endMs` on state and recompute on load/hand-change:_ rejected as
  redundant with the existing derived-getter pattern used by `startMs`.

- **Guard `endMs > startMs`.** With the clamp and the duration-inclusive max, any selection
  with ≥1 note yields `endMs > startMs`; the empty case falls back to `songEndMs`. No extra
  guard needed in `advance()` beyond the existing `> 0` check.

## Risks / Trade-offs

- [A held final note whose duration overruns `songEndMs`] → the clamp caps `endMs` at
  `songEndMs`, so it can never end *after* the raw end.
- [A piece that is entirely rests for a selection] → `visibleNotes` empty → fall back to
  `songEndMs`; identical to today.
- [Wait Mode expecting to gate the trailing rests] → rests are never gated today (they're not
  in `notes`), so completing at the last note is consistent, not a regression.
- [Progress bar / scrubbing extent] → unchanged; the bar still spans `songEndMs`. Only the
  auto-finish / loop threshold moves earlier. Acceptable and consistent with the trimmed start.

## Migration Plan

Pure additive logic change, no data migration. Rollback = revert the `advance()` threshold to
`s.songEndMs`. Covered by unit tests (`effective_end_test.dart`) and notifier end-of-song
tests before merge.
