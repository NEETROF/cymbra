## Why

Once drums exist, the app has two instruments and no way to say which one you are
here for. Every discovery surface — the Score Hub's filter, the home sections, the
sound picker, the courses — is implicitly keyboard today, and would either stay
keyboard (making drums invisible) or mix both (making each half noisier for
everyone).

The drum feature is meant to become generally available, not to stay a beta. So
whatever is built here has to work in **both** phases: while drums reach only staff
and the `midi-drums` campaign, and later when they reach everyone from a first
launch with no account.

## What Changes

**A persisted instrument context**

- One value — keyboard or drums — that seeds what the discovery surfaces show.
- **Sticky: it changes only when the user changes it.** Opening a score never
  modifies it.

**A one-time choice, at the moment drums first become visible**

- The trigger is "**the first time the drum feature is visible to this
  installation**": after sign-in for an eligible account during the beta, at first
  launch once the flag is global. One rule, two moments, no special case for the
  transition.
- Worded as a **choice** ("what would you like to start with?"), never as an unlock.
  "You've unlocked drums" is meaningless to a user for whom drums were always there,
  and would need rewriting the day the rollout completes.

**A permanent switcher in the home header**

- Not in settings. It is what makes a sticky context safe: a wrong choice costs one
  tap and the remedy is in view.
- It appears **only when drums are visible**. Everyone else sees today's home, with
  no new control and no question asked.

**Bundled drum scores**

- `welcome-onboarding` requires the core loop to be playable without an account,
  from bundled scores. Drums must satisfy the same requirement, or picking drums at
  a first launch leads to an empty app.
- The scores are **authored for the project**, as the existing bundled scores are
  (`assets/scores/CREDITS.md`), not sourced. A basic groove is an idiom, not a
  copyrightable work, so the licensing question that dogs drum repertoire does not
  arise. The public-domain drum scores already in the catalog are **not** promoted:
  they are crawler-classified and never human-reviewed, and shipping bytes inside
  the binary is a higher bar than holding them in a moderated catalog.

**Explicitly not changed**

- **The player.** A score carries its instrument; the player reads it directly.
  Opening a keyboard score while the context is drums works, with no warning and no
  switch. The context is never a lock.

## Capabilities

### New Capabilities

- `music-instrument-context`: the persisted instrument context — what it seeds, what
  it must never govern, when the one-time choice is offered, where the switcher
  lives, and how the context behaves when drums are withdrawn or when it has nothing
  to show.

### Modified Capabilities

- `welcome-onboarding`: the no-account core loop extends to drums, which requires
  bundled drum scores; and the instrument choice is introduced as a contextual
  moment rather than as a step of the first-run sequence, because during the beta
  drums are not knowable before sign-in.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the drum-visibility flag, the parser's instrument classification | the context preference, the choice modal, the home switcher, bundled drum scores |
| **Back-office / ID / Live / Site** | — | untouched |

**Code**

- `apps/music/lib/state/`: the persisted context beside the existing play
  preferences, and the seeding of the hub's filter.
- `apps/music/lib/screens/`: the home header switcher, the one-time modal.
- `apps/music/assets/scores/`: the bundled drum scores and their `CREDITS.md` rows.
- `apps/music/lib/l10n/`: the modal, the switcher and the empty-state copy (fr/en).

**Depends on** `add-drums-access` (the visibility flag and the instrument
classification). It does **not** depend on the audio, notation-rendering or kit-view
changes — but shipping it before them would let a user pick drums and find nothing
playable, so it should land after `add-drum-kit-view` in practice.

**Reference.** `mockups/context.html` shows the modal, the switcher in the home
header, and the split between what the context seeds and what it must never govern.
