## Why

The streak recovery offer is **use-it-or-lose-it, and playing loses it** — but the
only way to reach it is a dialog that opens itself at launch.

`advance` restarts the run on the first play after a gap
(`streak_core.rs:131`), so `recover_decision` then answers `Intact` and the
pre-break count is gone for good. The buy-back therefore exists only in the
window between opening the app and playing a note. Today that window is spent on
an unsolicited modal: the app interrupts someone who came to practise, asks them
for 30 points, and if they close it to go and play — the most natural thing to
do — the offer they declined "for now" is destroyed by the very act of
practising.

That is the wrong shape for the decision. It is not urgent because it is
important; it is urgent because we made it unreachable anywhere else.

A second, smaller problem: the spec's declined-offer rule no longer describes
what ships. It still says a refusal is remembered "on the device for the local
day it was declined on", reasoning that the grace window is one local day. PR
[#296](https://github.com/NEETROF/cymbra/pull/296) changed the code to remember
it **per break** (`streak_recovery_declined_run`, keyed on the run a recovery
would restore) precisely because `streak.grace_days` is a back-office flag and a
wider window re-asked the same question every morning. The behaviour is right;
the spec was not updated with it.

## What Changes

- **The standing chip becomes the way in.** The flame segment of the standing
  pill already shows the streak on every screen that hosts it; it becomes
  tappable, opening a streak sheet that states the standing and — while a break
  is recoverable — offers the buy-back with its cost. The offer stops depending
  on a modal the player did not ask for.
- **The launch dialog becomes a launch *cue*.** On a recoverable break the app
  still says something unprompted, because the window really does close on the
  next play — but as a dismissible in-app cue pointing at the chip, in the same
  register as the existing at-risk nudge, rather than a modal that blocks the way
  to the instrument. Confirming a spend still happens behind an explicit
  confirmation, unchanged: no silent debit, ever.
- **A refusal stops meaning "never mind, then".** Declining the cue silences the
  cue for that break (the rule that already ships), but the offer stays reachable
  from the chip for as long as the server still allows it. Saying "not now"
  currently costs the player the option; it should cost them the interruption.
- **BREAKING (spec only, no behaviour change): the declined-offer rule is
  restated per break rather than per local day**, matching what has shipped since
  #296. No code moves for this half — the spec catches up to it.

## Capabilities

### Modified Capabilities
- `practice-streak`: the recovery offer gains a reachable home (the standing
  chip) alongside the launch cue; the launch surface stops being modal; and the
  declined-offer rule is restated per break rather than per local day.

**Ordering constraint:** `practice-streak` is introduced by the in-flight
`add-practice-streak` change and is not yet in `openspec/specs/`. That change
must archive **before** this one, or this delta modifies requirements the base
spec never received. It has three tasks left (7.6, 10.4, 10.5), so the constraint
is live rather than theoretical.

## Impact

**Product: Cymbra Music only.** No backend, no gRPC surface, no schema. The
server's `recover_decision`, its idempotency key and the `streak.grace_days` /
`streak.freeze_cost` flags are consumed exactly as they are.

Consumed, unchanged:
- `StreakView` (`recoverable`, `recoverableStreak`, `recoverCost`, `atRisk`) —
  the client already receives everything the sheet needs to render;
- `Streak.recover()` and its fire-and-react outcome handling;
- `StreakRecoveryDecline`, whose per-break key ships already.

Modified in the app:
- `curator_chip.dart`: the streak segment becomes a tappable target;
- `streak_listener.dart`: the recovery modal becomes a cue; the outcome snackbars
  and the at-risk nudge are untouched;
- a new streak sheet (standing + recovery action + the cost), fr/en/es/it;
- `openspec/changes/add-practice-streak`'s delta text for the declined-offer
  rule, which this change's delta supersedes.

Deliberately out of scope: changing `streak.grace_days`, the freeze cost, or the
rule that playing forfeits the buy-back. The last one is the server's definition
of a streak and is not in question here — this change is about reaching the
decision, not about changing it.
