# add-lingua-data-pack — Cymbra Lingua : pack de données anglais → français

## Why

Le cœur d'analyse (`add-lingua-analysis`) et les étages qui le suivent testent sur des mini-fixtures synthétiques ; pour analyser du vrai texte il faut de vraies données : le FST formes→lemmes, les rangs de fréquence et les gloses. Ce change livre le **pack de données anglais → français** construit hors-ligne (AGID en FST, fréquences wordfreq, gloses kaikki) : format de pack versionné, clé par paire (langue étudiée → langue maternelle), pile de notices de licences embarquée. C'est aussi le change qui fixe l'hygiène de licences du produit (données à usage commercial autorisé, liste noire GPL/NC) et le budget de taille qui rend le pack embarquable dans une extension.

**Position dans la pile (4/12)** : add-lingua-analysis → add-lingua-knowledge-model → add-lingua-decks-review → **add-lingua-data-pack** → add-lingua-wasm → add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox → add-lingua-apple → add-lingua-agent → add-lingua-backend → add-lingua-connected-clients. **Prérequis explicite : `add-lingua-analysis`** (le cœur lit le format : la cascade de lemmatisation consomme le FST, le % consomme les rangs, et l'`analyzer_version` exposée par le cœur gate la compatibilité des packs).

## What Changes

- **Pipeline `scripts/lingua-data/`** : construction hors-ligne reproductible (sources datées, données brutes non commitées) — téléchargement AGID, export wordfreq, extrait kaikki fr-glosses ; construction du FST (AGID inversé), de la table de fréquence (rangs quantisés) et du `gloss.zst` offset-indexé (top lemmes, budget).
- **Format conteneur `pack.lingua`** : magic + TOC — `meta` (paire, `pack_version`, `analyzer_version` compatible, licences), `forms.fst`, `lemmas.bin`, `freq.bin`, `gloss.zst`, `NOTICE` — et **lecteur dans `lingua-core`** avec refus des versions incompatibles.
- **Garde-fous licences** : liste noire GPL/AGPL/NC documentée dans `scripts/lingua-data`, vérification du NOTICE au build.
- **Budget de taille** : échec de build si pack > 5 Mo ; la remédiation réduit la couverture des gloses, jamais le FST ni les fréquences.
- **Le pack n'est jamais commité** : reconstruit en CI (déterminisme testé) et mis en cache ; le dev local le construit une fois via le script.
- MVP : un seul pack (EN→FR), mais **tout le code est pair-keyed** — ajouter (ES→FR) = données, pas du code.

## Capabilities

### New Capabilities
- `lingua-data-packs` : format des packs de données par paire (L2→L1) — FST formes→lemmes, fréquences, gloses — versionnés, avec attributions de licences (AGID/wordfreq CC BY-SA/kaikki CC BY-SA) ; pipeline de construction hors-ligne reproductible.

### Modified Capabilities
_Aucune. `lingua-analysis` consomme le pack via ses API existantes (lookup FST, rangs de fréquence) — le contrat d'analyse ne bouge pas. La page « Attributions » exigée par la spec est un contrat de la capability, réalisé côté extension par `add-lingua-extension-reading`._

## Impact

- **Produits** : Lingua (données + lecteur de pack — rien de consommé hors de la pile Lingua) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés).
- **Arborescence** : `scripts/lingua-data/` (nouveau pipeline), `crates/lingua-core` (module `packs/` : lecteur du conteneur). Aucune nouvelle unité `apps/*`/`packages/*`/`crates/*` — rien à ajouter à `ci-units`.
- **CI** : la lane Rust existante couvre le lecteur (llvm-cov ≥ 80 %) ; le pack (en→fr) est construit en CI et mis en cache (jamais commité), avec test de reproductibilité.
- **Dépendances nouvelles** : données AGID + wordfreq + kaikki (buildées hors-ligne, non commitées brutes) ; compression zstd pour les gloses.
- **Hors périmètre** : la page « Attributions » dans l'extension (`add-lingua-extension-reading`), l'embarquement du pack dans le module WASM (`add-lingua-wasm`) et dans le bundle Apple (`add-lingua-apple`), les packs langues romanes (fr/it/es/pt — le format pair-keyed les attend).
