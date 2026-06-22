## Context

State was `PlayerState extends ChangeNotifier` with constructor-injected
`MidiService`/`ScoreSource`, consumed via `ListenableBuilder`. The team is
standardizing on Riverpod 2 + Freezed (codegen) for all state management.

## Goals / Non-Goals

**Goals:**
- One imposed state-management stack (Riverpod 2 + Freezed + build_runner).
- Keep the MIDI capability behavior identical; keep coverage ≥ 80%.
- Dependencies injectable via provider overrides for tests.

**Non-Goals:**
- No new features; no change to the Rust/FFI layer or the `midi` spec.
- Not migrating to Riverpod 3 (pinned to the 2.x line per request).

## Decisions

- **Riverpod 2 with codegen** (`@riverpod`): `Player` notifier + `playerProvider`,
  plus function providers `midiServiceProvider` / `scoreSourceProvider`.
  riverpod_generator 2.6.5 pulls **Freezed 3** (its analyzer utils require
  `freezed_annotation ^3`), so Freezed 3 syntax (`abstract class … with _$…`).
- **Immutable `PlayerData` (Freezed)** replaces the mutable notifier fields;
  derived helpers (`midiConnected`, `requiredNotesAt`) live on the model.
- **Ticker stays in the widget**: `PlayerScreen` (now `ConsumerStatefulWidget`)
  drives the Ticker and calls `ref.read(playerProvider.notifier).advance(dt*speed)`.
- **`ProviderScope` in `CymbraApp`** (not just `main`) so the app is self-contained
  and the integration test can pump `CymbraApp` directly.
- **Initial state computed in `build()`**: the notifier must not read/assign
  `state` before `build()` returns, so the initial MIDI status is read directly
  and returned; subsequent refreshes run from the 1s timer (post-build).
- **Generated files gitignored + CI codegen**: format runs before codegen (so it
  only checks hand-written source); analyze/lint/test run after.

## Risks / Trade-offs

- Codegen-before-analyze: missing generated files break analyze/IDE → CI runs
  `build_runner` first; documented locally. → Mitigation: `melos run generate`.
- Auto-dispose providers in tests dispose without a listener → tests keep them
  alive via `container.listen(...)` / `UncontrolledProviderScope`.
- custom_lint adds analyzer cost → run as a dedicated `dart run custom_lint` step.
