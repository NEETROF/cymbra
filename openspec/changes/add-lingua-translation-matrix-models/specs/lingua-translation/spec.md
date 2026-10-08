## ADDED Requirements

### Requirement: English and Spanish are translated into each other directly
The catalogue SHALL carry Mozilla's en-es model, version 2.1, pinned like every model and verifiable against Mozilla's publications of it. es-en's route SHALL be the es-en model alone and en-es's route the en-es model alone, with no pivot. A route SHALL be needed only by a reader whose pairs include its pair, so that a route of a pair not shipped downloads, keeps and loads nothing.

#### Scenario: A Spanish-native reader of English
- **WHEN** en-es ships and a reader whose native language is Spanish ticks « Traduction étendue »
- **THEN** the en-es model alone is downloaded, and an English sentence is translated by it in one step

#### Scenario: An English-native reader of Spanish
- **WHEN** es-en ships and a reader whose native language is English ticks « Traduction étendue »
- **THEN** the es-en model alone is downloaded, and nothing of en-fr is fetched

#### Scenario: Pinned against Mozilla's publications
- **WHEN** the en-es entry is written
- **THEN** each file's decompressed sha256 equals the one Mozilla publishes for en-es 2.1, and the model host's assembly keeps every file

## MODIFIED Requirements

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr, es-en and en-es models, the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, the es-en route is the es-en model alone, the en-es route is the en-es model alone, and a reader whose native language is French downloads, stores and loads as before

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

#### Scenario: A route of a pair not shipped
- **WHEN** the catalogue routes es-en and en-es and the shipped pairs are en-fr and es-fr
- **THEN** no reader's pairs need those routes, and nothing of them is downloaded, kept or loaded
