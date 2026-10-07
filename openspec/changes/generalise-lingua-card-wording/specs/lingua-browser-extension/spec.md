## ADDED Requirements

### Requirement: The word card describes a form once, and says it in the interface language
The word card SHALL describe a form in terms that name no language — the readings' parts of speech, genders, numbers, persons, moods, tenses and degrees as the engine's tags, the other dictionary forms and the pieces — and SHALL render that description with the interface language's own words, articles, elisions, joining, sense headings and order of tenses. Tense names SHALL be keyed by pair, so that a studied language's tense is named as a reader of the interface language names it. The French rendering SHALL be byte for byte what the card said before. The card SHALL carry the interface language as its `lang`, and each word of the studied language it shows SHALL carry that language's.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the card on any form of an English or a Spanish word
- **THEN** the lines read byte for byte as before, and the card's `lang` is `fr` with `es` or `en` on the studied word

#### Scenario: An English-native reader of Spanish
- **WHEN** the interface language is English and the card opens on « vino » with lemma « venir »
- **THEN** a line reads "3rd person singular of the preterite of venir", with "venir" marked as Spanish

#### Scenario: A Spanish-native reader of English
- **WHEN** the interface language is Spanish and the card opens on "went"
- **THEN** a line reads « pasado simple de go », with "go" marked as English

#### Scenario: The order of tenses in each language
- **WHEN** the card opens on « hable » in each interface language
- **THEN** the French line names the subjunctive before the imperative as before, and the English and Spanish lines follow their renderer's pinned order

#### Scenario: A tag the renderer must know
- **WHEN** the engine's closed vocabulary of tags is enumerated
- **THEN** the description names every member, and each renderer renders every one
