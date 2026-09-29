## RENAMED Requirements

- FROM: `### Requirement: The drum audience is staff plus the beta campaign`
- TO: `### Requirement: The drum feature is open to every player behind a kill-switch`

## MODIFIED Requirements

### Requirement: The drum feature is open to every player behind a kill-switch

The system SHALL expose the drum feature to every player. It SHALL remain behind
one feature-flag key scoped to the `music` app, defaulting to **on**, which serves
as the feature's **kill-switch**: turning the flag off in the back office SHALL
withdraw the feature from every caller, staff included, with no release. The
`midi-drums` campaign that carried the beta SHALL have no effect on the audience; a
second, parallel audience mechanism SHALL NOT be introduced.

The flag key SHALL be **`drums.enabled`**, following the registry's
`<feature>.enabled` convention (`rating.enabled`, `onboarding.enabled`, …) — named
here once so the registry constant, the app-snapshot read and the runbook all refer
to the same string instead of each inventing one.

Two vocabularies meet at this feature, and the mapping between them is stated here
once rather than left implicit in scattered scenarios: the stored score facet value
is **`percussion`** (`score-facets`), and the product feature the app and this gate
expose is **drums** (flag `drums.enabled`, UI copy, the hub's filter option). They
are two names for one axis — "drums" in any surface means "scores whose instrument
facet is `percussion`", gated by `drums.enabled`. The soundfont instrument column
speaks the **same** score vocabulary (`music.soundfonts.instrument`, `keyboard` |
`percussion` — bridged from its former `piano` spelling by
`add-drum-audio-channel`, whose upload boundary still normalises legacy `piano`
input). The one remaining other spelling is `music.courses.instrument`
(`DEFAULT 'piano'`), which belongs to the courses surface, is compared against
nothing in the drum feature, and is deliberately left alone.

#### Scenario: Every player reaches the feature

- **WHEN** any signed-in player is evaluated and `drums.enabled` holds its default
- **THEN** the drum feature is in effect for them, whatever their plan and whatever
  campaigns they belong to

#### Scenario: The kill-switch withdraws the feature from everyone

- **WHEN** an operator turns `drums.enabled` off
- **THEN** the drum feature is no longer in effect for any caller, staff included,
  at the next evaluation

#### Scenario: The former beta campaign changes nothing

- **WHEN** the `midi-drums` campaign is closed, or a player was never a member
- **THEN** their access to the drum feature is unchanged

### Requirement: The drum gate fails closed

The drum gate SHALL NOT let a degraded dependency decide the audience in either
direction. When the flag store cannot be read, the gate SHALL serve the flag's last-known value,
or its **code default** — on — when none was ever loaded, so a flag-store outage
neither hides a released feature from every player nor lifts a kill-switch the
service already knew about.
When the flag service itself is not wired into the serving module, the gate SHALL
treat the feature as off: that is a deployment defect, not an outage, and SHALL NOT
widen what an unwired module serves. Beta memberships SHALL play no part in the
decision.

Uncertainty about a **score** is the opposite case and SHALL NOT withhold it: the
gate withholds a score whose recorded instrument **is** `percussion`, and a score
recorded as `unknown` SHALL stay as reachable as it is today. Withholding `unknown`
rows would hide the large majority of the existing corpus from the users who can
reach it now — a regression, not a safety gain.

This is only safe once every stored row carries a **derived** instrument. The
corpus already contains percussion scores — they were ingested despite the
playable-notes gate — so a row recorded as `unknown` **can** be percussion until it
has been re-derived. The instrument backfill SHALL therefore classify from the
stored bytes rather than translate the former staff-count flag, and the gate SHALL
NOT be relied upon as a boundary before that pass has completed.

#### Scenario: An unreadable flag store serves the default

- **WHEN** the flag store cannot be read and no value was ever loaded
- **THEN** the drum feature follows the flag's code default, on, for every caller

#### Scenario: An unwired flag service denies

- **WHEN** the serving module has no flag service wired
- **THEN** the drum feature is treated as off for every caller

#### Scenario: Unknown scores stay reachable

- **WHEN** a caller without the feature reads a score whose recorded instrument is
  `unknown`
- **THEN** it is served exactly as it is today, rather than withheld on suspicion

#### Scenario: Existing percussion rows are classified, not assumed absent

- **WHEN** the instrument is backfilled over a corpus that already contains
  percussion scores
- **THEN** each row's instrument is derived from its stored bytes, so a percussion
  row is recorded as `percussion` and not left as `unknown`

#### Scenario: Only recorded percussion is withheld

- **WHEN** the gate decides whether to withhold a score
- **THEN** it withholds on the instrument being `percussion`, never on the
  instrument being merely unrecorded
