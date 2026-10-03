## ADDED Requirements

### Requirement: The models follow the reader's languages
While extended translation is on, the device SHALL keep the models that the routes of the reader's accepted languages need, and no other. Turning the setting on SHALL download every needed model. A model that the accepted languages no longer need SHALL be deleted, and a model several routes share SHALL be kept while any of them needs it. A needed model that was never downloaded SHALL NOT be fetched until the reader asks, and the setting SHALL say what it costs. A model stored before this requirement SHALL stay complete.

#### Scenario: Every reader today
- **WHEN** a reader of English alone ticks « Traduction étendue »
- **THEN** the en-fr model is downloaded, as before, and nothing else

#### Scenario: A language added while the setting is on
- **WHEN** a reader with the setting on and the en-fr model ready adds a language whose route needs a model not on the device
- **THEN** nothing is downloaded, and the setting offers to download the missing model with its size

#### Scenario: A language removed
- **WHEN** a reader of English and Spanish removes Spanish
- **THEN** the models only Spanish needed are deleted, and en-fr stays

#### Scenario: A model stored before the update
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** it is still complete, and English is still translated

### Requirement: A translation is asked in its document's language
A translation request SHALL name the language of the document the sentence was read in. The background SHALL answer it only when every model of that language's route is on the device, and SHALL otherwise answer as without a model. The engine SHALL translate the sentence through that language's route.

#### Scenario: An English page
- **WHEN** a reader with the en-fr model ready selects a phrase on an English page
- **THEN** the request names English, and the translation comes from the en-fr model

#### Scenario: A language whose models are not all there
- **WHEN** a reader selects a phrase on a page in a language whose route has a model that is not on the device
- **THEN** the card answers as it does without a model
