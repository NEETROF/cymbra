## ADDED Requirements

### Requirement: The studied-language filter lists the languages the usage report holds
The screen's studied-language filter SHALL offer every studied language present in the per-language breakdown of the usage report for the current window, together with the language currently selected and the choice of every language.
The breakdown is not itself filtered by language — only the per-day series are — so the filter
SHALL be drawn from it. A change of window SHALL NOT leave the filter on a language it no longer
offers.

#### Scenario: A language with activity in the window
- **WHEN** the usage report for the window breaks down activity for `en`
- **THEN** the filter offers `en` alongside every language

#### Scenario: The selected language has no activity in a new window
- **WHEN** the admin has selected `en` and switches to a window whose breakdown holds no `en`
- **THEN** the filter still offers `en`, and still shows it selected

## MODIFIED Requirements

### Requirement: Localised async states on the screen
Every asynchronous resource on the screen (aggregates, series) SHALL be modelled as an exhaustively matched `Async<T>` discriminated union, loaded exclusively through a Pinia store behind the `api()` seam; an RPC failure SHALL yield a localised error message inside the union — never a raw gRPC code or exception on screen, and never an API call from a component.

#### Scenario: An aggregates RPC fails
- **WHEN** `AdminGetLinguaUsage` fails during loading
- **THEN** the screen renders the error state with a localised message, the technical cause being logged only

## REMOVED Requirements

### Requirement: Registry of data-pack versions
**Reason**: It described the test fixture, never the pack readers have. Its source was a manifest
committed with the repo, kept fresh by a test that rebuilds the testdata pack because CI does not
download the real sources; the real pack is built only inside the release workflows. Production
showed `0.0.0-testdata`, 1 142 bytes. It drove no distribution, and it cost a manifest refresh
and a commit on every change to a pack's content.
**Migration**: None for readers — licence attribution reaches them through the extension's
credits, not the console. The studied-language filter it fed is now drawn from the usage report
(see "The studied-language filter lists the languages the usage report holds"). When
over-the-air pack updates are built, their registry is specified against the store they read.
