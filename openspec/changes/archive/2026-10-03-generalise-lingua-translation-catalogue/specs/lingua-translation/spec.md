## ADDED Requirements

### Requirement: Translation models are listed in one catalogue, with a route per studied language
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each studied language the route of models that translates it into French, in order. A route SHALL name only models of the catalogue, SHALL start from its language, SHALL chain each model's target to the next model's source, and SHALL end in French; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr model, the English route is that model alone, and the setting downloads, stores and loads it as before

#### Scenario: A route that does not reach French
- **WHEN** a catalogue's route for a language ends in English
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A model stored before the catalogue
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** the model is still complete, and nothing is downloaded again

#### Scenario: The host misses a model
- **WHEN** the model host does not serve a file of one of the catalogue's models
- **THEN** the check run before a submission fails, naming that model's file

### Requirement: The setting's cost comes from the catalogue
The setting SHALL state the download it costs and the room the model takes on the device as the sums, over the files of the models it downloads, of their sizes as served and of their decompressed sizes, read from the catalogue. Without those sums, the setting SHALL state its cost without a size.

#### Scenario: Every reader today
- **WHEN** a reader opens « Traduction étendue » before ticking it
- **THEN** it says it downloads 25,8 Mo once, and a storage failure says the model takes 36,7 Mo
