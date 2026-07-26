## 1. Shared controls (extract for reuse)

- [x] 1.1 Add `lib/widgets/setting_option_row.dart`: a public `SettingOptionRow` (selected/label/onTap radio row, from the drawer's `_option`) and `handLabel(AppLocalizations, Hand)`.
- [x] 1.2 Add `lib/widgets/difficulty_badge.dart`: a public `DifficultyBadge` moved from `score_card.dart`'s `_DifficultyBadge`; update `score_card.dart` to use it.
- [x] 1.3 Update `_SettingsDrawer` in `player_screen.dart` to use `SettingOptionRow` and `handLabel` (no behavior change).

## 2. The modal

- [x] 2.1 Add `lib/screens/pre_play_setup_modal.dart` with `Future<void> showPrePlaySetup(BuildContext, WidgetRef)` — a `Dialog` matching `showSessionSummary` (maxWidth 420, maxHeight 0.92×screen, scrollable body + pinned Validate, top-right close X, `barrierDismissible:false` + `PopScope`).
- [x] 2.2 Score-info section: title + composer (from `selectedScoreProvider`, fallback `NotationData.document.meta`), `DifficultyBadge` for level, and key/time/tempo from `PlayerData` — omit any unknown facet.
- [x] 2.3 Hands section: `SettingOptionRow` per `Hand`, shown only when `hasMultipleStaves`; edits the local draft.
- [x] 2.4 Tempo + metronome section: a speed slider (0.25–2.0) and a metronome toggle, editing the draft.
- [x] 2.5 MIDI device section: show connected device / none, an "Auto" row + a row per port (`SettingOptionRow`), reuse the Android `_OtgGuidance`/empty state; edits the draft device.
- [x] 2.6 Validate applies the draft via `setSelectedHands` / `setSpeed` / `toggleMetronome` (only if changed) / `selectMidiPort` (only if changed), then pops; close (X) pops without applying.

## 3. Trigger + l10n

- [x] 3.1 In `_PlayerScreenState`, show the modal once per opened score after the notation has a document (post-frame check + `ref.listen(notationProvider…)`), guarded by a `_setupShown` flag.
- [x] 3.2 Add l10n strings (modal title, section headers "Play with"/"Tempo"/"MIDI device", Validate) to `app_{en,fr,es,it}.arb`; reuse existing `handLeft/Right/Both`, `tempo`, level labels, MIDI device labels; regen localizations.

## 4. Persist the play settings

- [x] 4a.1 Add `lib/state/player_preferences.dart`: a keepAlive, device-persisted `PlayerPreferences` (Freezed `PlayerPrefs` — hands, speed, metronome, midiPort) using `preferencesServiceProvider` (JSON under one key), mirroring the `AppLocale` restore/persist pattern.
- [x] 4a.2 Seed `PlayerData` from it in `Player.build` (hands/speed/metronome, re-apply a remembered MIDI device), and write through in `setSelectedHands` / `setSpeed` / `toggleMetronome` / `selectMidiPort`; remove the old in-memory `MetronomeEnabled` provider.
- [x] 4a.3 Warm the provider at startup (library screen) so a cold-start open restores before the first player seeds.
- [x] 4a.4 Unit test `player_preferences_test.dart`: defaults, restore from a seeded store, each setter persists, corrupt value falls back.

## 5. Tests + validation

- [x] 4.1 Widget test: opening the player shows the modal (score title/composer visible); Validate with a changed hand applies it to `playerProvider` and closes; close (X) leaves settings unchanged.
- [x] 4.2 Widget test: single-staff piece → no hand chooser; multi-staff → hand chooser present.
- [x] 4.3 `flutter analyze`, `dart run custom_lint`, `flutter test --exclude-tags golden` green; `dart format` clean.
- [x] 4.4 `openspec validate add-pre-play-setup-modal --strict` passes.
