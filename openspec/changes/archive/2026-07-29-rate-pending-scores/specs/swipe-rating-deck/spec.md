## MODIFIED Requirements

### Requirement: Swipe rating deck over validated scores

The app SHALL present a card-stack ("deck") rating surface, available only to a
signed-in user, that sources catalog scores whose moderation status is **`pending` or
`accepted`** (never `rejected`) through the deck-sourcing read. The user SHALL be able
to rate the top card by swiping — **left = dislike**, **right = like**, **up = love** —
advancing to the next card. Each swipe action SHALL be recorded through the rating
operation; `dislike` is a negative verdict. The deck SHALL be driven through injectable
state so it is exercisable in tests without the native library or a live backend.

#### Scenario: Swipe records a rating and advances

- **WHEN** the user swipes the top card left, right, or up
- **THEN** the corresponding verdict (dislike / like / love) is submitted for that score
  and the next card is shown

#### Scenario: Deck shows pending and accepted scores, never rejected

- **WHEN** the deck loads its cards
- **THEN** every card is a `pending` or `accepted` score and no `rejected` score appears

#### Scenario: Deck empties when all sourced scores are rated

- **WHEN** the user has rated all currently sourced cards
- **THEN** the deck shows an empty/last-card state rather than repeating cards

### Requirement: In-card score info and read-only preview

Each card SHALL show the score's information (at least title, composer, difficulty when
known, and the licence/source attribution). A card for a `pending` score SHALL be
labelled as a **"potential new score"** the user is helping evaluate (a positive framing,
not a warning). The card SHALL offer a Play control that previews the score in the **same
horizontal game-score render mode used for play, but read-only** — the notation is shown
and the notes sound, while **no user interaction on the played score is possible** (no
input judging, editing, or scoring). For a `pending` score, the preview SHALL obtain its
bytes through the **deck-scoped rating-preview path** (it is not opened in the full
player). Stopping the preview SHALL return to the card.

#### Scenario: Card shows score info and attribution

- **WHEN** a card is displayed
- **THEN** it shows the score's title, composer, level (when known), and licence/source attribution

#### Scenario: Pending card is labelled a potential new score

- **WHEN** a card for a `pending` score is displayed
- **THEN** it is marked as a "potential new score" the user is helping evaluate, with attribution shown

#### Scenario: Preview plays read-only

- **WHEN** the user starts the in-card preview (for a pending or accepted score)
- **THEN** the score renders in the horizontal game mode and the notes sound, with no
  interaction on the played score and no scoring produced

#### Scenario: Pending preview does not open the full player

- **WHEN** the user previews a `pending` score in the deck
- **THEN** the bytes come from the rating-preview path and the score cannot be opened in the
  full player or saved to a library from the deck

#### Scenario: Preview is dismissible back to the card

- **WHEN** the user stops the preview
- **THEN** the deck returns to the card so the user can rate or advance
