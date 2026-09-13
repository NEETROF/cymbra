# lingua-decks-review — level-targeted deck seeding

## ADDED Requirements

### Requirement: Level-targeted deck seeding
The deck SHALL support seeding cards for a chosen set of lemmas — typically the lemmas of a
selected CEFR level (or frequency band) — without a real web encounter. Seeded cards SHALL
record the reserved `import` encounter source rather than fabricating a URL or an agent
session. Seeding SHALL be capped per operation, SHALL skip lemmas that already have a card
or an explicit status (idempotent), and SHALL let the caller choose the order in which
lemmas are taken from the level (commonest-first by default).

#### Scenario: Seed a level into the deck
- **WHEN** the user asks to add 20 words of level B2 with 5 of them already tracked
- **THEN** at most 15 new cards are created, each with encounter source `import`, and no already-tracked lemma is duplicated

#### Scenario: Cap respected
- **WHEN** the user requests more words than the per-operation cap allows
- **THEN** only up to the cap are seeded and the caller is told how many were added
