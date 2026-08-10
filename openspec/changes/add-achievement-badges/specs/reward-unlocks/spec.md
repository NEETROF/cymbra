## MODIFIED Requirements

### Requirement: Two point balances — lifetime and spendable

The system SHALL track two totals from the one append-only ledger: **lifetime earned
points** (the sum of all awards, which only ever rises) and a **spendable balance**
(lifetime earned minus points already redeemed). Levels SHALL be driven by **lifetime
earned** points, so **spending points in the reward shop never lowers a user's level**.
Badges are NOT driven by points at all — they are earned against activity counters owned by
the `achievement-badges` capability — so spending points can never remove a badge either.
The spendable balance SHALL decrease when the user redeems a reward and MUST never go
negative.

#### Scenario: Spending does not lower level

- **WHEN** a user redeems a reward and their spendable balance decreases
- **THEN** their level is unchanged, because it derives from lifetime earned points, and
  their badges are unchanged, because they derive from activity counters

#### Scenario: Balance cannot go negative

- **WHEN** a user attempts to redeem a reward that costs more than their spendable balance
- **THEN** the redemption is refused and the balance is unchanged

### Requirement: Badges earned by milestones

The curation domain SHALL contribute its **milestone counters** — number of ratings
recorded, number of ratings that settled aligned with ground truth, and number of scores the
user was the first to rate — to the badge registry owned by the `achievement-badges`
capability. The curation badge keys and thresholds already granted SHALL be preserved
unchanged, and the durable grant store SHALL remain the one this capability introduced, so
no badge already earned is lost. Curation SHALL NOT define, evaluate or render badges
itself; badges are **earned, not purchased** — they cost no points and cannot be redeemed.

#### Scenario: Curation counters feed the registry

- **WHEN** a user records a rating, has a rating settle aligned, or is the first to rate a
  score
- **THEN** the corresponding curation counter advances and any curation badge whose
  threshold it reaches is earned through the badge registry

#### Scenario: Existing curation badges are preserved

- **WHEN** a user had already earned curation badges before badges moved to the registry
- **THEN** those badges remain earned, under the same keys and thresholds, with their
  original earned moment

#### Scenario: Badges are not for sale

- **WHEN** a user browses the reward shop
- **THEN** badges are not purchasable there; they are only earned through milestones

### Requirement: Full-screen curator profile

The app SHALL provide a **full-screen** "curator profile" that shows the signed-in user:
their **current level and lifetime points** (with progress toward the next level); their
**spendable balance** and an entry into the **reward shop**; and their **personal curation
stats** — number of ratings, coverage contribution, and their own **alignment rate** (the
same reliability figure the back office shows moderators, here as a self-view). The badge
grid SHALL NOT live here: badges are presented in the profile's own Achievements section,
owned by the `achievement-badges` capability. This screen SHALL be driven through injectable
state so it is testable without the native library or a live backend.

#### Scenario: Profile shows level, balance and shop entry

- **WHEN** a signed-in user opens the curator profile
- **THEN** they see their level and lifetime points, their spendable balance, and a way into
  the reward shop

#### Scenario: Curator section carries no badge grid

- **WHEN** the user opens the curator profile
- **THEN** it shows no badge grid; badges appear in the Achievements section instead

#### Scenario: Personal stats include alignment rate

- **WHEN** the user opens the curator profile
- **THEN** it shows their rating count, coverage contribution, and their own alignment rate
