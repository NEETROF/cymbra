## ADDED Requirements

### Requirement: `/redeem` lets a signed-in user redeem an access code

The site SHALL serve `cymbra.app/redeem` (fr) and `/en/redeem` (en): the code field is prefilled
from `?code=` when present, the sign-in island gates the action, and submitting calls the web
plan API. On success the page SHALL show the campaign name, its kind (premium trial with its end
date, or feature beta) and tell the user to open or refresh the app; refusals SHALL be shown with
the API's neutral wording (invalid or used code, already enrolled, another trial running, beta
not open, too many attempts) — never a raw error, never a price or discount.

#### Scenario: Prefilled link redeems in one sitting

- **WHEN** a signed-in user opens `/redeem?code=…` and submits
- **THEN** the campaign name and end date are shown and the account is enrolled

#### Scenario: Not signed in yet

- **WHEN** a signed-out user opens `/redeem`
- **THEN** the sign-in island is shown first, the code stays prefilled, and the redemption proceeds after sign-in

#### Scenario: Neutral refusal

- **WHEN** the code is unknown or already used
- **THEN** the same neutral message is shown in both cases and nothing else changes

### Requirement: The store builds are never pointed at a code field

The redeem page is a web surface only: no store build SHALL link to it or embed it; the app's
own copy about betas points to the community (Discord) and the web site in general terms.

#### Scenario: App has no redeem entry

- **WHEN** the iOS / Android builds are inspected
- **THEN** no code field and no link to `/redeem` exist
