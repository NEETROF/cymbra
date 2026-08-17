## ADDED Requirements

### Requirement: `/checkout` hosts the merchant-of-record checkout

The site SHALL serve `/checkout`, a page that loads the merchant-of-record checkout script and
opens the checkout for the transaction id in `_ptxn` (created server-side and already bound to
the account), in sandbox or production per the site configuration. The page SHALL need no
sign-in, SHALL hold no session and no personal data of its own, and SHALL show a localized
message when `_ptxn` is missing.

#### Scenario: Desktop app hands off to the browser

- **WHEN** the app opens `/checkout?_ptxn=<id>` in the browser
- **THEN** the hosted checkout opens for that transaction

#### Scenario: Missing transaction

- **WHEN** `/checkout` is opened without `_ptxn`
- **THEN** a localized message explains that the purchase starts from the app or the account page

### Requirement: `/checkout/done` closes the loop

After a completed checkout the user SHALL land on `/checkout/done`, which tells them to go back
to the app and refresh (or opens `/account` when signed in). No plan state is decided here — the
webhook is the truth.

#### Scenario: Return after payment

- **WHEN** the checkout completes
- **THEN** the done page tells the user to refresh in the app and links to `/account`
