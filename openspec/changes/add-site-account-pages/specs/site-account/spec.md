## ADDED Requirements

### Requirement: `/account` shows the plan and offers the right management action

The site SHALL serve `/account` (fr/en) behind the sign-in island: the current plan (free /
premium / premium trial with its end), the source, the end or renewal date, the "rights end on
<date>" line when the plan will not renew, the active betas, and a manage action —
the provider portal (URL fetched at request time) for a `web` row, the App Store / Google Play
subscription page for a store row — plus sign-out. When the web channel is open and the account
may purchase, the page MAY offer the web checkout (calling the web plan API).

#### Scenario: Web subscriber manages on the portal

- **WHEN** a web subscriber opens `/account` and activates "manage"
- **THEN** a fresh provider portal URL is opened

#### Scenario: Store subscriber is sent to the store

- **WHEN** an Apple subscriber opens `/account`
- **THEN** the page says the subscription is managed on the App Store and links to its management page; no web purchase is offered

#### Scenario: Trial tester sees the end date

- **WHEN** a premium-trial tester opens `/account`
- **THEN** the trial campaign and its end date are shown with the rights-end wording

### Requirement: The account page is not the app's account management

The page SHALL NOT offer email/password changes, identity linking or account deletion — those
stay in the app; it links to the app for them.

#### Scenario: Scope stays narrow

- **WHEN** the page is inspected
- **THEN** it offers plan status, manage, sign-out and app links only
