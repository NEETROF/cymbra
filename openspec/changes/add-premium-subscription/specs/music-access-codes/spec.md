## ADDED Requirements

### Requirement: Campaigns carry the plan and a movable end date

The system SHALL let a music-scope admin create a **campaign** with a name, a plan (`beta` or
`premium`), an end date, and an open/closed state. Every entitlement produced by the campaign's
codes SHALL end at the campaign's end date unless the row carries an explicit override; **moving
the campaign end date SHALL move every such entitlement**. Closing a campaign SHALL stop new
redemptions and MUST NOT shorten existing entitlements by itself.

#### Scenario: Moving the end date extends everyone

- **WHEN** an admin moves a campaign's end date from March 1 to April 1
- **THEN** every entitlement produced by that campaign (without an override) now ends April 1

#### Scenario: Closing stops redemptions only

- **WHEN** an admin closes a campaign
- **THEN** new redemptions of its codes are refused and existing entitlements keep their end date

### Requirement: Codes are single-use, high-entropy, hashed at rest, campaign-bound

A code SHALL be at least 128 bits of randomness, shown once at mint time and stored only as a
hash. A code SHALL belong to exactly one campaign, SHALL be single-use by default (`max_uses = 1`),
and SHALL be revocable individually or by revoking its whole campaign. A revoked code MUST refuse
redemption; revoking a code MUST NOT remove an entitlement already produced by it (that is a
grant revocation).

#### Scenario: Code shown once

- **WHEN** a code is minted
- **THEN** its clear text is returned once and cannot be retrieved again from the system

#### Scenario: Second use is refused

- **WHEN** a single-use code is submitted after it has been redeemed
- **THEN** redemption is refused and nothing changes

#### Scenario: Whole-campaign revocation

- **WHEN** an admin revokes a campaign's codes
- **THEN** none of them can be redeemed and existing entitlements are untouched

### Requirement: One redemption per account per campaign, web-only, rate-limited

Redemption SHALL require a signed-in Cymbra account and SHALL be accepted only through the web
redeem surface (`cymbra.app/redeem`) or the RPC it calls; store builds MUST NOT offer a code entry.
An account SHALL redeem at most one code per campaign; a second code from the same campaign is
refused without consuming it. Redemption attempts SHALL be rate-limited per account and per
address through the platform rate limiter, and refusals MUST NOT reveal whether a code exists.

#### Scenario: Successful redemption creates the entitlement

- **WHEN** a signed-in user redeems a valid, unused code of an open campaign
- **THEN** an entitlement row `source = code` for the campaign's plan is created and the code is marked used

#### Scenario: Second code from the same campaign is refused

- **WHEN** a user who already redeemed a code of campaign C submits another code of C
- **THEN** redemption is refused, the second code is not consumed, and the user keeps their existing entitlement

#### Scenario: Unknown and revoked codes look alike

- **WHEN** a user submits an unknown code or a revoked code
- **THEN** the same neutral refusal is returned in both cases

#### Scenario: Brute force is throttled

- **WHEN** repeated invalid redemptions come from one account or address
- **THEN** further attempts are refused with a rate-limit answer before any lookup

### Requirement: Issuers mint through one port; codes mint free access only

Codes SHALL be minted through one issuing port used by every issuer (the back-office console,
the Discord `/beta` command, any future issuer). The port SHALL record who or what issued the code
and an optional recipient hint. A code MUST only ever grant plan access bounded by its campaign;
the system SHALL have no way to attach a price, a discount or a store offer to a code.

#### Scenario: Two issuers, one lifecycle

- **WHEN** a code minted from the back office and a code minted by the Discord command are redeemed
- **THEN** both follow the same single-use, one-per-account, campaign-bound rules

#### Scenario: Codes cannot express a discount

- **WHEN** the code model and issuing port are inspected
- **THEN** no field can carry a price, percentage or store-offer reference
