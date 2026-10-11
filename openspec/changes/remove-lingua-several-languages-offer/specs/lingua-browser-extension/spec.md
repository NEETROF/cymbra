## MODIFIED Requirements

### Requirement: The reader chooses the languages they study
The settings SHALL offer the reader the languages the package ships, each ticked when the reader studies it. Ticking SHALL add a language after the others and unticking SHALL remove it, and the last language SHALL NOT be removable. The choice SHALL be saved in the reader's profile, and every surface SHALL follow it. The choice SHALL be hidden when the package ships a single language. When the package ships several, onboarding SHALL offer the same choice before the level. The choice SHALL say nothing of price, in the settings or at onboarding, in any interface language.

#### Scenario: Adding Spanish
- **WHEN** the package ships en-fr and es-fr, and a reader who studies English ticks Spanish
- **THEN** the profile holds English then Spanish, and the settings show a level block for each

#### Scenario: The last language
- **WHEN** a reader studies a single language
- **THEN** its box cannot be unticked

#### Scenario: Every reader today
- **WHEN** the package ships en-fr alone
- **THEN** no choice of languages is shown, in the settings or at onboarding

#### Scenario: Nothing about price
- **WHEN** a reader whose native language is French, offered English and Spanish, opens the choice in the settings or at onboarding, studying one of them or both
- **THEN** the boxes are followed by the note that says in which language each page is read, and by no other line: « Plusieurs langues à la fois : gratuit pour l'instant. » is not shown

#### Scenario: Nothing about price in English or Spanish
- **WHEN** the choice is shown in the English or the Spanish interface, with two languages offered
- **THEN** it says nothing of price either: neither "Several languages at once: free for now." nor « Varios idiomas a la vez: gratis por ahora. » is shown
