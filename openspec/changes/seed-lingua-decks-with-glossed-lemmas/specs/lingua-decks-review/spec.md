## MODIFIED Requirements

### Requirement: Level-targeted deck seeding
The deck SHALL support seeding cards for a chosen set of lemmas — typically the lemmas of a selected CEFR level (or frequency band) — without a real web encounter, each card carrying the gloss that the pack of its studied language, glossed in the engine's native language, gives its lemma.
Seeded cards SHALL record the reserved `import` encounter source rather than fabricating a URL or an
agent session. Seeding SHALL be capped per operation, SHALL skip lemmas that already have a card or
an explicit status (idempotent), SHALL skip a lemma that pack does not gloss, so that no seeded card
arrives without a gloss, and SHALL let the caller choose the order in which lemmas are taken from the
level (commonest-first by default).

A skipped lemma SHALL take no place under the cap: the next lemma of the level, in the order chosen,
takes it, so that a seeding asked for N cards adds N whenever the level holds N lemmas that are
glossed and untracked, and fewer only when it does not. A lemma skipped for want of a gloss SHALL
receive no card and no status, and SHALL keep its level, its place in the level ladder, in the
vocabulary estimate and below a declared level, as a word of its language.

#### Scenario: Seed a level into the deck
- **WHEN** the user asks to add 20 words of level B2, commonest first, and 5 of the level's first 20 lemmas in that order are already tracked
- **THEN** 20 new cards are created, the next untracked lemmas of B2 taking the 5 tracked lemmas' places, each with encounter source `import` and its gloss, and no already-tracked lemma is duplicated

#### Scenario: Cap respected
- **WHEN** the user requests more words than the per-operation cap allows
- **THEN** only up to the cap are seeded and the caller is told how many were added

#### Scenario: The rarest words of a level have no gloss
- **WHEN** a French-native reader of English asks for 20 words of C1, rarest first, and the en-fr pack glosses none of the 20 rarest C1 lemmas (`unscathing`, `unmusically`, `unenviably`, …)
- **THEN** 20 cards are created from the rarest C1 lemmas the pack glosses, each with its French gloss, and none of the 20 unglossed lemmas gets a card or a status — where before every one of the 20 cards had no gloss

#### Scenario: A French level with no Spanish gloss
- **WHEN** a Spanish-native reader of French seeds level A1 until it is spent, and the fr-es pack does not gloss `part`, which French's levels place at A1
- **THEN** no card is created for `part`, which keeps its level, its place in the A1 row of the ladder and in the vocabulary estimate and gets no status, while an English-native reader, whose fr-en pack glosses it, is given its card

#### Scenario: Only lemmas without a gloss are left
- **WHEN** every lemma of a level that the pack glosses already has a card or a status, and the level's other lemmas have no gloss
- **THEN** seeding that level creates no card and the caller is told none was added, and the ladder still counts those other lemmas at that level

#### Scenario: A pack that glosses every levelled lemma
- **WHEN** the es-fr pack, which glosses each of Spanish's 8,302 levelled lemmas, seeds any level in either order
- **THEN** it creates exactly the cards it created before this rule
