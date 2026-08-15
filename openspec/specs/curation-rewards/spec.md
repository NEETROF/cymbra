# curation-rewards Specification

## Purpose
TBD - created by archiving change add-curation-rewards. Update Purpose after archive.
## Requirements
### Requirement: Coverage points awarded on rating

When a signed-in user rates a score from the app, the system SHALL award **coverage
points** immediately, sized by a **diminishing** function of how many ratings the score
already has (more for an under-rated score, approaching zero for a well-covered one),
subject to a **per-user daily cap**. Coverage points SHALL be awarded only when the user
**engaged** with the score first; a rating recorded without prior engagement records
normally but earns no coverage points. Engagement SHALL be recorded by **either**
previewing the score in the rating deck **or playing it**: opening a catalog score in the
player, and the ingest of a play session for that score, each record the same engagement
signal — the session ingest so that a score opened from the offline cache, which never
fetches bytes from the server, still counts. Engagement recording SHALL remain idempotent
per (user, score) and best-effort: failing to record it MUST NOT fail the preview, the
player open, or the session ingest. Point values, the diminishing curve, and the daily
cap SHALL be configuration.

#### Scenario: Rating an under-covered score earns more

- **WHEN** a user rates a score that has few existing ratings, after previewing it
- **THEN** they are awarded coverage points, more than for a score that is already well-covered

#### Scenario: Diminishing returns on well-covered scores

- **WHEN** a user rates a score that already has many ratings
- **THEN** the coverage points awarded approach zero

#### Scenario: Daily cap limits farming

- **WHEN** a user has reached the daily coverage-points cap
- **THEN** further ratings that day record normally but award no additional coverage points

#### Scenario: No engagement, no coverage points

- **WHEN** a rating is recorded without the user having previewed or played the score
- **THEN** the rating is stored but no coverage points are awarded

#### Scenario: Playing a score counts as engagement

- **WHEN** a user opens a catalog score in the player and later rates it
- **THEN** the rating is coverage-eligible exactly as if they had previewed it in the deck

#### Scenario: An offline-cached play still counts

- **WHEN** a user plays a catalog score served from the offline cache (no bytes fetched
  from the server) and the play session is ingested
- **THEN** engagement is recorded for that user and score

#### Scenario: Engagement recording is idempotent

- **WHEN** a user previews and also plays the same score, several times
- **THEN** engagement is recorded once for that (user, score) pair and the rating earns
  coverage points at most once

#### Scenario: Failing to record engagement does not fail the play path

- **WHEN** recording engagement fails while opening a catalog score in the player or
  ingesting a play session
- **THEN** the open and the ingest still succeed

### Requirement: Honesty bonus settled against ground truth

The system SHALL award an **honesty bonus** for a rating only at **settlement**, when a
ground truth for the score exists. Ground truth SHALL be the **community consensus** (the
score's aggregate once a minimum number of distinct raters is reached) and/or a
**moderator's accept/reject decision**, with the moderator decision weighted above
consensus when both exist. A rating that **aligns** with the ground truth SHALL earn the
full honesty bonus; a **misaligned** rating SHALL earn a small floor amount that is
**never negative**. Each rating's honesty bonus SHALL be awarded **at most once**.

#### Scenario: Aligned rating earns the honesty bonus

- **WHEN** a score settles and a user's rating agrees with the ground truth
- **THEN** that user is awarded the full honesty bonus for that rating

#### Scenario: Misaligned rating earns a non-negative floor

- **WHEN** a score settles and a user's rating disagrees with the ground truth
- **THEN** that user receives only the small floor amount and never loses points

#### Scenario: Moderator truth outweighs consensus

- **WHEN** both a community consensus and a moderator decision exist for a score and they differ
- **THEN** alignment is judged against the moderator decision

#### Scenario: Honesty is awarded once per rating

- **WHEN** a rating has already been settled
- **THEN** it is not awarded the honesty bonus again

### Requirement: No self-settlement of honesty

A user's rating SHALL NOT be settled for the honesty bonus against a moderation decision
that the **same user** made. Such a rating keeps its coverage points and its place in the
aggregate, but its honesty bonus SHALL wait for an **independent** ground truth (a
consensus formed by other raters, or another moderator's decision).

#### Scenario: Rater who also moderated the score is not self-settled

- **WHEN** a user rated a score from the app and later, as a moderator, decided that same score
- **THEN** their rating is not awarded an honesty bonus on the basis of their own decision

#### Scenario: Independent truth later settles it

- **WHEN** an independent ground truth (other raters' consensus or another moderator) later exists
- **THEN** that user's rating is settled against it, normally

### Requirement: App ratings earn points regardless of role; BO work does not

Points SHALL be awarded based on the **rating coming through the app**, independent of the
rater's role: an admin or moderator who rates from the app earns coverage and honesty
points exactly like any user. Back-office moderation actions (validate, reject, re-queue,
sort, browse) SHALL award **no** points. There is no rating action in the back office.

#### Scenario: Staff rating from the app earns points

- **WHEN** an admin or moderator rates a score from the app
- **THEN** they earn points on the same terms as any other user

#### Scenario: Moderation work earns nothing

- **WHEN** a moderator accepts, rejects, re-queues, or sorts scores in the back office
- **THEN** no points are awarded for those actions

### Requirement: Append-only points ledger

The system SHALL record every award as an entry in an **append-only** ledger (the user, the
kind of award, the amount, the related score, and when), and SHALL derive a user's balance
from it. Awards SHALL be **idempotent** with respect to their triggering event, so a retried
settlement or rating award does not double-count.

#### Scenario: Awards are recorded and summable

- **WHEN** a user earns coverage or honesty points
- **THEN** a ledger entry is appended and the user's balance reflects the sum of their entries

#### Scenario: Retried award does not double-count

- **WHEN** the same award-triggering event is processed more than once
- **THEN** only one award is recorded for it

### Requirement: Course-completion badge

The rewards system SHALL include a **course-completion badge** that is awarded when a user first
completes a notation course. It SHALL be awarded **once per course** (a replay does not re-award),
and it SHALL be surfaced through the existing badge feedback. This adds to the badge set only; the
points, shop, and other reward rules are unchanged.

#### Scenario: Awarded on first course completion

- **WHEN** a user completes a notation course for the first time
- **THEN** the course-completion badge is awarded and surfaced through the normal badge feedback

#### Scenario: Not re-awarded on replay

- **WHEN** a user replays a course they had already completed
- **THEN** no additional course-completion badge is granted for that course

