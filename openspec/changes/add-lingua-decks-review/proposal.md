# add-lingua-decks-review — Cymbra Lingua : decks, cartes et révision FSRS dans le cœur

## Why

`add-lingua-knowledge-model` sait dire ce que l'utilisateur connaît ; rien ne l'aide encore à apprendre ce qu'il ne connaît pas. Ce change ajoute au cœur la brique d'apprentissage : decks et cartes avec provenance (la phrase d'origine, capturable seulement au moment de la rencontre), planification de révision FSRS, et export Anki sans perte — le must-have de la catégorie, designé dans le schéma dès le jour 1. Tout reste de la logique pure `lingua-core`, host-testée : les surfaces d'UI (side panel, drawer, app conteneur) arrivent dans les changes suivants et consomment ce moteur tel quel.

**Position dans la pile (3/12)** : add-lingua-analysis → add-lingua-knowledge-model → **add-lingua-decks-review** → add-lingua-data-pack → add-lingua-wasm → add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox → add-lingua-apple → add-lingua-agent → add-lingua-backend → add-lingua-connected-clients. **Prérequis explicite : `add-lingua-knowledge-model`** (statuts, provenance `srs`, clé (langue, lemme) — le passage en « connu » depuis la révision écrit dans le knowledge model).

## What Changes

- **Module `decks/` dans `crates/lingua-core`** : schéma de carte sérialisable versionné — lemme, forme rencontrée, phrase de contexte d'origine, source de la rencontre (URL ou identifiant de session d'agent, horodatage), glose, emplacement de média optionnel (`media` avec `source: capture|banque|génération` et `sync_policy` — non peuplé dans ce change mais présent dans le schéma). Les expressions multi-mots sont des cartes de plein droit.
- **Intégration FSRS** (crate `fsrs`, version épinglée) : notation `again/hard/good/easy`, échéances, compteur de cartes dues calculable à tout instant ; paramètres stockés sur l'état.
- **« Je connais » en révision** → statut `known` provenance `srs` dans le knowledge model, carte conservée hors file (historique intact).
- **Export Anki CSV** : une colonne par champ du schéma, colonnes vides pour les champs non peuplés — jamais de perte silencieuse.
- Le requirement « révision au ras de la lecture » (compteur de dues visible, session lançable depuis le side panel/panneau injecté, réponse masquée) est posé ici comme contrat de la capability ; ses surfaces sont livrées par `add-lingua-extension-review` (puis `add-lingua-apple`), qui consomment ce moteur.

## Capabilities

### New Capabilities
- `lingua-decks-review` : decks et cartes — carte = lemme + forme vue + phrase de provenance + source + média optionnel (schéma jour 1, capture d'image différée), révision FSRS, export Anki sans perte, expressions multi-mots, révision accessible au ras de la lecture.

### Modified Capabilities
_Aucune. `lingua-knowledge-model` est consommée telle quelle (statuts et provenance `srs`)._

## Impact

- **Produits** : Lingua (nouveau module du cœur — rien de consommé hors de la pile Lingua) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés).
- **Arborescence** : `crates/lingua-core` (module `decks/`) uniquement — aucune nouvelle unité `apps/*`/`packages/*`.
- **CI** : couvert par la lane Rust existante (fmt/clippy/llvm-cov ≥ 80 %), déjà branchée sur le crate par `add-lingua-analysis` ; aucune nouvelle lane, rien à ajouter à `ci-units`.
- **Dépendances nouvelles** : `fsrs` (Rust), version épinglée.
- **Hors périmètre** : toute UI de révision (`add-lingua-extension-review`, `add-lingua-apple`), capture d'image sur les cartes (le schéma `media` est prêt, la capture est différée), sync des cartes (`add-lingua-backend` / `add-lingua-connected-clients`).
