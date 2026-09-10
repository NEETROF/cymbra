# Design — add-lingua-extension-review

## Context

Septième change de la pile Lingua, directement au-dessus d'`add-lingua-extension-reading` : l'extension Chromium lit, surligne et crée des cartes ; ce change lui donne ses surfaces de révision. Le moteur est hérité : état FSRS et export Anki dans `lingua-core` (`add-lingua-decks-review`), bindings (`add-lingua-wasm`), charte/tokens, storage et `AnalyzerPort` (`add-lingua-extension-reading`). Le preshot (`~/workspace/lingua-preshot`) avait validé les deux surfaces (panneau latéral / panneau flottant).

## Goals / Non-Goals

**Goals :**
- Boucle complète dans le navigateur : deck consultable, session de révision FSRS, export Anki — sur le même état local que la lecture.
- Deux surfaces : side panel natif par défaut, drawer injecté pour les micro-révisions — conçues pour être portées telles quelles (sidebar Firefox, drawer-only Safari) par les changes suivants.

**Non-Goals :**
- Badge « dues » sur l'icône, alarmes, notifications (v1 : compteur dans le popup d'icône et le side panel).
- Portage Firefox/Safari des surfaces (`add-lingua-firefox`, `add-lingua-apple`).
- Sync des cartes et de l'historique de révision (`add-lingua-backend`).

## Decisions

L'algorithme (FSRS, notation `again/hard/good/easy`, « Je connais » → `known` provenance `srs`) et le format d'export Anki (CSV, une colonne par champ, jamais de perte silencieuse) sont décidés par `add-lingua-decks-review` ; ce change ne décide que des surfaces.

### D1 — Deux surfaces d'UI : Side Panel API par défaut, drawer injecté en repli
UI : side panel (Side Panel API — la page est poussée, survit aux navigations) par défaut ; drawer overlay injecté (shadow DOM fermé) en repli et pour les micro-révisions — les deux partagent la même logique de session et opèrent sur le même état `chrome.storage.local`. Le badge de l'icône reste dédié au % de la page ; le compteur de cartes dues est visible dans le popup de l'icône et le side panel (pas de badge « dues » ni d'alarme en v1). Cette dualité est la cible de pile : Firefox exposera la même page via `sidebar_action` ; Safari, sans API de panneau, portera la révision navigateur sur le seul drawer. Export Anki : déclenché depuis le side panel, produit le CSV sans perte de `lingua-core`.

## Risks / Trade-offs

- [Deux surfaces pour une même logique = risque de divergence] → la session de révision est un module unique (testé vitest) ; side panel et drawer ne sont que deux hôtes de rendu de ce module.
- [Le drawer vit dans des pages hostiles (styles agressifs, z-index)] → shadow DOM fermé + tokens embarqués (charte d'`add-lingua-extension-reading`), lisibilité sur page claire comme sombre.
