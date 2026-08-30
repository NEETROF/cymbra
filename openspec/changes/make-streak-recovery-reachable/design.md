## Context

The streak recovery is a one-shot decision with a hard deadline the player
cannot see. `advance` (`streak_core.rs:126`) restarts the run on the first play
after a gap, so `recover_decision` then answers `Intact`: the pre-break count is
unrecoverable the moment the player does the thing they opened the app to do.

The server's rule is sound — a streak you can buy back after resuming is not a
streak. What is wrong is the client's only expression of it. `StreakListener`
opens an `AlertDialog` from `listenManual(..., fireImmediately: true)` as soon as
the standing loads, on either of the two screens that mount it. The player
arrives to practise and is met by a modal asking for 30 points. Closing it —
which is what someone in a hurry does — records a refusal, and then playing
destroys the offer for good.

So the offer is simultaneously **too loud** (a modal nobody asked for, at the
worst moment) and **unreachable** (nowhere else, ever). Those are the same bug
seen from two sides: it has no home, so it had to shout.

The standing pill already carries a flame segment with the day count on every
screen that hosts it (`curator_chip.dart`), and it is inert — a label, not a
control. That is the home.

## Goals / Non-Goals

**Goals:**
- Make the recovery reachable on the player's initiative, for as long as the
  server still allows it.
- Keep an unprompted signal, because the deadline is real and invisible — but at
  a volume proportional to it.
- Keep every guarantee the freeze already has: explicit confirmation, no silent
  debit, at most one charge per local day, the server as the only authority on
  whether recovery is allowed.
- Bring the spec's declined-offer rule back in line with what ships.

**Non-Goals:**
- Changing `streak.grace_days`, the freeze cost, or any server rule. The window
  and the price are back-office decisions and stay there.
- Changing the rule that playing forfeits the buy-back. That is the definition of
  a streak, not a defect.
- A streak *history* view. The sheet states the current standing and the one
  action available on it; a heatmap already exists elsewhere.
- Touching the at-risk nudge or the evening push reminder.

## Decisions

### D1 — The chip is the entry point, not a new destination

The flame segment becomes tappable and opens a sheet. It is where the streak
already lives, on every screen that shows the standing, and it is where a player
looks when they wonder about their streak — which is exactly the moment the
recovery is relevant.

*Why not a settings entry:* nobody goes to settings to save a streak. *Why not a
dedicated screen:* the content is one number and at most one action; a route
would be a heavier answer than the question deserves.

The chip stays informative when there is nothing to act on — tapping it always
opens the sheet, which then simply states the standing. A control that only
sometimes responds teaches players not to try it.

### D2 — The launch surface becomes a cue, not a modal

The unprompted signal stays, because the deadline closes on the next play and a
player who does not know that will lose the offer without ever seeing it. But it
becomes a dismissible in-app cue pointing at the chip — the same register as the
existing at-risk nudge, which already ships and already handles "say something
without blocking".

*Why not remove the unprompted signal entirely:* the offer would then be
reachable but undiscovered, and the deadline would pass silently. Making
something available is not the same as making it known.

*Why not keep the modal:* it is the one shape that guarantees the interruption
lands at the worst possible moment, since it fires the instant the standing
resolves — before the player has done anything at all.

### D3 — Declining silences the cue, never the offer

Today "not this time" is recorded and the offer never returns; on the next play
it is destroyed. So a refusal is indistinguishable from a decision to forfeit,
which is not what the player was asked.

The refusal keeps its current storage and its current grain — per break, keyed on
the run a recovery would restore — and keeps meaning exactly one thing: *stop
telling me*. The chip remains available until the server says the window closed.

### D4 — The spec catches up to the shipped per-break rule, and says why

The base text still reasons "since the grace window is one local day, declining
silences that offer for its whole life". That reasoning is wrong in a way worth
recording rather than quietly deleting: `streak.grace_days` is a **back-office
flag**, so the window is not one day by construction — it is one day by current
configuration. Widening it re-opened the same question every morning, which is
what the beta tester reported, and what #296 fixed by keying the refusal to the
break instead of the calendar.

No behaviour changes for this half. The delta restates the requirement and
records the reason, so the next reader does not re-derive the same wrong
inference from the same flag.

### D5 — An unaffordable break is shown disabled, not hidden

*Settled during implementation (task 2.3).* The open question assumed the client
might not be able to tell "too poor" from "too late". Checking the wire settles
it without a proto change: `to_proto_standing` flattens the server's decision
into one bool, but maps `InsufficientPoints` to `recoverable: false` **with
`recover_cost` set to what is needed**, while an intact or lapsed streak reports
a cost of zero. A cost with no offer therefore means exactly one thing.

So the sheet shows the recovery disabled with "you are N points short", because
that is actionable where silence is not — and a player who cannot tell the two
apart has no way to know whether earning points would help.

The discriminator is not guessable from the field names, so it is named once as
`StreakView.unaffordable` rather than re-derived at each call site.

### D6 — The cue's record silences the cue, and the names say so

*Settled during implementation (task 4.3).* The cue is raised once per break,
reusing the record that already ships — so the second screen mounting the
listener and the next launch both stay quiet.

That widened what the record means, and two names became lies:
`StreakRecoveryDecline` / `decline()` said the player forfeited something, and
`streakRecoveryOffered` said whether an offer existed. Neither is true any more —
the offer is reachable from the chip regardless. They are now
`StreakRecoveryCue` / `silence()` and `streakRecoveryCueDue`. The prefs key is
untouched, so there is no migration.

Worth the churn: "the offer is gone because the player was asked once" is exactly
the misreading that produced the original bug.

## Risks / Trade-offs

- **[A cue is easier to ignore than a modal]** → Intended. The cost of ignoring
  it is losing a buy-back the player did not want enough to act on; the cost of
  the modal is interrupting every practice session that follows a missed day.
  The asymmetry favours the cue.
- **[Two ways in could disagree]** → Both read the same `StreakView` and call the
  same `Streak.recover()`; the sheet is the only thing that can spend, and the
  cue only points at it. The confirmation stays where the debit is.
- **[The chip is not on every screen]** → It rides the standing pill, which is on
  the surfaces that host the streak listener today, so the reachable set is not
  smaller than the surfaces that can currently ask. If that changes, the cue is
  still what carries the deadline.
- **[Spec-only edit that changes no code]** → Deliberate, and separated in the
  tasks so a reviewer can see that nothing moves for it. The alternative — a spec
  that describes a rule the app has not followed since #296 — is worse than a
  documented correction.

## Open Questions

*Both settled during implementation:* the unaffordable case (D5) and the cue's
grain (D6).
