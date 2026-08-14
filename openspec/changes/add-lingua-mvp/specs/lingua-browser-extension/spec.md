# lingua-browser-extension — expérience navigateur

## ADDED Requirements

### Requirement: Surlignage in-place sans mutation du DOM
L'extension SHALL surligner les mots inconnus (et distinctement les mots « en cours ») via la CSS Custom Highlight API, sans envelopper les mots dans des éléments ni modifier le DOM de la page. Le contenu dynamique (SPA) SHALL être re-analysé par sous-arbre muté, avec débounce. L'analyse SHALL tourner dans le monde isolé du content script (WASM), derrière un port de messages (`AnalyzerPort`).

#### Scenario: Page anglaise surlignée
- **WHEN** une page d'article en anglais est chargée avec l'extension active
- **THEN** les mots inconnus apparaissent surlignés et le DOM de la page ne contient aucun nœud ajouté par l'extension (hors hôtes d'UI de l'extension)

#### Scenario: Contenu ajouté dynamiquement
- **WHEN** une SPA insère un nouveau paragraphe anglais
- **THEN** ce paragraphe est analysé et surligné sans re-analyse complète de la page

### Requirement: Pourcentage de la page visible en permanence
Le badge de l'icône SHALL afficher le pourcentage de mots connus de l'onglet actif, mis à jour à chaque changement d'état ; sur une page non analysable, il SHALL afficher un état neutre. Le popup de l'icône SHALL détailler : % connu, mots analysés, mots inconnus (occurrences et mots différents), et le réglage de calibration.

#### Scenario: Badge sur page analysée
- **WHEN** une page à 94 % de tokens connus est active
- **THEN** le badge affiche « 94% »

### Requirement: Popup de mot au clic
Cliquer un mot surligné SHALL ouvrir un panneau (shadow DOM fermé) affichant : la forme du dictionnaire, la forme vue si différente, la glose en langue maternelle (du pack, hors-ligne), le rang de fréquence vulgarisé, et les actions « Je connais », « + Deck », « Ignorer ». Chaque action SHALL mettre à jour le statut, re-peindre la page immédiatement et se propager aux autres onglets.

#### Scenario: Ajout au deck
- **WHEN** l'utilisateur clique « + Deck » sur un mot surligné
- **THEN** le mot passe au surlignage « en cours » sur cet onglet et sur tout autre onglet ouvert, et une carte est créée avec la phrase d'origine

### Requirement: Capture de sélection au raccourci clavier
Un raccourci clavier SHALL capturer la sélection courante (mot ou expression, bornée en longueur) et ouvrir le panneau avec la phrase d'origine extraite automatiquement, permettant l'ajout au deck comme carte d'expression.

#### Scenario: Capture d'une expression
- **WHEN** l'utilisateur sélectionne trois mots et presse le raccourci
- **THEN** le panneau affiche l'expression et sa phrase d'origine, et « + Deck » crée la carte

### Requirement: Deux surfaces de révision
L'extension SHALL offrir la révision dans le **side panel natif** (la page est poussée, le panneau survit aux navigations) et dans un **panneau injecté** repliable (shadow DOM) pour les micro-révisions. Les deux SHALL opérer sur le même état local.

#### Scenario: Side panel pendant la navigation
- **WHEN** l'utilisateur ouvre le side panel puis navigue vers une autre page
- **THEN** le side panel reste ouvert et sa session de révision continue

### Requirement: Identité visuelle Cymbra
Les surfaces d'UI possédées par l'extension (popup d'icône, side panel, pages d'extension) SHALL appliquer la charte graphique Cymbra — la palette « Sonic Luminescence » de `apps/music/lib/theme/cymbra_theme.dart`, déjà mirrorée en variables CSS par le back-office (`apps/back-office/src/styles.css`) — via une feuille de tokens unique embarquée dans l'extension. Les surfaces injectées dans les pages tierces (popup de mot, panneau flottant) SHALL consommer les mêmes tokens, la lisibilité sur page claire comme sombre primant sur la fidélité au thème sombre. Les teintes de surlignage SHALL dériver de l'ambre (`handLeft` — « en cours ») et du corail (`error` — « inconnu ») de la palette. Aucune couleur ne SHALL être codée en dur hors de la feuille de tokens.

#### Scenario: Cohérence des surfaces d'extension
- **WHEN** l'utilisateur ouvre le popup d'icône puis le side panel
- **THEN** les deux surfaces rendent avec les tokens Cymbra (fonds Midnight Navy, primaire violet, rayons de la charte) et aucun hex hors de la feuille de tokens n'existe dans les styles (vérifié par lint)

#### Scenario: Surlignage lisible sur page claire
- **WHEN** une page à fond clair est surlignée
- **THEN** les teintes ambre et corail des surlignages laissent le texte de la page dans sa couleur d'origine et restent distinguables entre elles

### Requirement: Posture de permissions minimale
L'extension SHALL s'installer avec `activeTab` et déclarer `<all_urls>` en permission optionnelle : « surligner cette page » SHALL fonctionner sans grant global ; « toujours surligner » SHALL demander le grant une seule fois. Aucun texte de page ne SHALL quitter l'appareil.

#### Scenario: Premier usage sans grant global
- **WHEN** l'utilisateur clique l'icône sur une page sans avoir accordé `<all_urls>`
- **THEN** la page courante est analysée et surlignée via `activeTab`

### Requirement: Aucune requête réseau
En v1, l'extension ne SHALL émettre aucune requête réseau : le pack et les gloses sont des assets locaux, et ni le texte des pages ni les données utilisateur ne quittent l'appareil.

#### Scenario: Fonctionnement hors ligne
- **WHEN** l'utilisateur lit une page déjà chargée sans connexion réseau
- **THEN** le surlignage, le popup de mot (glose comprise) et la révision fonctionnent intégralement

### Requirement: État local versionné
L'état (statuts, cartes, calibration, préférences) SHALL vivre dans `chrome.storage.local` sous un schéma versionné avec migration ascendante. Une réinitialisation complète SHALL être offerte.

#### Scenario: Migration de schéma
- **WHEN** l'extension démarre sur un état de version antérieure
- **THEN** l'état est migré sans perte et la version stockée est mise à jour
