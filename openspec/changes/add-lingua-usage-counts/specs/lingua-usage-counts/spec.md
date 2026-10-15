## ADDED Requirements

### Requirement: Six anonymous steps, each reported once per installation
The extension SHALL record, for each installation, six steps counted from the local calendar day the installation was made (day 0), and SHALL report each at most once, filed under the week of day 0 (the Monday that starts day 0's local week):
- **Installed:** the browser reported a new installation (not an update).
- **Set up:** the reader made a first level decision for any studied language, « Débutant » included, wherever it was made.
- **First page analysed:** the first page analysed in a studied language.
- **Back on day 1:** a use on day 1.
- **Back in days 7–13:** the first use on a day from 7 to 13.
- **Back in days 30–59:** the first use on a day from 30 to 59.

A use SHALL be a page analysed in a studied language or a Lingua surface the reader opened (popup, side panel, drawer, review, book reader); the welcome tab opening by itself at installation SHALL NOT be a use. After day 59 the device SHALL record no step; at the end of the week that follows day 59's week (73 days after day 0 at most) it SHALL forget day 0 and every step it still holds, and report nothing more for that installation. Every change to the device's counts — a wake, the first-run sentence shown, an answer, the switch — SHALL first apply that expiry. An installation that predates the first release carrying this requirement SHALL report nothing, SHALL never be asked about the counts, and SHALL register no leaving page.

#### Scenario: A reader who sets up and reads on day 0
- **WHEN** a reader installs Lingua on Wednesday 21 October, chooses B1 on the welcome tab and opens a Spanish page the same day
- **THEN** installed, set up and first page analysed are reported once each, under the week of Monday 19 October, from Monday 26 October

#### Scenario: A weekend reader
- **WHEN** a reader installed on a Monday uses Lingua again only on day 8
- **THEN** back in days 7–13 is reported once, and back on day 1 is never reported

#### Scenario: A late return
- **WHEN** a reader's only use after day 0 is on day 200
- **THEN** no return is reported, and the device holds no day 0 for that installation any more

#### Scenario: Past the installation's last week
- **WHEN** a Chrome reader installed on a Wednesday never answered, and on day 70 presses « Oui, compter », or a Safari reader sees the sentence for the first time on day 70
- **THEN** no step is reported, and the device holds no day 0 for that installation any more

#### Scenario: An update, not an installation
- **WHEN** an installation made before the first counting release is updated to it, on Chrome, Firefox or Safari
- **THEN** it reports no step, ever, the welcome tab and the popup never show the counts' sentence or question, and on Chrome and Firefox no leaving page is registered, even if the reader turns the switch on

#### Scenario: A page in no studied language
- **WHEN** a reader with the always-on reader browses only pages in no studied language on day 1
- **THEN** back on day 1 is not reported

### Requirement: A report carries no identifier and nothing of what is read
A count report SHALL carry only the week of day 0, the browser family (`chromium`, `firefox` or `safari`, taken from the build, never from the User-Agent), the extension's version as major.minor, and the steps it reports. It SHALL carry no identifier of any kind — no installation id, no device id, no account id even for a signed-in reader, no hash or derivative of one — and nothing about what the reader reads: no address, site, page text, word, studied language or level. Reports SHALL go through a transport that attaches no account token, to a service mounted with no authentication, so that no report can be tied to an account.

#### Scenario: A signed-in reader
- **WHEN** a signed-in reader's device reports a step
- **THEN** the request carries no `Authorization` header, and the server resolves no account for it

#### Scenario: Inspecting the contract
- **WHEN** the report message of `usage_counts.proto` is reviewed
- **THEN** its fields are the week of day 0, the browser family, the version and the steps, and the privacy allow-list test passes on it

#### Scenario: The sync's installation id
- **WHEN** a signed-in device that holds a sync installation id reports a step
- **THEN** that id appears nowhere in the report

### Requirement: Each browser's consent rule decides when counting starts, and one switch turns it off
The extension SHALL start counting as each browser's store requires, and SHALL offer, in Settings → Data, one switch that turns counting off on this device, with a note in the interface language (French, English or Spanish) saying what is counted, that nothing the reader reads is sent, and linking to the privacy policy.
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
- **THEN** installed and set up leave in one report at the first wake on or after the following Monday, and none before that press

#### Scenario: Chrome, never answered
- **WHEN** a Chrome reader closes the welcome tab and never answers the question in the popup either
- **THEN** no step is ever reported, and the held steps are forgotten at the end of the week that follows day 59's week

#### Scenario: Chrome, declined
- **WHEN** a Chrome reader presses the button declining the counts
- **THEN** the held steps and day 0 are deleted, no step is ever reported, and Settings → Data shows the switch off

### Requirement: The reader is told before any report leaves
The extension SHALL send no report before it has shown the reader, once, one sentence in the interface language saying what is counted, that nothing read leaves the device, and where to turn counting off — on the welcome tab or in the popup's first opening, whichever comes first. The sentence SHALL count as shown only once it is actually on screen. On Chromium the sentence SHALL carry the two buttons that agree and decline; on Firefox, the install prompt's data-collection consent SHALL count as that moment, and the sentence SHALL still be shown. The sentence SHALL NOT be shown as a notification, a badge, or over a page being read, and SHALL NOT be shown at all for an installation that predates the first counting release or has nothing left to count.

#### Scenario: Chrome, welcome tab
- **WHEN** a Chrome reader installs Lingua and the welcome tab opens
- **THEN** the tab shows the sentence with its two buttons, and no step is reported before one of them is pressed

#### Scenario: Safari, popup first
- **WHEN** a Safari reader enables Lingua and later opens its popup for the first time
- **THEN** the popup shows the sentence, and no step was reported before

#### Scenario: The popup opens on Réglages
- **WHEN** a Safari reader's first popup opens straight on Réglages › Données, the main view and the sentence in it hidden, and the reader closes it there
- **THEN** the sentence does not count as shown, and no step is reported

### Requirement: Reports ride the extension's own wake-ups
The extension SHALL send a step only at the first wake on or after the Monday, on the device's local calendar, that follows the week in which the step happened — never in the week it happened in — so that the moment a report arrives does not tell the day. At each wake — an installation, a browser start, a page analysis, a surface opening, a surface's message — it SHALL send, in one report, every step whose Monday has come, at most once an hour after a failure. It SHALL use no alarm and no timer to send. A step recorded offline SHALL be sent when the network returns, still filed under the week of day 0. A report the server accepted or refused as malformed SHALL NOT be sent again; only a refusal for too many requests, or a transport failure, SHALL be retried. Delivery SHALL be at least once: a report the server counted but whose outcome the device could not save is sent again.

#### Scenario: Sunday night and Monday morning
- **WHEN** a reader installed on a Wednesday chooses a level on the following Sunday at 23:59, then the browser wakes at 23:59 that Sunday and again at 00:00 on Monday
- **THEN** nothing is sent at 23:59, and installed and set up leave together at 00:00, filed under the week of day 0

#### Scenario: A daylight-saving week
- **WHEN** the week a step happened in is the one the device's clock leaves summer time
- **THEN** the step leaves at the first wake on or after the next local Monday 00:00, as in any other week

#### Scenario: A return waits for the end of its week
- **WHEN** a reader installed on a Monday comes back on day 8, a Tuesday
- **THEN** back in days 7–13 is not sent before day 14, the next Monday

#### Scenario: Offline for days
- **WHEN** a reader sets up offline on a Wednesday (day 0) and is online again only on the Thursday of the following week
- **THEN** the waiting steps are sent that Thursday, filed under the week of day 0

#### Scenario: Several steps at once
- **WHEN** three steps are waiting and their Monday has come
- **THEN** they leave in one report

#### Scenario: The server is down
- **WHEN** a report fails with a transport error and the reader opens five pages within the next ten minutes
- **THEN** no further attempt is made before an hour has passed

#### Scenario: Stopped before saving
- **WHEN** the server counts a report and the browser stops the extension's background before it saves the steps as sent
- **THEN** the same steps are sent again at the next wake

### Requirement: The server keeps week counters and nothing else
The Lingua backend SHALL add one to a counter keyed by the week of day 0, step, browser family and version for each step of an accepted report, and SHALL keep no row per report and no time of day, network address, header or account. It SHALL accept a report only when its values are bounded: steps among the six, one to six distinct steps, a browser family among the three, a version of the form major.minor with no leading zero, and a week that starts on a Monday. A week starting more than 18 weeks before the server's UTC day, or after the day after it, SHALL be acknowledged and not counted; any other malformed report SHALL be refused as invalid. A counter SHALL name a version only when it has the major of the latest released version — a runtime setting, `lingua.usage_counts.latest_version` — and a minor at most three past it; any other well-formed version SHALL be counted under `other`, so the versions a week holds stay bounded. Counters whose week is more than 25 months old SHALL be deleted once a day, whether counting is on or off and whether any report arrives. The service SHALL be mounted only when the Lingua database is configured, and a runtime flag (`lingua.usage_counts.enabled`, on by default) SHALL let an operator stop counting without a release: with it off, reports SHALL be acknowledged and not counted.

#### Scenario: A report counted
- **WHEN** a Firefox 1.10 device reports installed and set up for the week of Monday 19 October
- **THEN** two counters for that week, Firefox, 1.10 each increase by one, and no other row is written

#### Scenario: A week too old
- **WHEN** a report names a week that started 200 days ago
- **THEN** it is acknowledged and nothing is counted

#### Scenario: Not a week
- **WHEN** a report names a day that is not a Monday
- **THEN** it is refused as invalid and nothing is counted

#### Scenario: An unknown step
- **WHEN** a report names a step outside the six
- **THEN** it is refused as invalid and nothing is counted

#### Scenario: The operator stops counting
- **WHEN** an operator sets `lingua.usage_counts.enabled` off
- **THEN** reports are acknowledged and no counter changes, and extensions keep working

#### Scenario: Old counters
- **WHEN** a counter's week is 26 months old, counting has been off for months and no report arrives
- **THEN** it is no longer in the database the next day

#### Scenario: A forged version
- **WHEN** reports name the versions `999.999`, `1.14` and `1.13` while the latest released version is set to `1.10`
- **THEN** the first two are counted under `other` and the third under `1.13`, and a report naming `007.0` is refused as invalid

#### Scenario: Nothing in the server's logs
- **WHEN** reports are counted, dropped and refused while tracing output is captured
- **THEN** no network address, `Origin` value or `User-Agent` value appears in it

### Requirement: Floods are refused without keeping addresses
The count service and the leaving-reason route SHALL refuse more than 60 requests a clock hour from one network address — an IPv6 address counted by its /64 prefix — holding only a keyed hash of it, in the server process's memory, under a key drawn at start-up and never written, forgetting it when the clock hour ends, and holding a bounded number of entries: when the table is full, a request from an address it does not hold SHALL be refused as too many requests. Each request SHALL cost the guard a constant amount of work, whatever the table holds, and a request whose body cannot be read SHALL count against the limit like any other. The address SHALL NOT be written to the database, the cache, a log, a metric or a trace, and the public reverse proxy's logs SHALL NOT carry a client address or request headers either. A week's counter for one step (or reason) and one browser family, every version summed, SHALL stop growing at a ceiling an operator can change at runtime (`lingua.usage_counts.ceiling`, 5 000 by default), and a request past the ceiling SHALL be acknowledged.

#### Scenario: A flood from one address
- **WHEN** one address sends 61 reports within a clock hour
- **THEN** the 61st is refused as too many requests, and the database and cache hold no trace of the address

#### Scenario: One IPv6 network
- **WHEN** 61 reports arrive within a clock hour from 61 addresses of one IPv6 /64 prefix
- **THEN** the 61st is refused as too many requests

#### Scenario: Malformed requests
- **WHEN** one address sends 61 leaving answers whose bodies cannot be read within a clock hour
- **THEN** the first 60 are refused as invalid and the 61st as too many requests

#### Scenario: A full table under a spread-out flood
- **WHEN** the table is full and thousands of requests arrive from addresses it does not hold
- **THEN** each is refused in constant time, without the table being scanned

#### Scenario: A school behind one address
- **WHEN** 40 installations behind one address each report once within an hour
- **THEN** all 40 are counted

#### Scenario: The guard's table is full
- **WHEN** the table holds its maximum number of addresses and a report arrives from a new one
- **THEN** it is refused as too many requests, the device keeps its steps, and it tries again an hour later

#### Scenario: A counter at its ceiling
- **WHEN** a week's Firefox counter for one step, its versions summed, has reached the ceiling and another report for it arrives, from any version
- **THEN** the report is acknowledged, no counter of that week, step and browser changes, and the console marks the week as capped

#### Scenario: The backend fails behind the reverse proxy
- **WHEN** the server is down and a report reaches the reverse proxy, which answers with an error
- **THEN** the reverse proxy's log line for that error holds neither the client's address nor the request's headers

### Requirement: Leaving asks one question, counted the same way
While counting is on, for an installation made by a release that counts, the Chromium and Firefox variants SHALL register as their uninstall address the site's leaving page in the interface language, carrying the browser family and the version as its only parameters, and SHALL register none while counting is off or for an installation that predates the counts. The Lingua backend SHALL count a leaving reason sent from that page — by the page's script, or as a plain HTML form when the page runs without JavaScript — by the UTC week it arrived in, the reason, the browser family and the version, under the same flag, ceiling, version bound, retention and flood limit as the count reports. A reason SHALL be one of six fixed values, the browser family one of the three or `unknown`, the version major.minor or `unknown`, and no free text SHALL be accepted. A plain form SHALL be answered by sending the reader back to the page's thanks, in its language, on the site's own origin.

#### Scenario: A French reader removes Lingua from Chrome
- **WHEN** a reader whose interface is in French removes Lingua from Chrome after agreeing to the counts
- **THEN** Chrome opens `https://cymbra.app/lingua/au-revoir/` with only the browser family and the version as parameters

#### Scenario: Counting off
- **WHEN** a reader who turned counting off, or never agreed to it on Chromium, removes Lingua
- **THEN** no page opens

#### Scenario: Installed before the counts
- **WHEN** a reader who installed Lingua 1.9 on Firefox, updated it to the first counting release and left Firefox's data consent ticked, removes Lingua
- **THEN** no page opens

#### Scenario: A page without JavaScript
- **WHEN** the leaving page runs without JavaScript and the reader taps « Ça ralentissait mon navigateur »
- **THEN** `slow` is counted with `unknown` as browser family and version, and the browser lands on the page's thanks

#### Scenario: A reason sent
- **WHEN** the leaving page sends the reason `slow` for Firefox 1.10
- **THEN** that week's counter for `slow`, Firefox, 1.10 increases by one

#### Scenario: Free text
- **WHEN** a request to the leaving route carries a reason outside the six
- **THEN** it is refused as invalid and nothing is counted

#### Scenario: Safari
- **WHEN** a reader removes the Safari extension
- **THEN** no leaving page is involved, since Safari ignores the uninstall address
