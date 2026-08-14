## ADDED Requirements

### Requirement: Playing a piece earns points

When a signed-in user's **scored** play session is recorded, the system SHALL award
**performance points** into the same append-only ledger the rating awards use, so playing
raises the user's lifetime points, their level, and their spendable balance exactly as
rating does. A scoreless run SHALL NOT earn performance points, because it carries no
quality signal.

#### Scenario: A scored run earns points

- **WHEN** a signed-in user's scored play session is recorded
- **THEN** points are awarded for it and their lifetime total, level progress and spendable
  balance reflect the award

#### Scenario: A player who never rates still progresses

- **WHEN** a user only ever plays and never rates a score
- **THEN** they still accumulate lifetime points and gain levels

#### Scenario: A scoreless run earns no performance points

- **WHEN** a selective measure-range run is recorded
- **THEN** no performance points are awarded for it

### Requirement: A run below the quality floor earns nothing

Performance points SHALL be awarded only for a run whose overall accuracy reaches a
configured **floor**. A run below the floor SHALL be recorded as activity as it is today but
SHALL award nothing, so an idle keyboard, a piece abandoned partway, or random input pays
zero.

#### Scenario: A poor run pays nothing

- **WHEN** a user records a session whose overall accuracy is below the floor
- **THEN** the session is recorded and no points are awarded

#### Scenario: A good run pays

- **WHEN** a user records a session whose overall accuracy reaches the floor
- **THEN** points are awarded for it

### Requirement: Repeating the same piece pays less each time

The performance award SHALL be sized by a **diminishing** function of how many times that
**same piece** has already paid the user, approaching zero. Replaying one piece SHALL
therefore be worth progressively less, while playing a piece the user has not been paid for
recently SHALL be worth the full amount.

#### Scenario: The first good run of a piece pays most

- **WHEN** a user plays a piece well for the first time
- **THEN** they are awarded more than for a further good run of that same piece

#### Scenario: Grinding one piece approaches zero

- **WHEN** a user replays the same piece well many times
- **THEN** the points awarded for each further run approach zero

#### Scenario: A different piece is worth the full amount

- **WHEN** a user plays a piece they have not been paid for
- **THEN** the award is not reduced by how much they have played other pieces

### Requirement: Harder pieces are worth more

The performance award SHALL scale with the piece's **catalog difficulty**, using the same
difficulty weighting the global leaderboard ranks with, so a harder piece is worth more than
an easier one played equally well. A piece with no known difficulty SHALL be weighted
**neutrally**, never at zero.

#### Scenario: A harder piece pays more

- **WHEN** two pieces of different catalog difficulty are played equally well for the first
  time
- **THEN** the harder one awards more points

#### Scenario: An unleveled piece is not penalised

- **WHEN** a user plays a piece that carries no catalog difficulty, such as their own upload
- **THEN** it is weighted neutrally and still awards points

### Requirement: Play awards are capped per day

The system SHALL enforce a configured **per-user daily cap** on the points playing can earn.
Once reached, further sessions that day SHALL be recorded normally and award nothing.

#### Scenario: Daily cap limits volume

- **WHEN** a user has reached the daily play cap
- **THEN** further sessions that day are recorded normally and award no further points

#### Scenario: A new day restores the allowance

- **WHEN** a new day begins
- **THEN** the user can earn play points again

### Requirement: Practice earns a showing-up award once a day

A **scoreless** practice run over a chosen measure range SHALL earn a small fixed award, at
most **once per the player's local day**, regardless of how many practice sessions or
repetitions that day contained. The local day SHALL be the one the player was in when they
practised — the recorded timestamp shifted by its recorded UTC offset — the same bucketing
the activity surfaces use.

#### Scenario: Drilling a passage is acknowledged

- **WHEN** a user records a scoreless practice run on a day they have not yet been awarded
  for
- **THEN** the practice award is granted

#### Scenario: Further practice the same day awards nothing more

- **WHEN** a user records further practice runs on a day they have already been awarded for
- **THEN** those runs are recorded as activity and award no further points

#### Scenario: The player's local day decides

- **WHEN** two practice runs fall on the same local day but on different UTC days
- **THEN** the practice award is granted once, not twice

### Requirement: Every play award is paid exactly once

Each award SHALL be idempotent against the event that triggered it. Because session ingest is
**at-least-once** — the app retries from a durable outbox until acknowledged — a resent
session SHALL NOT pay a second time, and this SHALL hold independently of whether the session
record itself was newly stored.

#### Scenario: A resent session does not pay twice

- **WHEN** the same play session is ingested more than once
- **THEN** points are awarded for it exactly once

#### Scenario: A retried practice does not pay twice

- **WHEN** the same practice session is ingested more than once on the same local day
- **THEN** the practice award is granted exactly once

### Requirement: Failing to award never fails the session

Awarding points SHALL be **best-effort with respect to the ingest acknowledgement**: if the
award cannot be recorded, the session SHALL still be stored and acknowledged. A player MUST
NOT see their session fail to save because the points could not be written.

#### Scenario: An award failure still saves the session

- **WHEN** the points for a session cannot be recorded
- **THEN** the session is still stored and acknowledged to the app

### Requirement: The player is told what a session earned

The ingest acknowledgement SHALL carry the points awarded for that session, so the app can
show the amount at the end of the run without a further read. When nothing was awarded — a
run below the floor, a piece already paid out, the daily cap reached — the acknowledgement
SHALL report zero.

#### Scenario: A session summary shows what it earned

- **WHEN** a session that earned points is acknowledged
- **THEN** the acknowledgement carries the amount awarded, and the app shows it on the
  session summary

#### Scenario: Nothing earned reports zero

- **WHEN** a session that earned nothing is acknowledged
- **THEN** the acknowledgement reports zero awarded

### Requirement: Play award values are configuration

Every play award value SHALL be **configuration**, retunable without a database migration:
the quality floor, the per-piece diminishing curve, the difficulty weights, the daily cap and
the practice amount. A value lowered later SHALL apply only to future awards; points already
awarded SHALL NEVER be taken back.

#### Scenario: Retuning needs no migration

- **WHEN** an award value is retuned
- **THEN** the new value applies without a database migration

#### Scenario: Lowering a value is not retroactive

- **WHEN** an award value is lowered
- **THEN** points already awarded are unchanged and the user's lifetime total does not fall
