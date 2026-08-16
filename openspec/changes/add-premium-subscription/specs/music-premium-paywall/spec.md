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
leads to the paywall — replacing the "coming soon" placeholders. Account settings SHALL show the current plan,
its source, its end or renewal date, and a "manage" action that opens the store's subscription
management for store rows or the web provider portal for web rows. No raw provider or technical
string SHALL be shown.

#### Scenario: Quota lock upsells

- **WHEN** a free user hits the daily catalog quota
- **THEN** the lock sheet shows the premium benefit for the catalog and a way to the paywall

#### Scenario: Plan status for a subscriber

- **WHEN** a premium subscriber opens account settings
- **THEN** they see "premium", the source, the renewal date and a manage action opening the right portal

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
