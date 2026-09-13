## MODIFIED Requirements

### Requirement: Explicit statuses and implicit status by calibration
The model SHALL support the explicit statuses `learning`, `known` and `ignored`; the
absence of an entry means "new". A lemma with no explicit status SHALL be implicitly
known ("presumed") when the pack supplies a CEFR level and that level is below the user's
declared level, or, when no CEFR level is available for the studied language, when its
frequency rank is at or below the user's calibration threshold. Every `known` status
SHALL carry its provenance (`manual`, `calibration`, `srs`, `import`, `exposure`).

Removing a status with a timestamp — the user withdrawing a decision, such as putting a
known or ignored word back to learn — SHALL record a **withdrawal**. A lemma with a
withdrawal and no explicit status SHALL resolve as new, and SHALL NOT be presumed known
by the declared level or by the calibration. A later explicit status SHALL take
precedence over the withdrawal. A status removed without a timestamp (never stamped for
synchronisation) SHALL record no withdrawal and SHALL return to the implicit resolution.

#### Scenario: Calibration at startup
- **WHEN** the user sets their calibration to "I know the 3,000 most common words" without having marked any word
- **THEN** every lemma of rank ≤ 3,000 is classified as known by the analysis

#### Scenario: Presumed known below the declared level
- **WHEN** the user declares level B2 and the pack tags a lemma as A2, with no explicit status for it
- **THEN** the analysis classifies it as known with provenance `calibration` (presumed)

#### Scenario: The explicit status wins over calibration
- **WHEN** a lemma of rank 500 is explicitly marked `learning`
- **THEN** the analysis classifies it as learning despite the calibration

#### Scenario: A withdrawn decision is never presumed
- **WHEN** the user declares level B2, marks the A2 lemma `city` known, and then puts it back to learn
- **THEN** the analysis classifies `city` as unknown (highlighted), not as presumed known

#### Scenario: A new decision overrides a withdrawal
- **WHEN** a lemma was put back to learn and the user later marks it known again
- **THEN** the analysis classifies it as known

### Requirement: Exposure-confirmed known
The model SHALL provide an explicit, caller-driven operation that promotes a lemma to
`known` with provenance `exposure`. The operation SHALL promote a lemma only when ALL of:
it is below the user's declared level (presumed), it has neither an explicit status nor a
withdrawal, and it has been read on at least the configured number of distinct days
(default 4). It SHALL be a no-op for any lemma that has an explicit status or a
withdrawal, so any user interaction (marking, adding to a deck, ignoring, or putting a
word back to learn — including undoing a promotion) permanently blocks promotion. A
promoted `known` SHALL remain distinguishable by its `exposure` provenance so promotions
are reversible in bulk.

#### Scenario: Promotion after repeated reading
- **WHEN** a below-level lemma with no explicit status has been read on 4 distinct days and the promotion operation runs
- **THEN** it becomes `known` with provenance `exposure`

#### Scenario: An interaction blocks promotion
- **WHEN** a below-level lemma read on 6 distinct days was at any point added to the deck (an explicit status)
- **THEN** the promotion operation leaves it unchanged

#### Scenario: Undoing a promotion blocks re-promotion
- **WHEN** a lemma promoted to `known` by exposure is put back to learn, and the promotion operation runs again after further reading
- **THEN** the lemma is not promoted and stays unknown

#### Scenario: Recording alone never promotes
- **WHEN** exposures are recorded but the promotion operation is not called
- **THEN** no status changes, preserving behaviour for callers (e.g. the agent plugin) that never promote

## ADDED Requirements

### Requirement: Withdrawals synchronise as cleared statuses
The model SHALL export every withdrawal alongside the explicit statuses as a `cleared`
status operation, stamped with the time the withdrawal was made. Applying a pulled
`cleared` operation SHALL record a withdrawal under the same last-write-wins rule as any
other status change. Applying it SHALL be reported as a change whenever it alters how the
lemma resolves, even if the lemma had no explicit status on this device.

#### Scenario: An undo reaches another device
- **WHEN** the user marks `city` known on their Mac, syncs both devices, then puts `city` back to learn on the Mac and syncs again
- **THEN** the other device classifies `city` as unknown

#### Scenario: A stale decision loses to a newer withdrawal
- **WHEN** a lemma was withdrawn at time 2,000 and a `known` change stamped at time 1,500 is applied
- **THEN** the change is dropped and the lemma stays withdrawn

#### Scenario: A withdrawal survives a local wipe
- **WHEN** a signed-in user who put a word back to learn resets their statuses and the sync re-pulls the server state
- **THEN** the `cleared` operation restores the withdrawal and the word stays highlighted
