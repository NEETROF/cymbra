## Context

`add-unpitched-notation` gives the parser a real instrument classification but
leaves the admission gate shut, because admitting drum scores means deciding who
may see them. This change decides, enforces, and opens.

Three facts shape it.

**The audience mechanism already exists.** `RolloutScope::Beta(key) => self.staff ||
self.betas.contains(key)` (`feature-flags/src/context.rs:148`) with
`staff = roles ∋ admin | moderator` (`context.rs:108`) is exactly the required
audience. The platform spec already uses `beta:midi-drums` as its worked example,
and `CampaignKind::Feature` is described as a membership-only campaign the operator
closes when the feature stabilises.

**The music module can build an evaluation context today.** It already receives
`AuthIdentity` (with `roles`) at `music/src/grpc.rs:26`, and already depends on
`cymbra-plans` through a `PlanSource` port (`module.rs:164`, `with_plans`) whose
snapshot exposes `beta_keys()`. Only `FlagService` is missing, and
`notifications/src/dispatch.rs:38` shows the shape to copy.

**`is_piano` cannot survive.** Enforcement must ask "is this score percussion?" of
the database, and `is_piano` cannot answer: it stores `staves >= 2`, a proxy that
misreads single-staff keyboard pieces as non-piano and organ, two-staff
arrangements and two-staff drum kits as piano. The column has to become the real
instrument.

## Goals / Non-Goals

**Goals:**

- Restrict the drum feature to staff plus `midi-drums` campaign members, enforced
  server-side on every disclosing path.
- Replace the `is_piano` proxy with a stored instrument, migrating existing rows
  honestly.
- Open the admission gate, behind that enforcement.
- Replace the piano filter with an instrument filter in the app and the console.

**Non-Goals:**

- Percussion audio, notation rendering, the kit view, MIDI pad input,
  instrument-aware scoring — one change each, after this. Two of them are
  **doubled** and the duplication is easy to miss: percussion drawing lands in both
  `apps/music/lib/painters/` (Dart) and `apps/back-office/src/lib/notation/painter.ts`
  (TypeScript), and the percussion channel in both `apps/music/rust/src/api/` and
  `crates/audio-wasm/src/lib.rs`.
- SoundFont selection by instrument family, and verifying an uploaded font's
  declared family from its preset banks — `add-drum-audio-channel`, since a drum kit
  is only selectable once drums can sound.
- Making drum scores *useful*. Until the rendering and audio changes land, the app
  has no honest presentation of a drum score — and the dishonest one is worse than
  empty: per `add-unpitched-notation`'s schedule spec, unpitched notes enter the
  timed stream with GM numbers 35–81 in the MIDI slot, so the waterfall would draw
  them as falling *piano* notes and the piano synthesizer would sound them. That is
  precisely why the audience is restricted, and why the app shows an explicit
  "drums not playable yet" state instead of entering the player (the app-side
  mirror of the console guard below, removed by `add-drum-kit-view`).
- Scoring drum plays. Until `add-drum-scoring`, a percussion run engages no
  rewards, leaderboards, streak or daily-access consumption — those artifacts are
  permanent (ledger rows, monotone bests, badges never lost), so keyboard-shaped
  scoring of a drum part would bake wrong data nothing could cleanly unwind. The
  ingest sites fail closed on `instrument = 'percussion'`.
- Sourcing a drum corpus. The catalog can hold drum scores after this change; where
  they come from is an open product question.

## Decisions

### Enforcement lives in the module, resolved from identity — never from the request

The gate is a predicate the `music` module computes from `AuthIdentity.roles` and
the `PlanSource` snapshot, combined through `EvalContext` and the declared flag. No
request field participates.

*Alternative rejected:* letting the app send "I am a drummer" and trusting it. That
is the client-authoritative pattern the platform rule exists to forbid.

*Alternative rejected:* a gRPC interceptor gating whole methods. The disclosure is
row-level (a search returns a mix of keyboard and percussion rows), so it has to
reach into the query, not the method.

### An ineligible caller gets "not found", not "forbidden"

Withheld scores are absent from listings and unknown on direct fetch — and this
rule scopes to **read** paths. On the accepting paths (upload, propose) the caller
already holds the file, so nothing can be disclosed and a not-found answer would
be incoherent; there the refusal is typed and localisable.

*Rationale:* a distinguishable refusal on a read path is itself a disclosure — it
tells the caller that a percussion score exists at that id. This also keeps the
search path simple: one predicate in the `WHERE` clause rather than a post-filter
that would corrupt paging counts.

One carve-out, decided rather than left to emerge: the gate does **not** apply to
a caller's own uploads. Listing, fetching and deleting one's own scores always
works — there is no disclosure in showing someone their own file, and gating it
would strand a former tester with quota held by scores that are invisible in
their library yet deletable by id. Only `ProposeScore` (pushing toward the public
catalog) and uploading a *new* percussion score require eligibility.

### Fail closed about the caller, fail *open* about the score

Uncertainty about eligibility denies: an unreadable flag store, an unwired
`PlanSource`, unresolvable memberships all mean "not eligible". Uncertainty about a
score does the opposite — an `unknown` instrument is served normally.

*Rationale:* the two uncertainties are not symmetric. Denying on an unknown caller
costs a staff member a feature for a moment. Denying on an unknown *score* would
hide a slice of the existing corpus from users who can read it today — a live
regression, not a safety gain. But fail-open-about-the-score is only sound once
every row's instrument has been **derived from its bytes**: the corpus already
contains real percussion scores (they were ingested despite the playable-notes
gate — prod holds ~252 scores with unpitched notes, ~29 pure drum parts), so an
`unknown` row *can* be percussion until it has been re-derived. That is why the
gate is not treated as a boundary before the re-derivation pass below completes,
and why the flag is not activated before it has run.

### The backfill re-derives from the stored bytes; it is an application pass, not SQL

The instrument is filled by parsing each row's stored `.mxl` with the **new**
classifier from `add-unpitched-notation` — never by translating `is_piano`.

*Alternative rejected:* translating the flag (`true → keyboard`,
`false → unknown`). It reads cheaper and it is wrong on real rows: `false` covers
single-staff piano *and* every existing percussion score alike, so the translation
would record real drum parts as `unknown` — which the gate then serves to
everyone, defeating the change's central guarantee. `unknown` remains the honest
value only for rows whose bytes cannot be read or parsed.

*Alternative rejected:* mapping `false → other`. Asserting single-staff pieces are
not keyboard would permanently mislabel them, with no second signal to recover
from it.

**Mechanism.** The tables hold only an `object_key`; the bytes live in the private
object store, so a Postgres migration cannot do this. The schema migration only
**adds** the column (`NOT NULL DEFAULT 'unknown'`, CHECK on the three values — the
column is never NULL, so the gate and the filters test exactly one undeterminable
value). A separate one-shot **application-level backfill** — an admin command or
worker job, idempotent so it can resume — streams each object, parses it with the
new classifier, and updates the row; an unreadable or unparseable object leaves
`unknown`. Re-crawling still repopulates the *general* facets as `score-facets`
says, but it cannot substitute here: user uploads are never re-crawled, and the
gate needs every row classified, not just the crawler's.

### The Score Hub stops pinning a filter

The hub currently pins `isPiano: true` (`catalog_search_notifier.dart:270`). It
stops pinning anything and offers instrument as a user filter, listing drums only
for the drum audience.

*Rationale:* the pin existed because "the corpus is piano-only", a condition the
spec states explicitly and which no longer holds. Keeping the pin and adding a
drum-only escape hatch would leave two filters fighting. The backend already
withholds what a user may not see, so the hub does not need to defend anything.

### The rating deck predicate follows the same rule

`pg.rs:796` sources the deck with `AND cs.is_piano`. It becomes the explicit
predicate `instrument <> 'percussion' OR caller_is_eligible` — that is, keyboard
**and `unknown`** rows are dealt to every rater, percussion rows only to the
eligible — so a drummer can rate drum scores and a pianist is never handed one.

*Rationale:* left alone, this line would silently make drum scores unratable
forever — a defect nothing in the UI would reveal. Stating the predicate matters
because the naive translation is ambiguous about `unknown`: today's proxy excluded
every `is_piano = false` row, so the deck silently skipped single-staff keyboard
pieces and everything unclassified. That exclusion was the proxy's bug, not a
design intent — after re-derivation, `unknown` means "bytes we could not parse",
which is still worth a human rating — so the deck deliberately **widens** to
single-staff keyboard and `unknown` rows for everyone. The per-score preview bytes
path (`GetRatingPreviewBytes`) and the rating submission itself carry the same
eligibility predicate, since each would otherwise disclose (or accept ratings for)
a percussion score by id.

### The console refuses to draw percussion rather than drawing it wrong

Moderators are staff, so this change makes drum scores reachable in the console at
once — including its notation preview and its Play control. Both are keyboard-shaped:
the preview is the console's **own** TypeScript painter
(`lib/notation/painter.ts`, independent of the app's Dart painters), and Play runs
through `audio-wasm`, which carries its own `PIANO_CHANNEL = 0`. So a drum score
would be drawn on a treble staff with pitched noteheads, and auditioned on a piano
preset.

Rather than ship that, both surfaces declare the score not previewable yet, in a
state distinct from the existing decode/parse failures.

*Alternative rejected:* draw it anyway and let moderators infer. A confident wrong
rendering is indistinguishable from a corrupt file, and the moderator's available
action is to reject — so the cost of the honest-looking bug is real moderation
decisions made on false evidence.

*Alternative rejected:* hide percussion scores from the console until the renderer
is ready. That would leave uploaded drum scores unmoderatable and invisible to the
very people meant to be testing the feature.

### The restriction is a rollout stage, not a permanent boundary

The drum feature is intended to become generally available. The audience restriction
exists to keep an unfinished feature away from users who would only meet its broken
halves, and it is designed to **become a no-op** rather than to be removed: flipping
the flag's scope from `beta:midi-drums` to `global` makes every caller eligible,
and the enforcement code keeps running and keeps passing.

*Consequences worth stating, because they are easy to get wrong later:*

- **The flag stays after general availability**, as a kill-switch. It is not deleted
  once the rollout completes.
- **The enforcement is not a confidentiality mechanism** and must not be
  strengthened, audited or preserved as if it were. It gates an unfinished feature,
  not private data. Once the scope is `global` it protects nothing, by design.
- **Order of operations at general availability: widen the scope first, close the
  campaign second.** `beta:<key>` matches *staff or member*; closing the campaign
  ends every membership. Closing it before the scope is widened drops every tester
  out of the feature at once, with staff the only ones left seeing it — the exact
  symptom of a misconfigured rollout, produced by a correct-looking action.
- **The beta is not a go/no-go experiment.** Tester feedback tunes the design
  (lane order, bar attenuation, tom ordering); it is not being used to decide
  whether the feature ships.

## Risks / Trade-offs

**A breaking schema and wire change land together** → the column, the proto and both
front ends move in one commit. Mitigation: the migration is additive-then-drop
(add `instrument`, run the re-derivation pass, switch readers, drop `is_piano`), so
a rollback between steps is possible; and the generated clients are regenerated as
part of the change (`melos run gen-grpc` for the app, `yarn gen` for the console).
The in-repo clients are only half the story: **shipped app versions cannot be
regenerated** and keep sending `is_piano = 6` for months. The proto therefore
retires the field (`reserved 6; reserved "is_piano";`) and the instrument filter
takes a fresh number — an old client's pinned piano filter lands on a reserved
number and is silently ignored, so old-version hubs see unfiltered results
including `unknown`-instrument rows. Accepted: the pin's premise is gone, nothing
is disclosed that the server-side gate would withhold, and the alternative (reusing
number 6) would misdecode the old boolean as the new field.

**The gate is only as good as its least-guarded path** → the disclosing surface is
larger than the obvious call sites: beyond search, bytes, the listings and the
deck, a percussion score leaks through the metadata read by id, the deck's
per-score preview bytes, the success-versus-not-found answers of the save/rating
submissions, the daily unlock, and the HTTP audio-preview clip route — and missing
any one silently defeats the whole change. Mitigation: the spec enumerates the
known surface as a floor, not a ceiling; the tasks carry one ineligible-case test
per path rather than testing the predicate once in isolation; and any new
disclosing path inherits the obligation by spec.

**`music` becomes a flag consumer** → a new dependency on `FlagService` in a module
that had none, and a flag read on hot catalog paths. Mitigation: follow
`notifications/src/dispatch.rs`, which reads from the hot in-memory store rather
than the database; and default every unreadable value to disabled.

**The campaign is an operational prerequisite** → until the `midi-drums` campaign
exists and has members, `beta:midi-drums` reaches staff only, which will look like a
bug to a tester who was promised access. Mitigation: creating and populating the
campaign is a named manual task, done before the flag is switched on.

**Drum scores become uploadable before they are playable** → an eligible tester can
put a drum score in their library and find it is not playable yet: opening it shows
the localised interim state rather than the player, because the player's honest
alternative does not exist (the waterfall would draw and sound the part as piano).
Accepted and intended: that audience is the people building the feature. It is also
why the audience is not merely hidden but enforced.

**Drum proposals arrive before they can be judged** → public proposal stays open for
percussion, with moderation as the gate. But the console cannot draw a drum score
until `add-drum-notation-render`, so a moderator receives proposals they cannot
evaluate and can only leave pending. Accepted, because the audience able to propose
one is staff plus a handful of beta testers, so the queue cannot fill; the practical
mitigation is to land the render change before enrolling testers in numbers, and the
review row is badged with the instrument so the cause is visible without opening it.

## Migration Plan

1. Add `instrument TEXT NOT NULL DEFAULT 'unknown'` (CHECK: `keyboard` |
   `percussion` | `unknown`) to `music.catalog_scores` and `music.user_scores` —
   the schema migration does nothing else.
2. Run the one-shot application-level re-derivation pass: stream each row's object
   from the store, parse it with the new classifier, update the row; unreadable or
   unparseable objects stay `unknown`. The gate is not a boundary until this pass
   completes.
3. Switch every reader and writer to `instrument` (backend, crawler,
   `musicxml-core` summary, the frb bridge); regenerate the app and console
   clients.
4. Drop `is_piano` once no reader remains.
5. Declare the flag (`drums.enabled`, default off). Create the `midi-drums`
   campaign and enrol testers. Only then — and only after step 2 has been verified —
   set the override to `beta:midi-drums`.

Steps 1–2 are safe on their own and reversible (rerunning the pass is idempotent).
Step 4 is the point of no return; it should land after step 3 has been observed
working.

Rollback after step 5 is a flag edit, not a deploy: clearing the override closes the
feature for everyone including staff, leaving any drum score already uploaded in
place but unreachable.

## Open Questions

- **Where does a drum corpus come from?** The catalog can hold drum scores after
  this change. The existing corpus is public-domain keyboard repertoire; drum
  repertoire is largely modern and under copyright. This does not block the change
  but it does bound the feature's value, and it needs an answer before a global
  rollout.
- **How is a drum score's difficulty assessed?** The level filter still offers
  beginner/intermediate/advanced, but the facets behind it — ambitus, smallest note
  value, chord presence — are keyboard-shaped. Left as-is here; `add-drum-scoring`
  should revisit it.
- ~~**Should the crawler ingest drum scores at all yet?**~~ **Decided: yes, with no
  instrument restriction.** The crawler's admission rule is and stays the
  redistributable-licence whitelist (`license-filtering`); instrument is not a
  licensing question, so adding an instrument condition would be a second, unrelated
  gate on the same path. A crawled percussion score is treated exactly like any other
  crawled score — it lands in the catalog, is withheld from callers outside the drum
  audience by the enforcement above, and enters the rating deck subject to the same
  eligibility (see the deck predicate). Realistically the yield will be small, since
  freely-licensed drum repertoire is scarce, but it costs nothing to accept what
  exists.
