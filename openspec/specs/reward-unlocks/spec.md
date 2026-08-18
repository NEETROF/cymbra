# reward-unlocks Specification

## Purpose
TBD - created by archiving change add-curation-rewards. Update Purpose after archive.
## Requirements
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
— so the user is free to choose what to unlock. Redeemable rewards are **pianos/SoundFonts**
(integrating with the existing piano-selection catalog). **Premium access is not a shop
item**: points cannot buy a plan; a font flagged `redeemable = false` SHALL be presented as
**included in premium** (not "coming later") and SHALL refuse redemption by points. A caller
whose effective plan grants the `soundfonts.library` unlock SHALL see every shop item as
**owned** without a grant row and without being charged; the shop MUST NOT charge points for
an item the plan already unlocks.

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

#### Scenario: Premium-only item is not redeemable by points

- **WHEN** a user views a `redeemable = false` font in the shop
- **THEN** it is shown as included in premium and cannot be redeemed with points

#### Scenario: Plan holder owns the shop

- **WHEN** a user whose plan grants `soundfonts.library` opens the shop or attempts a redemption
- **THEN** every item is reported as owned and no points are charged

#### Scenario: A priced font still in review is not offered

- **WHEN** an operator prices a SoundFont that is still pending (or was rejected) and a user opens the shop
- **THEN** that font is absent from the listing

#### Scenario: A priced font still in review cannot be redeemed by key

- **WHEN** a user submits a redemption naming a priced but unaccepted SoundFont
- **THEN** the request reports not-found, nothing is granted and nothing is charged

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

When a user earns points for an action, the app SHALL give **immediate** feedback on that
action — a "+N" points cue where the work happened. This SHALL apply to **every** earning
action, not only rating: coverage points show their cue on the rating action, and points
earned by playing show theirs on the session summary at the end of the run. When a user
crosses a level, unlocks content, or earns a badge, the app SHALL show a **celebration**
moment for that event, whichever activity caused it.

#### Scenario: Coverage points shown immediately on rating

- **WHEN** a user rates a score and earns coverage points
- **THEN** a "+N" points cue is shown on the rating action

#### Scenario: Play points shown on the session summary

- **WHEN** a user finishes a run that earned points
- **THEN** a "+N" points cue is shown on the session summary

#### Scenario: A session that earned nothing shows no cue

- **WHEN** a user finishes a run that earned no points
- **THEN** no points cue is shown, and the session summary is otherwise unchanged

#### Scenario: Level-up / unlock / badge is celebrated

- **WHEN** the user crosses a level, unlocks a piano, or earns a badge
- **THEN** the app shows a celebration for that event

#### Scenario: A level crossed by playing is celebrated the same way

- **WHEN** a user crosses a level because of points earned by playing
- **THEN** the same celebration is shown as for a level crossed by rating

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

