## ADDED Requirements

### Requirement: Two point balances — lifetime and spendable

The system SHALL track two totals from the one append-only ledger: **lifetime earned
points** (the sum of all awards, which only ever rises) and a **spendable balance**
(lifetime earned minus points already redeemed). Levels and badges SHALL be driven by
**lifetime earned** points, so **spending points in the reward shop never lowers a user's
level or removes a badge**. The spendable balance SHALL decrease when the user redeems a
reward and MUST never go negative.

#### Scenario: Spending does not lower level

- **WHEN** a user redeems a reward and their spendable balance decreases
- **THEN** their level and badges are unchanged, because those derive from lifetime earned points

#### Scenario: Balance cannot go negative

- **WHEN** a user attempts to redeem a reward that costs more than their spendable balance
- **THEN** the redemption is refused and the balance is unchanged

### Requirement: Levels derived from lifetime points

The system SHALL derive a user's **level** from their **lifetime earned** points against a
set of configured thresholds. A user's level SHALL only ever rise (it never falls), and the
thresholds SHALL be configuration.

#### Scenario: Crossing a threshold raises the level

- **WHEN** a user's lifetime earned points cross a configured level threshold
- **THEN** their level increases to that tier

#### Scenario: Level does not regress

- **WHEN** a user's lifetime earned points only increase over time
- **THEN** their level never decreases

### Requirement: User redeems a chosen reward from a reward shop

The system SHALL offer a **reward shop** listing redeemable rewards, each with a **point
cost**, and SHALL let a signed-in user **choose which reward to redeem** by spending their
spendable balance. Redemption SHALL deduct the cost from the spendable balance, be
**idempotent** (a retried redemption charges once), and grant the reward **durably**.
Rewards SHALL be limited only by their cost against the user's balance — not gated by level
— so the user is free to choose what to unlock. Redeemable rewards in this change are
**pianos/SoundFonts** (integrating with the existing piano-selection catalog); **temporary
premium access** is listed as a **future** item and is NOT redeemable yet.

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

### Requirement: Badges earned by milestones

The system SHALL grant **badges** at defined milestones (for example a first set of ratings,
a count of aligned ratings, or coverage of rarely-rated scores). Badges are **earned, not
purchased** — they cost no points and cannot be redeemed — and once earned SHALL be kept.

#### Scenario: Milestone grants a badge

- **WHEN** a user meets a badge milestone (e.g. a number of aligned ratings)
- **THEN** the badge is granted and retained

#### Scenario: Badges are not for sale

- **WHEN** a user browses the reward shop
- **THEN** badges are not purchasable there; they are only earned through milestones

### Requirement: Full-screen curator profile

The app SHALL provide a **full-screen** "curator profile" that shows the signed-in user:
their **current level and lifetime points** (with progress toward the next level); their
**spendable balance** and an entry into the **reward shop**; a **badge grid** showing earned
badges and locked ones (locked shown with the milestone hint); and their **personal curation
stats** — number of ratings, coverage contribution, and their own **alignment rate** (the
same reliability figure the back office shows moderators, here as a self-view). This screen
SHALL be driven through injectable state so it is testable without the native library or a
live backend.

#### Scenario: Profile shows level, balance and shop entry

- **WHEN** a signed-in user opens the curator profile
- **THEN** they see their level and lifetime points, their spendable balance, and a way into the reward shop

#### Scenario: Badge grid shows earned and locked

- **WHEN** the user opens the profile
- **THEN** earned badges are shown as earned and unearned badges are shown locked with their milestone hint

#### Scenario: Personal stats include alignment rate

- **WHEN** the user opens the profile
- **THEN** it shows their rating count, coverage contribution, and their own alignment rate

### Requirement: Persistent reward indicator entry point

The app SHALL show a compact, persistent level/points indicator in the hub (and the rating
deck) app bar that opens the full-screen curator profile when tapped, so progress is
glanceable from the main surfaces.

#### Scenario: Indicator opens the profile

- **WHEN** a user taps the level/points indicator in the app bar
- **THEN** the full-screen curator profile opens

#### Scenario: Indicator reflects current standing

- **WHEN** the user's points or level change
- **THEN** the indicator reflects the updated level/points

### Requirement: Immediate and milestone reward feedback

When a user earns coverage points by rating, the app SHALL give **immediate** feedback (a
"+N" points cue on the action). When a user crosses a level, unlocks content, or earns a
badge, the app SHALL show a **celebration** moment for that event.

#### Scenario: Coverage points shown immediately on rating

- **WHEN** a user rates a score and earns coverage points
- **THEN** a "+N" points cue is shown on the rating action

#### Scenario: Level-up / unlock / badge is celebrated

- **WHEN** the user crosses a level, unlocks a piano, or earns a badge
- **THEN** the app shows a celebration for that event

### Requirement: Deferred honesty rewards are surfaced

The app SHALL surface deferred honesty awards after the fact — because the honesty bonus is
awarded later at settlement, not at rating time — via a notification cue on the profile
entry point when new awards have landed, and a **recent-activity** list in the profile
stating what was awarded and why (community consensus or a moderator decision).

#### Scenario: Notification cue for new deferred awards

- **WHEN** a user's ratings have been settled and awarded honesty points since they last looked
- **THEN** a notification cue appears on the profile entry point

#### Scenario: Activity list explains each award

- **WHEN** the user opens the profile's recent-activity list
- **THEN** each deferred award states the amount and whether it came from community consensus or a moderator decision

### Requirement: Redeemable content is visible where it is used

Redeemable content SHALL be shown in the place it is used with its locked state, its
**point cost**, and a way to redeem it. In particular, the piano/SoundFont selection UI
SHALL show a not-yet-redeemed piano with a lock affordance, its cost, and a redeem action
(subject to the user's spendable balance), so the reward is visible and actionable at the
point of use.

#### Scenario: Locked piano shows its cost and a redeem action

- **WHEN** a user opens the piano selection with a piano they have not yet redeemed
- **THEN** that piano is shown locked with its point cost and a redeem action

#### Scenario: Redeemed piano is selectable

- **WHEN** a user has redeemed a piano
- **THEN** that piano appears unlocked and selectable in the selection UI
