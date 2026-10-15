## ADDED Requirements

### Requirement: The console shows how installations convert and why readers leave
The Lingua console SHALL show, over its window, the anonymous counts of `lingua-usage-counts` grouped by installation week: for each week, the installations and how many of them set up, analysed a first page, came back on day 1, in days 7–13 and in days 30–59, each as a number and as a share of that week's installations, filterable by browser family. It SHALL also show, over the same window, how many readers gave each leaving reason.
- A return figure SHALL read "not yet" until its window has closed for every day of the week and the devices have sent it — the Monday after the week holding the window's last day for the week's last installation day — so a recent week never looks like a poor one.
- A week whose counter reached the ceiling SHALL be marked as capped.
- The screen SHALL state that these are anonymous counts from installations that count — on Chromium, only those whose reader agreed; installations from before the first counting release never — that a reinstall and each device count as one installation, and that the latest week stays incomplete until the following week, since devices send each step the week after it happens.
- No figure SHALL be finer than a week and a browser family.

The figures SHALL come from `LinguaAdminService`, under the console's existing authorisation (admin in the `lingua` scope), and SHALL carry no account, identifier or per-installation detail, like every figure of this console. Strings SHALL be in English and French, through the store's `Async<T>` union.

#### Scenario: A week of installations
- **WHEN** a lingua admin opens the console on a window holding a week with 40 installations, 25 first pages analysed and 8 returns in days 7–13
- **THEN** that week's row shows 40, 25 (63 %) and 8 (20 %) in those columns

#### Scenario: A week too recent
- **WHEN** the window holds the current week
- **THEN** its day-1 return reads "not yet" until the Monday two weeks after the week's Monday, its days 7–13 until three weeks after, its days 30–59 until ten weeks after, and the note says the latest week is incomplete until the following week

#### Scenario: One browser
- **WHEN** the admin filters on Firefox
- **THEN** every figure of the section, leaving reasons included, counts Firefox installations only

#### Scenario: Nothing personal
- **WHEN** the response messages of the funnel and leaving-reason RPCs are inspected
- **THEN** none carries an account, an identifier or a per-installation field, and the privacy allow-list test passes on them
