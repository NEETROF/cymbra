## ADDED Requirements

### Requirement: Six anonymous steps, each reported once per installation
The extension SHALL record, for each installation, six steps and SHALL report each at most once, filed under the local calendar day the installation was made (day 0):
- **Installed:** the browser reported a new installation (not an update).
- **Set up:** the reader made a first level decision for any studied language, « Débutant » included, wherever it was made.
- **First page analysed:** the first page analysed in a studied language.
- **Back on day 1:** a use on day 1.
- **Back in days 7–13:** the first use on a day from 7 to 13.
- **Back in days 30–59:** the first use on a day from 30 to 59.

A use SHALL be a page analysed in a studied language or a Lingua surface the reader opened (popup, side panel, drawer, review, book reader); the welcome tab opening by itself at installation SHALL NOT be a use. After day 59 the device SHALL forget day 0 and report nothing more for that installation. An installation that predates the first release carrying this requirement SHALL report nothing.

#### Scenario: A reader who sets up and reads on day 0
- **WHEN** a reader installs Lingua, chooses B1 on the welcome tab and opens a Spanish page the same day
- **THEN** installed, set up and first page analysed are reported once each, under that day

#### Scenario: A weekend reader
- **WHEN** a reader installed on a Monday uses Lingua again only on day 8
- **THEN** back in days 7–13 is reported once, and back on day 1 is never reported

#### Scenario: A late return
- **WHEN** a reader's only use after day 0 is on day 200
- **THEN** no return is reported, and the device holds no day 0 for that installation any more

#### Scenario: An update, not an installation
- **WHEN** an installation made before the first counting release is updated to it
- **THEN** it reports no step, ever

#### Scenario: A page in no studied language
- **WHEN** a reader with the always-on reader browses only pages in no studied language on day 1
- **THEN** back on day 1 is not reported

### Requirement: A report carries no identifier and nothing of what is read
A count report SHALL carry only day 0, the browser family (`chromium`, `firefox` or `safari`, taken from the build, never from the User-Agent), the extension's version as major.minor, and the steps it reports. It SHALL carry no identifier of any kind — no installation id, no device id, no account id even for a signed-in reader, no hash or derivative of one — and nothing about what the reader reads: no address, site, page text, word, studied language or level. Reports SHALL go through a transport that attaches no account token, to a service mounted with no authentication, so that no report can be tied to an account.

#### Scenario: A signed-in reader
- **WHEN** a signed-in reader's device reports a step
- **THEN** the request carries no `Authorization` header, and the server resolves no account for it

#### Scenario: Inspecting the contract
- **WHEN** the report message of `usage_counts.proto` is reviewed
- **THEN** its fields are day 0, the browser family, the version and the steps, and the privacy allow-list test passes on it

#### Scenario: The sync's installation id
- **WHEN** a signed-in device that holds a sync installation id reports a step
- **THEN** that id appears nowhere in the report

### Requirement: Counting is on by default, and one switch turns it off
The extension SHALL count by default and SHALL offer, in Settings → Data, one switch that turns counting off on this device, with a note in the interface language (French, English or Spanish) saying what is counted, that nothing the reader reads is sent, and linking to the privacy policy.
- **Firefox:** the switch SHALL be Firefox's "technical and interaction data" consent — read from the extension's granted data collection, requested and withdrawn through the permissions API, and followed when the reader changes it in the browser's add-on manager. Where Firefox has no data-collection consent (desktop before 140, Android before 142), the extension SHALL count nothing and SHALL NOT show the switch.
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

### Requirement: The reader is told before any report leaves
The extension SHALL send no report before it has shown the reader, once, one sentence in the interface language saying what is counted, that nothing read leaves the device, and where to turn counting off — on the welcome tab or in the popup's first opening, whichever comes first. On Firefox, the install prompt's data-collection consent SHALL count as that moment. The sentence SHALL NOT be shown as a notification, a badge, or over a page being read.

#### Scenario: Chrome, welcome tab
- **WHEN** a Chrome reader installs Lingua and the welcome tab opens
- **THEN** the tab shows the sentence, and the installed step is reported only after it was shown

#### Scenario: Safari, popup first
- **WHEN** a Safari reader enables Lingua and later opens its popup for the first time
- **THEN** the popup shows the sentence, and no step was reported before

### Requirement: Reports ride the extension's own wake-ups
The extension SHALL try to send every waiting step, in one report, when it records a step, and SHALL otherwise retry waiting steps when it wakes for an installation, a browser start, a page analysis or a surface opening — at most once an hour after a failure. It SHALL use no alarm and no timer to send. A step recorded offline SHALL be sent when the network returns, still filed under day 0. A report the server accepted or refused as malformed SHALL NOT be sent again; only a refusal for too many requests, or a transport failure, SHALL be retried.

#### Scenario: Offline for three days
- **WHEN** a reader sets up offline on day 0 and is online again on day 3
- **THEN** the waiting steps are sent on day 3, filed under day 0

#### Scenario: Several steps at once
- **WHEN** three steps are waiting
- **THEN** they leave in one report

#### Scenario: The server is down
- **WHEN** a report fails with a transport error and the reader opens five pages within the next ten minutes
- **THEN** no further attempt is made before an hour has passed

### Requirement: The server keeps day counters and nothing else
The Lingua backend SHALL add one to a counter keyed by day 0, step, browser family and version for each step of an accepted report, and SHALL keep no row per report and no time of day, network address, header or account. It SHALL accept a report only when its values are bounded: steps among the six, one to six distinct steps, a browser family among the three, a version of the form major.minor, and a day 0 from 120 days before the server's UTC day to one day after it. A malformed report SHALL be refused as invalid; an out-of-range day 0 SHALL be acknowledged and not counted. The service SHALL be mounted only when the Lingua database is configured, and a runtime flag (`lingua.usage_counts.enabled`, on by default) SHALL let an operator stop counting without a release: with it off, reports SHALL be acknowledged and not counted.

#### Scenario: A report counted
- **WHEN** a Firefox 1.10 device reports installed and set up for day 0 = 20 October
- **THEN** two counters for 20 October, Firefox, 1.10 each increase by one, and no other row is written

#### Scenario: A day 0 too old
- **WHEN** a report names a day 0 from 200 days ago
- **THEN** it is acknowledged and nothing is counted

#### Scenario: An unknown step
- **WHEN** a report names a step outside the six
- **THEN** it is refused as invalid and nothing is counted

#### Scenario: The operator stops counting
- **WHEN** an operator sets `lingua.usage_counts.enabled` off
- **THEN** reports are acknowledged and no counter changes, and extensions keep working

#### Scenario: Nothing in the logs
- **WHEN** reports are counted, dropped and refused while tracing output is captured
- **THEN** no network address, `Origin` value or `User-Agent` value appears in it

### Requirement: Floods are refused without keeping addresses
The count service and the leaving-reason route SHALL refuse more than 60 requests an hour from one network address, holding only a keyed hash of the address, in memory, under a key drawn at start-up and never written, and forgetting it when its hour ends. The address SHALL NOT be written to the database, the cache, a log, a metric or a trace. Each counter SHALL stop growing at a ceiling an operator can change at runtime (`lingua.usage_counts.ceiling`, 5 000 by default), and a request past the ceiling SHALL be acknowledged.

#### Scenario: A flood from one address
- **WHEN** one address sends 61 reports within an hour
- **THEN** the 61st is refused as too many requests, and the database and cache hold no trace of the address

#### Scenario: A school behind one address
- **WHEN** 40 installations behind one address each report once within an hour
- **THEN** all 40 are counted

#### Scenario: A counter at its ceiling
- **WHEN** a counter has reached the ceiling and another report for it arrives
- **THEN** the report is acknowledged and the counter does not change

### Requirement: Leaving asks one question, counted the same way
While counting is on, the Chromium and Firefox variants SHALL register as their uninstall address the site's leaving page in the interface language, carrying the browser family and the version as its only parameters, and SHALL register none while counting is off. The Lingua backend SHALL count a leaving reason sent from that page by the UTC day it arrived, the reason, the browser family and the version, under the same flag, ceiling and flood limit as the count reports. A reason SHALL be one of six fixed values, the browser family one of the three or `unknown`, the version major.minor or `unknown`, and no free text SHALL be accepted.

#### Scenario: A French reader removes Lingua from Chrome
- **WHEN** a reader whose interface is in French removes Lingua from Chrome with counting on
- **THEN** Chrome opens `https://cymbra.app/lingua/au-revoir/` with only the browser family and the version as parameters

#### Scenario: Counting off
- **WHEN** a reader who turned counting off removes Lingua
- **THEN** no page opens

#### Scenario: A reason sent
- **WHEN** the leaving page sends the reason `slow` for Firefox 1.10
- **THEN** that day's counter for `slow`, Firefox, 1.10 increases by one

#### Scenario: Free text
- **WHEN** a request to the leaving route carries a reason outside the six
- **THEN** it is refused as invalid and nothing is counted

#### Scenario: Safari
- **WHEN** a reader removes the Safari extension
- **THEN** no leaving page is involved, since Safari ignores the uninstall address
