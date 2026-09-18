# lingua-privacy Specification

## Purpose
TBD - created by archiving change add-lingua-privacy-controls. Update Purpose after archive.
## Requirements
### Requirement: A card's page address stays on the device
Lingua SHALL keep the address of the page a card was captured from on the device only: it SHALL NOT be sent to the server, and the server SHALL NOT store one.
- The extension SHALL push every card with an empty `source`.
- Applying a pulled card SHALL keep the local address when the pulled `source` is empty.
- The server SHALL ignore any `source` it receives, return an empty one, and keep no stored address, including those stored before this change.
- `CardOp.source` SHALL remain in the protocol as a deprecated field so installed clients keep working.

#### Scenario: Capturing a word from a page
- **WHEN** a signed-in reader adds a word to the deck from a web page and the extension syncs
- **THEN** the pushed card carries the lemma, surface form, sentence, gloss and review state with an empty source, and the local card still holds the page address

#### Scenario: A card edited on another device
- **WHEN** a card captured on the Mac is reviewed on the iPhone and the Mac then pulls that change
- **THEN** the Mac's card takes the iPhone's review state and keeps its own page address

#### Scenario: An installed extension still sends an address
- **WHEN** an extension built before this change pushes a card with a page address
- **THEN** the server accepts the card, stores no address, and returns the card with an empty source on the next pull

#### Scenario: Addresses stored before the change
- **WHEN** the backend migration of this change runs
- **THEN** no page address previously stored for any card remains in the database

### Requirement: Erasing Lingua data keeps the Cymbra account
A signed-in reader SHALL be able to erase all of their Lingua data from the extension without deleting their Cymbra account.
- **Server:** the erasure SHALL delete the reader's word statuses, declared levels, cards and daily stats, and record an erasure mark with the server time. It SHALL leave the account, its sessions and every other product's data intact.
- **Device:** the erasure SHALL empty the local Lingua store (statuses, deck, level, calibration, exposure counters, local stats) and the sync cursors.
- **Confirmation:** the erasure SHALL require an explicit confirmation stating that it is irreversible, applies to every device, and keeps the Cymbra account and Music.

#### Scenario: Confirmed erasure
- **WHEN** a signed-in reader confirms « Effacer mes données Lingua »
- **THEN** the server holds no Lingua status, level, card or stat for that account, the device's Lingua store is empty, and the reader is still signed in

#### Scenario: The account keeps working elsewhere
- **WHEN** the same reader then opens Cymbra Music
- **THEN** they are signed in to the same account with their Music data unchanged

#### Scenario: Erasure not confirmed
- **WHEN** the reader dismisses the confirmation
- **THEN** nothing is erased on the server or the device

#### Scenario: The server cannot be reached
- **WHEN** the erasure call fails
- **THEN** the device keeps its local store and the reader sees a categorized error, never a raw message

### Requirement: An erasure reaches every device
Every sync SHALL read the account's erasure mark before pushing anything.
- **Device that has not seen the mark:** it SHALL empty its local Lingua store and cursors first, then sync.
- **Server:** it SHALL drop any status, level or card op dated at or before the mark, and any daily stat for a day before the mark's UTC day, while still acknowledging the push.
- **Ops made after a wipe:** they SHALL be dated after the mark, whatever the device clock says.

#### Scenario: Another signed-in device syncs after the erasure
- **WHEN** a second device that still holds the erased deck runs its next sync
- **THEN** it empties its local store before pushing, and nothing erased reappears on the server

#### Scenario: An extension that predates the mark
- **WHEN** an extension built before this change pushes its old statuses, cards and past-day stats after the erasure
- **THEN** the server acknowledges the push and stores none of those old entries

#### Scenario: Learning again after the erasure
- **WHEN** the reader marks a word as known after the erasure, on a device whose clock runs behind the server's
- **THEN** that decision is stored and synced to their other devices

### Requirement: Cymbra account deletion is reachable from Lingua
The extension SHALL offer a signed-in reader a link to delete their whole Cymbra account on the Cymbra site, in the reader's language.
- The link SHALL be introduced by a warning that the account, and the data of every Cymbra app including Music, will be deleted.
- The warning SHALL point to « Effacer mes données Lingua » for removing Lingua alone.

#### Scenario: A French-speaking reader opens their account
- **WHEN** a signed-in reader with a French browser opens the account page
- **THEN** they see « Supprimer mon compte Cymbra » with the warning, and the link opens `https://cymbra.app/suppression-compte/` in a tab

#### Scenario: Another language
- **WHEN** the browser language is not French
- **THEN** the link opens `https://cymbra.app/en/delete-account/`

#### Scenario: Reaching it from the popup
- **WHEN** a signed-in reader chooses « Gérer mes données » in the popup
- **THEN** the account page opens on the section holding the erasure and the deletion link

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French and English) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address.
- **Synced for a signed-in reader:** statuses, level, deck without addresses, daily stats, and a random installation identifier.
- **Removal:** the Lingua-only erasure and account deletion.

The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/` or `https://cymbra.app/en/privacy/`
- **THEN** a Lingua annex lists what is synced, what stays on the device and how to erase it

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking

