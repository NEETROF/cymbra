## ADDED Requirements

### Requirement: A Spanish apocope reads as its full word
The es-fr pack SHALL read an adjective's or a determiner's apocope as a form of its full word, and SHALL NOT read the full word as a form of its apocope: `buen` and `bueno` are forms of *bueno*, `gran` of *grande*, `algún` of *alguno*. An adverb or a numeral that the dictionary calls apocopic SHALL stay a word of its own.

#### Scenario: The commonest adjective
- **WHEN** the reader opens the card of `bueno`, or of `buen` in `buen hombre`
- **THEN** the card is keyed by *bueno* and shows its French gloss « Bon »

#### Scenario: A full word the dictionary lists under its apocope
- **WHEN** the reader opens the card of `malo`
- **THEN** the card is keyed by *malo* and shows « Mauvais, méchant », not the noun *mal*

#### Scenario: An adverb of its own
- **WHEN** the reader opens the card of `muy`
- **THEN** the card is keyed by *muy* and shows « Très », not *mucho*
