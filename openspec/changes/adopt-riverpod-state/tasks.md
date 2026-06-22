## 1. Dependencies + tooling

- [x] 1.1 Add flutter_riverpod/riverpod_annotation/freezed_annotation + dev riverpod_generator/build_runner/freezed/riverpod_lint/custom_lint
- [x] 1.2 analysis_options: custom_lint plugin + exclude generated; .gitignore `*.g.dart`/`*.freezed.dart`; melos `generate` script

## 2. State stack

- [x] 2.1 `midiServiceProvider` / `scoreSourceProvider` in midi_service.dart
- [x] 2.2 Freezed `PlayerData` model (+ RenderMode, TimedNote) in player_data.dart
- [x] 2.3 `@riverpod` `Player` notifier in player_notifier.dart (ported logic; initial state computed in build)
- [x] 2.4 Remove `PlayerState`; ProviderScope in CymbraApp; PlayerScreen → ConsumerStatefulWidget

## 3. Tests

- [x] 3.1 player_notifier_test.dart via ProviderContainer + overrides (replaces player_state_test.dart)
- [x] 3.2 player_screen_test.dart via UncontrolledProviderScope + overrides; keep 1024px overflow regression
- [x] 3.3 painters import player_data.dart; integration test boots via ProviderScope

## 4. CI + convention

- [x] 4.1 flutter.yml/sonar.yml: `melos run generate` before analyze/test; custom_lint step
- [x] 4.2 CLAUDE.md: State-management convention section
- [ ] 4.3 Confirm in CI: codegen + analyze + custom_lint + tests + coverage ≥ 80% (Rust + merged Flutter)

## 5. Validate

- [x] 5.1 `openspec validate adopt-riverpod-state --strict`
- [ ] 5.2 After review, `openspec archive adopt-riverpod-state`
