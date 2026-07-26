## Context

The player screen (`lib/screens/player_screen.dart`) pushes straight to the play
surface. All the tunables already exist in `PlayerData` + the `Player` notifier —
`selectedHands`/`setSelectedHands`, `speed`/`setSpeed` (0.25–2.0), `metronomeEnabled`/
`toggleMetronome`, `midiPorts`/`connectedDevice`/`selectMidiPort` — but they're only
reachable through the end-drawer, and the score's own details aren't surfaced. The
`openScore` guard already pre-loads the notation before the player mounts, so the
metadata is available on the first frame.

Controls we want to reuse (`_option` radio row, `_handLabel`, the MIDI device list,
`_DifficultyBadge`) are currently private to `player_screen.dart` / `score_card.dart`.

## Goals / Non-Goals

**Goals:**
- One centered modal per opened score, before play, bundling score info + the key
  setup controls.
- Reuse existing state/mutators and share the small controls (no duplication).
- Draft semantics: Validate applies, close (X) cancels; user stays on the player.

**Non-Goals:**
- No new player state or backend change.
- Not moving the settings out of the drawer — the drawer stays; the modal is an
  additional, pre-play surface.
- No "don't show again" preference (the user asked for it on every open).

## Decisions

### D1 — Trigger once per opened score
`PlayerScreen` is pushed fresh on each open, so a per-instance flag suffices. In
`_PlayerScreenState`, guard with a `bool _setupShown` and show the modal after the
notation has a document: check on the first post-frame callback and also via
`ref.listen(notationProvider, …)` for the async-arrival case. This mirrors the
existing `ref.listen(performanceScorerProvider…)` → `showSessionSummary` idiom.

### D2 — Draft state, apply on Validate
The modal is a `StatefulWidget` seeded from the current `PlayerData` (hands, speed,
metronome, selected device). Edits mutate local draft only. **Validate** applies each
changed value through the notifier mutators then pops; **close (X)** pops without
applying. This gives the "validate vs cancel" semantics the user asked for, and
avoids the mid-edit side effects of `setSelectedHands` (which resets the playhead)
until the user commits.

### D3 — Reuse via small shared widgets
Extract `SettingOptionRow` (the radio row) and `handLabel(l10n, Hand)` into
`lib/widgets/setting_option_row.dart`, and `DifficultyBadge` into
`lib/widgets/difficulty_badge.dart` (moved out of `score_card.dart`). The drawer and
`score_card` are updated to consume them, so the drawer and modal share one
implementation. The MIDI device rows + Android `_OtgGuidance` logic are simple enough
to render in the modal from the same `midiPorts`/`connectedDevice` data.

### D4 — Modal layout (matches `showSessionSummary`)
A `Dialog` on `CymbraColors.surfaceContainerLow`, 16px corners,
`ConstrainedBox(maxWidth: 420, maxHeight: screenHeight*0.92)`, a scrollable body
(score info → hands → tempo/metronome → device) and a pinned bottom Validate button,
with a top-right close `IconButton`. `barrierDismissible: false` + `PopScope` so it's
dismissed only via X or Validate. Uses `context.isPhoneLayout` to compact on
phone-landscape.

### D5 — Score info sources
Composer, difficulty (`level`) and title come from the selected `CatalogEntry`
(`selectedScoreProvider`); key/time/tempo come from `PlayerData` (authoritative
post-load) with `NotationData.document.meta` as a fallback for composer/title on
bundled entries. Null facets are omitted (per score-facets: missing tempo stays
unknown).

## Risks / Trade-offs

- **Modal not showing / showing twice** → the `_setupShown` guard + the
  document-ready check must be idempotent; cover with a widget test (shows once,
  re-shows on a fresh instance).
- **`setSpeed`/`setSelectedHands` side effects** → applied only on Validate, before
  play starts, so the playhead reset is harmless.
- **Duplication creep** → extract the shared controls now so the drawer and modal
  don't diverge (D3).

## Migration Plan

Pure additive UI change; no migration. Ships behind no flag (the user wants it
always). Rollback = revert the modal trigger + files.
