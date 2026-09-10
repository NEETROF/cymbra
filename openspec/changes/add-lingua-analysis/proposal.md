# add-lingua-analysis — Cymbra Lingua : fondations monorepo + pipeline d'analyse

## Why

Les produits existants de lecture-en-langue-étrangère (LingQ, Readlang, Migaku) comptent les formes de surface au lieu des lemmes — leurs compteurs « mots connus » mentent. Le comptage honnête par lemme est le différenciateur n°1 de Cymbra Lingua, et il repose entièrement sur un pipeline d'analyse déterministe : tokenisation, lemmatisation en cascade, détection de langue, % de tokens connus. Ce change pose **la première brique** du produit : le crate `lingua-core` dans le monorepo (avec sa lane CI/coverage et le préfixe OpenSpec `lingua-*`) et le pipeline d'analyse complet, testé sur mini-fixtures. Tout le reste de la pile — knowledge model, decks, packs, WASM, extension, app Apple, plugin agent, backend — consomme ce pipeline.

**Position dans la pile** (12 changes, ordre d'implémentation) : **1/12 — tête de pile, aucun prérequis.** Suite : add-lingua-knowledge-model → add-lingua-decks-review → add-lingua-data-pack → add-lingua-wasm → add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox → add-lingua-apple → add-lingua-agent → add-lingua-backend → add-lingua-connected-clients.

## What Changes

- **Nouveau préfixe OpenSpec `lingua-*`** ajouté à `openspec/config.yaml`. Ce change ne touche aucune capability existante.
- **Nouveau crate `crates/lingua-core`** dans le workspace Cargo racine : tokenisation UAX #29 avec pré-passe par langue, lemmatisation par table FST + repli morphologique + repli pluriel hors-lexique, détection de langue par bloc, calcul du % de tokens connus, `analyzer_version` contractuelle. Logique pure host-testable (convention `*_core.rs`) ; layout de modules (`analysis/`, `knowledge/`, `decks/`, `packs/`) préparé pour les changes suivants.
- **CI** : le crate est couvert par la lane Rust existante (fmt/clippy/llvm-cov ≥ 80 %) ; l'exclusion du futur glue wasm-bindgen est ajoutée au `--ignore-filename-regex`.
- Les tests de ce change utilisent des **mini-fixtures synthétiques** (FST/fréquences de test) ; le vrai pack (AGID/wordfreq/kaikki) arrive avec `add-lingua-data-pack`.

## Capabilities

### New Capabilities
- `lingua-analysis` : pipeline d'analyse de texte — tokenisation, pré-passe par langue (élisions/clitiques), lemmatisation (FST + repli + repli pluriel hors-lexique), détection de langue par bloc, calcul du % de tokens connus. Déterministe à `analyzer_version` donnée.

### Modified Capabilities
_Aucune. Ce change est local au nouveau crate : il ne consomme ni ne modifie `id-*`/`platform-*`._

## Impact

- **Produits** : Lingua (nouveau) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés). Le socle n'est consommé qu'à un endroit : la convention de coverage et la CI.
- **Arborescence** : `crates/lingua-core` (workspace Cargo racine) ; `openspec/config.yaml` (domaine `lingua-*`).
- **CI** : lane Rust existante (`cargo --workspace`) couvre le crate automatiquement ; `.github/coverage-ignore-regex.txt` mis à jour pour le futur glue wasm-bindgen.
- **Dépendances nouvelles** : `fst`, `unicode-segmentation`, `whichlang`, `serde` (le crate `fsrs` arrive avec `add-lingua-decks-review`).
- **Hors périmètre** : statuts/calibration (`add-lingua-knowledge-model`), decks/révision, pack réel, cible WASM et test de parité natif/WASM (`add-lingua-wasm`), toute surface utilisateur.
