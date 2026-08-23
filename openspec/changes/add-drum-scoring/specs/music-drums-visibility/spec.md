## RENAMED Requirements

- FROM: `### Requirement: Percussion plays engage no scoring artifact before drum scoring exists`
  TO: `### Requirement: Percussion plays engage every scoring artifact`

## MODIFIED Requirements

### Requirement: Percussion plays engage every scoring artifact

A percussion score's play submissions SHALL engage the scoring artifacts exactly
as a keyboard score's do: instrument-aware scoring now exists
(`music-drum-scoring`), so each ingest site that failed closed on
`instrument = 'percussion'` is lifted — deliberately, one by one, each with what
"instrument-aware" meant for it:

- **Coverage engagement** (the post-play rating signal) SHALL be recorded for a
  percussion play. It never needed instrument awareness — playing a piece is
  genuine engagement whatever the instrument — and was withheld only to keep the
  interim total.
- **Play rewards** SHALL pay a percussion session under the unchanged rules of
  `music-play-rewards`: the quality floor reads the percussion synchronization
  percentage produced by the drum matcher (timing and correctness, no sustain),
  and the difficulty weight reads the piece's catalog level, now honestly graded
  for percussion (`corpus-manifest`). Curve, cap, floor and idempotence are the
  same configuration for both instruments.
- **Per-piece leaderboard bests** SHALL be maintained from a percussion
  session's sub-scores under the unchanged rules of `leaderboards`. The
  sub-scores come from the drum matcher; the boards are per-piece, so a drum
  board ranks drum plays against drum plays by construction.
- **Global season bests** SHALL accumulate from percussion sessions under the
  unchanged rules of `global-leaderboard`, difficulty-weighted by the same level
  scale as every piece.
- **Streak accounting** SHALL credit a percussion play's day: the streak
  measures showing up, which is instrument-agnostic.
- **Daily-access consumption** SHALL apply to a percussion open like any other:
  it consumes a day slot and the piece can be day-locked. The exemption existed
  only because consuming quota for a score that could not honestly be played
  would have been wrong; that premise is gone.

**Nothing is retroactive.** A percussion session stored during the interim SHALL
stay inert forever: every engagement effect is keyed to its own ingest event
(the session id for awards and bests, the player's local day for the streak, the
server day for slots) and applied at ingest time only — no pass re-scans stored
sessions, and nothing minted before the lift needs repudiating because nothing
was minted. Idempotence is inherited unchanged: a resent percussion session pays
once, and a replayed one never lowers or duplicates a best.

The interim requirement's "leaves no permanent trace" scenario is inverted here
**by design**: the interim existed only until instrument-aware scoring did, and
this requirement is its stated successor.

#### Scenario: A percussion run engages the artifacts

- **WHEN** an eligible player completes a scored percussion run above the
  quality floor
- **THEN** engagement is recorded, points are awarded, the per-piece and season
  bests are maintained, and the day counts toward their streak

#### Scenario: A percussion open consumes the daily quota

- **WHEN** the daily-access gate is on and a player opens a distinct percussion
  catalog piece
- **THEN** the open consumes a day slot exactly as a keyboard open would, and an
  over-quota percussion piece is presented as locked

#### Scenario: A below-floor percussion run pays nothing

- **WHEN** a percussion session's overall accuracy is below the configured floor
- **THEN** the session is stored and no points are awarded — the same floor,
  read from the percussion blend

#### Scenario: Interim sessions stay inert

- **WHEN** the lift lands over a history of percussion sessions stored during
  the interim
- **THEN** those sessions engage nothing — no award, no best, no streak day, no
  slot — and only sessions ingested after the lift engage

#### Scenario: A resent percussion session pays once

- **WHEN** the same percussion session is ingested more than once
- **THEN** points are awarded for it exactly once and its bests are not
  duplicated

#### Scenario: Keyboard plays are unaffected

- **WHEN** any caller completes a run on a keyboard score
- **THEN** rewards, leaderboards, streak and daily access behave exactly as
  before

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
a percussion score); the daily unlock (`UnlockCatalogScoreForToday`); the
non-gRPC audio-preview route (`GET /scores/{catalog_id}/preview`), whose
200-versus-404 answer is an existence oracle and whose body is an audible clip of
the score; and — now that a percussion play is scorable — the **per-piece
leaderboard reads** (`GetLeaderboard`, and the batched `GetMyStandings` behind
the score cards and the post-session standing), since boards are keyed by
catalog piece and a percussion piece's board existing is itself an existence
oracle. For an ineligible caller, a percussion piece's board read SHALL answer
exactly as for a piece that has no board. The **global** board reads
(`GetGlobalLeaderboard`, `ListGlobalSeasons`) remain outside the surface: their
entries disclose players, scores and a contributing-piece count — never a piece
identity. Any disclosing or accepting path added later inherits the same
obligation. This closes the exemption the previous revision recorded for the
leaderboard surface, on the condition it named: a percussion play is now
scorable.

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
of a drum part is never produced or served, to eligible callers included.

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

#### Scenario: A percussion piece's board is not disclosed

- **WHEN** an ineligible caller requests a percussion piece's leaderboard by id
- **THEN** the answer is exactly the answer for a piece with no board, even when
  eligible players are ranked on it

#### Scenario: Batched standings skip percussion for the ineligible

- **WHEN** an ineligible caller's batched standings read includes a percussion
  piece's id
- **THEN** that piece contributes no standing and no board signal to the
  response, indistinguishable from a boardless piece

#### Scenario: Global boards stay readable

- **WHEN** an ineligible caller reads the global boards or their season history
- **THEN** the read succeeds as before — entries name players and scores, never
  pieces — even when percussion sessions contributed to the standings

#### Scenario: An eligible caller is unaffected

- **WHEN** an eligible caller performs the same reads
- **THEN** percussion scores are returned normally

#### Scenario: Enforcement ignores client claims

- **WHEN** a request carries any client-supplied assertion of eligibility
- **THEN** it is disregarded and eligibility is resolved from the caller's identity
  and memberships
