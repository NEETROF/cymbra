## ADDED Requirements

### Requirement: A single cross-domain badge registry

The system SHALL define every badge in **one registry**. Each badge SHALL declare a stable
`key`, a **family**, the **metric** it is measured against, the **threshold** that earns it,
and — when it belongs to a graduated series — a **track** and a **tier** within that track.
No other component SHALL enumerate badges: the profile grid, the awarding logic and the wire
projection SHALL all derive from the registry. Adding a badge to the registry SHALL be
sufficient for it to appear, be awarded and be rendered, **without an app release**.

#### Scenario: A badge added to the registry appears everywhere

- **WHEN** a new badge entry is added to the registry
- **THEN** it is listed in the badge read, evaluated for awarding, and rendered in the app
  with its label and description, with no app release

#### Scenario: Nothing enumerates badges outside the registry

- **WHEN** the badge grid is rendered
- **THEN** every tile comes from the registry, and the client holds no hard-coded list of
  badge keys, labels or metrics

### Requirement: Badge families span the app's activity, not only curation

The registry SHALL organise badges into families covering the user's activity across the
app: **play**, **consistency**, **ranking**, **contribution** and **curation**. A
**learning** family SHALL be declared for course achievements but SHALL ship no badge in
this change, because no server-side course progress exists to measure. A family with no
badges SHALL NOT render.

#### Scenario: A player who never rates can still earn badges

- **WHEN** a user records play sessions but has never rated a score
- **THEN** they earn play and consistency badges, and their achievements surface is not a
  wall of locked curation badges

#### Scenario: Empty learning family is not shown

- **WHEN** the achievements surface is rendered and the learning family has no badges
- **THEN** no learning section appears

### Requirement: Badge metrics are sourced from existing activity records

Badge counters SHALL be derived from the records the system already keeps, with no new
per-user tracking table:

- **play** — recorded play sessions, distinct pieces played, and sessions above an accuracy
  threshold
- **consistency** — distinct local days played and the longest run of consecutive local
  days, bucketed by the player's local day (the recorded session timestamp shifted by its
  recorded UTC offset), the same bucketing the activity heatmap uses
- **ranking** — per-piece board placements and closed-season standings
- **contribution** — accepted catalog score proposals and accepted SoundFont contributions
  attributed to the user
- **curation** — the existing rating, aligned-rating and first-rater counters

All counters for a badge read SHALL be gathered in **one** repository call.

#### Scenario: Consistency uses the player's local day

- **WHEN** a player records two sessions that fall on the same local day but on different
  UTC days
- **THEN** the consistency counters count one day, not two

#### Scenario: Counters are fetched once per read

- **WHEN** the achievements surface is read
- **THEN** the counters for every badge are gathered in a single repository call, not one
  call per badge

### Requirement: Awarding is idempotent and retroactive

A badge SHALL be awarded when its counter reaches its threshold, and the award SHALL be
recorded durably with the **moment it was earned**. Awarding SHALL be **idempotent**: a
repeated evaluation SHALL NOT create a second award nor change the recorded moment. A badge
whose criterion a user **already** satisfies when the badge is first defined SHALL be
awarded to them **without a backfill**, on their next badge read.

#### Scenario: Repeated evaluation awards once

- **WHEN** a user's badges are evaluated repeatedly after they crossed a threshold
- **THEN** the badge is awarded once and its earned moment does not change

#### Scenario: A newly defined badge is awarded retroactively

- **WHEN** a badge is added whose criterion a user already satisfies
- **THEN** that user is shown as having earned it on their next badge read, with no backfill
  step

### Requirement: An earned badge is never lost

A badge SHALL be shown as **earned** when it has ever been awarded **or** when its counter
currently clears its threshold. Consequently a badge SHALL NOT be lost when the underlying
activity records change — for example when a rated or played piece is removed from the
catalog, when a contribution returns to review, or when old activity ages out of retention.
Spending points SHALL NOT remove a badge, and badges SHALL NOT be purchasable.

#### Scenario: Purged activity does not remove a badge

- **WHEN** a user has earned a badge and the activity records behind its counter are
  subsequently removed
- **THEN** the badge is still shown as earned, with its original earned moment

#### Scenario: Badges are earned, not bought

- **WHEN** a user browses anything that can be redeemed with points
- **THEN** no badge is offered for purchase, and redeeming does not remove any badge

### Requirement: The badge read carries progress, provenance and identity

The badge read SHALL return, for **every** badge in the registry — earned or not — its key,
family, metric, threshold, track and tier, whether it is **earned**, the user's **current
value** for that metric, the **moment it was earned** when it has been awarded, and a
**localized label and description**. The current value SHALL be **clamped to the threshold**
for an earned badge, so an earned badge never renders as incomplete. Localized text SHALL be
delivered as a per-language map that the client resolves against its active display
language, falling back to English.

#### Scenario: Locked badge exposes how far along the user is

- **WHEN** a user has 12 of the 25 occurrences a badge requires
- **THEN** the read returns the badge as not earned with a current value of 12 and a
  threshold of 25, so the app can show "12/25"

#### Scenario: Earned badge reports its date and full progress

- **WHEN** a badge has been awarded
- **THEN** the read returns it as earned, with the moment it was earned, and its current
  value clamped to the threshold

#### Scenario: Label follows the displayed language

- **WHEN** the app's display language differs from the account's stored language
- **THEN** badge labels and descriptions render in the **displayed** language, falling back
  to English when that language is missing

### Requirement: Badges are read through their own endpoint

The system SHALL expose a **dedicated authenticated read** for the signed-in user's badges,
separate from the curation rewards read, because badges are no longer curation-scoped. The
curation rewards read SHALL keep returning its curation badges, marked deprecated, so an
already-released app version keeps working.

#### Scenario: Signed-in user reads their badges

- **WHEN** a signed-in user requests their badges
- **THEN** they receive the full registry projected against their own counters

#### Scenario: Older app version keeps its grid

- **WHEN** an app version that predates this change reads curation rewards
- **THEN** it still receives the curation badges it knows how to render

### Requirement: Achievements are their own profile section

The app SHALL present badges in a dedicated **Achievements** section of the user profile,
**grouped by family**, with earned badges shown before locked ones. Each badge SHALL show
its own icon, its label, and — when locked — a **progress indicator** toward its threshold.
A badge earned since the user last visited the section SHALL be marked as **new**. The
section SHALL be driven through injectable state so it is testable without the native
library or a live backend.

#### Scenario: Grid groups by family, earned first

- **WHEN** a user opens the Achievements section
- **THEN** badges are grouped by family, and within a family earned badges precede locked
  ones

#### Scenario: Locked badge shows progress

- **WHEN** a locked badge's metric stands at 12 of 25
- **THEN** its tile shows a progress indicator reflecting 12 of 25, not only the target

#### Scenario: Newly earned badge is marked

- **WHEN** a user earned a badge since their last visit to the section
- **THEN** that badge is marked as new, and the mark clears once the section has been seen

### Requirement: A graduated series renders as one tile

A badge track SHALL render as a **single tile** showing the **highest tier the user has
earned** and the progress toward the next tier — not one tile per tier. Opening a badge
SHALL present a **detail view** stating what it takes to earn it, where the user currently
stands, when they earned it if they have, and, for a track, **every tier in the ladder**
with the ones already earned marked.

#### Scenario: Track collapses to the current tier

- **WHEN** a user has earned tier 2 of a three-tier track
- **THEN** the grid shows one tile for that track at tier 2 with progress toward tier 3

#### Scenario: Detail view shows the whole ladder

- **WHEN** the user opens a badge that belongs to a track
- **THEN** the detail view lists every tier with its threshold, marking the ones earned and
  showing the user's current standing
