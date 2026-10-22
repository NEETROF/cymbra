## ADDED Requirements

### Requirement: A translation line can be shown under the caption
While the caption mode is on, the extension SHALL offer a line under its caption line that shows the translation of the caption on screen, hidden until the reader shows it, and SHALL keep that choice per device (French interface: « Afficher la traduction des sous-titres »).
The choice SHALL be offered by a control beside the caption line and in Réglages, and SHALL hold on
every video until the reader changes it on that device. The translation line SHALL follow the
caption line through play, pause, seek and a change of playback rate, in the default, theatre and
full-screen layouts, and SHALL be hidden while an advertisement plays.

#### Scenario: A first video
- **WHEN** a reader who never showed the translation line starts the caption mode on a video
- **THEN** the caption line is shown without a translation line, and the control to show it is there

#### Scenario: The choice is kept on this device
- **WHEN** the reader shows the translation line on one video and later starts the caption mode on another
- **THEN** the translation line is shown on the other video too

#### Scenario: Another device
- **WHEN** the reader showed the translation line on one device and starts the caption mode on another
- **THEN** the translation line is hidden on the other device

### Requirement: The translation line comes from the engine when there is one, from YouTube otherwise
The translation line SHALL show the translation engine's translation of the caption sentence when the reader has an engine ready for English, and otherwise SHALL show only YouTube's own automatic translation of the caption.
With an engine, a sentence's translation SHALL be computed ahead of its line being shown — a few
sentences ahead of the playhead, one at a time, never the whole track at once — and a translation
the reader asks for on a card SHALL wait at most for the one sentence already being translated,
never for the sentences ahead. A model still downloading SHALL count as no engine until it is
ready. Without an engine, the translation SHALL be YouTube's automatic translation of the track
into the language of the reader's glosses, fetched once per video when the line is first shown,
shown whole-line under a label in the interface language (French interface: « Traduction
automatique YouTube »). It SHALL NOT be presented as aligned word by word, SHALL NOT mark a word,
SHALL NOT be mixed with an engine's answer, and SHALL NOT be offered outside YouTube. For a track
YouTube does not translate, or whose translation YouTube does not return, with no engine, the line
SHALL say that no translation is available
(French interface: « Pas de traduction pour ces sous-titres »). In degraded mode, YouTube's
translation SHALL NOT be shown; an engine SHALL translate the line on screen.

#### Scenario: No engine
- **WHEN** a reader with no engine shows the translation line on a video
- **THEN** the line shows YouTube's automatic translation, labelled « Traduction automatique YouTube », and no model is downloaded

#### Scenario: No engine, for a Spanish-speaking reader
- **WHEN** a reader whose native language is Spanish, with no engine, shows the translation line on a video with English captions
- **THEN** the line shows YouTube's automatic translation into Spanish, labelled as YouTube's automatic translation in the reader's interface language

#### Scenario: No engine, for an Italian-speaking reader
- **WHEN** a reader whose native language is Italian, with no engine, shows the translation line on a video with English captions
- **THEN** the line shows YouTube's automatic translation into Italian, labelled as YouTube's automatic translation in the reader's interface language

#### Scenario: A track YouTube does not translate
- **WHEN** a reader with no engine, outside degraded mode, shows the translation line on a video whose track YouTube does not translate, or whose translation request YouTube does not answer
- **THEN** the line says that no translation is available for these captions, in the reader's interface language (French interface: « Pas de traduction pour ces sous-titres »), and no model is downloaded

#### Scenario: An engine ready
- **WHEN** a reader whose engine is ready for English shows the translation line and plays the video
- **THEN** each line shows the engine's translation of its sentence as the line appears, and no request goes to YouTube for a translation

#### Scenario: A card opened while sentences are translated ahead
- **WHEN** the reader selects several words on the caption line while the line's next sentences are being translated ahead
- **THEN** the card's translation waits at most for the one sentence already being translated, and the sentences ahead wait for the card

#### Scenario: A model still downloading
- **WHEN** the reader's model is still downloading and the reader shows the translation line
- **THEN** the line shows YouTube's automatic translation until the model is ready, then the engine's

#### Scenario: Degraded mode with no engine
- **WHEN** the caption mode is degraded and the reader, with no engine, shows the translation line
- **THEN** no translation from YouTube is fetched or shown

### Requirement: YouTube's translation is never kept
YouTube's automatic translation SHALL be held for display only, for the video being watched, and SHALL NOT be stored on a card, used as a card's gloss or translation, synchronised, or sent anywhere.
A card captured while the translation line is shown SHALL keep exactly what a card captured on a
page keeps. A translation the engine computed for the line SHALL follow the rules of every engine
translation.

#### Scenario: A card from a caption whose YouTube translation was shown
- **WHEN** the reader adds a word to the deck from a caption line whose YouTube translation is shown
- **THEN** the card holds the caption sentence and what a card captured on a page holds, and nothing of YouTube's translation
