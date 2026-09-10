# Tasks — add-lingua-analysis

## 1. Fondations monorepo

- [ ] 1.1 Déclarer le domaine `lingua-*` dans `openspec/config.yaml` (table des préfixes + règles par artefact si besoin)
- [ ] 1.2 Créer `crates/lingua-core` (lib) dans le workspace Cargo racine ; dépendances `unicode-segmentation`, `fst`, `whichlang`, `serde` ; module layout `analysis/`, `knowledge/`, `decks/`, `packs/`
- [ ] 1.3 Brancher le crate sur la lane CI Rust (fmt/clippy/llvm-cov) et ajouter l'exclusion du glue wasm-bindgen au `--ignore-filename-regex` (lane CI **et** commande documentée dans CLAUDE.md) — la logique reste host-testée, seuls les bindings sont exclus

## 2. Pipeline d'analyse (spec lingua-analysis)

_Ce change teste sur des mini-fixtures synthétiques (FST/fréquences de test) ; le vrai pack arrive avec `add-lingua-data-pack`._

- [ ] 2.1 Tokenisation UAX #29 + pré-passe anglaise (contractions, apostrophes de bord, mots d'une lettre du lexique) avec tests
- [ ] 2.2 Format FST formes→lemmes : lecture depuis un slice (`include_bytes!`-compatible), API de lookup, mini-FST de test
- [ ] 2.3 Cascade de lemmatisation : irréguliers → FST → repli morphy → repli pluriel hors-lexique → identité ; fixtures de non-régression (min. 100 cas, incluant `endeavors→endeavor`, `bigger→big`, `went→go`)
- [ ] 2.4 Détection de langue par bloc (`whichlang`) + règle « page non analysable » ; tests FR/EN mélangés
- [ ] 2.5 Calcul du % de tokens connus à partir de classifications fournies en entrée (occurrences, ignorés=connus, en-cours=inconnus, noms propres hors-lexique exclus) — l'intégration avec les statuts réels arrive avec `add-lingua-knowledge-model`
- [ ] 2.6 `analyzer_version` exposée + test de déterminisme sur corpus de fixtures
