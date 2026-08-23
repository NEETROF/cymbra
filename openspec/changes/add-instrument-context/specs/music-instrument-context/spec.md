## ADDED Requirements

### Requirement: A persisted instrument context seeds discovery

The app SHALL hold one **instrument context** — keyboard or drums — persisted across
launches, which seeds what the discovery surfaces present: the Score Hub's
instrument filter, the home sections, and the courses surfaced. It SHALL default to
keyboard, which is the app's existing behaviour.

The sound picker is deliberately **not** among the seeded surfaces. It is a
player-path surface, and the player follows the score: a keyboard score opened under
a drums context still needs piano fonts, so the *score's* instrument must decide the
family there. Family filtering of the picker is owned by `add-drum-audio-channel` —
the deferral `add-drums-access` already records — not by this change.

Courses are seeded like the rest, with a present-tense caveat: no drum courses exist
yet, so under a drums context the course surface shows the same explicit invitation
as an empty context (see "The context degrades gracefully"), never a bare blank. The
course cards in `mockups/context.html` are illustrative of the seeded layout, not of
available content.

The context SHALL be **sticky**: it changes only when the user changes it. Opening,
playing or finishing a score SHALL NOT modify it, whatever that score's instrument.

Inferring a lasting preference from a single action fails silently — the user is
not told their home changed, so they cannot connect the effect to its cause, and an
incoming shared link would be enough to reconfigure the app.

#### Scenario: The context seeds the discovery surfaces

- **WHEN** the context is drums
- **THEN** the hub's instrument filter, the home sections and the surfaced courses
  all start from drums

#### Scenario: A drums context with no drum courses explains itself

- **WHEN** the context is drums and no drum course exists yet
- **THEN** the courses surface shows the explicit empty-context invitation, not a
  bare blank

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

Changing the **context** — from the switcher or the choice modal — SHALL re-seed the
filter immediately, even over an adjustment made this session: an explicit act on
the context outranks the session's working state, which exists to survive mere
navigation, never a deliberate re-pointing. Navigation alone SHALL never re-seed.

This separates a durable preference from a working state: a user can explore
another instrument's catalogue at length without either being nagged back to their
context on every return, or having their whole app quietly re-pointed because they
went looking.

#### Scenario: The filter starts from the context

- **WHEN** the user opens the hub
- **THEN** the instrument filter starts on the context's instrument

#### Scenario: An adjusted filter is kept for the session

- **WHEN** the user changes the hub's instrument filter and navigates away and back
  without touching the context
- **THEN** the filter is still on their adjusted value

#### Scenario: Switching the context re-seeds an adjusted filter

- **WHEN** the user has adjusted the hub's filter this session and then changes the
  context
- **THEN** the filter is re-seeded to the new context's instrument

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

The offer SHALL be presented **on the home only**: the next time the user is on, or
arrives at, the home while drums are visible — never over the player or any other
surface. Visibility resolves from an asynchronous flag fetch (bootstrap, a periodic
poll, resume, an identity change), so the flip can land at any moment, including
mid-play; when it does, the offer defers to the next arrival at the home.

The choice SHALL be worded as a **choice**, not as an unlock or a reward: a user for
whom drums were always available has unlocked nothing, and unlock wording would have
to be rewritten the day the rollout completes. It SHALL state that the choice can be
changed later, and SHALL NOT block: either option proceeds.

#### Scenario: Offered on first visibility, whenever that is

- **WHEN** the drum feature becomes visible to an installation for the first time
- **THEN** the instrument choice is offered on the next presence on the home

#### Scenario: A mid-play visibility flip defers the offer

- **WHEN** drums become visible while the user is in the player
- **THEN** nothing interrupts the session, and the choice is offered on the next
  arrival at the home

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

The app SHALL degrade gracefully when the context cannot be honoured, and the
degradation SHALL be **presentational only**: neither the stored context nor the
offered-once marker is ever modified by a change of visibility. When the context
names an instrument that is not currently visible — a beta campaign closed without
the feature having been widened, or simply a flag snapshot that has not resolved
yet — the home SHALL render as keyboard **silently**, without an error, leaving the
user on a working home rather than an empty one; the moment drums are visible
again, the stored context reapplies unchanged.

Persisting the fallback is the wrong implementation, not merely an unspecified one:
the flag snapshot is empty on every cold launch and resets on sign-out, so a
fallback that wrote back would silently erase the user's choice on every launch and
every account switch — and since the choice is offered at most once, erase it
permanently.

When the context is in force and there is nothing to show for it — drums chosen but
no drum score reachable — the app SHALL surface an explicit invitation naming the
cause and offering to switch, never a bare empty screen.

#### Scenario: Withdrawn instrument falls back silently

- **WHEN** the context is drums and the drum feature stops being visible
- **THEN** the home renders as keyboard without an error, and the stored context is
  still drums

#### Scenario: A cold start does not erase the context

- **WHEN** the context is drums and the app cold-starts with the flag snapshot not
  yet resolved
- **THEN** the home first renders as keyboard, then reapplies drums once the flag
  resolves, and the stored value was never modified

#### Scenario: A sign-out/sign-in round trip keeps the context

- **WHEN** the context is drums, the user signs out — drums no longer visible — and
  signs back in on an eligible account
- **THEN** the context is drums again, without the choice being re-offered

#### Scenario: An empty context explains itself

- **WHEN** the context is drums and no drum content is reachable
- **THEN** the home shows an explicit invitation to switch, not an unexplained empty
  state
