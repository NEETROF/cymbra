## ADDED Requirements

### Requirement: The word card describes a form once, and says it in the interface language
The word card SHALL describe a form in terms that name no language — the readings' parts of speech, genders, numbers, persons, moods, tenses and degrees as the engine's tags, merged and deduplicated by tag, the other dictionary forms and the pieces — and SHALL render that description with the interface language's own words, articles, elisions, joining, sense headings and order of tenses. What depends on the studied language — tense names, the gerund's name, whether the infinitive is named, which moods are named, the order of tenses — SHALL be keyed by studied language in each renderer. A renderer SHALL name what the French card names today, and SHALL leave unnamed what it leaves unnamed. The French rendering SHALL be byte for byte what the card said before. Each word of the studied language the card shows SHALL carry that language as its `lang`.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the card on any form of an English or a Spanish word
- **THEN** the lines read byte for byte as before, and each studied word carries `lang="es"` or `lang="en"`

#### Scenario: An English-native reader of Spanish
- **WHEN** the interface language is English and the card opens on « vino » with lemma « venir »
- **THEN** a line reads "third-person singular preterite indicative of venir", with "venir" marked as Spanish

#### Scenario: A Spanish-native reader of English
- **WHEN** the interface language is Spanish and the card opens on "went"
- **THEN** a line reads « pasado simple de go », with "go" marked as English

#### Scenario: The order of tenses in each language
- **WHEN** the card opens on « hable » in each interface language
- **THEN** the French line names the subjunctive before the imperative as before, and the English and Spanish lines follow their renderer's pinned order

#### Scenario: What each renderer names
- **WHEN** the engine's closed vocabulary of tags is enumerated
- **THEN** the description names every member, each renderer names the same members the French one names, and a form the French card says nothing about gets no line in any language

## MODIFIED Requirements

### Requirement: A Spanish card names its forms as French schools do
A Spanish form SHALL be named as the interface language's grammar names it: in French, in French school terms — présent and imparfait de l'indicatif, passé simple, futur, conditionnel, présent, imparfait and futur du subjonctif, impératif, infinitif, gérondif, participe passé with its agreement — with French articles and elision; in Spanish, in RAE/ASALE terms; in English, in the English Wiktionary's form-of wording. A card of an English word SHALL read as before.

#### Scenario: A form of two persons
- **WHEN** the card opens on a Spanish form that is both the first and the third person of one tense and number
- **THEN** the line names both persons once, in the interface language's grammar — in French « 1re et 3e personnes du singulier … »

#### Scenario: A subjunctive and an imperative
- **WHEN** the card opens on « hable »
- **THEN** the line names the subjunctive readings and the imperative one, in the interface language's order

#### Scenario: An adjective's agreement
- **WHEN** the card opens on « rápidas »
- **THEN** the line names the feminine plural of « rápido », in the interface language's words

#### Scenario: The dictionary form itself
- **WHEN** the card opens on « hablar »
- **THEN** no line describes the form

#### Scenario: Elision before an accented vowel
- **WHEN** a French line names a form « d'él »
- **THEN** the elision applies as before; the English and Spanish lines use their own grammar

#### Scenario: English reads as before
- **WHEN** a reader of French opens the card on an English form
- **THEN** the lines read as before this requirement
