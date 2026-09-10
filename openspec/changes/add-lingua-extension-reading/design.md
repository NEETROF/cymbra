# Design — add-lingua-extension-reading

## Context

Sixième change de la pile Lingua. Le cœur est déjà là : analyse déterministe (`add-lingua-analysis`), knowledge model lemma-first (`add-lingua-knowledge-model`), decks/FSRS et export Anki (`add-lingua-decks-review`), pack EN→FR (`add-lingua-data-pack`), bindings WASM et parité natif/WASM (`add-lingua-wasm`). Ce change industrialise la partie extension du **preshot fonctionnel** (`~/workspace/lingua-preshot`, extension MV3 JS pur) qui a validé sur pièces : le rendu CSS Custom Highlight API sur page réelle, la boucle popup → statuts → repaint, la calibration par curseur, la capture de sélection.

Contraintes héritées du monorepo : Yarn pour le JS, pas de logique métier dans les coquilles. Contraintes produit actées : jamais de silo de lecture, jamais « lemme » à l'écran, local-only par défaut, comptage honnête par lemme.

## Goals / Non-Goals

**Goals :**
- Un utilisateur (le fondateur d'abord) lit le web en anglais sur Chrome/Edge avec surlignage des mots inconnus, % par page honnête, popup de mot hors-ligne, capture d'expressions et création de cartes — **sans compte, sans réseau**.
- Poser la couture `AnalyzerPort` qui permettra Firefox (WASM en event page) et Safari (nativeMessaging) sans toucher au content script.

**Non-Goals :**
- Surfaces de révision dans l'extension — side panel, drawer, export Anki (`add-lingua-extension-review`).
- Variantes de build/manifest Firefox et Safari (`add-lingua-firefox`, `add-lingua-apple`).
- Sync multi-device, comptes, backend (`add-lingua-backend`, `add-lingua-connected-clients`) ; plugin agent (`add-lingua-agent`).

## Decisions

Les décisions du cœur (clé `(langue, lemme)` sans POS, cascade de lemmatisation, format de pack, planning FSRS, cible WASM) sont héritées des changes amont de la pile et ne sont pas re-décidées ici.

### D1 — Rendu : CSS Custom Highlight API, interaction par `caretRangeFromPoint`
Deux registres (`lingua-unknown`, `lingua-learning`), zéro mutation du DOM (pas de guerre avec React/hydration — validé par le preshot). Clic : `caretRangeFromPoint` → index de tokens trié (portable, Safari-compatible plus tard) ; `highlightsFromPoint` (Chromium 140+) en amélioration progressive. Repli `<span>` non implémenté en v1 (toutes les cibles MVP supportent l'API). MutationObserver débouncé avec re-scan **par sous-arbre muté** (le preshot re-scannait tout — suffisant pour juger, pas pour Gmail).

### D2 — Où vit le WASM : dans le content script
Le monde isolé de Chrome autorise `wasm-unsafe-eval` ; le module (code wasm ~1 Mo + pack EN ≤ 5 Mo) s'instancie par onglet, zéro IPC pour analyser. Le service worker ne garde que l'orchestration (badge, commandes). Les résultats par forme sont mémoïsés côté JS. Alternative rejetée : WASM dans le SW (aller-retours par page, SW tué à 30 s). L'analyse est exposée derrière un **`AnalyzerPort`** par messages malgré tout — c'est la couture qui permettra Firefox (WASM en event page) et Safari (nativeMessaging) sans toucher au content script.

### D3 — Permissions : `activeTab` + `optional_host_permissions <all_urls>`
Install sans avertissement effrayant ; « surligner cette page » marche immédiatement ; « toujours surligner » demande le grant une fois. Aligne Chrome sur le modèle imposé par Firefox/Safari (cibles ultérieures de la pile) et dé-risque la review du store. Le badge % fonctionne dans les deux modes.

### D4 — État extension : `chrome.storage.local`, schéma versionné
Statuts (map lemme→statut compact), cartes (JSON), calibration, préférences — sous une clé racine versionnée avec migration. Le pack est un asset de l'extension (pas dans storage). L'extension et le futur plugin agent (`add-lingua-agent`) auront chacun leur store local ; la **réconciliation est le problème des changes de sync** (`add-lingua-backend`/`add-lingua-connected-clients`), pas de celui-ci — un faux sync local serait du travail jeté.

### D5 — Charte graphique : tokens Cymbra partagés
Source de vérité = `CymbraColors` (« Sonic Luminescence », `apps/music/lib/theme/cymbra_theme.dart`). L'extension embarque une `tokens.css` qui la mirrore — exactement le précédent du back-office (`apps/back-office/src/styles.css` mirrore déjà le thème Flutter). Surfaces possédées par l'extension = charte pleine (Midnight Navy, violet primaire, rayons 12/18) ; surfaces injectées en page tierce = mêmes tokens mais lisibilité d'abord (les pages hôtes sont claires ou sombres). Coïncidence exploitée : la palette contient déjà l'ambre (`handLeft`, sémantique « pending ») et le corail (`error`) — ils deviennent les teintes de surlignage « en cours »/« inconnu », rendant le surlignage nativement Cymbra. Un lint interdit tout hex hors de `tokens.css` ; les surfaces de révision (`add-lingua-extension-review`) consommeront la même feuille.

### D6 — Monorepo : `apps/lingua-extension`, TS sans framework
TS sans framework (le DOM injecté n'a pas besoin de Vue), vitest pour la logique TS, build Yarn ; le module WASM est celui produit par `add-lingua-wasm` (wasm-pack `--target web`). Pas de Flutter, pas de Tauri dans ce change.

## Risks / Trade-offs

- [Perf du re-scan sur SPA lourdes (Gmail, feeds virtualisés)] → re-scan par sous-arbre + IntersectionObserver (analyse du visible d'abord) + budget par frame ; page pathologique = dégradation douce (surlignage partiel), jamais de jank imputable à l'extension.
- [Taille WASM + pack (~6 Mo) instanciée par onglet] → mémoïsation par forme, instanciation lazy (au premier bloc anglais détecté), un seul module partagé par frame principale ; mesurer avant d'optimiser (cible < 50 ms d'init).
- [Store review Chrome (`<all_urls>` optionnel, lecture de texte de page)] → posture D3 + privacy policy « le texte lu ne quitte jamais l'appareil » (vraie par construction — requirement « Aucune requête réseau »).
