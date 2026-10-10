## ADDED Requirements

### Requirement: The Lingua page says nothing of price
The site's Lingua page SHALL say nothing of price, in every site language it is built in. Its card on the choice of languages SHALL say only that the reader chooses their languages in the extension's settings and that each page is read in its own.

#### Scenario: Today's pages
- **WHEN** the site is built with en-fr and es-fr
- **THEN** the languages card of `/lingua/` reads « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. » and that of `/en/lingua/` reads "Choose your languages in Settings: each page is read in its own.", and neither page shows « Plusieurs langues à la fois : gratuit pour l'instant. » or "Several languages at once: free for now."

#### Scenario: The Spanish page
- **WHEN** a pair glossed in Spanish ships and `/es/lingua/` is built
- **THEN** its languages card reads « Elige tus idiomas en los Ajustes: cada página se lee en el suyo. », and the page does not show « Varios idiomas a la vez: gratis por ahora. »
