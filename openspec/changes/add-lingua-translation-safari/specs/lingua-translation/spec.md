## MODIFIED Requirements

### Requirement: Extended translation is offered where it has been measured
The setting SHALL be offered on Chromium, on Firefox desktop, on Firefox for Android and on Safari — iPhone, iPad and Mac.
Everywhere it is offered, the setting, its download, its deletion and its states SHALL be those of
Firefox desktop, and the engine SHALL run in the extension's own package: no variant downloads code,
only the model. A platform SHALL join this list on a measurement on its devices, recorded in the
change that adds it.

#### Scenario: Firefox for Android
- **WHEN** a reader opens the settings in Firefox for Android
- **THEN** the extended translation setting is there, off, stating the download size and the memory used, exactly as on Firefox desktop

#### Scenario: Safari on iPhone, iPad and Mac
- **WHEN** a reader opens the extension's settings in Safari on an iPhone, an iPad or a Mac
- **THEN** the extended translation setting is there, off, stating the download size and the memory used, exactly as on Firefox desktop

#### Scenario: The Safari package
- **WHEN** the Safari package is inspected
- **THEN** it carries the pinned engine and the model's manifest, and no model file
