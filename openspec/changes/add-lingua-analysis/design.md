# Design — add-lingua-analysis

## Context

Exploration produit complète menée en amont (2026-08-10) : étude concurrentielle (LingQ/Readlang/Migaku/Lute/jpdb), faisabilité par surface, et un **preshot fonctionnel** (`~/workspace/lingua-preshot`, extension MV3 JS pur) qui a validé la boucle produit sur pièces. Ce change industrialise le cœur du preshot : le pipeline d'analyse passe en Rust, dans un crate du monorepo, avec le déterminisme comme contrat.

Contraintes héritées du monorepo : cœurs host-testables (`*_core.rs`), coverage ≥ 80 %, pas de logique métier dans les coquilles. Contrainte produit actée : comptage honnête par lemme.

## Goals / Non-Goals

**Goals :**
- Un pipeline d'analyse complet et déterministe dans `crates/lingua-core` : texte → tokens → lemmes → classement → % connu, testé sur mini-fixtures.
- Les fondations monorepo du produit Lingua : préfixe OpenSpec, crate dans le workspace, lane CI/coverage.

**Non-Goals :**
- Statuts, calibration, profil L1/L2 (`add-lingua-knowledge-model`) ; decks/FSRS (`add-lingua-decks-review`) ; pack réel (`add-lingua-data-pack`) ; cible WASM (`add-lingua-wasm`) ; toute surface utilisateur (extension, app, plugin — changes ultérieurs de la pile).

## Decisions

### D1 — Arborescence : `crates/lingua-core` dans le workspace Cargo racine
Le crate rejoint le workspace Cargo racine et est donc couvert par la lane llvm-cov existante (fmt/clippy/coverage ≥ 80 % sans nouvelle lane). Layout de modules posé dès maintenant — `analysis/`, `knowledge/`, `decks/`, `packs/` — pour que les changes suivants remplissent des cases prévues au lieu de réorganiser. La logique reste pure et host-testée ; seuls les futurs bindings (wasm-bindgen) seront exclus du coverage, exclusion ajoutée dès ce change au `--ignore-filename-regex` (lane CI **et** commande documentée dans CLAUDE.md). Pas de Flutter, pas de Tauri.

### D2 — Tokenisation : UAX #29 + pré-passe par langue étudiée
Tokenisation `unicode-segmentation` (UAX #29), précédée d'une pré-passe **par langue** qui absorbe les particularités de surface — pour l'anglais : contractions (`don't` → `do` + `not`), apostrophes de bord retirées, mots d'une lettre comptés seulement s'ils appartiennent au lexique (« I », « a »). La pré-passe est le point d'extension pour les langues romanes (élisions/clitiques) : ajouter une langue = ajouter une pré-passe, pas toucher le tokeniseur.

### D3 — Lemmatisation anglaise : FST AGID + morphy en repli + repli pluriel hors-lexique
Pipeline : exceptions/irréguliers → lookup FST (AGID inversé formes→lemmes, ~0,5 Mo) → morphy (règles WordNet, crate `wordnet-lemmatizer` ou port interne) → repli pluriel simple pour les mots hors lexique (leçon du preshot : `endeavors`→`endeavor`, sinon le comptage ment). Alternatives rejetées : `rust-stemmers` (des stems, pas des lemmes — jamais montrables), `nlprule` (binaires LGPL lourds), spacy-lookups-data EN (couverture inférieure à AGID). Dans ce change le FST est lu depuis un slice (`include_bytes!`-compatible) sur des mini-FST de test ; le format conteneur et le pack réel arrivent avec `add-lingua-data-pack`.

### D4 — Détection de langue par bloc : `whichlang`
La détection se fait **par bloc de texte**, pas par document : les pages réelles mélangent les langues (UI française, citations, code). Un bloc hors langue étudiée est exclu de l'analyse — dont les blocs en langue maternelle ; un document sans contenu suffisant dans la langue étudiée est « non analysable ». `whichlang` : Rust pur, rapide, sans données externes — compatible avec la future cible WASM.

### D5 — Déterminisme contractuel : `analyzer_version`
À `analyzer_version` égale et pack égal, sortie identique octet pour octet, quelle que soit la cible de compilation. C'est un contrat, pas un vœu : le futur TextProfile communautaire, la sync et la fusion des stores en dépendent (une analyse non reproductible rend les comptes incomparables). Testé dès ce change par un test de déterminisme sur corpus de fixtures (double exécution native) ; le test de **parité croisée natif/WASM** arrive avec `add-lingua-wasm` (même `analyzer_version` ⇒ sorties identiques).

## Risks / Trade-offs

- [Qualité lemmatiseur v1 (AGID+morphy sans POS)] → suffisant pour le comptage (l'ambiguïté se résout côté knowledge model, pro-apprenant) ; les erreurs résiduelles sont le différenciateur du pack v2, pas un bloquant MVP. Fixtures de non-régression dès le jour 1 (min. 100 cas).
- [Tests sur mini-fixtures seulement] → assumé : la qualité *du pipeline* (cascade, déterminisme) se teste sur fixtures synthétiques ; la qualité *des données* se validera avec le vrai pack (`add-lingua-data-pack`), qui réutilisera les mêmes fixtures de non-régression.
- [Layout de modules posé avant leurs changes] → coût quasi nul (dossiers + `mod` vides ou minimaux) contre un bénéfice réel : les 11 changes suivants ne déplacent pas de code.

## Migration Plan

Rien à migrer (nouveau crate, aucune surface). Rollback = retirer le crate du workspace.

## Open Questions

- `wordnet-lemmatizer` (crate existant) vs port interne des règles morphy — trancher à l'implémentation selon l'état du crate (licence Apache-2.0 confirmée, mais maintenance à vérifier) ; le contrat de la cascade ne bouge pas.
