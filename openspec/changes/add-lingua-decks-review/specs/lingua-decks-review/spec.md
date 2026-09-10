# lingua-decks-review — decks, cartes et révision

## ADDED Requirements

### Requirement: Schéma de carte avec provenance
Une carte SHALL porter au minimum : le lemme, la forme rencontrée, la phrase de contexte d'origine, la source de la rencontre (URL ou identifiant de session d'agent, horodatage), la glose, et un emplacement de média optionnel (`media`, avec `source: capture|banque|génération` et `sync_policy`) — non peuplé dans ce change mais présent dans le schéma. Les expressions multi-mots SHALL être des cartes de plein droit.

#### Scenario: Carte créée depuis le popup
- **WHEN** l'utilisateur ajoute `conundrum` au deck depuis une page web
- **THEN** la carte contient le lemme, la phrase d'origine complète et l'URL de la page

#### Scenario: Carte d'expression
- **WHEN** l'utilisateur capture la sélection « compelling starting point » au raccourci clavier
- **THEN** une carte d'expression est créée avec la phrase d'origine

### Requirement: Planification de révision FSRS
La révision SHALL être planifiée par FSRS dans le cœur : chaque carte porte son état (stabilité, difficulté, échéance) et les quatre réponses (`again`, `hard`, `good`, `easy`) SHALL mettre à jour l'état et l'échéance. Le nombre de cartes dues SHALL être calculable à tout instant.

#### Scenario: Réponse « good » repousse l'échéance
- **WHEN** une carte due est notée `good`
- **THEN** son échéance devient strictement postérieure à maintenant et son état FSRS est mis à jour

### Requirement: Passage en connu depuis la révision
Marquer une carte « je connais » pendant la révision SHALL passer le lemme en statut `known` (provenance `srs`) et le retirer des cartes dues, sans supprimer la carte ni son historique.

#### Scenario: Mot appris
- **WHEN** l'utilisateur répond « je connais » sur la carte `seldom`
- **THEN** `seldom` passe en statut `known` (provenance `srs`) et la carte sort de la file de révision

### Requirement: Export Anki sans perte silencieuse
Le système SHALL exporter les decks en CSV importable dans Anki, avec une colonne par champ de carte (lemme, forme vue, phrase, glose, source, statut, échéance, état FSRS). Un champ non peuplé SHALL produire une colonne vide ; aucun champ peuplé ne SHALL être omis.

#### Scenario: Export d'un deck
- **WHEN** l'utilisateur exporte son deck de 20 cartes
- **THEN** le CSV contient 20 lignes, toutes les colonnes du schéma présentes, les champs peuplés fidèlement reportés

### Requirement: Révision présente au ras de la lecture
La révision SHALL être accessible sans quitter le navigateur : le compteur de cartes dues SHALL être visible dans le popup de l'icône et le side panel, et une session de révision SHALL pouvoir être lancée depuis le side panel ou le panneau injecté. La réponse d'une carte SHALL être masquée jusqu'à une action explicite de révélation.

#### Scenario: Micro-session depuis le side panel
- **WHEN** l'utilisateur ouvre le side panel et lance « Réviser maintenant » avec 3 cartes dues
- **THEN** les cartes défilent une par une, réponse masquée puis révélée, et le compteur de dues décroît
