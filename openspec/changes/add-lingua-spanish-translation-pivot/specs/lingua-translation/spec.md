## MODIFIED Requirements

### Requirement: The answer is the reader's sentence with their selection marked
A translation SHALL answer with the whole sentence the selection sits in, and SHALL mark, within that translated sentence, the span corresponding to what the reader selected, in a language whose marks have been measured: English. In another language, the selection SHALL be sent untagged, in a single request, and the answer SHALL be the translated sentence without a mark. Spanish is such a language until its marks are measured (`release-lingua-spanish-translation`).
The selection SHALL be translated in its sentence rather than on its own, so that its form
carries the grammar the context imposes. The marked span SHALL be identifiable in the answer
without the caller re-reading the source text.

#### Scenario: A fragment carries the sentence's grammar
- **WHEN** the reader selects a verb phrase inside a sentence whose subject and tense determine its form
- **THEN** the marked span holds that form, not the form the fragment would take alone

#### Scenario: A fragment whose translation is shorter than itself
- **WHEN** the reader selects several words that correspond to a single word in the target language
- **THEN** the marked span covers that single word

#### Scenario: The whole sentence is available too
- **WHEN** a translation is returned
- **THEN** the caller can show the translated sentence as well as the marked span

#### Scenario: A Spanish sentence
- **WHEN** a reader selects a word in a Spanish sentence
- **THEN** the answer is the whole sentence in French without a mark, from a single request to the engine

## ADDED Requirements

### Requirement: Spanish is translated through English
The catalogue SHALL carry Mozilla's es-en model, pinned like every model, and Spanish's route SHALL be es-en then en-fr. The engine SHALL load the models of a two-model route once, and SHALL translate a sentence through both in one request. For a reader whose accepted languages include Spanish, the setting SHALL state the download of both models.

#### Scenario: A reader of Spanish turns the setting on
- **WHEN** a reader who accepts Spanish opens « Traduction étendue » before ticking it
- **THEN** it says it downloads 52,0 Mo once, and ticking it stores both models

#### Scenario: A reader of English alone
- **WHEN** a reader who accepts English alone opens « Traduction étendue »
- **THEN** it says it downloads 25,8 Mo once, as before

#### Scenario: A Spanish selection
- **WHEN** a reader with both models selects a phrase on a Spanish page
- **THEN** the French sentence comes back from es-en then en-fr, in one request to the engine

#### Scenario: The host misses the es-en model
- **WHEN** the model host does not serve a file of the es-en model
- **THEN** the check run before a submission fails, naming that file
