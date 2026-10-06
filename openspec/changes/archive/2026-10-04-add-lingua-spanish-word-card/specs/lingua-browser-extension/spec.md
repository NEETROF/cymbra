## ADDED Requirements

### Requirement: A Spanish card names its forms as French schools do
A card SHALL name the forms of the language the word was met in. A Spanish form SHALL be named in French school terms — présent and imparfait de l'indicatif, passé simple, futur, conditionnel, présent, imparfait and futur du subjonctif, impératif, infinitif, gérondif, participe passé with its agreement — with its person and number, a form of several persons in one tense naming them once. A noun's, adjective's, determiner's or pronoun's form SHALL name its gender and number. A reading that merely says what the card's dictionary form is SHALL give no line. French articles and elision SHALL apply, before accented vowels too. A card of an English word SHALL read as before.

#### Scenario: A form of two persons
- **WHEN** the reader opens the card of `hablaba`, a Spanish word
- **THEN** the card headed `hablar` says it is the 1re et 3e personnes du singulier de l'imparfait de l'indicatif of `hablar`

#### Scenario: A subjunctive and an imperative
- **WHEN** the reader opens the card of `hable`
- **THEN** the card says, in one statement, that it is the 1re et 3e personnes du singulier du présent du subjonctif and the 3e personne du singulier de l'impératif of `hablar`

#### Scenario: An adjective's agreement
- **WHEN** the reader opens the card of `rápidas`
- **THEN** the card headed `rápido` says it is its féminin pluriel

#### Scenario: The dictionary form itself
- **WHEN** the reader opens the card of `hablar`, or of `casa`
- **THEN** the card says nothing about the form being the infinitive, or the singular

#### Scenario: Elision before an accented vowel
- **WHEN** a line names a form of the Spanish pronoun `él`
- **THEN** it reads « d'él », not « de él »

#### Scenario: English reads as before
- **WHEN** the reader opens the card of `went`
- **THEN** the card says that `went` is the prétérit of `go`, as before
