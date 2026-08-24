## MODIFIED Requirements

### Requirement: The drum audience is staff plus the beta campaign

The system SHALL expose the drum feature to admin and moderator identities, and to
identities holding an **active membership** of the `midi-drums` beta campaign, and
to no one else — including premium subscribers outside the campaign. This audience
SHALL be obtained by declaring one feature-flag key scoped to the `music` app,
defaulting to **off**, and giving it the existing `beta:midi-drums` rollout scope,
which the platform already resolves from the caller's identity and active
memberships. A second, parallel audience mechanism SHALL NOT be introduced.

The flag key SHALL be **`drums.enabled`**, following the registry's
`<feature>.enabled` convention (`rating.enabled`, `onboarding.enabled`, …) — named
here once so the registry constant, the app-snapshot read and the runbook all refer
to the same string instead of each inventing one.

Three vocabularies meet at this feature, and the mapping between them is stated
here once rather than left implicit in scattered scenarios: the stored score facet
value is **`percussion`** (`score-facets`), the product feature the app and this
gate expose is **drums** (flag `drums.enabled`, UI copy, the hub's filter option),
and the campaign key is **`midi-drums`**. They are three names for one axis —
"drums" in any surface means "scores whose instrument facet is `percussion`",
gated by `drums.enabled` under the `beta:midi-drums` scope. The soundfont
instrument column speaks the **same** score vocabulary
(`music.soundfonts.instrument`, `keyboard` | `percussion` — bridged from its
former `piano` spelling by `add-drum-audio-channel`, whose upload boundary still
normalises legacy `piano` input). The one remaining fourth spelling is
`music.courses.instrument` (`DEFAULT 'piano'`), which belongs to the courses
surface, is compared against nothing in the drum feature, and is deliberately
left alone.

#### Scenario: Staff reach the feature

- **WHEN** an admin or moderator identity is evaluated
- **THEN** the drum feature is in effect for them

#### Scenario: Campaign members reach the feature

- **WHEN** an identity holds an active `midi-drums` membership
- **THEN** the drum feature is in effect for them, whatever their plan

#### Scenario: Nobody else reaches it

- **WHEN** an identity is neither staff nor an active campaign member
- **THEN** the drum feature is not in effect for them, even on a premium plan

#### Scenario: Closing the campaign withdraws the feature

- **WHEN** the `midi-drums` campaign is closed
- **THEN** its former members lose the feature at the next evaluation, with no flag
  edit

### Requirement: The backend enforces the drum audience

The backend SHALL enforce the drum audience on **every path that can disclose or
accept a percussion score**. This is an obligation on the whole serving surface,
not a closed list — one missed content-serving path defeats the change — and today
that surface comprises at least: catalog search; the catalog score metadata read
by id (`GetCatalogScore`, which returns title, composer and facets); catalog and
user score bytes (`GetCatalogScoreBytes`; `GetScoreBytes` for a score the caller
does not own); the saved-catalog listing (`ListSavedCatalogScores`); upload; the
rating deck **and** its per-score preview bytes (`GetRatingPreviewBytes`, which
today serves any pending or accepted score's bytes to any signed-in caller,
bypassing the deck listing entirely); the save and rating submissions
(`SaveCatalogScore`, `SubmitScoreRating` — whose success-versus-not-found answers
are existence oracles, and the latter of which would let an ineligible caller rate
a percussion score); the daily unlock (`UnlockCatalogScoreForToday`); and the
non-gRPC audio-preview route (`GET /scores/{catalog_id}/preview`), whose
200-versus-404 answer is an existence oracle and whose body is an audible clip of
the score. Any disclosing or accepting path added later inherits the same
obligation. The leaderboard read surface is exempt only because no percussion play
can be scored yet (see the interim-scoring requirement below); `add-drum-scoring`
SHALL take on this obligation for the leaderboard surface the moment it makes a
percussion play scorable.

Hiding the feature in the app is defence in depth, not the gate. Enforcement SHALL
be resolved from the caller's own identity and memberships, never from a value
supplied in the request.

On a **read** path, a caller without the feature SHALL receive the same answer as
if the score did not exist — absent from listings, not-found by id — rather than a
distinguishable refusal, so the gate does not itself reveal which scores are
percussion. On an **accepting** path (upload, proposal), the caller already holds
the file, so nothing can be disclosed and the not-found shape is incoherent; the
refusal there is necessarily distinguishable and SHALL be a typed refusal the app
maps to its localised reason, never a raw technical string.

The rating deck SHALL keep offering **keyboard and `unknown`** rows to every
rater, and offer **percussion** rows only to the eligible — so a drummer can rate
drum scores and a pianist is never handed one, while the deck no longer inherits
the old grand-staff proxy's silent exclusion of single-staff and unclassified
scores.

The audio-preview **render** job SHALL render a percussion score only with a
**percussion-family font on the drum channel** (`music-score-audio-preview`,
`music-drum-audio`): while no kit preview font is configured, an accepted
percussion score is simply left without a preview — its row unmarked, so the
standard backfill covers it once a kit font is configured — and a piano-font clip
of a drum part is never produced or served, to eligible callers included. The
interim skip installed when this requirement was first written is lifted by
exactly this rule: percussion previews go from "never rendered" to "rendered
with a kit, or honestly absent".

#### Scenario: Search withholds percussion scores

- **WHEN** an ineligible caller searches the catalog
- **THEN** no percussion score appears in the results, whatever filter they supplied

#### Scenario: Direct fetch by id is refused

- **WHEN** an ineligible caller requests a percussion score's metadata or bytes by
  id
- **THEN** the backend withholds it, answering as it would for an unknown id

#### Scenario: Rating preview bytes are withheld

- **WHEN** an ineligible caller requests a percussion score's rating-preview bytes
  by id
- **THEN** the backend answers as it would for an unknown id, rather than serving
  the bytes because the score is pending or accepted

#### Scenario: The audio-preview clip does not leak

- **WHEN** an ineligible caller requests `GET /scores/{catalog_id}/preview` for a
  percussion score
- **THEN** the route answers not-found, indistinguishable from a piece that has no
  preview or does not exist

#### Scenario: A drum score's preview is rendered with a kit, never a piano

- **WHEN** a percussion score is accepted while the kit preview font is
  configured
- **THEN** its preview render is enqueued and produced with the kit font on the
  drum channel, and no piano-font rendition of a drum part is ever produced or
  served

#### Scenario: No kit font configured leaves the preview honestly absent

- **WHEN** a percussion score is accepted while no kit preview font is
  configured
- **THEN** no clip is produced, the row stays unmarked, and the piece renders on
  a later backfill once a kit font is configured

#### Scenario: The deck never deals percussion to the ineligible

- **WHEN** an ineligible rater is dealt a rating deck
- **THEN** it may contain keyboard and `unknown` scores but never a percussion one,
  and a rating submitted against a percussion score's id answers as for an unknown
  id

#### Scenario: An eligible caller is unaffected

- **WHEN** an eligible caller performs the same reads
- **THEN** percussion scores are returned normally

#### Scenario: Enforcement ignores client claims

- **WHEN** a request carries any client-supplied assertion of eligibility
- **THEN** it is disregarded and eligibility is resolved from the caller's identity
  and memberships
