## ADDED Requirements

### Requirement: A persisted instrument context seeds discovery

The app SHALL hold one **instrument context** — keyboard or drums — persisted across
launches, which seeds what the discovery surfaces present: the Score Hub's
instrument filter, the home sections, the family offered by the sound picker, and
the courses surfaced. It SHALL default to keyboard, which is the app's existing
behaviour.

The context SHALL be **sticky**: it changes only when the user changes it. Opening,
playing or finishing a score SHALL NOT modify it, whatever that score's instrument.

Inferring a lasting preference from a single action fails silently — the user is
not told their home changed, so they cannot connect the effect to its cause, and an
incoming shared link would be enough to reconfigure the app.

#### Scenario: The context seeds the discovery surfaces

- **WHEN** the context is drums
- **THEN** the hub's instrument filter, the home sections, the sound picker's family
  and the surfaced courses all start from drums

#### Scenario: Playing a score never changes the context

- **WHEN** the context is drums and the user opens and plays a keyboard score
- **THEN** the context is still drums afterwards

#### Scenario: The context survives a relaunch

- **WHEN** the user sets a context and relaunches the app
- **THEN** the same context is in force

#### Scenario: Keyboard is the default

- **WHEN** a user has never made a choice
- **THEN** the context is keyboard

### Requirement: The context seeds the hub filter without governing it

The Score Hub's instrument filter SHALL be **seeded** from the context and then
remain independently adjustable, retaining the user's adjustment for the session.
Adjusting it SHALL NOT write back to the context.

This separates a durable preference from a working state: a user can explore
another instrument's catalogue at length without either being nagged back to their
context on every return, or having their whole app quietly re-pointed because they
went looking.

#### Scenario: The filter starts from the context

- **WHEN** the user opens the hub
- **THEN** the instrument filter starts on the context's instrument

#### Scenario: An adjusted filter is kept for the session

- **WHEN** the user changes the hub's instrument filter and navigates away and back
- **THEN** the filter is still on their adjusted value

#### Scenario: Adjusting the filter leaves the context alone

- **WHEN** the user changes the hub's instrument filter
- **THEN** the context is unchanged, and the home still reflects it

### Requirement: The context never governs the player

The context SHALL NOT influence what can be opened or how it is played. A score
carries its own instrument and the player reads it from the score. Opening a score
whose instrument differs from the context SHALL work normally, without a warning,
a confirmation, or a change of context.

The player never needs the context, so letting it interfere would buy no capability
and only produce refusals the user cannot explain.

#### Scenario: A score of the other instrument opens normally

- **WHEN** the context is drums and the user opens a keyboard score
- **THEN** it opens and plays normally, with no warning and no context change

#### Scenario: A shared link cannot reconfigure the app

- **WHEN** the user opens a score from a link or from their library
- **THEN** the context is untouched

### Requirement: The instrument choice is offered once, when drums first become visible

The app SHALL offer the instrument choice **once**, the first time the drum feature
is visible to this installation — after sign-in for an eligible account while the
feature is beta-scoped, and at first launch once it is generally available. The same
rule SHALL cover both, so the rollout's completion needs no special case.

The choice SHALL be worded as a **choice**, not as an unlock or a reward: a user for
whom drums were always available has unlocked nothing, and unlock wording would have
to be rewritten the day the rollout completes. It SHALL state that the choice can be
changed later, and SHALL NOT block: either option proceeds.

#### Scenario: Offered on first visibility, whenever that is

- **WHEN** the drum feature becomes visible to an installation for the first time
- **THEN** the instrument choice is offered

#### Scenario: Offered only once

- **WHEN** the user has already been offered the choice
- **THEN** it is not offered again, on any later launch

#### Scenario: Never offered when drums are not visible

- **WHEN** the drum feature is not visible to this installation
- **THEN** the choice is never offered and the app behaves as it does today

### Requirement: A permanent switcher, not a settings entry

While drums are visible, the app SHALL present the context switcher **persistently
in the home header**, not behind a settings screen. When drums are not visible it
SHALL NOT be shown at all: there is nothing to choose between.

A sticky context is only safe when correcting it is obvious and immediate. Placing
the control where it is always in view is what removes the need for the app to guess
on the user's behalf.

#### Scenario: The switcher is visible on the home

- **WHEN** drums are visible and the user is on the home
- **THEN** the context switcher is present without opening any menu

#### Scenario: No switcher when there is nothing to switch

- **WHEN** drums are not visible
- **THEN** no switcher is shown and no instrument question is asked anywhere

### Requirement: The context degrades gracefully

The app SHALL degrade gracefully when the context cannot be honoured. When the
context names an instrument that is no longer visible — a beta campaign closed
without the feature having been widened — it SHALL fall back to keyboard
**silently**, without an error, leaving the user on a working home rather than an
empty one.

When the context is in force and there is nothing to show for it — drums chosen but
no drum score reachable — the app SHALL surface an explicit invitation naming the
cause and offering to switch, never a bare empty screen.

#### Scenario: Withdrawn instrument falls back silently

- **WHEN** the context is drums and the drum feature stops being visible
- **THEN** the context falls back to keyboard without an error, and the home works

#### Scenario: An empty context explains itself

- **WHEN** the context is drums and no drum content is reachable
- **THEN** the home shows an explicit invitation to switch, not an unexplained empty
  state
