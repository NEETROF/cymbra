# lingua-browser-extension — révision dans le navigateur

## ADDED Requirements

### Requirement: Deux surfaces de révision
L'extension SHALL offrir la révision dans le **panneau natif du navigateur** quand il existe (Side Panel sur Chromium, sidebar sur Firefox — la page est poussée, le panneau survit aux navigations) et dans un **panneau injecté** repliable (shadow DOM) pour les micro-révisions. Sur Safari, qui n'a pas d'API de panneau, le panneau injecté SHALL porter seul la révision dans le navigateur. Toutes les surfaces SHALL opérer sur le même état local. L'export Anki sans perte (défini par `lingua-decks-review`) SHALL être accessible depuis le side panel.

#### Scenario: Side panel pendant la navigation
- **WHEN** l'utilisateur ouvre le side panel puis navigue vers une autre page
- **THEN** le side panel reste ouvert et sa session de révision continue

#### Scenario: Export Anki depuis le side panel
- **WHEN** l'utilisateur déclenche l'export Anki depuis le side panel
- **THEN** un fichier CSV est produit avec une colonne par champ du schéma de carte (colonnes vides pour les champs non peuplés), sans perte silencieuse
