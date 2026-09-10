# lingua-apple-app — app conteneur Safari (iOS + macOS)

## ADDED Requirements

### Requirement: Une app avec des fonctions propres, une fiche universelle
L'app conteneur SHALL être une app à part entière (decks, révision FSRS, réglages, import LingQ) et non une coquille pour l'extension, et SHALL être publiée sous **une seule fiche App Store universelle couvrant iOS et macOS** (universal purchase, un seul projet Xcode). Elle SHALL embarquer les packs de données dans son bundle — jamais dans le storage de l'extension.

#### Scenario: L'app vit sans l'extension
- **WHEN** l'utilisateur ouvre l'app sans avoir activé l'extension Safari
- **THEN** il peut consulter ses decks, réviser ses cartes dues et lancer la calibration ou l'import LingQ

### Requirement: Activation guidée de l'extension
L'app SHALL guider l'activation de l'extension Safari, qui arrive désactivée par défaut : sur iOS, un walkthrough pas-à-pas (l'OS n'offre ni invite ni API d'état) avec détection d'activation par **heartbeat App Group** (l'extension écrit un battement à chaque exécution, l'app le lit) ; sur macOS, un bouton ouvrant directement le panneau Extensions de Safari (`SFSafariApplication.showPreferencesForExtension`) et l'état réel via `SFSafariExtensionManager`. L'écran d'accueil SHALL refléter l'état d'activation.

#### Scenario: Premier lancement iOS
- **WHEN** l'utilisateur ouvre l'app pour la première fois sur iOS sans extension active
- **THEN** le walkthrough d'activation s'affiche en premier, et disparaît de lui-même une fois le heartbeat de l'extension observé

#### Scenario: Activation macOS en deux clics
- **WHEN** l'utilisateur clique « Activer dans Safari » sur macOS
- **THEN** le panneau Extensions de Safari s'ouvre à la bonne entrée, et l'app affiche « extension active » dès que l'API d'état le confirme

### Requirement: Analyse native partagée avec l'extension
L'extension Safari SHALL obtenir l'analyse via nativeMessaging (event page → `SafariWebExtensionHandler`), le handler exécutant `lingua-core` compilé en natif — aucun WASM ne SHALL être instancié dans les contextes d'extension Safari. Les requêtes SHALL être des lots (par viewport), avec mémoïsation par forme côté content script, et une panne du pont SHALL dégrader en douceur (pas de surlignage) sans bloquer la page.

#### Scenario: Analyse d'une page dans Safari
- **WHEN** une page anglaise est chargée dans Safari avec l'extension active
- **THEN** le surlignage apparaît, l'analyse ayant transité par le handler natif, et aucun module WASM n'a été chargé dans l'extension

### Requirement: État local par appareil, prêt pour la sync
Chaque appareil SHALL conserver son état local (statuts, cartes, calibration) sous le même schéma versionné que l'extension Chromium/Firefox, amorcé par calibration ou import LingQ ; aucune synchronisation ne SHALL exister dans ce change, et les types partagés de `lingua-core` SHALL garantir qu'une fusion ultérieure (change de sync) est mécanique.

#### Scenario: iPhone et Mac indépendants
- **WHEN** l'utilisateur marque un mot « connu » sur son Mac
- **THEN** l'état de son iPhone est inchangé (aucune sync dans ce change), sans erreur ni écart de schéma
