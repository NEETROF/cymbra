## Context

Every prerequisite of drum scoring exists by the time this change lands: the
parser resolves each unpitched note to a General MIDI percussion number
(`add-unpitched-notation` — `NoteEvent.unpitched.gm_number`, already corrected
for MusicXML's one-based `<midi-unpitched>`: GM number = element value − 1); the
backend knows which scores are percussion and fails every scoring ingest site
closed on them (`add-drums-access`); the cascade and the pad strip present the
score (`add-drum-kit-view`), the staff draws it (`add-drum-notation-render`),
the kit sounds (`add-drum-audio-channel`) and strokes arrive through the same
note-on entry points the keyboard uses (`add-drum-input-mapping`).

What is missing is the **judgment**: the scorer classifies "the player's attack
of that pitch", and for a drum part "that pitch" is undefined. Three facts shape
the answer.

**The equivalence question is already half-answered.** `music-drum-kit-view`
requires General MIDI numbers that denote one physical piece to share one lane,
and the implemented table (`apps/music/lib/state/drum_kit.dart`) records the
groups: hi-hat {42, 46}, snare {37, 38, 40}, ride {51, 53, 59}, kick {35, 36},
each tom and each accent cymbal alone. The display already decides what the
player can *see*; the matcher must not ask for more than that.

**The keyboard scorer's third dimension does not exist on a kit.** The blend is
timing 0.5 / correctness 0.3 / sustain 0.2 (`performance_scoring_core.dart`).
A stroke has no meaningful duration — an e-kit's note-off timing is a property
of the module's firmware, not of the player.

**Difficulty is load-bearing, not cosmetic.** The play-reward award and the
global season score are both weighted by the catalog `level` through one shared
function (`difficulty_weight_of`, `backend/music/src/global_leaderboard_core.rs`
— deliberately one scale, not two). The crawler's heuristic
(`crates/score-crawler/src/difficulty.rs`) counts only **pitched** notes: on a
drum part its density, range, leap and accidental terms are all zero, so every
groove would grade Beginner. Two sibling designs deferred this exact question
here.

## Goals / Non-Goals

**Goals:**

- Define stroke identity — what satisfies a written drum note — once, and make
  every consumer (Wait gate, scorer, extra-stroke detection, pad feedback) use
  that one definition.
- Make a percussion run produce an honest quality signal: verdicts, a sync%,
  sub-scores a leaderboard can rank.
- Offer Wait Mode for percussion, with gate semantics that acknowledge a stroke
  is an attack, not a hold.
- Complete the core loop `add-instrument-context` names as the GA prerequisite:
  gauge and end-of-session summary on a drum score.
- Give percussion an honest difficulty on the shared level scale, and re-grade
  the mis-graded rows.
- Lift each interim fail-closed ingest site deliberately, and take on the
  leaderboard read surface `music-drums-visibility` delegated here.

**Non-Goals:**

- **Velocity, ghost notes, accents.** MusicXML drum parts rarely mark per-stroke
  dynamics and pad velocity curves are hardware artefacts; judging them now
  would score the kit, not the drummer. Keyboard scoring ignores velocity too,
  so the omission is symmetric.
- **Per-instrument global boards, or a percussion handicap.** See the
  comparability decision below.
- **Scoring changes for keyboard.** Windows, weights, verdicts, ingest,
  boards — all byte-for-byte unchanged for a keyboard score.
- **Practice scoring.** Selective (measure-range) runs stay unscored **by
  construction** — `measure-range-practice` never arms the scorer for a
  selective run, an instrument-agnostic rule that covers drums with no
  carve-out. Stated here so nobody adds one: the decision is that there is
  nothing to decide.
- **A new audience or gating mechanism.** The audience is
  `music-drums-visibility`'s, inherited; at general availability the read gate
  added here becomes a no-op exactly like the rest of the enforcement.

## Decisions

### Stroke identity is matched at the kit piece's grain

An incoming General MIDI number satisfies a written note when both resolve to
the same **kit piece** — the named-piece groups of `music-drum-kit-view`'s
table, not raw number equality, and not the score-derived lane membership (an
incoming 40 satisfies a written 38 even in a score that never contains a 40).
Pieces with their own lanes — each tom, each accent cymbal, each terminal-bucket
number — match only themselves. Kick 35 and 36 are one pedal and satisfy each
other.

*Rationale:* the governing principle is **never demand a distinction the
cascade does not draw**. The display collapses 38/40/37 into one snare lane with
one look; a matcher that failed a side-stick against a written acoustic snare
would punish the player for a difference they cannot see — and cannot control:
which number an e-kit sends for a given zone is a module-mapping artefact.
Conversely the toms and accent cymbals have separate lanes because they are
separate aim points, so hitting the wrong one is exactly the error the game
exists to catch.

*Alternative rejected:* exact General MIDI equality. Fails real hardware (zone
mappings differ per module), demands invisible distinctions, and would make Wait
Mode block on a number the player's kit cannot emit.

*Alternative rejected:* any-piece-counts ("just hit something on the beat").
Destroys the aiming exercise; the cascade's whole design encodes *where to aim*.

*Alternative rejected:* keying equivalence on the derived lane layout instead of
the static table. The layout's groups are intersected with the numbers present
in the score, so a number absent from the score has no lane and would misread
as a wrong note. The table is the identity; the layout is a view of it.

### Open versus closed hi-hat shades the verdict and never gates

The two numbers are one piece for binding and gate release; a stroke with the
wrong articulation binds, releases the gate, and has its timing verdict capped
below `perfect`.

*Rationale:* the distinction is musical and the cascade draws it (the hollow
variant), so ignoring it entirely would erase information the display promises.
But it is also the one distinction whose production depends on hardware many
players lack — an e-kit without a hi-hat controller can only ever send closed.
Strict matching would wall those players out of Wait Mode (the gate would block
forever on a stroke they cannot produce). Shading resolves both: the run
completes, the difference costs something, and a player with the pedal is
rewarded for using it.

*Alternative rejected:* strict articulation matching — blocks Wait Mode on
hardware grounds, the one thing a practice gate must never do.

*Alternative rejected:* full equivalence — the only in-lane distinction the view
deliberately draws would be worth nothing.

### Timing windows are reused; velocity is ignored

The percussion scorer uses the keyboard windows (free-run signed offset:
perfect 40 ms / good 90 ms / bind 160 ms; Wait reaction: 120 / 300 ms) and the
same verdict scale.

*Rationale:* the windows are tunable constants explicitly marked "not
spec-locked; validate on-device". No data yet supports drum-specific values,
and the beta's feel pass is precisely where such data comes from. Diverging now
would also complicate the cross-instrument comparability argument below for no
demonstrated gain. Velocity is ignored because keyboard scoring ignores it too
and pad velocity is dominated by hardware curves.

*Alternative rejected:* tighter drum windows a priori ("drummers are the
timekeepers"). Plausible, unproven, and cheap to do later as a constant change;
wrong to bake into a spec before a single on-device session.

### A percussion run has no sustain dimension; the blend renormalizes

The percussion sync% blends timing and correctness only, with their keyboard
ratio preserved (0.5 : 0.3 → 0.625 : 0.375). Sustain aggregates and per-note
sustain fields are **absent** from a percussion session result — absent like a
mode sub-score with no onsets, never zero. The keyboard blend is untouched.

*Rationale:* a stroke's duration is not player intent. Measuring it would score
firmware; fixing it at 1.0 would leak a constant 20% of free credit into every
percussion run (the exact dilution `sustainScore`'s `anyOnsetJudged` guard
exists to prevent) and make percussion percentages incomparable upward.
Renormalizing keeps the two real dimensions at their established relative
importance.

*Alternative rejected:* constant full sustain — free credit, and a do-nothing
run would score 20% instead of 0.

*Alternative rejected:* measuring note-off anyway — judges the module, not the
player.

### One stroke identity, every consumer

The gate's required set, the scorer's binding, the extra-stroke detection and
the pad feedback all call one shared identity function, sourced from the
kit-piece table.

*Rationale:* this is `add-drum-kit-view`'s "one derived layout, two consumers"
argument restated over judgment: two independent equivalence tables would
drift, and the failure mode is vicious — a gate that releases on a stroke the
scorer then counts as wrong, or the reverse. One function makes the
inconsistency inexpressible.

### Wait Mode gates on strokes at the gate — no hold carve-out

The gate freezes at each onset and releases when every required stroke (of the
selected hands/feet) has been **struck while the gate is active**. A stroke
before the playhead reaches the onset does not pre-satisfy; there is no
percussion counterpart of the keyboard's held-pitch satisfaction.

*Rationale:* the keyboard carve-out exists to tolerate sustained and tied notes
carried into the onset where they first sound — a situation a kit cannot
produce. A stroke is an attack; "already holding it" is meaningless. Adding an
early-stroke grace window instead would blur the exercise Wait Mode is (strike
*when the music asks*), and free-run scoring already exists for players who
want tolerance rather than a gate.

*Alternative rejected:* an early-stroke grace (bind strokes arriving up to N ms
before the gate). It converts Wait Mode into a worse free run and creates a
second window constant to tune with no user asking for it.

### Difficulty: drum-shaped features onto the shared level scale

The heuristic becomes instrument-aware. For a percussion score it estimates
from: stroke density (unpitched notes per measure), tempo, fastest subdivision,
limb simultaneity (simultaneous distinct pieces at one onset; two-voice
writing), and kit breadth (count of distinct pieces). It emits the same
beginner/intermediate/advanced vocabulary, under the same provenance rules
(a heuristic estimate is never recorded as a source grade). The thresholds are
calibrated against the authored bundled drum scores, which exist at each tier
by construction (`add-instrument-context`'s bundling task covers the three
levels). Rows previously graded by the keyboard heuristic are re-graded —
`level_source = heuristic` rows only; `source` and `manual` grades are never
overwritten, which is exactly the overwrite the provenance rule was designed to
permit.

*Rationale for one scale rather than a percussion axis:* the difficulty weight
is deliberately **one** function shared by play rewards and the global boards
("rank difficulty on ONE scale, not two" — `play_rewards_core.rs`). A
percussion-specific weight table would fork that scale and force every consumer
to know the instrument. Slotting percussion into the existing levels keeps the
keyboard weights untouched — the brief's hard constraint — and keeps the
anti-farming analysis intact: floor, diminishing curve, cap and weight are all
instrument-agnostic once the level is honest.

*Alternative rejected:* leaving the keyboard heuristic. Its pitched-note terms
degenerate to zero on drum parts; every groove grades Beginner, which both
underpays honest play and — worse — makes the *easiest* farming target the drum
corpus the moment percussion pays.

*Alternative rejected:* refusing to grade percussion (level null, neutral
weight). Safer against overpayment but permanently flattens the reward and
ranking landscape for a whole instrument, and neutral-weights an advanced solo
identically to a two-note exercise.

### The ingest sites lift one by one, and history stays inert

Each formerly fail-closed site starts engaging at ingest time, with
"instrument-aware" made concrete per site (enumerated in the
`music-drums-visibility` delta). Sessions stored during the interim are never
re-processed: every effect is keyed to its own ingest event (session id for
awards and bests, local day for streaks, server day for slots), and no pass
scans stored sessions.

*Rationale:* the interim's whole point was that these artifacts are permanent —
ledger rows, monotone bests, badges. Retroactively scoring interim sessions
would mint artifacts from judgments produced by the keyboard scorer's read of a
drum part, exactly the wrong data the interim existed to keep out. Inertness
also falls out of the architecture for free: engagement happens in the ingest
handlers, so not scanning history is the default, stated so it stays deliberate.

### The per-piece leaderboard read surface joins the enforcement; global reads do not

`GetLeaderboard` and the batched `GetMyStandings` answer for a percussion piece,
to an ineligible caller, exactly as for a piece with no board — the same
not-found-shaped answer the rest of the read surface gives. The global boards
(`GetGlobalLeaderboard`, `ListGlobalSeasons`) stay ungated.

*Rationale:* per-piece boards are keyed by catalog piece, so a board's mere
existence is an existence oracle — the disclosure `music-drums-visibility`
explicitly delegated to this change "the moment it makes a percussion play
scorable". A modified client could probe board reads even though search already
withholds the piece, so the gate must sit on the read itself. Global entries
carry players, scores and a contributing-piece *count* — no piece identity — so
gating them would protect nothing while breaking a community surface. Like the
rest of the enforcement, this gate is a rollout stage: at general availability
it keeps running and keeps passing.

### Percussion sub-scores are globally comparable — accepted, watched

A percussion best enters the global season score on the same 0–1 scale and the
same level weights as a keyboard best.

*Rationale:* the boards already mix incommensurable metrics by design — the
reaction board and the tempo board measure different things, and a `mixed` run
feeds both. The sync% is a quality percentage in both instruments. Splitting
the boards per instrument would halve a small community and create a drummers'
board with three players.

*Trade-off accepted, named:* a two-dimension blend may be systematically easier
to saturate than a three-dimension one (no sustain to lose). If the first
season with drums shows percussion entries crowding the top, the lever is
configuration — the blend weights and the level weights — not a schema change.

## Risks / Trade-offs

**Old app versions submit keyboard-shaped percussion sessions after the lift** →
a shipped app without the percussion scorer could, in principle, arm the
keyboard scorer on a drum score and submit its garbage sub-scores once the
backend accepts percussion sessions. Mitigation: the current app never enters a
scored percussion run (Wait Mode absent, and the interim states of
`add-drums-access`/`add-drum-kit-view` keep the piano path off drum scores), so
the exposure is limited to versions between the kit view and this change; the
audience is the beta cohort, who update; and the integrity checks bound
sub-scores to plausible ranges. Accepted for the beta; the backend lift lands
**after** the app-side scorer ships (see Migration Plan) so the in-repo path is
never wrong.

**The heuristic mis-grades and the weight overpays** → an Advanced-graded
trivial groove would pay and rank above its worth. Mitigation: the quality
floor, the per-piece diminishing curve and the daily cap bound the damage
regardless of weight (the farming analysis never rested on the grade being
right); thresholds are calibrated against the authored tiered scores; and every
heuristic grade remains overwritable by curation without touching real grades.

**The blend saturates too easily** → see the comparability decision; watched,
tunable, not structural.

**The equivalence table is a judgment call** → side-stick-satisfies-snare and
ride-bell-satisfies-ride rest on the lane design plus one drummer's feedback.
Mitigation: the table lives in one place (`drum_kit.dart`) and the matcher
consumes it, so revising a group is a one-line change plus tests; the beta's
feel pass exercises it with real hardware.

**Wait Mode with no hold semantics may feel strict on flams/drag rudiments** →
grace-note clusters could gate pedantically. Mitigation: grace notes are
already outside the timed stream's scored onsets (they carry no duration); the
feel pass drives rudiment-heavy scores explicitly before the windows are
declared final.

## Migration Plan

1. **App first**: the percussion matcher, blend, Wait gate, gauge and summary
   land behind the existing audience gate. A percussion run now produces an
   honest session result locally; the backend still ignores it (interim sites
   unchanged). No schema, no wire change — the session payload already carries
   the sub-scores and the record's absent-field convention covers the missing
   sustain.
2. **Difficulty**: the instrument-aware heuristic lands in the crawler; the
   re-grade pass runs over `level_source = 'heuristic'` percussion rows.
   Verified before step 3 so the first paid percussion runs meet honest
   weights.
3. **Backend lift**: the six ingest sites and the leaderboard read gate land
   together, after step 1 is verified on-device. From this point percussion
   runs pay, rank, streak and consume.
4. Nothing to migrate in storage: interim-stored sessions stay where they are,
   inert; no backfill touches them.

Rollback: step 3 is a revert (the interim branches return); steps 1–2 are
app/crawler code with no stored-state coupling. Nothing minted before a
rollback needs unwinding, because ledger rows and bests minted after the lift
are honest by construction — rollback stops new engagement, it does not
repudiate old.

## Open Questions

- **Single-crash kits.** A score written with two accent cymbals cannot be
  played faithfully on a one-crash kit; today the second cymbal's strokes are
  wrong notes. A per-run leniency ("collapse accent cymbals") is a plausible
  later setting; deciding it needs a real score and a real complaint.
- **Velocity as a later dimension.** If it ever lands, it should be a separate
  opt-in dimension with per-device calibration, never folded silently into the
  blend. Left open until MusicXML drum dynamics show up in the corpus.
- **Do the reused windows survive contact with drummers?** The feel pass
  answers it; the constants are one file.
- **Global-board pressure from the two-dimension blend.** Watch the first
  season with percussion entries; the lever is configuration.
- **The chick, again.** When `music-drum-kit-view`'s deferred encoding lands
  (hollow bar or far-left lane), its scoring is already covered — it is a foot
  piece matching itself — but its Wait-gate ergonomics (foot-only onsets) should
  be re-felt then.
