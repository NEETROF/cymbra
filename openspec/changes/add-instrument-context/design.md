## Context

Every discovery surface in the app is implicitly keyboard: the Score Hub pins the
piano filter, the home sections and courses assume one instrument. With drums
added, each of these either stays keyboard — hiding drums — or shows both, which
makes each half noisier for everyone. (The sound picker also lists every font, but
it is a player-path surface, not a discovery one — see Non-Goals.)

Two facts shape the answer, and they pull in opposite directions.

**Drums are beta-scoped now and general later.** The audience is resolved from the
caller's identity (staff, or a `midi-drums` campaign member), and the flag's scope
will eventually be widened to `global`. Anything built here has to work in both
phases without a flag day.

**The first run happens without an account, on purpose.** `welcome-onboarding`
guarantees it. So while drums are beta-scoped, an anonymous first-run user evaluates
flags with no staff role and no memberships: drums are *never* visible at that
moment. A "piano or drums?" step in the first-run sequence would offer one option to
everyone, for the whole length of the rollout.

## Goals / Non-Goals

**Goals:**

- Let a user say which instrument they are here for, and have the app follow.
- Work identically during the beta and after general availability.
- Keep the drum path usable from a first launch with no account, which means
  bundled drum scores.

**Non-Goals:**

- Governing the player. A score carries its instrument; the context is for
  discovery only.
- Seeding the sound picker. It is a player-path surface where the score's
  instrument must decide the family — a keyboard score under a drums context still
  needs piano fonts — and its family filtering is owned by `add-drum-audio-channel`,
  exactly where `add-drums-access` deferred it.
- Supporting a "both" context. The score is keyboard **or** drums, and a mixed view
  makes each half noisier; a user who plays both switches, which is one tap.
- Inferring the context from behaviour — see Decisions.

## Decisions

### The trigger is "first visible to this installation", not a fixed moment

*Rationale:* it is one rule that resolves to the right moment in each phase — after
sign-in while beta-scoped, at first launch once global. The alternative, naming a
moment, forces a second rule and a migration the day the rollout completes.

The trigger *arms* the offer; the home *presents* it. Visibility resolves from an
asynchronous flag fetch — bootstrap, a periodic poll, resume, an identity change —
so the flip can land at any moment, including mid-play. Left unbounded, "offered
when visible" mandates a dialog over whatever screen the user is on. The offer is
therefore bound to the home: the next time the user is on, or arrives at, the home
while drums are visible, and never over the player or any other surface.

*Alternative rejected:* an instrument step in the first-run sequence. It cannot work
while drums are beta-scoped, because the first run is deliberately account-less and
drums are resolved per identity.

*Alternative rejected:* offering the choice at every sign-in until answered. Noisier
for no gain: the switcher is permanently visible, so a user who dismissed the modal
has not lost access to the decision.

### The wording is a choice, never an unlock

*Rationale:* "you've unlocked drums" is true during the beta and false afterwards.
Since the same modal serves both phases, unlock wording would have to be rewritten
at exactly the moment nobody is thinking about copy. "What would you like to start
with?" is true in both.

### The context is sticky, and the switcher is what makes that safe

*Rationale:* the context is a durable preference; opening a score is a transient
act. Deriving the former from the latter is the classic over-inference bug, and its
failure is silent — the user is not told their home changed, so they cannot connect
cause to effect. A shared link would be enough to re-point the app.

Stickiness is only acceptable because the switcher is permanently in the home
header. If the control were in settings, the app would have to guess, and the
trade-off would invert.

*Alternative rejected:* the context follows the last opened score. The decisive
argument is that **the player never needs the context** — it reads the instrument
from the score — so following it buys no capability and only produces surprise.

*Alternative rejected:* sticky with a prompt after N scores of the other instrument.
Better on paper, and it needs a threshold, a counter and prompt-fatigue handling —
three values guessed with no data. Worth revisiting if usage shows people forget to
switch; the data will then say what N is.

### The hub filter is seeded, not governed

*Rationale:* this resolves the one scenario where stickiness loses — the user who
deliberately goes looking at the other instrument's catalogue and is bounced back on
every return. Separating the durable preference from the session's working state
costs one extra piece of state and removes the annoyance without making the app
guess.

The working state yields to exactly one thing: an explicit context change. Flipping
the switcher (or answering the modal) re-seeds the filter even over a session
adjustment — the act's whole meaning is "re-point the app" — while navigation alone
never re-seeds. Retention protects the user from the app; it does not protect the
session from the user.

### Bundled drum scores are authored, not promoted from the catalog

*Rationale:* the existing bundled scores are authored for the project
(`assets/scores/CREDITS.md`), with the *work* in the public domain and the
transcription ours. For drums this is easier still: a basic groove is an idiom, not
a copyrightable work.

*Alternative rejected:* promoting the public-domain drum scores already in the
catalog. They carry a **crawler-assigned** licence classification and are `pending`,
never human-reviewed. `CREDITS.md` sets the bundling rule explicitly — drop anything
whose licence cannot be confirmed — and a classifier's output is not a confirmation.
Shipping bytes inside a binary is a higher bar than holding them in a moderated
catalog.

*Side benefit:* authoring at a chosen level sidesteps the unresolved
drum-difficulty question, which only bites for catalog content.

### Degradation is presentational-only, because the invisible state is routine

A context naming a not-currently-visible instrument renders as keyboard silently; a
context with nothing to show explains itself. The fallback never writes: neither
the stored context nor the offered-once marker is modified by a visibility change,
and the context reapplies unchanged the moment drums are visible again.

*Rationale:* both are states a user lands in without having done anything wrong, and
both would otherwise present as "the app is broken". And the invisible state is not
the narrow campaign-abandonment case it first looks like: the flag client begins
every cold launch on an empty snapshot and resets it on sign-out, so *every* launch
and *every* account switch passes through a moment where drums are invisible. A
fallback that persisted would erase the user's choice on each of them — and since
the choice is offered at most once, erase it permanently, with no re-offer to
recover it. Presentation-only is the only implementation that survives the flag
client's own lifecycle.

## Risks / Trade-offs

**Shipping this before drums are playable** → a user could pick drums and find
nothing to play. It has no code dependency on the audio, rendering or kit-view
changes, which makes the mistake easy. Mitigation: the sequencing constraint in the
proposal — land after `add-drum-kit-view`, so a bundled drum score at least opens
and renders — with this change's own gates scoped to exactly that, the full loop
(gauge and summary) named a general-availability prerequisite on
`add-drum-audio-channel`, `add-drum-input-mapping` and `add-drum-scoring`, and the
empty-context invitation as a backstop rather than a plan.

**Two surfaces can disagree about visibility** → the modal trigger and the
switcher's presence both depend on "are drums visible", read at different times.
Mitigation: one predicate, read from the same state, never duplicated.

**"Once" needs a durable marker** → the choice is offered once per installation, so
a stored flag must survive relaunch and must not be reset by signing out. Mitigation:
an explicit test for the sign-out/sign-in cycle, which is exactly where a naive
implementation re-prompts.

**Authoring musical content is not a coding task** → the bundled scores need someone
to write four grooves, tier them, and record credits. It sits in a change otherwise
made of code and could stall it. Mitigation: it is a separable task group; if it
becomes a bottleneck the change can ship the mechanism and the scores can follow —
but not to general availability, where they are a prerequisite.

## Migration Plan

None. One added preference defaulting to keyboard, so an existing user is unaffected
and sees no new control until drums become visible to them.

Rollback is a revert; the stored preference is inert if the code that reads it is
gone.

## Open Questions

- **Where exactly does the switcher sit on a phone?** The mockup places it under the
  home header, which is comfortable on a tablet. On a phone-class viewport it
  competes with the header itself; the responsive placement is a layout decision
  best made against the real screen.
- **How many bundled drum scores, and at which levels?** Four is the working
  assumption — enough to cover the three tiers and to exercise unpitched notes, the
  part-list instrument table, the percussion clef, two voices and open/closed
  hi-hat. The right number is whatever makes the no-account experience feel
  non-empty, which is a judgement to make once they exist.
