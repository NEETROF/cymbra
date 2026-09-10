# add-lingua-extension-review — Cymbra Lingua : la révision dans l'extension

## Why

`add-lingua-extension-reading` crée des cartes (« + Deck », capture d'expressions) mais n'offre aucun moyen de les réviser dans le navigateur : le planning FSRS existe dans `lingua-core` (`add-lingua-decks-review`) sans surface d'UI. Ce change ferme la boucle lecture → deck → révision → export, là où l'utilisateur lit déjà — sans back-office, sans app à part. La décision UX est actée : la page est **poussée**, pas recouverte (Side Panel API), avec un drawer injecté pour les micro-révisions.

**Position dans la pile (12 changes) : 7ᵉ.** Prérequis direct : `add-lingua-extension-reading` (qui tire `add-lingua-decks-review`, `add-lingua-data-pack`, `add-lingua-wasm`). Les changes suivants (`add-lingua-firefox`, `add-lingua-apple`) porteront ces surfaces : sidebar Firefox = même page que le side panel ; Safari, sans API de panneau, s'appuiera sur le seul drawer — le requirement le prévoit dès maintenant.

## What Changes

- **Side panel** (Side Panel API, permission `sidePanel` ajoutée au manifest) : deck, session de révision FSRS (réponse masquée/révélée), compteur de cartes dues — la page est poussée, le panneau survit aux navigations.
- **Drawer injecté** repliable (shadow DOM fermé) partageant la même logique de révision, pour les micro-révisions sans quitter la page.
- **Sauvegarde/restauration depuis le side panel** : la sauvegarde sans perte de `lingua-decks-review` (téléchargement du fichier + ré-import), rendue accessible dans le navigateur — le filet de sécurité de la phase locale.
- **Storage versionné complété** : migrations + réinitialisation complète.
- **Page attributions** (NOTICE du pack) + privacy note « rien ne quitte l'appareil ».
- Parcours manuel complet documenté (calibration → lecture → +Deck → révision → export) sur 5 sites réels.

## Capabilities

### New Capabilities
_Aucune._

### Modified Capabilities
- `lingua-browser-extension` (créée par `add-lingua-extension-reading`) : ajout du requirement « Deux surfaces de révision » — side panel natif + drawer injecté sur le même état local, sauvegarde/restauration accessible depuis le side panel, posture Safari (drawer seul) anticipée.

## Impact

- **Produits** : Lingua uniquement ; Cymbra ID / Music / Live / back-office intacts.
- **Arborescence** : `apps/lingua-extension` (pages side panel/drawer, logique de session de révision, export, attributions) — aucune nouvelle unité, les lanes CI de `add-lingua-extension-reading` couvrent tout (vitest, lints « lemme »/hex).
- **Dépendances** : aucune nouvelle — FSRS et la sauvegarde/restauration viennent de `lingua-core` via les bindings d'`add-lingua-wasm`.
- **Hors périmètre** : badge « cartes dues » et alarmes/notifications (v1 : le compteur vit dans le popup d'icône et le side panel), sidebar Firefox et drawer Safari (changes `add-lingua-firefox`/`add-lingua-apple` — ils réutilisent ces surfaces), sync des cartes (`add-lingua-backend`).
