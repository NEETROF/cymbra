# lingua-stats — agrégats d'apprentissage : écran de stats des clients

## ADDED Requirements

### Requirement: Écran de stats dans l'extension et l'app
L'extension et l'app conteneur SHALL offrir un écran de stats d'apprentissage — mots appris, révisions faites, expositions, par jour et par langue — alimenté par la lecture consolidée quand l'utilisateur est connecté, et par les agrégats locaux de l'appareil sinon ; l'écran SHALL indiquer la portée affichée (tous les appareils ou cet appareil) et que les sessions d'agents (plugin Claude Code, local-only) n'y figurent pas.

#### Scenario: Stats consolidées une fois connecté
- **WHEN** un utilisateur connecté ouvre l'écran de stats de l'extension après avoir révisé sur deux appareils
- **THEN** les totaux affichés couvrent les deux appareils et l'écran indique la portée « tous les appareils »

#### Scenario: Stats locales sans compte
- **WHEN** un utilisateur sans compte ouvre l'écran de stats
- **THEN** les agrégats locaux de l'appareil s'affichent avec la portée « cet appareil », sans aucune requête réseau

### Requirement: Vocabulaire des stats sans jargon
Les écrans et réponses de stats ne SHALL PAS afficher le terme « lemme » : les comptes de lemmes uniques SHALL être libellés « mots différents » et la forme canonique « forme du dictionnaire », conformément à la règle de vocabulaire du produit.

#### Scenario: Libellé des mots appris
- **WHEN** l'écran de stats affiche le compte de mots appris de la semaine
- **THEN** le libellé emploie « mots » ou « mots différents », et le terme « lemme » n'apparaît nulle part
