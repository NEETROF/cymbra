## ADDED Requirements

### Requirement: The console shows how installations convert and why readers leave
The Lingua console SHALL show, over its window, the anonymous counts of `lingua-usage-counts` grouped by installation day, latest first: for each day, the installations and how many of them set up, analysed a first page, came back on day 1, in days 7–13 and in days 30–59, each as a number and as a share of that day's installations, filterable by browser family. It SHALL also offer the same figures by installation week, the days summed, and SHALL show, over the same window, how many readers gave each leaving reason.
- A return figure SHALL read "not yet" until its window has closed: until the day after day 1, 13 or 59 counted from the installation day — for a week, from its last day — so a recent day never looks like a poor one.
- A day whose counter reached the ceiling SHALL be marked as capped, and so SHALL a week holding one.
- The screen SHALL state that these are anonymous counts from installations that count — on Chromium, only those whose reader agreed; installations from before the first counting release never — that a reinstall and each device count as one installation, and that devices send each step as it happens, filed by installation day.
- No figure SHALL be finer than a day and a browser family.

The figures SHALL come from `LinguaAdminService`, under the console's existing authorisation (admin in the `lingua` scope), and SHALL carry no account, identifier or per-installation detail, like every figure of this console. Strings SHALL be in English and French, through the store's `Async<T>` union.

#### Scenario: A day of installations
- **WHEN** a lingua admin opens the console on a window holding a day with 40 installations, 25 first pages analysed and 8 returns in days 7–13
- **THEN** that day's row shows 40, 25 (63 %) and 8 (20 %) in those columns, above the rows of the days before it

#### Scenario: By week
- **WHEN** the admin switches to the week view
- **THEN** each week's row sums its days, and is marked capped if one of its days is

#### Scenario: A day too recent
- **WHEN** the window holds today's installations
- **THEN** their day-1 return reads "not yet" until two days after the installation day, their days 7–13 until day 14, their days 30–59 until day 60, and no note says the figures wait for a following week

#### Scenario: One browser
- **WHEN** the admin filters on Firefox
- **THEN** every figure of the section, leaving reasons included, counts Firefox installations only

#### Scenario: Nothing personal
- **WHEN** the response messages of the funnel and leaving-reason RPCs are inspected
- **THEN** none carries an account, an identifier or a per-installation field, and the privacy allow-list test passes on them
