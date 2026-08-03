## MODIFIED Requirements

### Requirement: Owner lists and deletes their contributions

A signed-in user SHALL be able to see the scores they have contributed and delete
any of them. Deleting a contribution MUST request its removal from the backend
and reflect the result; the user MUST NOT be offered deletion of scores they do
not own or of bundled-catalog scores. For each contributed score, the list SHALL
also surface its **public-catalog proposal state** — not proposed, `pending`,
`accepted`, or `rejected` — and, for a not-yet-proposed score, offer the opt-in
"propose to the public catalog" action (see `score-catalog-proposal`). A
contribution that has not been proposed remains private; the list is the entry point
to the proposal flow, and once a score is proposed the propose action is hidden for
it.

#### Scenario: Owner sees their contributions

- **WHEN** a signed-in user opens their contributed scores
- **THEN** only that user's contributions are listed

#### Scenario: Owner deletes a contribution

- **WHEN** the owner deletes one of their contributions
- **THEN** the app requests deletion from the backend and, on success, removes it
  from the list

#### Scenario: No delete affordance for non-owned scores

- **WHEN** a bundled-catalog score or a score the user does not own is shown
- **THEN** no delete affordance is offered for it

#### Scenario: Contribution shows its proposal state

- **WHEN** a contribution has been proposed to the public catalog
- **THEN** the list shows its proposal state (`pending`/`accepted`/`rejected`) and does
  not offer the propose action for it

#### Scenario: Not-yet-proposed contribution offers propose

- **WHEN** a contribution has not been proposed
- **THEN** the list marks it as private/not-proposed and offers the opt-in propose action
