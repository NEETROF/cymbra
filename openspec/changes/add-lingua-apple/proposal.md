# add-lingua-apple — app conteneur Apple + extension Safari (macOS + iOS)

## Why

Sur iOS, Safari est la seule voie d'extension (règle Apple), assumée — et l'extension y arrive **désactivée** par défaut : le décrochage documenté n°1 de la catégorie. La réponse n'est pas un README mais une app conteneur à part entière (guideline 4.4) : elle héberge l'extension Safari convertie, porte les écrans decks/révision, l'analyse **native** (pas de WASM chez Apple), et le parcours d'activation guidé. Ce change complète la matrice de navigateurs du MVP : la variante `safari` rejoint le build multi-cibles introduit par `add-lingua-firefox`.

**Position dans la pile** (12 changes) : 9ᵉ, après `add-lingua-firefox`. Prérequis explicites : `add-lingua-extension-review` (extension complète — lecture + révision, drawer injecté) et `add-lingua-firefox` (le système de variantes de manifest que la variante `safari` rejoint).

## What Changes

- **Nouvelle app conteneur Apple** (`apps/lingua-apple`) : un projet Xcode, **une fiche App Store universelle (iOS + macOS)** qui héberge l'extension Safari convertie, l'**analyse native** (`lingua-core` lié en natif via nativeMessaging — pas de WASM chez Apple, packs dans le bundle), les écrans decks/révision, et le **parcours d'activation** de l'extension (walkthrough pas-à-pas sur iOS + détection par heartbeat App Group ; deep link et API d'état sur macOS).
- **Variante `safari` du build de l'extension** : troisième variante de manifest depuis la même source (`safari-web-extension-converter`), analyse via nativeMessaging derrière l'`AnalyzerPort`, drawer injecté seul pour la révision dans le navigateur (Safari n'a pas d'API de panneau).
- **Signing/TestFlight** : pattern `release-build` de music cloné (chaîne Apple existante) ; dogfooding iOS via TestFlight interne.
- Les canaux « tier 3 » (Edge Canary Android par ID, stores curés Edge/Samsung, forks Chromium) sont explicitement **non supportés** : le build standard peut y tourner, rien n'y est promis ni testé.

## Capabilities

### New Capabilities
- `lingua-apple-app` : l'app conteneur Safari (iOS + macOS) — fonctions propres (decks/révision), activation guidée de l'extension (walkthrough + heartbeat iOS, deep link + API d'état macOS), analyse native via nativeMessaging, packs dans le bundle, fiche App Store universelle.

### Modified Capabilities
- `lingua-browser-extension` : ajout du requirement « Variante Safari » — la variante convertie hébergée par l'app conteneur rejoint la matrice de build (chromium/firefox/safari), avec la limite « tier 3 non promis ».

## Impact

- **Produits** : Lingua ; la chaîne de signing Apple de music est **consommée** (pattern cloné), pas modifiée ; aucun proto, aucun crate backend touchés.
- **Arborescence** : `apps/lingua-apple` (projet Xcode : app conteneur + extension Safari + handler natif) ; `apps/lingua-extension` gagne la variante `safari` dans son build multi-cibles.
- **CI** : lane de signing Apple clonée du pattern `release-build` de music (TestFlight interne pour le dogfooding iOS) ; `apps/lingua-apple` ajouté au filtre `ci-units`.
- **Stores** : App Store (une fiche universelle iOS/macOS). Cadence assumée : les fixes Safari passent par la review Apple — les comportements se rodent d'abord sur Chromium/Firefox.
- **Dépendances nouvelles** : aucune côté Rust ; outillage `safari-web-extension-converter` (Xcode) côté build.
