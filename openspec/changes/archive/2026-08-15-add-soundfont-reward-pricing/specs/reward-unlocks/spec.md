## MODIFIED Requirements

### Requirement: User redeems a chosen reward from a reward shop

The system SHALL offer a **reward shop** listing redeemable rewards, each with a **point
cost**, and SHALL let a signed-in user **choose which reward to redeem** by spending their
spendable balance. Redemption SHALL deduct the cost from the spendable balance, be
**idempotent** (a retried redemption charges once), and grant the reward **durably**.
Rewards SHALL be limited only by their cost against the user's balance — not gated by level
— so the user is free to choose what to unlock. Redeemable rewards in this change are
**pianos/SoundFonts** (integrating with the existing piano-selection catalog); **temporary
premium access** is listed as a **future** item and is NOT redeemable yet.

The shop SHALL offer **only accepted** SoundFonts. Pricing a font is deliberately
independent of moderation — an operator may set a price while the font is still in review,
so it is ready the moment it is accepted — but an unvalidated (pending or rejected) font
SHALL NOT appear in the shop and SHALL NOT be redeemable by key, so a price never makes an
unvalidated font reachable from the app. This rule applies to both the shop listing and the
redemption lookup.

#### Scenario: User redeems an affordable piano

- **WHEN** a user with enough spendable balance redeems a piano from the shop
- **THEN** the cost is deducted, the piano is granted durably, and it becomes selectable in the app

#### Scenario: Free choice among affordable rewards

- **WHEN** a user can afford several shop rewards
- **THEN** they may choose any of them to redeem, independent of their level

#### Scenario: Redemption is charged once

- **WHEN** the same redemption is submitted more than once
- **THEN** the user is charged only once and the reward is granted once

#### Scenario: Premium item is not redeemable yet

- **WHEN** a user views the future temporary-premium item in the shop
- **THEN** it is shown as coming later and cannot be redeemed

#### Scenario: A priced font still in review is not offered

- **WHEN** an operator prices a SoundFont that is still pending (or was rejected) and a user opens the shop
- **THEN** that font is absent from the listing

#### Scenario: A priced font still in review cannot be redeemed by key

- **WHEN** a user submits a redemption naming a priced but unaccepted SoundFont
- **THEN** the request reports not-found, nothing is granted and nothing is charged
