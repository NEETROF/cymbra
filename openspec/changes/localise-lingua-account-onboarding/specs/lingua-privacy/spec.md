## MODIFIED Requirements

### Requirement: Cymbra account deletion is reachable from Lingua
The extension SHALL offer a signed-in reader a link to delete their whole Cymbra account on the Cymbra site, in the reader's interface language: the French page for French, the English page otherwise (until the site has a Spanish page).
- The link SHALL be introduced by a warning that the account, and the data of every Cymbra app including Music, will be deleted.
- The warning SHALL point to « Effacer mes données Lingua » for removing Lingua alone.

#### Scenario: A French-speaking reader opens their account
- **WHEN** a signed-in reader whose interface language is French opens the account page, whatever their browser's language
- **THEN** they see « Supprimer mon compte Cymbra » with the warning, and the link opens `https://cymbra.app/suppression-compte/` in a tab

#### Scenario: Another language
- **WHEN** the interface language is not French
- **THEN** the link opens `https://cymbra.app/en/delete-account/`

#### Scenario: Reaching it from the popup
- **WHEN** a signed-in reader chooses « Gérer mes données » in the popup
- **THEN** the account page opens on the section holding the erasure and the deletion link
