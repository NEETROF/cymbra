## ADDED Requirements

### Requirement: Swipe rating deck over validated scores

The app SHALL present a card-stack ("deck") rating surface, available only to a
signed-in user, that sources **validated (`accepted`)** catalog scores through the
existing catalog search. The user SHALL be able to rate the top card by swiping —
**left = pass**, **right = like**, **up = love** — advancing to the next card. Each
swipe action SHALL be recorded through the rating operation. The deck SHALL be driven
through injectable state so it is exercisable in tests without the native library or a
live backend.

#### Scenario: Swipe records a rating and advances

- **WHEN** the user swipes the top card left, right, or up
- **THEN** the corresponding verdict (pass / like / love) is submitted for that score
  and the next card is shown

#### Scenario: Deck shows only validated scores

- **WHEN** the deck loads its cards
- **THEN** every card is an `accepted` score (no `pending` or `rejected` scores appear)

#### Scenario: Deck empties when all sourced scores are rated

- **WHEN** the user has rated all currently sourced cards
- **THEN** the deck shows an empty/last-card state rather than repeating cards

### Requirement: Tap-button parity for swipe actions

The deck SHALL present on-screen buttons under the card that perform the same actions
as the swipes (pass / like / love), so the rating flow is fully usable without
swiping. Swiping is a shortcut, not the only path.

#### Scenario: Buttons mirror swipes

- **WHEN** the user taps the pass, like, or love button
- **THEN** the same rating is recorded and the deck advances exactly as the equivalent swipe

#### Scenario: Rating without swiping

- **WHEN** a user never swipes and only uses the buttons
- **THEN** they can rate every card and progress through the deck

### Requirement: Star rating on the card

Tapping the card SHALL open a 1–5 star rating for the shown score, recording an
explicit star value through the rating operation (reconciled with the swipe verdict so
one rating is stored per user per score).

#### Scenario: Setting stars records an explicit rating

- **WHEN** the user opens a card and selects a star value
- **THEN** that star value is submitted as the user's rating for the score

#### Scenario: Stars and swipe stay one rating

- **WHEN** a user both swipes and sets stars on the same score
- **THEN** a single reconciled rating is stored, not two conflicting ratings

### Requirement: In-card score info and read-only preview

Each card SHALL show the score's information (at least title, composer, difficulty when
known, and the licence/source attribution). The card SHALL offer a Play control that
previews the score in the **same horizontal game-score render mode used for play, but
read-only** — the notation is shown and the notes sound, while **no user interaction on
the played score is possible** (no input judging, editing, or scoring). Stopping the
preview SHALL return to the card.

#### Scenario: Card shows score info and attribution

- **WHEN** a card is displayed
- **THEN** it shows the score's title, composer, level (when known), and licence/source attribution

#### Scenario: Preview plays read-only

- **WHEN** the user starts the in-card preview
- **THEN** the score renders in the horizontal game mode and the notes sound, with no
  interaction on the played score and no scoring produced

#### Scenario: Preview is dismissible back to the card

- **WHEN** the user stops the preview
- **THEN** the deck returns to the card so the user can rate or advance

### Requirement: First-run coaching for the rating gestures

On first use of the deck, the app SHALL show a one-time coaching hint explaining that
the user can swipe or tap the stars, and SHALL not repeat it on later visits. A subtle
affordance MAY hint that the top card is interactive.

#### Scenario: Coach mark shown once

- **WHEN** the user opens the deck for the first time
- **THEN** a one-time hint explains swiping and star rating

#### Scenario: Coach mark not repeated

- **WHEN** the user returns to the deck after seeing the hint
- **THEN** the hint is not shown again
