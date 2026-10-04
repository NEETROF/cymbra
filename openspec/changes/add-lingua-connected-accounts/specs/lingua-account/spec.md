## ADDED Requirements

### Requirement: Connected accounts reachable from the extension
The extension SHALL let a signed-in reader see and manage the sign-in methods linked to their Cymbra account, following Cymbra ID's `account-linking` rules, on the account page (a tab), reached from Réglages › Données › Compte and from the account page's signed-in view. It SHALL list every linked method — Google, Apple, email and password — with the date it was linked, and the email address of an email-and-password method. A reader who is not signed in SHALL NOT reach it.

#### Scenario: Opened from Réglages
- **WHEN** a signed-in reader chooses « Comptes connectés » in Réglages › Données › Compte, in the popup, the side panel or the in-page drawer
- **THEN** the account page opens on the connected accounts, listing each linked method with the date it was linked

#### Scenario: Signed out
- **WHEN** a reader who is not signed in opens the account page on the connected accounts
- **THEN** the page shows sign-in instead, and no method is listed

#### Scenario: The list cannot be read
- **WHEN** the server cannot be reached while the connected accounts load
- **THEN** the page says so in plain words and offers to try again, without listing anything as linked or missing

### Requirement: Set a password from the extension
The extension SHALL offer « Définir un mot de passe » only when the account has no email-and-password method, and SHALL then take the address and the password, send the emailed code, and ask for that code on the same page while the reader stays signed in. The method SHALL appear only once the code is confirmed; from then on that address and password SHALL sign in to this same account on any browser, including one without Google or Apple. The password SHALL NOT be written to any storage area or log.

#### Scenario: A Google account gains a password
- **WHEN** a reader whose account has only Google submits an address and a valid password, then enters the emailed code
- **THEN** the list shows the email-and-password method with that address, and the reader is still signed in

#### Scenario: Signing in on Firefox for Android afterwards
- **WHEN** that reader then signs in with that address and password on Firefox for Android, which offers no Google
- **THEN** they land on the same Cymbra account — same handle, same deck — and no second account exists

#### Scenario: The code page is left for the mailbox
- **WHEN** the reader switches to their mailbox for the code and comes back
- **THEN** the account page is still on the code step for that address

#### Scenario: The address already has an account
- **WHEN** the address submitted already signs in to a Cymbra account
- **THEN** the page says that this address is already used by an account, sends no code and links nothing

#### Scenario: Already set
- **WHEN** the account already has an email-and-password method
- **THEN** « Définir un mot de passe » is not offered

### Requirement: Link Google or Apple where the browser can
The extension SHALL offer « Lier Google » and « Lier Apple » only for a provider the account does not have yet and that the current browser can run, by the same rule as signing in with it (a configured client id and `identity.launchWebAuthFlow`); it SHALL NOT offer them in Safari. Linking SHALL run that provider's flow and attach the resulting identity to the signed-in account, never sign in to another one. An identity already linked SHALL be listed whatever the browser.

#### Scenario: Linking Google in Chrome
- **WHEN** a reader whose account has only an email and password chooses « Lier Google » in Chrome and completes Google's consent
- **THEN** the list shows Google, and the reader is still signed in to the same account

#### Scenario: The provider's window is closed
- **WHEN** the reader closes Google's or Apple's window without consenting
- **THEN** nothing is linked and no error is shown

#### Scenario: Firefox for Android and Safari
- **WHEN** the connected accounts are shown on Firefox for Android or in Safari
- **THEN** no « Lier Google » or « Lier Apple » is offered, while « Définir un mot de passe » and the methods already linked are

#### Scenario: The provider account belongs to another Cymbra account
- **WHEN** the Google account chosen already signs in to a different Cymbra account
- **THEN** the page says that this Google account is already linked to another Cymbra account, and neither account changes

### Requirement: Remove a method, never the last one
The extension SHALL let the reader remove a linked method after confirming it, and SHALL NOT offer to remove the only method left, saying why instead. A removal the server refuses as the last method SHALL be shown as such. Removing the email-and-password method SHALL free its address, so a password can be set again later.

#### Scenario: Removing Google from an account that has a password
- **WHEN** a reader with Google and an email-and-password method removes Google and confirms
- **THEN** the list shows only the email-and-password method

#### Scenario: The only method
- **WHEN** the account has a single method
- **THEN** it has no « Retirer » action, and the page says that the only sign-in method cannot be removed

#### Scenario: Changing one's mind
- **WHEN** the reader asks to remove a method and then cancels the confirmation
- **THEN** nothing is removed

### Requirement: Linking errors in plain words
The extension SHALL word each linking failure for what it is — a provider already linked to another account, an address already used, the only method, a password too weak, a wrong or expired code, an expired session, the server unreachable — and SHALL NOT show any of them as a wrong password or a raw error. No failure SHALL merge two accounts.

#### Scenario: Expired session while linking
- **WHEN** the session has expired by the time the reader links, removes or sets a password
- **THEN** the page asks the reader to sign in again and changes nothing

#### Scenario: A wrong code
- **WHEN** the reader enters a wrong or expired code for the password
- **THEN** the page says the code is invalid or expired, and the method is not added

### Requirement: Signed-out Compte points at linking
When the reader is signed out, Réglages › Données › Compte SHALL say that an account created with Google or Apple signs in on a browser without them once a password is set from a browser that has them — so that creating an account with the same address is not taken for the way in.

#### Scenario: On the Boox
- **WHEN** a signed-out reader opens Réglages › Données on Firefox for Android
- **THEN** the Compte block, beside the email form, says how to reach a Google or Apple account from there
