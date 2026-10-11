## MODIFIED Requirements

### Requirement: Campaigns have a kind that defines their effect and their clock

The system SHALL let an admin of a product create a **beta campaign** for that product with a key, a name and a
kind. A **`premium_trial`** campaign SHALL carry a `duration_days` (default 90, fixed at
creation) and SHALL grant each enrolled tester `premium` for that many days **from their own
enrolment**; its enrolment MAY be closed, which stops new enrolments and MUST NOT shorten any
existing trial. A **`feature`** campaign SHALL grant no plan, SHALL have no end date, and SHALL be
**closed by the operator** when the feature is stable, which ends every membership at once. A
closed campaign MUST refuse enrolment. Only an admin of a campaign's product SHALL create it,
enrol an account into it, revoke a membership of it, or close or reopen it. A Lingua campaign
SHALL be of kind `feature`.

#### Scenario: Trial campaign grants per-tester premium

- **WHEN** a tester enrols in a 90-day trial campaign
- **THEN** they receive a `premium` row ending 90 days after their enrolment and a membership with the same end

#### Scenario: Closing enrolment leaves trials running

- **WHEN** an admin closes enrolment of a trial campaign
- **THEN** further enrolments are refused and every running trial keeps its end date

#### Scenario: Feature campaign grants membership only

- **WHEN** a tester enrols in a feature campaign
- **THEN** a membership without end date is created and no plan row is written

#### Scenario: Closing a feature campaign ends early access

- **WHEN** an admin closes a feature campaign
- **THEN** every membership becomes inactive and enrolment is refused

#### Scenario: An admin of another product

- **WHEN** an admin of Music only tries to enrol an account in a Lingua campaign
- **THEN** the enrolment is refused as not permitted and nothing changes
