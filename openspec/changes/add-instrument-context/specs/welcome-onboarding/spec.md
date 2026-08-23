## MODIFIED Requirements

### Requirement: The core loop is experienceable without an account

The app SHALL provide a way to experience the **core loop** — playing a piece and seeing the
live synchronization gauge and the end-of-session summary — **without an account**, so the value
lands before any sign-up. This no-account experience SHALL use the app's **already-included
bundled scores** (playable without the backend), exercise the real player and scoring, and MUST
NOT require opening the authenticated catalog to anonymous users.

This SHALL hold for **every instrument the app offers without an account**. Once drums are
generally available they are offered from a first launch, so bundled **drum** scores SHALL ship
alongside the keyboard ones — otherwise choosing drums leads to an app with nothing to play,
which is the same failure the requirement exists to prevent.

The bundled drum scores SHALL be **authored for the project**, as the existing bundled scores
are, rather than sourced. A score whose licence cannot be confirmed SHALL NOT be bundled, and a
catalog row's crawler-assigned licence classification SHALL NOT be treated as such a
confirmation: shipping bytes inside the binary is a higher bar than holding them in a moderated
catalog.

#### Scenario: Try the core loop with no account

- **WHEN** a user chooses to try from the welcome without signing in
- **THEN** they can play an included bundled score and see the live gauge and end-of-session summary

#### Scenario: Trying uses included scores, not the authenticated hub

- **WHEN** the no-account try runs
- **THEN** it plays an included bundled score and does not require anonymous access to the authenticated catalog/hub

#### Scenario: Drums are playable without an account once generally available

- **WHEN** drums are generally available and a user picks drums without an account
- **THEN** they can play a bundled drum score and see the gauge and the summary, exactly as for
  keyboard

#### Scenario: A bundled score's licence is confirmed, never inferred

- **WHEN** a score is considered for bundling
- **THEN** it is included only if its licence is confirmed, and a crawler classification alone
  does not qualify

## ADDED Requirements

### Requirement: The instrument choice is contextual, not a first-run step

The instrument choice SHALL NOT be added to the first-run sequence that precedes the welcome.
While the drum feature is beta-scoped it is resolved from the caller's identity, and the first
run happens deliberately **without an account** — so drums are not knowable at that point and
the question would offer a single option to everyone, for as long as the rollout lasts.

The choice SHALL instead be offered when the drum feature first becomes visible to the
installation (see `music-instrument-context`), which lands inside the first-run sequence
naturally once the feature is generally available, and after sign-in before then.

#### Scenario: No instrument question during the beta's first run

- **WHEN** a user launches the app for the first time while drums are beta-scoped
- **THEN** the first-run sequence is unchanged and asks nothing about instruments

#### Scenario: The question arrives when it can be answered

- **WHEN** drums become visible to the installation
- **THEN** the choice is offered at that moment, whether that falls during first run or after a
  later sign-in
