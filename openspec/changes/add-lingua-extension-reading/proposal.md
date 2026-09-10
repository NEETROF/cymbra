# add-lingua-extension-reading — Cymbra Lingua : l'extension qui lit (Chromium)

## Why

Les changes précédents de la pile livrent le cerveau (analyse, knowledge model, decks/FSRS, pack EN→FR, cible WASM) — mais rien que l'utilisateur puisse ouvrir. Ce change livre la première surface utilisable : l'instrumentation de lecture dans le navigateur, **sur Chromium (Chrome + Edge, même build)** — le dogfooding quotidien du fondateur, sous Chrome sur macOS. Le principe produit n°1 s'incarne ici : jamais de silo — on lit le web **sur place**, surligné, avec un % honnête par lemme.

**Position dans la pile (12 changes) : 6ᵉ.** Prérequis directs : `add-lingua-decks-review` (cartes créées par « + Deck », compteur de dues), `add-lingua-data-pack` (gloses/fréquences hors-ligne), `add-lingua-wasm` (bindings d'analyse) — qui tirent eux-mêmes `add-lingua-analysis` et `add-lingua-knowledge-model`. Les variantes Firefox (`add-lingua-firefox`) et Safari/Apple (`add-lingua-apple`) arrivent plus loin dans la pile : ce change pose la couture (`AnalyzerPort`) qui les rendra possibles sans toucher au content script. Les surfaces de révision (side panel/drawer) sont le change suivant, `add-lingua-extension-review`.

## What Changes

- **Nouvelle extension navigateur MV3** (`apps/lingua-extension`), ciblant Chromium : surlignage des mots inconnus via CSS Custom Highlight API (zéro mutation du DOM), % de mots connus par page (badge + popup d'icône avec calibration par curseur), popup de mot au clic (forme du dictionnaire, glose, rareté, actions statuts), capture de sélection multi-mots au raccourci clavier avec phrase d'origine, création de cartes « + Deck ».
- **`AnalyzerPort`** : le content script consomme l'analyse exclusivement par messages ; l'implémentation de ce change est le WASM instancié dans le content script.
- **Posture de permissions minimale** (`activeTab` + `<all_urls>` optionnel), **aucune requête réseau**, tout l'état en `chrome.storage.local` sous schéma versionné.
- **Charte Cymbra** : `tokens.css` mirrorant `CymbraColors`, surlignages dérivés de l'ambre/corail de la palette, lint « aucun hex hors tokens.css ».
- Vocabulaire UI : le mot « lemme » n'apparaît **jamais** à l'écran (« forme du dictionnaire », « mots différents ») — lint des chaînes UI.

## Capabilities

### New Capabilities
- `lingua-browser-extension` : l'expérience de **lecture** dans le navigateur — surlignage in-place, % par page, popup de mot, capture de sélection, calibration, identité visuelle Cymbra, posture de permissions, zéro réseau, état local versionné. Les requirements de révision (side panel/drawer) et la matrice multi-navigateurs sont ajoutés à cette capability par les changes suivants de la pile.

### Modified Capabilities
_Aucune._

## Impact

- **Produits** : Lingua uniquement ; Cymbra ID / Music / Live / back-office intacts.
- **Arborescence** : `apps/lingua-extension` (TS sans framework, wasm-pack, Yarn, vitest, build esbuild/vite) — nouvelle unité `apps/*`, ajoutée au filtre `ci-units` avec sa lane vitest/lint.
- **CI** : lane vitest + lints (« lemme », hex hors tokens) sur `apps/lingua-extension` ; le build WASM et les tests de parité sont déjà couverts par `add-lingua-wasm`.
- **Distribution** : load unpacked pour le dogfooding ; publication (Chrome Web Store + Microsoft Add-ons, même build) quand stable — recommandation héritée : unlisted dès que stable, pour roder la review. Les canaux tier 3 (forks Chromium, stores curés) restent non supportés.
- **Hors périmètre** : side panel/drawer et export Anki dans l'extension (`add-lingua-extension-review`), variantes de manifest Firefox/Safari (`add-lingua-firefox`, `add-lingua-apple`), sync/compte (`add-lingua-backend`, `add-lingua-connected-clients`).
