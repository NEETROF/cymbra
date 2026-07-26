## ADDED Requirements

### Requirement: Score Load Feedback Across Render Modes

The Flutter player SHALL surface the load state of the selected score in **every**
render mode — the default time-based mode (Synthesia), the Staff mode, and the
Partition mode — not only in Partition mode. While a selected score's bytes are
being fetched and parsed, each mode SHALL show a **loading indicator**; when the
load fails (fetch error, undecodable bytes, or parse error), each mode SHALL show
an **error message**. The player SHALL distinguish "no score selected" (a neutral
empty prompt) from "loading in progress" (the loading indicator), so a fetch in
flight is never rendered as a silent blank surface.

The error message SHALL be a **localized** message only — the raw technical cause
(exception text, gRPC status) SHALL NOT be shown to the user (it is logged
instead).

Opening a score from a browse/library surface SHALL be **guarded**: the score is
pre-loaded (behind a blocking progress indicator) and the player is entered only
once the notation has loaded. On a load failure the user SHALL remain on the
originating screen and be shown a localized notification, rather than being
navigated into a player that then displays an error.

#### Scenario: Loading indicator in the default mode
- **WHEN** a score is selected and its bytes are being fetched/parsed while the
  player is in the default (Synthesia) render mode
- **THEN** a loading indicator is shown, not a blank canvas

#### Scenario: Error surfaced in the default and Staff modes
- **WHEN** loading the selected score fails (e.g. the bytes cannot be fetched) and
  the player is in the Synthesia or Staff render mode
- **THEN** an error message is shown in that mode, rather than an empty, silent
  render

#### Scenario: No selection is distinct from loading
- **WHEN** no score is selected
- **THEN** the player shows a neutral empty prompt, distinct from the loading
  indicator shown while a selected score is being fetched

#### Scenario: Error message is localized, not the raw cause
- **WHEN** a score fails to load with a technical error (e.g. a gRPC/auth failure)
- **THEN** the user sees a localized "could not load" message, and the raw
  exception/gRPC text is not displayed (it is logged)

#### Scenario: Failed open is guarded before entering the player
- **WHEN** the user taps a score in a browse/library surface and it cannot be
  loaded
- **THEN** they stay on that surface with a localized notification, and the
  player screen is not shown
