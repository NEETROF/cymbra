# music-access-codes Specification

## Purpose
TBD - created by archiving change add-premium-subscription. Update Purpose after archive.
## Requirements
### Requirement: Campaigns have a kind that defines their effect and their clock

The system SHALL let a music-scope admin create a **beta campaign** with a key, a name and a
kind. A **`premium_trial`** campaign SHALL carry a `duration_days` (default 90, fixed at
creation) and SHALL grant each enrolled tester `premium` for that many days **from their own
enrolment**; its enrolment MAY be closed, which stops new enrolments and MUST NOT shorten any
existing trial. A **`feature`** campaign SHALL grant no plan, SHALL have no end date, and SHALL be
**closed by the operator** when the feature is stable, which ends every membership at once. A
closed campaign MUST refuse enrolment.

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

### Requirement: Codes are single-use, high-entropy, hashed at rest, campaign-bound

A code SHALL be at least 128 bits of randomness, shown once at mint time and stored only as a
hash. A code SHALL belong to exactly one campaign, SHALL be single-use by default (`max_uses = 1`),
and SHALL be revocable individually or for a whole campaign. A revoked code MUST refuse
redemption; revoking a code MUST NOT end a membership or trial already produced by it (that is a
membership revocation).

#### Scenario: Code shown once

- **WHEN** a code is minted
- **THEN** its clear text is returned once and cannot be retrieved again from the system

#### Scenario: Second use is refused

- **WHEN** a single-use code is submitted after it has been redeemed
- **THEN** redemption is refused and nothing changes

#### Scenario: Whole-campaign code revocation

- **WHEN** an admin revokes a campaign's codes
- **THEN** none of them can be redeemed and existing memberships and trials are untouched

### Requirement: Redeeming a code enrols the account, once per campaign, web-only, rate-limited

Redemption SHALL require a signed-in Cymbra account and SHALL be accepted only through the web
redeem surface (`cymbra.app/redeem`) or the RPC it calls; store builds MUST NOT offer a code entry.
Redemption SHALL enrol the account in the code's campaign in one transaction (code marked used,
membership inserted, and for trials the premium row upserted). An account SHALL hold at most one
membership per campaign; an account with an active premium trial SHALL be refused enrolment in
another trial campaign. Attempts SHALL be rate-limited per account and per address through the
platform rate limiter, and refusals MUST NOT reveal whether a code exists.

#### Scenario: Successful redemption enrols

- **WHEN** a signed-in user redeems a valid, unused code of an open campaign
- **THEN** the code is marked used, a membership is created and, for a trial, the premium row is created

#### Scenario: Second code from the same campaign is refused

- **WHEN** a user already enrolled in campaign C submits another code of C
- **THEN** redemption is refused, the second code is not consumed, and the membership is unchanged

#### Scenario: Concurrent trials are refused

- **WHEN** a user with an active premium trial redeems a code of another trial campaign
- **THEN** redemption is refused and the code is not consumed

#### Scenario: Unknown and revoked codes look alike

- **WHEN** a user submits an unknown code or a revoked code
- **THEN** the same neutral refusal is returned in both cases

#### Scenario: Brute force is throttled

- **WHEN** repeated invalid redemptions come from one account or address
- **THEN** further attempts are refused with a rate-limit answer before any lookup

### Requirement: Issuers mint through one port; codes mint free access only

Codes SHALL be minted through one issuing port used by every issuer (the back-office console,
the Discord `/beta` command, any future issuer), which records who or what issued the code and an
optional recipient hint. Nominative enrolment by handle from the console SHALL go through the
same enrolment path without a code (`source = admin`). A code MUST only ever produce a membership
and, for trials, a time-bounded `premium` row; the system SHALL have no way to attach a price, a
discount or a store offer to a code.

#### Scenario: Two issuers, one lifecycle

- **WHEN** a code minted from the back office and a code minted by the Discord command are redeemed
- **THEN** both follow the same single-use, one-per-account-per-campaign, campaign-bound rules

#### Scenario: Nominative enrolment needs no code

- **WHEN** an admin enrols a handle in a campaign from the console
- **THEN** the same membership (and trial row, if applicable) is created with source `admin`

#### Scenario: Codes cannot express a discount

- **WHEN** the code model and issuing port are inspected
- **THEN** no field can carry a price, percentage or store-offer reference

