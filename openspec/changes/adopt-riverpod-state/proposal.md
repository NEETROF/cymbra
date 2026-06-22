## Why

State was a hand-rolled `ChangeNotifier` with bespoke constructor DI. To
standardize how the app manages state (and keep it testable and consistent as it
grows), the project adopts **Riverpod 2 + Freezed with code generation** as the
single, imposed state-management stack.

## What Changes

- Add Riverpod 2 (`flutter_riverpod`, `riverpod_annotation`, `riverpod_generator`)
  and Freezed (`freezed`, `freezed_annotation`) with `build_runner`.
- Replace `PlayerState extends ChangeNotifier` with an immutable Freezed
  `PlayerData` model and a generated `@riverpod` `Player` notifier.
- Expose dependencies as providers (`midiServiceProvider`, `scoreSourceProvider`),
  overridden with fakes in tests instead of constructor injection.
- Wrap the app in `ProviderScope`; `PlayerScreen` becomes a `ConsumerStatefulWidget`.
- Impose the convention via `riverpod_lint`/`custom_lint`, CI codegen, and docs.
- Generated `*.g.dart`/`*.freezed.dart` are gitignored and produced in CI.

## Capabilities

### New Capabilities
- `state-management`: how the app manages UI state — Riverpod providers, immutable
  Freezed models, and provider-overridable dependencies for testing.

### Modified Capabilities
<!-- None: `midi` behavior is unchanged (pure refactor). -->

## Impact

- Dart: `lib/state/player_data.dart` (new), `lib/state/player_notifier.dart` (new),
  `lib/services/midi_service.dart` (providers added), `lib/main.dart`,
  `lib/screens/player_screen.dart`; removed `lib/state/player_state.dart`.
- Tooling: `pubspec.yaml`, `analysis_options.yaml` (custom_lint), `melos.yaml`
  (`generate`), `.gitignore`, `.github/workflows/{flutter,sonar}.yml` (codegen step).
- Tests: `test/player_notifier_test.dart` (replaces `player_state_test.dart`),
  `test/widgets/player_screen_test.dart` (ProviderScope overrides).
- No user-facing behavior change; coverage stays ≥ 80%.
