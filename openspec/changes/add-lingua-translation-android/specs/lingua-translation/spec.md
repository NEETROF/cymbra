## MODIFIED Requirements

### Requirement: The engine is loaded only while it is used
The engine SHALL be loaded when a translation is asked, when a reader begins a selection while a model is ready, or when a page that asked for a translation within the idle period becomes visible again; it SHALL stay loaded while reading tabs keep asking for translations, and SHALL be released after an idle period in which none is asked.
Beginning a selection and returning to such a page SHALL count as asking, for the idle period
only: they load the engine early and arm its release, and they translate nothing. Nothing SHALL be
loaded for them when no model is ready. Turning extended translation on SHALL NOT by itself load
the engine, and neither SHALL opening a page. A keep-warm signal from a reading tab SHALL NOT count
as a translation asked.

#### Scenario: Reading stops
- **WHEN** no translation has been asked for the idle period, even with reading tabs still open
- **THEN** the engine is released, and the next translation loads it again

#### Scenario: The setting just turned on
- **WHEN** the model has finished downloading and the reader has asked for nothing yet
- **THEN** the engine is not loaded

#### Scenario: A selection begins
- **WHEN** a model is ready and the reader starts selecting text, before the selection settles
- **THEN** the engine starts loading, and the translation asked when the selection settles uses it

#### Scenario: A selection that needs no translation
- **WHEN** the reader selects a single word the pack glosses, after the engine was loaded for that selection
- **THEN** nothing is translated, and the engine is released once the idle period passes with no translation asked

#### Scenario: Back to a page that was translating
- **WHEN** a page that asked for a translation less than the idle period ago becomes visible again and the engine is no longer loaded
- **THEN** the engine starts loading before the reader selects anything

#### Scenario: A page opened, nothing selected
- **WHEN** the reader opens or returns to a page that has asked for no translation within the idle period
- **THEN** the engine is not loaded

#### Scenario: No model
- **WHEN** extended translation is off, or its model is not ready, and the reader begins a selection
- **THEN** nothing is loaded and nothing is sent to the engine's host

### Requirement: Extended translation is offered where it has been measured
The setting SHALL be offered on Chromium, on Firefox desktop and on Firefox for Android, and SHALL NOT be offered on Safari.
On Firefox for Android the setting, its download, its deletion and its states SHALL be those of
Firefox desktop. Safari's package SHALL carry no engine; offering it there is decided by a change
of its own.

#### Scenario: Firefox for Android
- **WHEN** a reader opens the settings in Firefox for Android
- **THEN** the extended translation setting is there, off, stating the download size and the memory used, exactly as on Firefox desktop

#### Scenario: Safari
- **WHEN** the Safari package is inspected
- **THEN** it carries no engine and its settings offer no extended translation

## ADDED Requirements

### Requirement: A selection already translated on the page is not translated again
A page SHALL keep the translations it has received, and SHALL answer a request identical to one already answered — the same sentence and the same selection within it — from them, without asking the engine.
Only translations SHALL be kept, not the absence of one. The kept answers SHALL belong to the page:
they SHALL NOT be stored, SHALL NOT be shared with another tab, and SHALL go when the page goes. A
different selection in the same sentence SHALL be asked for, since the engine marks the selection
in its answer.

#### Scenario: Handles back on a span already answered
- **WHEN** the reader adjusts the selection handles and comes back to a selection translated a moment ago on the same page
- **THEN** the card shows that translation without the engine running again

#### Scenario: A different span of the same sentence
- **WHEN** the reader selects other words of a sentence already translated on the page
- **THEN** the engine is asked, and the card marks the new selection

#### Scenario: No translation last time
- **WHEN** a request was answered without a translation (no model ready, the engine did not answer)
- **THEN** the same request asks again next time
