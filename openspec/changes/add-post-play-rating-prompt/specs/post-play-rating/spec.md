## ADDED Requirements

### Requirement: Rating offered at the end of a played run

The end-of-song summary SHALL offer to rate the piece just played, whenever a scored run
reaches the end of the piece and that score is **eligible for rating**, carrying a 1–5 star
control whose verdict is derived exactly as the swipe deck derives it (so a star rating
and a swipe fold into the same single rating). Submitting the rating SHALL go through
the same rating operation as the deck — there is at most one rating per user per score,
and re-rating upserts. The affordance SHALL be optional: it MUST NOT become a fourth
mandatory choice, MUST NOT dismiss the summary, and leaving without answering it MUST
have no consequence at all.

Unlike the deck, the offer SHALL NOT be gated on a listening threshold: the player has
just performed the piece, which is stronger evidence than a preview.

#### Scenario: Summary offers to rate the piece just played

- **WHEN** a scored run ends on an eligible catalog score
- **THEN** the summary presents a rating affordance for that score alongside the
  existing statistics and actions

#### Scenario: Rating from the summary is recorded

- **WHEN** the player picks a star value in the summary
- **THEN** a rating carrying that star value and its derived verdict is submitted for
  that catalog score

#### Scenario: Rating does not dismiss the summary

- **WHEN** the player submits a rating from the summary
- **THEN** the summary stays open and still awaits the explicit see-mistakes / retry /
  quit choice

#### Scenario: The rating affordance is never mandatory

- **WHEN** the player chooses see-mistakes, retry, or quit without rating
- **THEN** the chosen action proceeds normally and no rating is recorded

#### Scenario: No listening gate after a run

- **WHEN** the summary's rating affordance is shown
- **THEN** it is immediately usable, with no listening-progress unlock

### Requirement: Non-blocking rating offer on early exit

The app SHALL present a compact rating surface offering the same 1–5 star rating when the
user leaves the player **before** the end of the piece on a score eligible for rating.
That surface MUST NOT block or cancel the exit: rating it, closing it,
tapping outside it, or triggering a back gesture SHALL all result in leaving the player.
It SHALL NOT ask the user to confirm the exit.

#### Scenario: Leaving early offers the rating

- **WHEN** the user leaves the player before the piece ends, on an eligible catalog score
- **THEN** a compact rating surface is presented

#### Scenario: Rating on the way out submits and leaves

- **WHEN** the user picks a star value on that surface
- **THEN** the rating is submitted for that score and the player is left

#### Scenario: Dismissing leaves the player

- **WHEN** the user closes the surface, taps outside it, or triggers a back gesture
- **THEN** no rating is recorded and the player is left

#### Scenario: The exit is never cancelled

- **WHEN** the rating surface is shown on exit
- **THEN** the user is never returned to the player and is never asked to confirm leaving

### Requirement: Eligibility of a played score for the rating prompt

A played score SHALL be eligible for the rating prompt only when **all** of the
following hold: the user is signed in; the score is a **public-catalog** score (bundled
and user-contributed scores are not rateable and MUST NOT prompt); the caller has **not
already rated** it; the player has **not explicitly refused** to rate it on this
device; and the playhead has passed at least a configured **minimum share of the
piece's notes** — 25% by default. That share SHALL be counted **in notes** (how many of
the score's notes the playhead reached, over the score's total note count), not in
elapsed time and not as a fraction of the piece's duration, so it is independent of the
playback tempo and of how the piece's time is laid out. It SHALL be counted over the
**whole** score rather than only the selected hand, so muting a hand does not inflate
it, and SHALL use the furthest point reached, so pausing or seeking backwards never
lowers it. When eligibility cannot be determined — for example the already-rated state
is unknown because the device is offline — the prompt MUST NOT be shown.

#### Scenario: A bundled score never prompts

- **WHEN** a run ends, or the player is left, on a bundled or user-contributed score
- **THEN** no rating prompt is presented

#### Scenario: A guest is never prompted

- **WHEN** a signed-out user plays a catalog score
- **THEN** no rating prompt is presented

#### Scenario: An already-rated score never prompts

- **WHEN** the caller has already rated the played catalog score, on this or another device
- **THEN** no rating prompt is presented

#### Scenario: A refused score never prompts

- **WHEN** the player has explicitly refused to rate the played score
- **THEN** no rating prompt is presented

#### Scenario: A negligible run does not prompt

- **WHEN** the user opens the player and leaves it having reached less than the
  configured share of the score's notes
- **THEN** no rating prompt is presented

#### Scenario: The share is counted in notes, not in time

- **WHEN** the user plays the same portion of a piece at half tempo, taking twice as long
- **THEN** eligibility is unchanged, because the same number of notes was reached

#### Scenario: Muting a hand does not inflate the share

- **WHEN** the user plays with one hand muted
- **THEN** the share is still counted over the whole score's notes, not only the
  audible hand's

#### Scenario: Seeking backwards does not lower the share

- **WHEN** the user reaches past the threshold and then pauses or seeks back before leaving
- **THEN** the furthest point reached still counts and the prompt is presented

#### Scenario: Practising a short range of a long piece does not prompt

- **WHEN** the user loops a few measures of a long piece and leaves
- **THEN** the share of the score's notes reached stays below the threshold and no
  prompt is presented

#### Scenario: Unknown rated state suppresses the prompt

- **WHEN** the caller's existing rating of the score could not be determined (offline or
  a failed read)
- **THEN** no rating prompt is presented

### Requirement: A score stops being offered only when rated or refused

Suppression SHALL be **per catalog score**, never a global nag budget, and it SHALL
be triggered only by an **answer**: rating the score, or explicitly refusing to rate
it. Merely having been *shown* the prompt MUST NOT retire a score — closing a
summary or dismissing the exit sheet to leave the player is not a statement about
the piece, so the offer stands and returns on the next run.

Every surface that shows the prompt SHALL therefore expose an explicit refusal
control, since that (or a rating) is the only way for the user to stop being asked
about a piece. A refusal SHALL be persisted through the injectable preferences seam
and the persisted value SHALL be bounded in size so it cannot grow without limit.
There SHALL be no global dismissal count and no cross-score snooze window.

#### Scenario: Being shown the prompt does not consume the offer

- **WHEN** the prompt is shown and the player leaves without rating or refusing
- **THEN** nothing is recorded, and the next run of that score offers it again

#### Scenario: A rating ends the offers

- **WHEN** the player rates the score
- **THEN** it is never offered again, on this or any other device

#### Scenario: An explicit refusal ends the offers

- **WHEN** the player uses the refusal control
- **THEN** the refusal is recorded and that score is never offered again on this device

#### Scenario: Every surface can refuse

- **WHEN** the prompt is shown, on either the summary or the exit sheet
- **THEN** an explicit refusal control is available on it

#### Scenario: Dismissing a surface is not a refusal

- **WHEN** the exit sheet is dismissed by its barrier or a back gesture rather than
  its refusal control
- **THEN** no refusal is recorded and the offer stands

#### Scenario: Refusing one score does not silence others

- **WHEN** the player refuses one score and then plays a different eligible score
- **THEN** the rating prompt is presented for that other score

#### Scenario: No global stop after repeated refusals

- **WHEN** the player has refused many different scores
- **THEN** a newly-played, never-refused eligible score still presents the prompt

#### Scenario: A refusal survives a restart

- **WHEN** the app is restarted after a score was refused
- **THEN** that score is still not offered again

#### Scenario: The refusal memory stays bounded

- **WHEN** the player has refused more scores than the retained maximum
- **THEN** the persisted memory keeps at most that maximum, discarding the oldest entries

### Requirement: A prompt failure is never surfaced as a technical error

Submitting a rating from a player prompt SHALL be best-effort with respect to the play
experience: a failed submission MUST NOT block leaving the player, MUST NOT interrupt the
summary flow, and MUST NOT surface a raw transport or exception string. The user SHALL
see only a localized message, if anything at all.

#### Scenario: A failed submission does not block the exit

- **WHEN** the rating submitted on early exit fails
- **THEN** the player is still left and no raw error string is shown

#### Scenario: A failed submission does not break the summary

- **WHEN** a rating submitted from the summary fails
- **THEN** the summary remains usable and the see-mistakes / retry / quit actions still work
