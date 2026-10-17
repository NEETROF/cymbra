# Proposal

## Why

A player's best result on a piece already exists — it feeds the per-piece leaderboards —
but the app never shows it as something to improve. A percentage on a board is a
comparison with others; **stars** on the piece itself are a promise to oneself: two stars
means "I can come back for the third". This change turns the stored best into one to three
stars on every surface where the piece appears.

## What Changes

- **One to three stars from the best overall synchronisation.** One star from 60 %, two
  from 80 %, three from 95 %. The thresholds are app constants, aligned with the piece
  step's default pass threshold (80 % = two stars) so "passed" and "two stars" mean the
  same thing.
- **Derived from the best the server already keeps.** The per-piece best used by the
  leaderboards is read through the existing standings read, extended with the best overall
  percentage. Stars never decrease, because the best never does.
- **Shown where the piece is.** On score cards in the library and the hub, on the learning
  path's piece nodes, in the end-of-run summary (stars reached by this run, and whether a
  new star was earned), and on the pieces of today's session.
- **A new star is a moment.** Earning a star the player did not have raises the celebration
  cue; the visuals are the motion change's, the trigger is here.
- **Pieces without a scored run show empty stars**, never a dash, so the invitation is
  visible.

## Capabilities

### New Capabilities
- `music-piece-stars`: the star thresholds, their derivation from the best overall
  synchronisation, the surfaces that show them and the new-star cue.

### Modified Capabilities
- None. The standings read is extended with an additive field; the leaderboards' own
  requirements do not change.

## Impact

Products: **Music only**. Consumed, not new: the per-piece best kept by the leaderboards,
the coalesced standings read used by score cards, the session result, the path nodes from
`add-music-piece-lessons`, the celebration cue. New: a stars derivation, a stars widget, one
additive proto field.

- Flutter: `state/piece_stars.dart` (pure derivation + provider over `MyStandings`),
  `widgets/star_row.dart`, score card, path node, summary and daily-session block changes.
- Backend: `LeaderboardStanding` gains `best_overall_sync_pct` in the `GetMyStandings`
  response — **additive**, `buf breaking` passes; the value is the best already stored.
- Tests: derivation at every threshold edge, provider merge with standings, widget states
  (none, one, two, three, new star), Rust response mapping with a mockall repo double.
