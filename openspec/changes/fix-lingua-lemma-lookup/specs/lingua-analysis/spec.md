## ADDED Requirements

### Requirement: A dictionary form is read as itself
The core SHALL read a dictionary form's gloss, the parts of speech of its senses, the readings of a written form as that dictionary form and the other dictionary forms filed under it from the pack's entry for that very dictionary form, found among the pack's lemmas, and SHALL NOT read them from the entry of the lemma its spelling is a form of.
A string the pack holds as no lemma — a form of another word, as `saw` is of *see* in the (en → fr)
pack, or a word the pack does not hold — SHALL have no gloss, no senses, no reading and no other
dictionary form from the pack, wherever it is asked about: a word card, the page analysis's gloss of
a token, a phrase gloss, or a caller of the engine's gloss. Review shows such a card the gloss the
card stored, as it does any card whose word the pack does not gloss. A token the analysis reads as
such a string, as the English and Spanish plural fallbacks read `buildings` as `building` and
`ablativas` as `ablativa`, is glossed by nothing rather than by another word's gloss.

The frequency rank, the CEFR level and the dictionary-word mark the pack answers for a string SHALL
remain those of the lemma the pack's forms read that string as: they estimate whether the reader
knows a word, and no token's class, no count and no percentage moves with this rule. The rule moves
no analyser version and no pack byte.

#### Scenario: A form asked about as its own dictionary form
- **WHEN** the core is asked about `saw` for the dictionary form `saw`, and the (en → fr) pack holds `saw` only as a form of `see`
- **THEN** it answers no gloss, no sense, no reading and no other dictionary form, where it answered *see*'s gloss grouped by *see*'s parts of speech and the past tense of *see* as a reading of `saw`

#### Scenario: A Spanish plural the pack does not list
- **WHEN** the analysis reads `ablativas` as `ablativa`, which the (es → fr) pack holds only as a form of `ablativo`
- **THEN** the card of `ablativas` names no reading — not *ablativo*'s masculine plural — and no gloss, and the page analysis glosses the token with nothing

#### Scenario: An English plural the pack does not list
- **WHEN** the analysis reads `buildings` as `building`, which the (en → fr) pack holds only as a form of `build`
- **THEN** the token has no gloss and its card no reading, where they showed *build*'s « Construire, édifier » and a present-tense third person singular, and the token's class is the one *build*'s rank gives it, as before

#### Scenario: A dictionary form keeps its own
- **WHEN** the core is asked about `went` for `go`, and about `leaves` for `leave` with `leaf` in the pack
- **THEN** it answers as before: `go`'s gloss with its senses and the past tense, and `leave`'s reading with `leaf` named as another dictionary form

#### Scenario: The gloss of a form
- **WHEN** the engine is asked the gloss of `are`, which the (en → fr) pack holds as a form of `be`
- **THEN** it answers none, and asked the gloss of `be`, it answers *be*'s

#### Scenario: What the invariance baselines show
- **WHEN** the English, Spanish, French, es-en and en-es invariance baselines run after this change
- **THEN** only the probes that ask a form as a dictionary form or the gloss of a form, and the tokens a plural fallback reads as a form, move to no gloss, no sense and no reading — `saw`, `lay`, `thought` and `more` asked as their own dictionary forms, the glosses of `more`, `are` and `has`, and the tokens `lowers`, `findings` and `strangers` in en-fr's and en-es's; the glosses of `cuenta` and `llama` and the token `quebrantos` in es-fr's and es-en's; the glosses of `vis`, `as`, `été` and `est` and `été` asked as its own dictionary form in the French one — and every other probe, every token's lemma and class, every count and percentage and every analyser version is byte for byte as before
