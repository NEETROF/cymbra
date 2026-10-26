## ADDED Requirements

### Requirement: The reader chooses the speaking rate of read-aloud
The "Lecture à voix haute" block of Réglages SHALL offer a choice of speaking rate among a short
list of presets — very slow, slow, normal — with normal selected until the
reader chooses otherwise. Every reading SHALL be spoken at the chosen rate: the listen buttons of
the word card and the selection card, and the preview of a voice in Réglages. The rate SHALL be
one choice per device for every studied language, kept as a preference and never as the reader's
data, and SHALL change neither which voice speaks nor the rule that text is read only by an
eligible voice. A change SHALL take effect on the next reading; a reading already under way SHALL
NOT be interrupted or re-timed by it. The choice SHALL be offered wherever the voice choice is, and
absent wherever the voice choice is absent.

#### Scenario: Normal by default
- **WHEN** a reader who has never chosen a speaking rate presses a listen button
- **THEN** the text is spoken at the normal rate, and Réglages shows "normal" selected

#### Scenario: A slower rate applies to the card
- **WHEN** the reader chooses a slower rate in Réglages and then presses a listen button on a card
- **THEN** the selection or its sentence is spoken at that slower rate

#### Scenario: The preview uses the chosen rate
- **WHEN** the reader chooses a rate and then presses the button that lets them hear a voice
- **THEN** the sample sentence is spoken at the chosen rate, so the reader judges it as they will
  hear it on a page

#### Scenario: One rate for every language
- **WHEN** the reader chooses a slower rate while studying one language and then studies another
- **THEN** readings in the second language are spoken at the same slower rate, with no second
  choice to make

#### Scenario: A change does not re-time a reading under way
- **WHEN** a reading is under way and the reader changes the rate in Réglages
- **THEN** that reading finishes at the rate it started with, and the next one uses the new rate

#### Scenario: The choice survives a restart and follows every surface
- **WHEN** the reader chooses a rate in one Réglages (side panel, in-page drawer or toolbar popup)
  and opens another, or restarts the browser
- **THEN** the other Réglages shows the same choice, and readings use it

#### Scenario: No voice, no rate choice
- **WHEN** the browser has no synthesiser, or lists no voice at all, so that Réglages hides the
  voice choice
- **THEN** the speaking-rate choice is hidden with it

#### Scenario: The rate does not open a door for remote voices
- **WHEN** the rate is changed to any value
- **THEN** the set of voices allowed to speak, and what the on-device-only and opt-in rules say,
  are exactly what they were
