## RENAMED Requirements

- FROM: `### Requirement: Translation models are listed in one catalogue, with a route per studied language`
  TO: `### Requirement: Translation models are listed in one catalogue, with a route per pair`

- FROM: `### Requirement: The models follow the reader's languages`
  TO: `### Requirement: The models follow the reader's pairs`

- FROM: `### Requirement: A language's marks are measured before they are shown`
  TO: `### Requirement: A pair's marks are measured before they are shown`

## ADDED Requirements

### Requirement: A translation goes through the reader's pair
The background SHALL form the pair of a translation from the language the page asked in — the document's — and the reader's native language, read from their stored profile, and SHALL ask the engine for that pair's route; a page SHALL keep asking in the document's language and SHALL NOT name a pair. A pair the catalogue lists no route for SHALL be unavailable, and the engine SHALL NOT be started for it. The ready pairs recorded on the device SHALL be read by pair where the background gates, and by studied language where a page gates: a page may ask in a language that a ready pair studies.

#### Scenario: Every reader today
- **WHEN** a reader whose native language is French selects words on an English page
- **THEN** the background asks for en-fr's route, and the sentence is translated as before

#### Scenario: The same page for another native language
- **WHEN** a reader whose native language is Spanish selects words on an English page, and en-es ships with its route
- **THEN** the background asks for en-es's route, and nothing of en-fr is loaded

#### Scenario: A pair without a route
- **WHEN** a reader whose native language is Spanish selects words on an English page, and no en-es ships
- **THEN** the answer is unavailable, and the engine is not started

#### Scenario: A state recorded before pairs
- **WHEN** the extension updates on a device whose stored state names the languages `en` and `es` as ready
- **THEN** en-fr and es-fr are ready, English and Spanish pages are translated as before, and the next reconciliation records the pairs

## MODIFIED Requirements

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr and es-en models, the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, and the setting downloads, stores and loads them as before

#### Scenario: A route that does not reach French
- **WHEN** a catalogue's es-fr route ends in English
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not reach its native language
- **WHEN** a catalogue's en-es route ends in French
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not start from its studied language
- **WHEN** a catalogue's en-es route starts with a model that translates from Spanish
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route whose key is no pair
- **WHEN** a catalogue keys a route `en` or `en-`
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A model stored before the catalogue
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** the model is still complete, and nothing is downloaded again

#### Scenario: The host misses a model
- **WHEN** the model host does not serve a file of one of the catalogue's models
- **THEN** the check run before a submission fails, naming that model's file

### Requirement: The models follow the reader's pairs
While extended translation is on, the device SHALL keep the models that the routes of the reader's pairs need, and no other; the reader's pairs are the shipped pairs of their native language that study each of their accepted languages. Turning the setting on SHALL download every needed model. A model that the reader's pairs no longer need SHALL be deleted, and a model several routes share SHALL be kept while any of them needs it. A needed model that was never downloaded SHALL NOT be fetched until the reader asks, and the setting SHALL say what it costs. A model stored before this requirement SHALL stay complete.

#### Scenario: Every reader today
- **WHEN** a reader of English alone, whose native language is French, ticks « Traduction étendue »
- **THEN** the en-fr model is downloaded, as before, and nothing else

#### Scenario: A language added while the setting is on
- **WHEN** a reader with the setting on and the en-fr model ready adds a language whose pair's route needs a model not on the device
- **THEN** nothing is downloaded, and the setting offers to download the missing model with its size

#### Scenario: A language removed
- **WHEN** a reader of English and Spanish, whose native language is French, removes Spanish
- **THEN** the models only es-fr needed are deleted, and en-fr stays

#### Scenario: The native language changes
- **WHEN** a reader of English whose native language was French now has Spanish as their native language, and en-es ships with its route
- **THEN** the models en-es's route needs are the needed ones, the ones only en-fr needed are deleted, and nothing is fetched until the reader asks

#### Scenario: A model stored before the update
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** it is still complete, and English is still translated

### Requirement: A pair's marks are measured before they are shown
A pair SHALL have its selection marked only once its marks, measured on the committed corpus of its studied language's selections with the extension's own marking through that pair's route, reach the programme's first tier: at least 90 % of the shown marks correct, and at most 25 % of the selections without a mark. The pairs that reached it SHALL be listed by name in the extension, and a pair outside the list SHALL be translated without a mark. The corpus, the rule that chose its selections, the harness and every judgment SHALL be committed, filed by pair, so that the measurement can be run again and checked.

#### Scenario: Measuring again
- **WHEN** the harness runs for a pair on the committed corpus of its studied language with the catalogue's models
- **THEN** it produces each selection's translated sentence and mark, as the extension would through that pair's route, and the judged results give that pair's share of correct and withheld marks

#### Scenario: The shipped pairs today
- **WHEN** a reader whose native language is French selects words on an English or a Spanish page
- **THEN** the selection is marked in the translated sentence, as en-fr's and es-fr's measurements allow

#### Scenario: Spanish below the first tier
- **WHEN** es-fr's measured marks fall short of 90 % correct or exceed 25 % withheld
- **THEN** its sentences stay translated without a mark

#### Scenario: Spanish on the first tier
- **WHEN** es-fr's measured marks reach 90 % correct with at most 25 % withheld
- **THEN** its selection is marked in the translated sentence, as en-fr's is

#### Scenario: A pair measured in another native language
- **WHEN** es-en is listed in the catalogue and its marks are not yet measured
- **THEN** es-en is translated without a mark, although es-fr's marks are measured

### Requirement: A translation is asked in its document's language
A translation request SHALL name the language of the document the sentence was read in. The background SHALL form the reader's pair from that language and the reader's native language, SHALL answer the request only when every model of that pair's route is on the device, and SHALL otherwise answer as without a model. The engine SHALL translate the sentence through that pair's route.

#### Scenario: An English page
- **WHEN** a reader of French with the en-fr model ready selects a phrase on an English page
- **THEN** the request names English, and the translation comes through en-fr's route

#### Scenario: A language whose models are not all there
- **WHEN** a reader selects a phrase on a page in a language whose pair's route has a model that is not on the device
- **THEN** the card answers as it does without a model

### Requirement: The answer is the reader's sentence with their selection marked
A translation SHALL answer with the whole sentence the selection sits in, and SHALL mark, within that translated sentence, the span corresponding to what the reader selected, for a pair whose marks have been measured: en-fr, and es-fr, whose marks through English `release-lingua-spanish-translation` measured on the programme's first tier. For a pair whose marks are not measured, the selection SHALL be sent untagged, in a single request, and the answer SHALL be the translated sentence without a mark.
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
- **WHEN** a reader of French selects a word in a Spanish sentence
- **THEN** the answer is the whole sentence in French, through English, the selection marked

#### Scenario: A language whose marks are not measured
- **WHEN** a translation is asked for a pair whose marks were not measured
- **THEN** the answer is the whole sentence in the reader's native language without a mark, from a single request to the engine

### Requirement: Spanish is translated through English
The catalogue SHALL carry Mozilla's es-en model, pinned like every model, and es-fr's route SHALL be es-en then en-fr. The engine SHALL load the models of a two-model route once, and SHALL translate a sentence through both in one request. For a reader of French whose accepted languages include Spanish, the setting SHALL state the download of both models.

#### Scenario: A reader of Spanish turns the setting on
- **WHEN** a reader of French who accepts Spanish opens « Traduction étendue » before ticking it
- **THEN** it says it downloads 52,0 Mo once, and ticking it stores both models

#### Scenario: A reader of English alone
- **WHEN** a reader of French who accepts English alone opens « Traduction étendue »
- **THEN** it says it downloads 25,8 Mo once, as before

#### Scenario: A Spanish selection
- **WHEN** a reader of French with both models selects a phrase on a Spanish page
- **THEN** the French sentence comes back from es-en then en-fr, in one request to the engine

#### Scenario: The host misses the es-en model
- **WHEN** the model host does not serve a file of the es-en model
- **THEN** the check run before a submission fails, naming that file
