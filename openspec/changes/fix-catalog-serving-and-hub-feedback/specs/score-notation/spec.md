## ADDED Requirements

### Requirement: Score Load Feedback Across Render Modes

The Flutter player SHALL surface the load state of the selected score in **every**
render mode — the default time-based mode (Synthesia), the Staff mode, and the
Partition mode — not only in Partition mode. While a selected score's bytes are
being fetched and parsed, each mode SHALL show a **loading indicator**; when the
load fails (fetch error, undecodable bytes, or parse error), each mode SHALL show
an **error message** describing the failure. The player SHALL distinguish
"no score selected" (a neutral empty prompt) from "loading in progress" (the
loading indicator), so a fetch in flight is never rendered as a silent blank
surface.

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
