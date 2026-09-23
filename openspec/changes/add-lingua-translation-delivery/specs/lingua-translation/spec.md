## ADDED Requirements

### Requirement: Extended translation is chosen per device, off by default
The reader SHALL turn extended translation on or off for each device separately, it SHALL be off until they do, and nothing but the reader SHALL change it.
The choice SHALL be stored on the device and SHALL NOT be synchronised. The stored value SHALL name
where the engine runs — today nowhere or on the device — so that another host can be added without
migrating it. The setting SHALL state, before it is turned on, how much it downloads and roughly how
much memory the engine uses while translating.

#### Scenario: A fresh install
- **WHEN** a reader installs the extension and never opens the setting
- **THEN** extended translation is off, no model is downloaded, and no engine runs

#### Scenario: Two devices on one account
- **WHEN** a signed-in reader turns extended translation on on the Mac, and the tablet then synchronises
- **THEN** extended translation is still off on the tablet, and nothing is downloaded there

### Requirement: The model is downloaded only when the reader turns extended translation on
The model SHALL be downloaded only after the reader turns extended translation on, only from Cymbra's static host, and SHALL be used only once every file matches its pinned hash.
The download SHALL show its progress, SHALL be cancellable, and SHALL report a failure in the
reader's language without a raw technical message. A file that does not match its hash SHALL be
discarded and never loaded. A cancelled download SHALL keep nothing and SHALL leave the setting
off. A failed one SHALL leave the setting on, offering to try again, with every surface answering
as it does without a model.

#### Scenario: Turning it on
- **WHEN** the reader turns extended translation on
- **THEN** the model is downloaded with its progress shown, verified, and translation becomes available once it is

#### Scenario: A file that is not the model
- **WHEN** a downloaded file does not match its pinned hash
- **THEN** it is discarded, the engine is not started, and the setting says the download failed and offers to try again

#### Scenario: Cancelling
- **WHEN** the reader cancels a download in progress
- **THEN** no part of the model is kept and extended translation is off

### Requirement: Turning extended translation off deletes the model
Turning extended translation off SHALL stop the engine and delete every stored file of the model from the device.

#### Scenario: Turning it off
- **WHEN** the reader turns extended translation off after the model was downloaded
- **THEN** the engine stops, the model's files are gone from the device's storage, and turning it on again downloads them again

### Requirement: The engine is part of the extension, and only its model is fetched
The engine's code SHALL be packaged inside the extension, and the extension SHALL NOT download, load or execute code from anywhere else.
Only the model — data — SHALL be fetched. A package that offers extended translation SHALL carry
the engine built from the pinned source.

#### Scenario: Inspecting a shipped package
- **WHEN** a shipped Chromium or Firefox package is inspected
- **THEN** it contains the engine, and no request the extension makes fetches JavaScript or WebAssembly

### Requirement: A model the browser removed is not fetched again unasked
When extended translation is on and its model is no longer on the device, the extension SHALL NOT download it again without the reader asking, and the setting SHALL say that the model must be downloaded again.
Until it is, every surface SHALL answer as it does without a model.

#### Scenario: Storage cleared by the browser
- **WHEN** the browser has removed the extension's stored model while extended translation is on
- **THEN** nothing is downloaded, cards answer from the pack, and the setting offers to download the model again

### Requirement: The engine is loaded only while it is used
The engine SHALL be loaded on the first translation asked, SHALL stay loaded while reading tabs keep asking for translations, and SHALL be released after an idle period in which none is asked.
Turning extended translation on SHALL NOT by itself load the engine. A keep-warm signal from a
reading tab SHALL NOT count as a translation asked.

#### Scenario: Reading stops
- **WHEN** no translation has been asked for the idle period, even with reading tabs still open
- **THEN** the engine is released, and the next translation loads it again

#### Scenario: The setting just turned on
- **WHEN** the model has finished downloading and the reader has asked for nothing yet
- **THEN** the engine is not loaded

### Requirement: Extended translation is offered where it has been measured
The setting SHALL be offered on Chromium and on Firefox desktop, and SHALL NOT be offered on Firefox for Android or on Safari.
Firefox for Android, which shares the Firefox package, SHALL hide the setting at run time and SHALL
NOT download a model. Safari's package SHALL carry no engine. Each is decided by a change of its
own.

#### Scenario: Firefox for Android
- **WHEN** a reader opens the settings in Firefox for Android
- **THEN** there is no extended translation setting, and no model can be downloaded

#### Scenario: Safari
- **WHEN** the Safari package is inspected
- **THEN** it carries no engine and its settings offer no extended translation

## MODIFIED Requirements

### Requirement: Without a model the extension behaves as it does today
When no model is ready, every surface SHALL answer exactly as it does without the engine.
No model is ready when extended translation is off, when its model is downloading, when the
download failed, or when the model was removed. The engine SHALL NOT be a condition for any
existing behaviour. On a reading surface its absence SHALL NOT be reported to the reader, as an
error or as a translation on its way; only the setting SHALL say that the model is not ready, and
why.

#### Scenario: No model available
- **WHEN** the reader selects a phrase and no model is present
- **THEN** the card answers from the pack exactly as it did before the engine existed, with no mention of a missing engine

#### Scenario: The model still downloading
- **WHEN** the reader selects a phrase while the model is downloading
- **THEN** the card answers from the pack, with no line saying a translation is on its way

### Requirement: The translation is shown as a machine translation, in the reader's sentence
A card SHALL show a translation as the reader's sentence, labelled as a machine translation, with the selection's place in it marked, and SHALL render every part of it as text.
Beside a translation the card SHALL show no word-by-word rows and no note that the pack has no
translation; an expression's dictionary gloss SHALL remain. A selection of several words SHALL be
translated. A single word SHALL be translated only when the pack has no gloss for it and it is not
a proper noun outside the lexicon, and then in its sentence with the word marked, never alone; a
single word the pack glosses keeps its dictionary card.

#### Scenario: A translated phrase
- **WHEN** the engine translates a selection of several words
- **THEN** the card shows the sentence under a label saying it is a machine translation, with the selection's place marked

#### Scenario: Markup from the page
- **WHEN** the translated sentence contains characters that would read as markup
- **THEN** the card shows them as text, and no element is created from them

#### Scenario: A single word the pack glosses
- **WHEN** the reader selects a single word the pack has a gloss for
- **THEN** the engine is not asked, and the card is the word's dictionary card

#### Scenario: A single word the pack does not know
- **WHEN** the reader selects "disambiguation", which the pack has no gloss for, with a model ready
- **THEN** the card shows the sentence translated with the word's place marked, labelled as a machine translation

#### Scenario: A proper noun outside the lexicon
- **WHEN** the reader selects a single word the analysis classes as a proper noun outside the lexicon
- **THEN** the engine is not asked
