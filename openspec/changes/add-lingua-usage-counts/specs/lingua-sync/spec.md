## MODIFIED Requirements

### Requirement: Optional account, local-first preserved
The extension and the container app SHALL remain fully usable without an account: highlighting, statuses, decks, review and local stats all work with no connection and no sign-in, and synchronisation SHALL start only after an explicit sign-in (opt-in). Signing out SHALL stop synchronisation without altering local state. The anonymous usage counts (`lingua-usage-counts`) are not synchronisation: they carry no account and SHALL NOT depend on signing in.

#### Scenario: Account-free use unchanged
- **WHEN** a user without an account reads a page with the extension installed
- **THEN** highlighting, the word popup and review work in full and no synchronisation request is sent to the Cymbra backend

#### Scenario: Signing out loses nothing
- **WHEN** a signed-in user signs out of the extension
- **THEN** their local statuses, cards and stats stay intact and usable offline, and no further sync request is sent

#### Scenario: Counts are not tied to signing in
- **WHEN** a reader with the anonymous usage counts on signs in, then signs out
- **THEN** the counts' reports carry no account before, during or after, and signing in or out neither starts nor stops them

### Requirement: Synchronisation keeps up with the reader
A signed-in device SHALL exchange with the server without the reader having to restart their browser: after a local mutation, when a Lingua surface opens (popup, in-page drawer, side panel), when a page loads or the reader returns to a tab, and when the extension's background wakes. Those triggers SHALL be throttled — at most one exchange every 10 s for a surface the reader opened, every 60 s for a page load — counted from the last successful one, which SHALL survive a background restart. A trigger SHALL be acknowledged only once its exchange is over, so the requesting surface keeps the background alive until then. Réglages SHALL show when this device last synced and SHALL offer a manual exchange, reporting a failure by category, never as a raw message. No exchange SHALL happen while signed out.

#### Scenario: A card captured on another device
- **WHEN** a reader adds a card on their phone, then opens the extension's panel on their Mac
- **THEN** the Mac exchanges with the server without being restarted, and the panel shows the new card

#### Scenario: Repeated openings
- **WHEN** the reader opens the popup three times within a few seconds
- **THEN** a single exchange runs

#### Scenario: A background the browser may suspend
- **WHEN** a surface triggers an exchange
- **THEN** it is told the exchange is done only when it is, and the exchange completes even though the browser suspends an idle background

#### Scenario: Synchronising on demand
- **WHEN** the reader chooses « Synchroniser maintenant » in Réglages while the server is unreachable
- **THEN** they see a categorized message and the displayed last-sync time is unchanged

#### Scenario: Signed out
- **WHEN** any of those moments happens on a signed-out device
- **THEN** no synchronisation request is sent
