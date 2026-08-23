## MODIFIED Requirements

### Requirement: Riverpod-managed state

App UI state SHALL be exposed through Riverpod 3 providers generated with
`riverpod_generator` (`@riverpod`). New stateful features SHALL NOT use
`ChangeNotifier` or `setState` for application state.

State SHALL NOT be written after its provider has been disposed. Code that resumes
after an `await` SHALL check the notifier is still mounted before assigning `state`,
using the framework's own mounted check rather than a hand-rolled disposal flag.

#### Scenario: UI reads state from a provider
- **WHEN** a widget needs player state
- **THEN** it watches a Riverpod provider (`playerProvider`) rather than a
  `ChangeNotifier` instance

#### Scenario: A late completion does not write to a disposed provider
- **WHEN** an async operation completes after its provider was disposed
- **THEN** the result is discarded without writing state and without throwing

### Requirement: Immutable Freezed state models

State models SHALL be immutable Freezed types, mutated only via `copyWith`.

Because the framework compares successive values with `==` to decide whether to notify
listeners, a state type SHALL define value equality over the fields the UI depends on.
A state carrying a field with identity equality only (such as a byte buffer or a plain
list) SHALL NOT rely on that field alone to trigger a rebuild.

#### Scenario: State update produces a new value
- **WHEN** a notifier changes a field
- **THEN** it assigns `state = state.copyWith(...)` and the previous value is unchanged

#### Scenario: A meaningful change notifies listeners
- **WHEN** a notifier assigns a state that differs in a field the UI reads
- **THEN** watchers rebuild, and are not filtered out by value comparison

## ADDED Requirements

### Requirement: Provider failure policy is explicit

The behaviour of a provider that fails SHALL be a stated decision, not a framework
default inherited by accident. The application SHALL configure whether a failed
provider is retried, and SHALL do so in one place rather than per screen.

Where a failure is a **terminal, user-meaningful outcome** — an unreachable backend, a
score that is not available offline — the provider SHALL surface it as an error state
the UI can render, and SHALL NOT sit in a loading state while the framework retries
behind the user's back.

#### Scenario: A user-meaningful failure is shown, not retried silently

- **WHEN** a provider fails with a classified, user-facing cause
- **THEN** the UI shows the corresponding message rather than remaining in a loading
  state while retries proceed invisibly

#### Scenario: The retry policy is set once

- **WHEN** the application configures its provider container
- **THEN** the retry behaviour is defined there, and individual providers do not each
  restate it

### Requirement: A wrapped provider error is classified by its cause

Code that classifies a failure SHALL classify the **underlying cause**, not any wrapper
the framework adds when propagating an error through a provider read. A classification
that inspects the error type SHALL keep working across that wrapping.

#### Scenario: Offline classification survives error wrapping

- **WHEN** a backend call fails with an unreachable-backend cause and the error reaches
  a consumer through a provider read
- **THEN** the consumer classifies it as unreachable and takes the offline path, exactly
  as it would for an unwrapped error

### Requirement: Connectivity events are not missed while unobserved

A provider exposing an event stream the application relies on for correctness SHALL
keep receiving events for as long as the behaviour depending on them is live, even when
no widget is currently watching it. Connectivity transitions are such a stream: the
offline paths depend on them.

#### Scenario: An offline transition is observed during a load

- **WHEN** a network load is in flight and the device goes offline while no widget is
  watching the connectivity provider
- **THEN** the transition is still delivered to the code awaiting it
