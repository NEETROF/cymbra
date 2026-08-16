## ADDED Requirements

### Requirement: Channel-aware purchase entry per platform

The app SHALL offer at most one purchase channel per platform, decided server-side and delivered
in the plan snapshot: the App Store flow on iOS and macOS App Store builds, the Google Play flow
on Android, and the web checkout (opened in the browser) on Linux and Windows. Store builds MUST
NOT show links, prices or copy that steer to an external purchase, MUST NOT offer a code entry,
and MUST NOT mention discounts outside the store's own offer presentation. Desktop builds MAY
deep-link the web checkout and the web redeem page.

#### Scenario: iOS shows only the App Store flow

- **WHEN** a free user opens the paywall on the iOS build
- **THEN** the only purchase action starts the App Store purchase; no web link and no code field is shown

#### Scenario: Windows opens the browser

- **WHEN** a free user activates "subscribe" on the Windows build
- **THEN** the hosted web checkout opens in the browser and the app waits with a refresh action

#### Scenario: Purchase hidden while managed elsewhere

- **WHEN** a user already premium via another channel opens the paywall
- **THEN** no purchase action is shown and the surface says where the subscription is managed

### Requirement: Locked surfaces upsell in place, plan status lives in account settings

The app SHALL show, on every locked surface (daily catalog quota reached, locked SoundFont,
private-library quota reached), a localized upsell that names what premium unlocks there and
leads to the paywall — replacing the "coming soon" placeholders. Account settings SHALL show the current plan
(`free` / `premium`, with **essai** marked for a trial), its source, its end or renewal date, the
active beta memberships by campaign, and — whenever the plan will end without renewal (trial,
cancelled subscription, comp) — an explicit **"rights end on <date>"** line naming what will be
withdrawn then (offline library, premium SoundFonts), plus a "manage" action that opens the
store's subscription management for store rows or the web provider portal for web rows. No raw
provider or technical string SHALL be shown.

#### Scenario: Quota lock upsells

- **WHEN** a free user hits the daily catalog quota
- **THEN** the lock sheet shows the premium benefit for the catalog and a way to the paywall

#### Scenario: Plan status for a subscriber

- **WHEN** a premium subscriber opens account settings
- **THEN** they see "premium", the source, the renewal date and a manage action opening the right portal

#### Scenario: Rights-end date when not renewing

- **WHEN** a trial tester, or a subscriber who cancelled, opens account settings
- **THEN** they see "rights end on <date>" and that the offline library and premium SoundFonts will be withdrawn then

#### Scenario: Provider errors are localized

- **WHEN** a purchase or restore fails
- **THEN** the user sees a localized message and the cause is logged, never a raw store or gRPC string

### Requirement: Trial testers see their trial end and can still subscribe; betas are listed

A user whose premium comes from a **premium trial** SHALL see "premium trial until <date> —
subscribe to keep it" in the plan status, and SHALL be able to purchase premium at any time on a
platform with a purchase channel; the paywall MUST NOT be hidden for trial testers. Once a trial
tester holds a paid row, the trial mention disappears from purchase surfaces. Account settings
SHALL list the user's **active beta memberships** (campaign name, kind, joined date) so a tester
knows which unfinished features they are seeing; a feature-beta member on the free plan sees the
normal free paywall.

#### Scenario: Trial status is explicit

- **WHEN** a trial tester opens account settings
- **THEN** they see premium, the trial campaign, its end date and a subscribe action

#### Scenario: Trial tester can subscribe

- **WHEN** a trial tester opens the paywall on a platform with a purchase channel
- **THEN** the purchase action is available exactly as for a free user

#### Scenario: Betas are listed

- **WHEN** a member of the `midi-drums` feature beta opens account settings
- **THEN** the beta appears in a "betas" section with its name and join date, whatever their plan

### Requirement: Restore purchases and refresh are always reachable

Store builds SHALL offer "restore purchases" on the paywall and in the plan status; every build
SHALL refresh the plan on app resume, after a purchase or restore, and on explicit user request.
A refresh failure SHALL keep the last-known plan rather than degrading the UI to free.

#### Scenario: Restore on a new device

- **WHEN** a subscriber signs in on a new device and taps restore
- **THEN** the store transactions are re-asserted and the plan shows premium

#### Scenario: Offline keeps last-known plan

- **WHEN** the plan refresh fails because the device is offline
- **THEN** the UI keeps the last-known plan and shows no error

### Requirement: Plan-only downloads are purged at the next connection after the plan lapses

At the first connection after the effective plan has dropped to `free` (past grace), the app SHALL
delete its local copies of **plan-only** downloads: catalog SoundFonts it no longer owns
(neither free, nor points-redeemed, nor imported) and the offline cache of **catalog** scores;
cached copies of the user's own uploads SHALL be kept. The purge SHALL be driven by the server's
answer (plan snapshot and rotated cache secret), not by the device clock alone; as a local belt,
the app SHALL stop opening plan-only content once `ends_at + grace` from the last known snapshot
has passed while offline. The purge SHALL be announced with a localized notice — never silent —
and the user SHALL have been told the date in advance in the plan status. If a paid row is
active, nothing is purged.

#### Scenario: Purge on reconnect after a trial

- **WHEN** a former trial tester opens the app online after their trial ended
- **THEN** premium SoundFont files and cached catalog scores are deleted, own uploads' cache is kept, and a localized notice explains it

#### Scenario: Nothing purged while premium

- **WHEN** a trial tester who purchased premium reconnects after the trial end
- **THEN** no local content is deleted

#### Scenario: Offline device past the end

- **WHEN** a device stays offline past `ends_at + grace` of its last known snapshot
- **THEN** the app stops opening plan-only content until it reconnects

#### Scenario: Purge is not silent

- **WHEN** the purge runs
- **THEN** the user sees a localized message and no raw technical string
