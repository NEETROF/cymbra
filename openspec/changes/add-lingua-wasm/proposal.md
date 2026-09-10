# add-lingua-wasm — Cymbra Lingua : cible WASM et parité native/WASM

## Why

Le principe « un seul cerveau » de Lingua exige que la même analyse (même `analyzer_version`) produise les mêmes résultats dans l'extension (WASM) et le plugin (natif). Le cœur et le pack existent (`add-lingua-analysis`, `add-lingua-data-pack`) mais ne compilent qu'en natif ; ce change livre la **cible WASM** de `lingua-core` — bindings wasm-bindgen, build `wasm-pack --target web` — et fait de la **parité native/WASM un contrat testé en CI**, avant que la moindre surface navigateur n'en dépende. Les changes d'extension (`add-lingua-extension-reading` et suivants) consomment ce module tel quel.

**Position dans la pile (5/12)** : add-lingua-analysis → add-lingua-knowledge-model → add-lingua-decks-review → add-lingua-data-pack → **add-lingua-wasm** → add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox → add-lingua-apple → add-lingua-agent → add-lingua-backend → add-lingua-connected-clients. **Prérequis explicites : `add-lingua-analysis`** (l'analyseur, son `analyzer_version` et son corpus de fixtures) **et `add-lingua-data-pack`** (le pack chargé par les deux cibles — la parité se teste à pack égal).

## What Changes

- **Crate/feature `lingua-wasm`** : bindings wasm-bindgen (analyse par lot de blocs → tokens classés, statuts, %, gloses) ; build wasm-pack `--target web`.
- **Tests de parité natif/WASM** sur les fixtures : même `analyzer_version` (et même pack) ⇒ sorties identiques octet pour octet.
- **Lane CI** : build wasm + exécution des tests de parité.
- La logique reste host-testée dans `lingua-core` ; seul le glue wasm-bindgen est exclu du coverage (convention `--ignore-filename-regex`, même traitement que le glue frb de music).

## Capabilities

### New Capabilities
_Aucune._

### Modified Capabilities
- `lingua-analysis` : ajout du requirement « Parité native/WASM » — le pipeline d'analyse devient compilable en module WASM, avec parité contractuelle vérifiée en CI. Aucun requirement existant ne bouge.

## Impact

- **Produits** : Lingua (nouvelle cible de compilation du cœur — rien de consommé hors de la pile Lingua) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés).
- **Arborescence** : `crates/lingua-core` (feature/crate compagnon `lingua-wasm`) — le module WASM sera embarqué par `apps/lingua-extension` au change suivant. Si `lingua-wasm` est un crate séparé du workspace, il est ajouté au filtre `ci-units` avec la lane wasm qui le surveille.
- **CI** : nouvelle lane build wasm-pack + tests de parité ; la lane Rust existante continue de couvrir la logique (llvm-cov ≥ 80 %, glue wasm-bindgen exclu via le `--ignore-filename-regex` partagé).
- **Dépendances nouvelles** : `wasm-bindgen` / wasm-pack.
- **Hors périmètre** : où vit le WASM dans l'extension (content script vs event page — la couture `AnalyzerPort` d'`add-lingua-extension-reading`), le spike CSP Firefox (`add-lingua-firefox`), toute UI.
