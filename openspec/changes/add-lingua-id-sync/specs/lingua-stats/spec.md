# lingua-stats — agrégats d'apprentissage et écran de stats

## ADDED Requirements

### Requirement: Agrégats par jour, par langue et par appareil
`StatsService` SHALL stocker des agrégats d'apprentissage à la clé (jour UTC, langue étudiée, appareil) — expositions, mots appris, révisions faites — poussés par upsert idempotent depuis chaque appareil ; aucun évènement fin horodaté ne SHALL être stocké serveur, le grain jour × langue étant le plus fin autorisé.

#### Scenario: Deux appareils actifs le même jour
- **WHEN** l'utilisateur fait 20 révisions sur son Mac et 10 sur son iPhone le même jour
- **THEN** chaque appareil upserte sa propre ligne d'agrégat et la lecture consolidée du jour rapporte 30 révisions

#### Scenario: Upsert rejoué sans double compte
- **WHEN** un appareil re-pousse la ligne d'agrégat d'un jour déjà envoyé
- **THEN** la ligne est remplacée (upsert par clé), jamais additionnée à elle-même

### Requirement: Lecture consolidée multi-appareils
`StatsService` SHALL exposer une lecture des agrégats consolidés — sommés sur les appareils, en séries par jour et par langue, sur une plage de dates demandée — pour l'utilisateur authentifié uniquement (jamais les stats d'un autre compte).

#### Scenario: Série sur trente jours
- **WHEN** un client demande les stats des 30 derniers jours pour l'anglais
- **THEN** la réponse contient au plus une valeur par jour et par mesure, sommée sur tous les appareils du compte

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
