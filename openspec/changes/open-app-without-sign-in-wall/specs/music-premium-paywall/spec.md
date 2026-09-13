## ADDED Requirements

### Requirement: Guests see the offer and are invited to sign in to subscribe

The paywall SHALL, for a guest with no account session, show what premium includes, state that
premium is attached to the Cymbra account — which is what makes it available on every device the
user signs in on — and offer a sign-in action that names subscribing as its benefit. For a guest
the paywall MUST NOT show a purchase action, a price or a restore action, and MUST NOT call the
plan service or the store. Declining the invitation SHALL leave the guest on the paywall.
Completing sign-in SHALL return the user to the paywall, where the plan is read for the new
session and the purchase actions for their platform become available.

#### Scenario: Guest opens the paywall
- **WHEN** a guest opens the paywall from the account menu or from a locked surface
- **THEN** they see what premium includes, that it is attached to a Cymbra account, and a sign-in action, with no purchase action, price or restore action

#### Scenario: No backend or store call for a guest
- **WHEN** a guest views the paywall
- **THEN** neither the plan service nor the store client is called

#### Scenario: Guest signs in to subscribe
- **WHEN** a guest accepts the invitation and completes sign-in
- **THEN** they are returned to the paywall and the purchase actions for their platform are shown

#### Scenario: Guest declines the invitation
- **WHEN** a guest dismisses the sign-in invitation from the paywall
- **THEN** they stay on the paywall as a guest

## MODIFIED Requirements

### Requirement: Restore purchases and refresh are always reachable

Store builds SHALL offer "restore purchases" on the paywall and in the plan status to a user with
an account session; a guest is invited to sign in first, since a restore re-asserts purchases on
an account. Every build SHALL refresh the plan on app resume, after a purchase or restore, and on
explicit user request. A refresh failure SHALL keep the last-known plan rather than degrading the
UI to free.

#### Scenario: Restore on a new device

- **WHEN** a subscriber signs in on a new device and taps restore
- **THEN** the store transactions are re-asserted and the plan shows premium

#### Scenario: Restore is not offered to a guest

- **WHEN** a guest opens the paywall on a store build
- **THEN** no restore action is shown, and the sign-in invitation is offered instead

#### Scenario: Offline keeps last-known plan

- **WHEN** the plan refresh fails because the device is offline
- **THEN** the UI keeps the last-known plan and shows no error
