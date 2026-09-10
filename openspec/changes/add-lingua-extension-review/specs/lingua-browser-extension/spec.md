# lingua-browser-extension — révision dans le navigateur

## ADDED Requirements

### Requirement: Deux surfaces de révision
L'extension SHALL offrir la révision dans le **panneau natif du navigateur** quand il existe (Side Panel sur Chromium, sidebar sur Firefox — la page est poussée, le panneau survit aux navigations) et dans un **panneau injecté** repliable (shadow DOM) pour les micro-révisions. Sur Safari, qui n'a pas d'API de panneau, le panneau injecté SHALL porter seul la révision dans le navigateur. Toutes les surfaces SHALL opérer sur le même état local. La sauvegarde/restauration sans perte (définie par `lingua-decks-review`) SHALL être accessible depuis le side panel (téléchargement du fichier et ré-import).

#### Scenario: Side panel pendant la navigation
- **WHEN** l'utilisateur ouvre le side panel puis navigue vers une autre page
- **THEN** le side panel reste ouvert et sa session de révision continue

#### Scenario: Sauvegarde depuis le side panel
- **WHEN** l'utilisateur déclenche la sauvegarde depuis le side panel
- **THEN** un fichier de sauvegarde versionné est téléchargé, contenant l'état complet (cartes champ par champ, statuts, calibration, paramètres FSRS), et son ré-import restaure l'état à l'identique
