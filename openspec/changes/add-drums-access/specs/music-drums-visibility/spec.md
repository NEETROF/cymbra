## ADDED Requirements

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
gated by `drums.enabled` under the `beta:midi-drums` scope. The pre-existing
soundfont and course instrument columns use a fourth spelling, `piano`
(`music.soundfonts.instrument`, `music.courses.instrument`, both
`DEFAULT 'piano'`); bridging that to `keyboard` is `add-drum-audio-channel`'s
obligation, noted here so it is not re-derived from scratch.

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

The audio-preview **render** job is the server-side twin of the console's Play
guard and SHALL NOT bake a piano-font clip of a percussion score: an accepted
percussion score is simply left without a preview until `add-drum-audio-channel`
can render it with a drum kit, so the gated route above has nothing wrong to
serve to anyone — eligible callers included.

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

#### Scenario: No piano clip is baked for a drum score

- **WHEN** a percussion score is accepted into the catalog
- **THEN** no preview render is enqueued for it, so no piano-font rendition of a
  drum part is ever produced or served

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

### Requirement: The gate does not apply to a caller's own uploads

The drum gate SHALL NOT apply to a caller's **own** user scores: listing one's
uploads, fetching their bytes and deleting them SHALL work regardless of drum
eligibility. There is no disclosure in showing a caller a file they themselves
uploaded, and gating these paths would produce two real harms for a former tester
(the campaign closes, or the flag is cleared): storage quota held by scores they
can no longer see or remove, and the incoherent surface where a score is invisible
in their library yet deletable by id. Pushing a score toward the **public**
catalog is different — `ProposeScore` SHALL require drum eligibility for a
percussion score, and uploading a **new** percussion score stays gated (see the
upload requirement).

#### Scenario: A former tester keeps their own drum uploads

- **WHEN** a caller who is no longer drum-eligible lists their uploaded scores or
  fetches one of their own drum scores' bytes
- **THEN** the drum scores they uploaded are listed and served normally

#### Scenario: A former tester can delete their own drum upload

- **WHEN** a caller who is no longer drum-eligible deletes a drum score they
  uploaded
- **THEN** the deletion succeeds

#### Scenario: Proposing still requires eligibility

- **WHEN** a caller who is no longer drum-eligible proposes one of their own drum
  scores to the public catalog
- **THEN** the proposal is refused

### Requirement: Percussion plays engage no scoring artifact before drum scoring exists

A percussion score's play submissions SHALL NOT engage play rewards,
leaderboards, streak accounting or daily-access consumption until
instrument-aware scoring exists (`add-drum-scoring`): each of those ingest sites
SHALL fail closed when the played score's instrument is `percussion`. The rationale is that these artifacts
are permanent by design — reward ledger rows, monotone leaderboard bests, badges
that are never lost — while the scorer that would feed them is keyboard-shaped:
running it against a drum part's GM key numbers would bake wrong data that cannot
be cleanly unwound when real drum scoring lands. A tester's percussion run is
simply unscored; nothing is stored that `add-drum-scoring` would have to repudiate.

#### Scenario: A percussion run leaves no permanent trace

- **WHEN** an eligible tester completes a run on a percussion score
- **THEN** no reward points are granted, no leaderboard best is ingested, no
  streak day is credited and no daily-access slot is consumed

#### Scenario: Keyboard plays are unaffected

- **WHEN** any caller completes a run on a keyboard score
- **THEN** rewards, leaderboards, streak and daily access behave exactly as before

### Requirement: The app declares a percussion score not playable yet

While the app lacks a percussion presentation, opening a percussion score SHALL
show a localised "drums not playable yet" state instead of entering the player.
This is the app-side mirror of the console's preview-and-Play guard, and it exists
for the same reason: without it the player would render the score confidently and
wrongly — the waterfall would draw its GM key numbers as falling **piano** notes
(the numbers 35–81 all sit inside the piano range) and the piano synthesizer would
sound them — which a tester would reasonably read as the feature being broken
rather than unbuilt. The state SHALL be distinct from error states: the file is
fine, the presentation is not built yet. This requirement is interim by
construction and is removed by `add-drum-kit-view`, which supplies the percussion
presentation.

#### Scenario: Opening a drum score shows the interim state

- **WHEN** an eligible tester opens a percussion score in the app
- **THEN** a localised "drums not playable yet" state is shown and the player is
  not entered

#### Scenario: Keyboard scores open normally

- **WHEN** any user opens a keyboard score
- **THEN** the player opens exactly as before

### Requirement: The drum gate fails closed

The drum gate SHALL deny on every uncertainty **about the caller**: an unreadable or
unreachable flag store, an unwired plan source, or memberships that cannot be
resolved SHALL each resolve to **not eligible**, never to a grant. A degraded
dependency must not silently widen the audience.

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

#### Scenario: An unreadable flag store denies

- **WHEN** the flag store cannot be read
- **THEN** the drum feature is treated as off for every caller

#### Scenario: Unresolvable memberships deny

- **WHEN** a caller's beta memberships cannot be resolved
- **THEN** they are treated as holding none, so only staff retain the feature

#### Scenario: Unknown scores stay reachable

- **WHEN** an ineligible caller reads a score whose recorded instrument is `unknown`
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
