## MODIFIED Requirements

### Requirement: Campaign and code management, member export

The console SHALL let an admin create campaigns (key, name, kind, and `duration_days` for
trials), close enrolment, close a feature campaign, **reopen a closed campaign**, mint N codes
(clear text shown once, downloadable once), revoke codes individually or per campaign, and list
members per campaign with their join date and, for trials, their end date. It SHALL export a
campaign's members (handles and dates) so the group can be announced to, thanked, or targeted
with a store offer.

**Closing a campaign is a PAUSE, not an end.** It records that the campaign is closed and SHALL
NOT touch any membership: reopening therefore restores every member who was not individually
revoked. The per-person gesture already exists and is separate — a revoked membership never
returns, whatever the campaign does — so the global control stays global. The alternative,
deleting or stamping memberships at close time, would destroy the record of who took part,
which is precisely what the member list and the export exist to keep.

Reopening SHALL clear only the campaign's closed state. It SHALL NOT reopen **enrolment**,
which is a separate decision with its own control: a campaign may legitimately be live for its
members and closed to newcomers.

Because reopening restores people, it SHALL say so before it acts: the console SHALL state how
many memberships the reopening will reactivate. Symmetrically, a closed campaign's member list
SHALL show its members as **inactive** rather than listing them as though nothing had happened —
an operator must not have to remember the campaign's state to read its members correctly.

Closing a `premium_trial` campaign SHALL NOT revoke the entitlements it already granted: a trial
given is a right acquired for its term, and it runs to its own end date. Closing therefore ends
a **feature** beta immediately but leaves a **trial**'s premium running, and reopening a trial
campaign restores no entitlement — it only makes the campaign live again. The console SHALL make
this difference visible where the two kinds are managed, rather than leaving an operator to
discover it by surprise.

#### Scenario: Mint and show once

- **WHEN** an admin mints 20 codes for a campaign
- **THEN** the 20 clear-text codes are shown once and are not retrievable afterwards

#### Scenario: Close a feature beta

- **WHEN** an admin closes the `midi-drums` campaign
- **THEN** its members are listed as ended and the campaign disappears from the flags console's beta selector

#### Scenario: Reopen it, and the members come back

- **WHEN** the admin reopens that campaign
- **THEN** every member who was not individually revoked is active again, with no flag edit and
  no re-enrolment, and the campaign reappears in the flags console's beta selector

#### Scenario: A revoked member does not come back

- **WHEN** a member's membership was revoked before the campaign was closed, and the campaign is
  reopened
- **THEN** that member stays out — the per-person revocation outlives the campaign's state

#### Scenario: Reopening announces who it restores

- **WHEN** the admin asks to reopen a campaign holding 12 unrevoked memberships
- **THEN** the console states that 12 memberships will be reactivated before the action is
  confirmed

#### Scenario: A closed campaign's members read as inactive

- **WHEN** an admin lists the members of a closed campaign
- **THEN** each row is shown as inactive, not as a live membership

#### Scenario: Reopening enrolment is a separate act

- **WHEN** a campaign whose enrolment was closed is reopened
- **THEN** its members are active again and new redemptions are still refused, until enrolment
  is reopened in its own right

#### Scenario: A closed trial keeps its granted premium

- **WHEN** a `premium_trial` campaign is closed while a member's trial still has days to run
- **THEN** that member keeps premium until the trial's own end date, and reopening the campaign
  grants no new entitlement

#### Scenario: Member export

- **WHEN** an admin exports a campaign's members
- **THEN** a file with the member handles and dates is produced
