## MODIFIED Requirements

### Requirement: Redeeming a code enrols the account, once per campaign, web-only, rate-limited

Redemption SHALL require a signed-in Cymbra account and SHALL be accepted only through the web
redeem surface — the site page `cymbra.app/redeem` calling the browser JSON route
(`music-plan-web-api`) with a `web`-audience session, the RPC `RedeemAccessCode` remaining the
seam behind it; store builds MUST NOT offer a code entry, and a `music`-audience token MUST be
refused. Redemption SHALL enrol the account in the code's campaign in one transaction (code marked
used, membership inserted, and for trials the premium row upserted). An account SHALL hold at most
one membership per campaign; an account with an active premium trial SHALL be refused enrolment in
another trial campaign. Attempts SHALL be rate-limited per account and per address through the
platform rate limiter, and refusals MUST NOT reveal whether a code exists.

#### Scenario: Successful redemption enrols

- **WHEN** a signed-in user redeems a valid, unused code of an open campaign
- **THEN** the code is marked used, a membership is created and, for a trial, the premium row is created

#### Scenario: Store-build token is refused

- **WHEN** a redemption arrives with a `music`-audience token
- **THEN** it is refused and nothing is consumed

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
