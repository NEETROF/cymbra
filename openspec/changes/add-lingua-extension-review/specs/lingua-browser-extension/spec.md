# lingua-browser-extension — review in the browser

## ADDED Requirements

### Requirement: Two review surfaces
The extension SHALL offer review in the **browser's native panel** where one exists (Side
Panel on Chromium, sidebar on Firefox — the page is pushed, the panel survives navigation)
and in a collapsible **injected panel** (shadow DOM) for micro-reviews. On Safari, which
has no panel API, the injected panel SHALL carry in-browser review on its own. Every
surface SHALL operate on the same local state. Lossless backup/restore (defined by
`lingua-decks-review`) SHALL be reachable from the side panel (file download and
re-import).

#### Scenario: Side panel during navigation
- **WHEN** the user opens the side panel and then navigates to another page
- **THEN** the side panel stays open and its review session continues

#### Scenario: Backup from the side panel
- **WHEN** the user triggers a backup from the side panel
- **THEN** a versioned backup file is downloaded containing the complete state (cards field by field, statuses, calibration, FSRS parameters), and re-importing it restores the state identically
