# Design — add-lingua-firefox

## Context

L'extension Chromium existe et est complète (`add-lingua-extension-reading` + `add-lingua-extension-review`) : source WebExtension MV3 unique, analyse consommée par le content script via l'`AnalyzerPort` (couture posée par `add-lingua-wasm`, impl Chromium = WASM dans le content script), side panel de révision. Ce change la porte sur Firefox desktop + Android et introduit le système de variantes de manifest. Toutes les décisions amont (surlignage Highlight API, storage versionné, permissions `activeTab` + optionnelles, charte Cymbra) sont héritées telles quelles des changes précédents de la pile.

## Decisions

### D1 — Un artefact, des variantes de build (part Firefox de D12 du design source)

Une seule source WebExtension MV3 ; le build produit des variantes de manifest — `chromium` et `firefox` avec ce change (la variante `safari` rejoindra la matrice avec `add-lingua-apple`). Ce qui varie est confiné derrière deux coutures : l'**`AnalyzerPort`** (Chrome/Edge = WASM dans le content script ; Firefox = WASM dans l'event page — sa CSP bloque le WASM en content script, spike jour 1) et la **surface de panneau** (Side Panel API sur Chromium ; `sidebar_action` sur Firefox — la même page d'extension dans les deux cas). Firefox : `background.scripts` (event page) déclaré à côté du `service_worker`, host permissions optionnelles à l'install (prompt), même zip publié desktop + Android sur AMO.

## Risks / Trade-offs

- [WASM dans les content scripts Firefox (CSP)] → spike jour 1 du port ; le repli est déjà le design (WASM en event page + messages via l'`AnalyzerPort`) — le verdict du spike ne change pas l'architecture, seulement l'opportunité d'une optimisation ultérieure.
- [Event page tuée entre deux requêtes] → requêtes par lots + mémoïsation par forme côté content script ; la ré-instanciation du module est idempotente.
- [Deux variantes pour un solo dev] → un artefact unique + coutures ; l'ordre d'implémentation reste Chromium → Firefox → Apple ; parcours manuel par navigateur avant chaque release.
