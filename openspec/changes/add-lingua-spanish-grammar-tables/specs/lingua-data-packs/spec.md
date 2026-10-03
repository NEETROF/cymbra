## ADDED Requirements

### Requirement: The Spanish pack's word grammar
The es-fr pack SHALL carry the grammar of the forms its forms table holds, read from kaikki's tags as Universal Dependencies tags: a verb form's mood, tense, person and number, or its infinitive, gerund or agreed participle; a noun's gender on its own form and on its plural; an adjective's, determiner's or pronoun's agreement. A reading of another dictionary form the pack keeps SHALL be marked so that the card names it. A form that combines a verb with clitic pronouns SHALL carry no reading.

#### Scenario: A verb form says what it is
- **WHEN** the card asks the grammar of `hablábamos` as *hablar*
- **THEN** it answers the indicative imperfect, first person plural (`VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin`)

#### Scenario: A noun says its gender
- **WHEN** the card asks the grammar of `casas` as *casa*
- **THEN** it answers a feminine plural noun (`NOUN|Gender=Fem|Number=Plur`)

#### Scenario: A homograph names its other dictionary form
- **WHEN** the card asks the grammar of `vino` as the noun *vino*
- **THEN** it also names *venir*, whose preterite third person singular `vino` is
