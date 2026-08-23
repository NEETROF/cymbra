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
- Making drum scores *useful*. Until the rendering and audio changes land, a drum
  score in the app is an empty staff and silence. That is precisely why the audience
  is restricted.
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

Withheld scores are absent from listings and unknown on direct fetch.

*Rationale:* a distinguishable refusal is itself a disclosure — it tells the caller
that a percussion score exists at that id. This also keeps the search path simple:
one predicate in the `WHERE` clause rather than a post-filter that would corrupt
paging counts.

### Fail closed about the caller, fail *open* about the score

Uncertainty about eligibility denies: an unreadable flag store, an unwired
`PlanSource`, unresolvable memberships all mean "not eligible". Uncertainty about a
score does the opposite — an `unknown` instrument is served normally.

*Rationale:* the two uncertainties are not symmetric. Denying on an unknown caller
costs a staff member a feature for a moment. Denying on an unknown *score* would
hide a slice of the existing corpus from users who can read it today — a live
regression, not a safety gain. It is sound because every `unknown` row predates the
gate opening and the gate refused percussion, so no `unknown` row can be percussion.
This invariant is worth stating in the migration, because it stops holding the
moment anyone inserts a row without classifying it.

### The backfill records `unknown`, not a definite family

`is_piano = true → keyboard`; `is_piano = false → unknown`.

*Alternative rejected:* mapping `false → other`. It reads tidier and it is wrong:
`false` meant "fewer than two staves", which includes single-staff piano pieces.
Asserting they are not keyboard would permanently mislabel them, and the catalog has
no second signal to recover from it. `unknown` is the truthful value, it preserves
current behaviour (those rows are already excluded by the pinned piano filter), and
re-crawling replaces it with the derived value through the existing ingest path —
which is how `score-facets` already says the corpus gets repopulated.

### The Score Hub stops pinning a filter

The hub currently pins `isPiano: true` (`catalog_search_notifier.dart:270`). It
stops pinning anything and offers instrument as a user filter, listing drums only
for the drum audience.

*Rationale:* the pin existed because "the corpus is piano-only", a condition the
spec states explicitly and which no longer holds. Keeping the pin and adding a
drum-only escape hatch would leave two filters fighting. The backend already
withholds what a user may not see, so the hub does not need to defend anything.

### The rating deck predicate follows the same rule

`pg.rs:796` sources the deck with `AND cs.is_piano`. It becomes an instrument
predicate honouring the caller's eligibility, so a drummer can rate drum scores and
a pianist is never handed one.

*Rationale:* left alone, this line would silently make drum scores unratable
forever — a defect nothing in the UI would reveal.

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
(add `instrument`, backfill, switch readers, drop `is_piano`), so a rollback between
steps is possible; and the generated clients are regenerated as part of the change
(`melos run gen-grpc` for the app, `yarn gen` for the console).

**The gate is only as good as its least-guarded path** → six call sites can disclose
a score (search, catalog bytes, user bytes, saved list, owned list, rating deck) and
missing one silently defeats the whole change. Mitigation: enumerate them in the
tasks, and test each for the ineligible case rather than testing the predicate once
in isolation.

**`music` becomes a flag consumer** → a new dependency on `FlagService` in a module
that had none, and a flag read on hot catalog paths. Mitigation: follow
`notifications/src/dispatch.rs`, which reads from the hot in-memory store rather
than the database; and default every unreadable value to disabled.

**The campaign is an operational prerequisite** → until the `midi-drums` campaign
exists and has members, `beta:midi-drums` reaches staff only, which will look like a
bug to a tester who was promised access. Mitigation: creating and populating the
campaign is a named manual task, done before the flag is switched on.

**Drum scores become uploadable before they are playable** → an eligible tester can
put a drum score in their library and find it renders as an empty staff. Accepted
and intended: that audience is the people building the feature. It is also why the
audience is not merely hidden but enforced.

**Drum proposals arrive before they can be judged** → public proposal stays open for
percussion, with moderation as the gate. But the console cannot draw a drum score
until `add-drum-notation-render`, so a moderator receives proposals they cannot
evaluate and can only leave pending. Accepted, because the audience able to propose
one is staff plus a handful of beta testers, so the queue cannot fill; the practical
mitigation is to land the render change before enrolling testers in numbers, and the
review row is badged with the instrument so the cause is visible without opening it.

## Migration Plan

1. Add `instrument TEXT` to `music.catalog_scores` and `music.user_scores`, nullable.
2. Backfill: `true → 'keyboard'`, `false → 'unknown'`.
3. Switch every reader and writer to `instrument`; regenerate the app and console
   clients.
4. Drop `is_piano` once no reader remains.
5. Declare the flag (default off). Create the `midi-drums` campaign and enrol
   testers. Only then set the override to `beta:midi-drums`.

Steps 1–2 are safe on their own and reversible. Step 4 is the point of no return;
it should land after step 3 has been observed working.

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
