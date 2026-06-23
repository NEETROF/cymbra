## ADDED Requirements

### Requirement: Riverpod-managed state

App UI state SHALL be exposed through Riverpod providers generated with
`riverpod_generator` (`@riverpod`). New stateful features SHALL NOT use
`ChangeNotifier` or `setState` for application state.

#### Scenario: UI reads state from a provider
- **WHEN** a widget needs player state
- **THEN** it watches a Riverpod provider (`playerProvider`) rather than a
  `ChangeNotifier` instance

### Requirement: Immutable Freezed state models

State models SHALL be immutable Freezed types, mutated only via `copyWith`.

#### Scenario: State update produces a new value
- **WHEN** a notifier changes a field
- **THEN** it assigns `state = state.copyWith(...)` and the previous value is unchanged

### Requirement: Overridable dependency providers

External dependencies (the MIDI engine, the score source) SHALL be exposed as
Riverpod providers so tests can replace them via provider overrides rather than
constructor injection.

#### Scenario: Tests inject fakes
- **WHEN** a test needs a fake MIDI engine
- **THEN** it overrides `midiServiceProvider` (and `scoreSourceProvider`) in a
  `ProviderScope`/`ProviderContainer` with a fake implementation

### Requirement: Generated code is reproducible and not committed

Generated sources (`*.g.dart`, `*.freezed.dart`) SHALL be excluded from version
control and produced by `build_runner` locally and in CI before analyze/test.

#### Scenario: CI regenerates before checks
- **WHEN** CI runs analyze, lints, or tests
- **THEN** it first runs `build_runner` (e.g. `melos run generate`) so generated
  files exist
