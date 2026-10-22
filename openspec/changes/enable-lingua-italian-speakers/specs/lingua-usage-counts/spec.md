## MODIFIED Requirements

### Requirement: Each browser's consent rule decides when counting starts, and one switch turns it off
The extension SHALL start counting as each browser's store requires, and SHALL offer, in Settings → Data, one switch that turns counting off on this device, with a note in the interface language (French, English, Spanish or Italian) saying what is counted, that nothing the reader reads is sent, and linking to the privacy policy.
- **Safari:** counting SHALL be on by default.
- **Firefox:** the switch SHALL be Firefox's "technical and interaction data" consent — read from the extension's granted data collection, requested and withdrawn through the permissions API, and followed when the reader changes it in the browser's add-on manager. Where Firefox has no data-collection consent (desktop before 140, Android before 142), the extension SHALL count nothing and SHALL NOT show the switch.
- **Chromium:** counting SHALL start only once the reader has agreed, by a button that says so, to the sentence that discloses it; until then steps SHALL be recorded and held, never sent. Declining SHALL be the switch turned off.
- **Turned off:** the device SHALL delete the steps it has not sent and its day 0, SHALL send nothing more, and SHALL unregister the leaving page.
- **Turned on again:** the device SHALL report nothing more for this installation.
- **Full reset:** a full reset of the device's Lingua data SHALL also delete day 0 and the unsent steps, and SHALL leave the switch as it was.

#### Scenario: Turning counting off with steps waiting
- **WHEN** a reader whose device is offline with two unsent steps turns the switch off, then goes back online
- **THEN** neither step is ever sent

#### Scenario: Turning it on again
- **WHEN** a reader turns counting off on day 2 and on again on day 3, then uses Lingua on day 8
- **THEN** no step is reported for this installation

#### Scenario: Declined in Firefox's install prompt
- **WHEN** a Firefox 140 reader unticks "technical and interaction data" while installing
- **THEN** no step is ever reported, and Settings → Data shows the switch off

#### Scenario: Changed in the add-on manager
- **WHEN** a Firefox reader withdraws the data-collection consent from the browser's add-on manager
- **THEN** the extension behaves as with the switch turned off

#### Scenario: An older Firefox
- **WHEN** Lingua runs on Firefox 128 for desktop
- **THEN** no step is ever reported and the switch is not shown

#### Scenario: Chrome, agreed on the welcome tab
- **WHEN** a Chrome reader installs Lingua on a Wednesday, chooses a level, then presses the button agreeing to the counts on the welcome tab
- **THEN** installed and set up leave in one report at that press, and none before it

#### Scenario: Chrome, never answered
- **WHEN** a Chrome reader closes the welcome tab and never answers the question in the popup either
- **THEN** no step is ever reported, and the held steps are forgotten at day 60

#### Scenario: Chrome, declined
- **WHEN** a Chrome reader presses the button declining the counts
- **THEN** the held steps and day 0 are deleted, no step is ever reported, and Settings → Data shows the switch off
