# add-lingua-firefox — port Firefox (desktop + Android)

## Why

L'extension Chromium livrée par les changes amont couvre le dogfooding quotidien du fondateur (Chrome sur macOS), mais Firefox est le seul navigateur Android à store ouvert (AMO) : le même zip MV3 y livre l'extension sur desktop **et** mobile, quasi gratuitement. C'est aussi ce change qui introduit le **système de variantes de manifest** — l'extension devient un artefact multi-navigateurs construit depuis une source unique, fondation sur laquelle la variante `safari` se branchera au change suivant (`add-lingua-apple`).

**Position dans la pile** (12 changes) : 8ᵉ, après `add-lingua-extension-review`. Prérequis explicites : `add-lingua-extension-review` (extension Chromium complète — lecture + révision) et, par transitivité, `add-lingua-wasm` (l'`AnalyzerPort` et la cible WASM que la variante Firefox recâble).

## What Changes

- Le build de `apps/lingua-extension` produit désormais **deux variantes de manifest depuis la même source** : `chromium` (existante) et `firefox` (nouvelle). Ce qui varie est confiné derrière les coutures existantes (`AnalyzerPort`, surface de panneau).
- **Variante Firefox** : event page (`background.scripts` déclaré à côté du `service_worker`), **WASM chargé dans l'event page** derrière l'`AnalyzerPort` (la CSP Firefox bloque le WASM en content script — spike jour 1), host permissions optionnelles demandées à l'install (prompt), panneau via `sidebar_action` (même page que le side panel).
- **Publication AMO** : desktop + Android, même zip.

## Capabilities

### New Capabilities
_Aucune._

### Modified Capabilities
- `lingua-browser-extension` : ajout du requirement « Variante Firefox » — la promesse multi-navigateurs (source unique, variantes de build) entre dans la spec avec sa première variante ; le comportement Chromium existant est inchangé.

## Impact

- **Produits** : Lingua uniquement ; aucune app existante, aucun crate backend, aucun proto touchés.
- **Arborescence** : `apps/lingua-extension` (builds multi-cibles `chromium`/`firefox`) ; aucune nouvelle unité — le filtre `ci-units` est inchangé.
- **CI** : la lane extension construit désormais les deux variantes de manifest ; parcours manuel Firefox desktop + Android documenté (`web-ext run` / adb).
- **Stores** : AMO (desktop + Android, même zip). Les canaux « tier 3 » restent non supportés (formalisé au change `add-lingua-apple` avec la matrice complète).
- **Dépendances nouvelles** : outillage `web-ext` (dev + publication AMO).
