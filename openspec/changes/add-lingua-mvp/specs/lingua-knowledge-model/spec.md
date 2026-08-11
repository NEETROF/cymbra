# lingua-knowledge-model — état de connaissance lexicale

## ADDED Requirements

### Requirement: Clé de connaissance par paire langue-lemme
L'état de connaissance SHALL être stocké par clé `(langue étudiée, lemme)` — sans partie du discours. Les expressions multi-mots SHALL être des lemmes à part entière (texte avec espaces). Un token SHALL être considéré connu si **au moins un** de ses lemmes candidats est connu.

#### Scenario: Ambiguïté résolue en faveur de l'apprenant
- **WHEN** le lemme `can` est marqué connu et le token `cans` est analysé
- **THEN** le token compte comme connu

### Requirement: Statuts explicites et statut implicite par calibration
Le modèle SHALL supporter les statuts explicites `learning`, `known`, `ignored` ; l'absence d'entrée signifie « nouveau ». Un lemme sans statut explicite SHALL être implicitement connu si son rang de fréquence est inférieur ou égal au seuil de calibration de l'utilisateur. Chaque statut `known` SHALL porter sa provenance (`manual`, `calibration`, `srs`, `import`).

#### Scenario: Calibration au démarrage
- **WHEN** l'utilisateur règle sa calibration à « je connais les 3 000 mots les plus courants » sans avoir marqué aucun mot
- **THEN** tout lemme de rang ≤ 3 000 est classé connu par l'analyse

#### Scenario: Le statut explicite prime sur la calibration
- **WHEN** un lemme de rang 500 est explicitement marqué `learning`
- **THEN** il est classé « en cours » par l'analyse malgré la calibration

### Requirement: Profil L1/L2
Le profil utilisateur SHALL distinguer `native_language` (langue de confort : gloses, futures traductions) des langues étudiées, et toutes les données dépendantes de la langue (gloses, packs, état de connaissance) SHALL être clées par paire (langue étudiée → langue maternelle). Le MVP SHALL livrer la paire (anglais → français) uniquement, sans que l'ajout d'une paire n'exige de changement de code.

#### Scenario: Gloses dans la langue maternelle
- **WHEN** un utilisateur de langue maternelle `fr` consulte le popup d'un mot anglais
- **THEN** la glose affichée provient du pack (en → fr)

### Requirement: Import LingQ pour le démarrage à froid
Le système SHALL importer un export LingQ (CSV) et marquer `known` (provenance `import`) les lemmes correspondants, après lemmatisation des entrées importées.

#### Scenario: Import d'un export LingQ
- **WHEN** un CSV LingQ contenant `running` est importé
- **THEN** le lemme `run` est marqué connu avec provenance `import`

### Requirement: Compteurs d'exposition
Le modèle SHALL maintenir, par (langue étudiée, lemme), un compteur d'expositions (occurrences rencontrées, source de la dernière rencontre, horodatage). En v1, l'exposition ne SHALL PAS modifier le statut d'un lemme — c'est une donnée d'entrée pour l'inférence future (« connu » déduit du SRS/de l'exposition).

#### Scenario: Ingestion d'une session d'agent
- **WHEN** une session contenant 2 occurrences d'un lemme sans statut est ingérée
- **THEN** le compteur d'exposition du lemme augmente de 2 et son statut reste « nouveau »

### Requirement: Vocabulaire d'interface sans jargon
Les surfaces utilisateur — extension **et** plugin (statusline, `/vocab`, réponses MCP) — ne SHALL PAS afficher le terme « lemme ». Les libellés SHALL utiliser « forme du dictionnaire » (pour la forme canonique) et « mots différents » (pour les comptes de lemmes uniques).

#### Scenario: Popup d'un mot fléchi
- **WHEN** l'utilisateur clique sur `pitfalls`
- **THEN** le popup titre `pitfall` et indique « forme vue : “pitfalls” », sans le mot « lemme »
