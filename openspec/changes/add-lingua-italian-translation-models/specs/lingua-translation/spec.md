# Spec Delta

## ADDED Requirements

### Requirement: Italian is translated to and from English directly, and to and from French and Spanish through English
The catalogue SHALL carry Mozilla's it-en model, version 2.0, and en-it model, version 2.1, each pinned like every model and verifiable against Mozilla's publications of it. it-en's route SHALL be the it-en model alone and en-it's the en-it model alone; it-fr's route SHALL be it-en then en-fr, it-es's it-en then en-es, fr-it's fr-en then en-it, and es-it's es-en then en-it, each a route of two models the engine loads once and translates through in one request. A file the two Italian models share — their vocabulary decompresses to the same bytes — SHALL be stored once on a device and kept while either model is kept. These routes SHALL be inert until their pair ships: a reader's pairs are the shipped pairs of their native language, so nothing SHALL be downloaded, kept or loaded for a route whose pair no package ships.

#### Scenario: An English-native reader of Italian
- **WHEN** it-en ships and a reader whose native language is English ticks « Traduction étendue »
- **THEN** the it-en model alone is downloaded, and an Italian sentence is translated by it in one step

#### Scenario: A French-native reader of Italian
- **WHEN** it-fr ships and a reader whose native language is French ticks « Traduction étendue »
- **THEN** the it-en and en-fr models are downloaded, and an Italian sentence comes back in French from it-en then en-fr, in one request to the engine

#### Scenario: An Italian-native reader of English and French
- **WHEN** en-it and fr-it ship and a reader whose native language is Italian translates on an English page, then on a French page, then on an English page again
- **THEN** the device and the engine hold the en-it and fr-en models, en-it serving both routes, and nothing is deleted

#### Scenario: Every native language's pairs
- **WHEN** the routes of the pairs glossed in each native language are taken together, Italian's included
- **THEN** they need three models each: en-fr, es-en and it-en for French; es-en, fr-en and it-en for English; en-es, fr-en and it-en for Spanish; en-it, fr-en and es-en for Italian — and the engine still holds two at a time, a route's

#### Scenario: Pinned against Mozilla's publications
- **WHEN** the it-en and en-it entries are written
- **THEN** each file's decompressed sha256 equals the one Mozilla publishes for it-en 2.0 and en-it 2.1, and the model host's assembly keeps every file

#### Scenario: A vocabulary the two models share
- **WHEN** a device holds the it-en model and then needs the en-it model
- **THEN** the vocabulary is not downloaded again, and it is kept while either model is kept

#### Scenario: A route of an Italian pair before it ships
- **WHEN** the catalogue routes the six Italian pairs and no shipped pair studies Italian or is glossed in it
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and no reader fetches the it-en or the en-it model

#### Scenario: The host misses an Italian model
- **WHEN** the model host does not serve a file of the it-en or the en-it model
- **THEN** the check run before a submission fails, naming that file

## MODIFIED Requirements

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr, es-en, en-es, fr-en, it-en and en-it models; the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, the es-en route is the es-en model alone, the en-es route is the en-es model alone, the fr-en route is the fr-en model alone, the fr-es route is fr-en then en-es, the it-en route is the it-en model alone, the en-it route is the en-it model alone, the it-fr route is it-en then en-fr, the it-es route is it-en then en-es, the fr-it route is fr-en then en-it, the es-it route is es-en then en-it; and a reader whose native language is French downloads, stores and loads as before

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
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and the es-en model is held only as es-fr's first model, as before

#### Scenario: A route of a pair studying French
- **WHEN** the catalogue routes fr-en and fr-es and no shipped pair studies French
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and no reader fetches the fr-en model

#### Scenario: A route of an Italian pair
- **WHEN** the catalogue routes it-en, en-it, it-fr, it-es, fr-it and es-it and no shipped pair studies Italian or is glossed in it
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and no reader fetches the it-en or the en-it model
