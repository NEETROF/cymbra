## 1. The spec correction (no code moves)

- [ ] 1.1 Confirm on the shipped code that the refusal is keyed to the break, not
  the day: `StreakRecoveryDecline.prefsKey == 'streak_recovery_declined_run'` and
  `streakRecoveryOffered` compares against `value.recoverableStreak`. The delta
  restates behaviour that already ships since #296 — if this does not match, the
  delta is wrong and not the code
- [ ] 1.2 Nothing else in this section. Kept separate on purpose so a reviewer can
  see the spec-only half is spec-only

## 2. The streak surface (spec: `practice-streak`)

- [ ] 2.1 A sheet stating the standing: current run, longest, whether today is
  secured. Reads `StreakView` only — no new server call, no new field
- [ ] 2.2 The recovery action, shown whenever `recoverable` is true, carrying
  `recoverCost` and `recoverableStreak`. Fires `Streak.recover()` and reacts to
  the resulting state; never awaits its return to decide anything (architecture
  rule 3)
- [ ] 2.3 Settle the open question: is the action shown **disabled with the
  shortfall** when the balance is short, or hidden? Decide against the real
  numbers a player would see and record it in `design.md`. Note the server
  answers `InsufficientPoints` separately from `Allow`, so the client can tell
  the two apart without guessing
- [ ] 2.4 The outcome (restored / refused) surfaces exactly as it does today —
  the existing snackbars, localized, never a raw server string
- [ ] 2.5 Empty and signed-out states: a guest has no streak, and the sheet says
  so rather than rendering a zero that looks like a lost run
- [ ] 2.6 Localised fr/en/es/it, every locale aligned with the template

## 3. The chip becomes the way in

- [ ] 3.1 `curator_chip.dart`: the streak segment becomes a tappable target with
  its own key and a 48 px touch target, opening the sheet. It already carries a
  `Semantics` label and a tooltip — extend them to say it is actionable
- [ ] 3.2 It responds whether or not a recovery is available (design D1): a
  control that only sometimes works teaches players not to try it. Test both
- [ ] 3.3 Widget test that the segment's existing muted/lit/warm states are
  unchanged by becoming interactive — this chip is on every surface with a
  standing pill, so a regression here is everywhere
- [ ] 3.4 Riverpod layering: the chip opens a surface, the surface calls notifier
  methods; no service read from a widget, no provider invalidating a sibling

## 4. The launch prompt becomes a cue

- [ ] 4.1 `streak_listener.dart`: replace the recovery `AlertDialog` with a
  dismissible cue pointing at the chip, in the same register as the at-risk nudge
  that already ships beside it
- [ ] 4.2 Declining the cue silences it for that break — the existing
  `StreakRecoveryDecline` path, unchanged — and **does not** withdraw the offer:
  assert the sheet still offers the recovery afterwards (design D3). This is the
  behaviour change with teeth; test it from both sides
- [ ] 4.3 The re-entrancy guard and the "asked once" grain: settle whether the cue
  is once per session or once per break (design open question) and record it.
  Leaning per break, so dismissing it on one screen does not re-raise it on the
  next
- [ ] 4.4 The at-risk nudge and the evening push reminder are untouched —
  asserted, not assumed
- [ ] 4.5 Test that nothing can spend from the cue: the confirmation lives with
  the debit, in the sheet

## 5. Gates

- [ ] 5.1 `dart format --output=none --set-exit-if-changed .` **from the repo
  root** (the CI's own command — a run scoped to `apps/music` misses `packages/`)
- [ ] 5.2 `flutter analyze` and `dart run custom_lint` clean
- [ ] 5.3 `cd apps/music && flutter test --coverage --exclude-tags golden`,
  coverage ≥ 80%
- [ ] 5.4 `openspec validate make-streak-recovery-reachable --strict`
- [ ] 5.5 **Ordering:** `add-practice-streak` must be archived **before** this
  change is, or this delta modifies `practice-streak` requirements the base spec
  in `openspec/specs/` never received. That change has three tasks left (7.6, 10.4
  and 10.5), so the constraint is live — see §6

## 6. Manual verification

- [ ] 6.1 Refuse the cue, then open the chip: the recovery is still there with its
  cost. This is the whole point of the change
- [ ] 6.2 Refuse the cue, force-quit, relaunch: the cue does not return and the
  chip still offers it
- [ ] 6.3 Recover from the sheet: the streak is restored, the points are debited
  once, and the standing chip updates without a relaunch
- [ ] 6.4 Play without recovering: the offer disappears on its own, because the
  run restarted — confirm the sheet stops offering it rather than failing on
  confirmation
- [ ] 6.5 A user with too few points sees whatever 2.3 decided, and no debit is
  attempted
- [ ] 6.6 Read `streak.grace_days` in production — already tracked as
  `add-practice-streak` task 10.4, and it is what tells us whether the daily
  re-ask this change builds on was ever legitimate behaviour. Record the value
  there, not here
