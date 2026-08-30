# add-lingua-mvp — Cymbra Lingua : cœur d'analyse, extension navigateur, capture Claude Code

## Why

Les produits existants de lecture-en-langue-étrangère (LingQ, Readlang, Migaku) comptent les formes de surface au lieu des lemmes — leurs compteurs « mots connus » mentent — et enferment la lecture dans leur silo (copie de la page dans leur reader). Trois gaps de marché sont confirmés par l'étude concurrentielle : le comptage honnête par lemme, la lecture instrumentée **sur place** (jamais de silo), et l'ingestion du corpus quotidien du développeur (sessions d'agents IA) qu'aucun produit n'exploite. Cymbra Lingua occupe ces trois gaps, avec un cœur Rust unique compilé pour chaque surface.

Ce premier change livre un produit **entièrement local, sans compte ni backend** : la valeur se valide sur l'utilisateur fondateur avant tout investissement serveur. La sync Cymbra ID (audience `lingua`), l'extension Safari/iOS, l'app mobile et le catalogue de livres sont des changes ultérieurs déjà explorés.

## What Changes

- **Nouveau produit et nouveau préfixe OpenSpec `lingua-*`** (ajouté à `openspec/config.yaml`). Ce change ne touche aucune capability existante.
- **Nouveau crate `lingua-core`** dans le workspace Cargo : tokenisation, lemmatisation par table FST + repli morphologique, détection de langue par bloc, knowledge model lemma-first, decks/cartes avec provenance, planification FSRS, calibration du démarrage à froid, import LingQ, export Anki. Compilé en natif (CLI/plugin) et en WASM (extension). Logique pure host-testable (convention `*_core.rs`).
- **Nouvelle extension navigateur MV3** (`apps/lingua-extension`) : surlignage des mots inconnus via CSS Custom Highlight API (zéro mutation du DOM), % de mots connus par page (badge + popup), popup de mot (forme du dictionnaire, glose, rareté, statuts), capture de sélection multi-mots au raccourci clavier avec phrase d'origine, side panel deck/révision (page poussée) + drawer overlay injecté (repli), calibration par curseur. Tout l'état en `chrome.storage.local`.
- **Nouveau plugin Claude Code** (`apps/lingua-agent`) : binaire Rust `lingua` + hook `Stop` (ingestion des transcripts JSONL), statusline « % connus · N nouveaux », skill `/vocab`, serveur MCP pour les opérations de deck. Ingestion **local-only par construction** (les transcripts sont confidentiels). Architecture `SessionSource` extensible aux autres agents (Codex, Aider — hors périmètre de ce change).
- **Pack de données anglais → français** construit hors-ligne (AGID en FST, fréquences wordfreq, gloses kaikki) : format de pack versionné, clé par paire (langue étudiée → langue maternelle), pile de notices de licences embarquée.
- Vocabulaire UI : le mot « lemme » n'apparaît **jamais** à l'écran (« forme du dictionnaire », « mots différents »).

## Capabilities

### New Capabilities
- `lingua-analysis` : pipeline d'analyse de texte — tokenisation, pré-passe par langue (élisions/clitiques), lemmatisation (FST + repli + repli pluriel hors-lexique), détection de langue par bloc, calcul du % de tokens connus. Déterministe à `analyzer_version` donnée.
- `lingua-knowledge-model` : l'état de connaissance par lemme et par langue étudiée — statuts (nouveau/en cours/connu/ignoré), « connu » inférable du SRS, calibration par rang de fréquence au démarrage, import LingQ (CSV), profil L1/L2 (langue maternelle ≠ langue étudiée, tout est clé par paire).
- `lingua-decks-review` : decks et cartes — carte = lemme + forme vue + phrase de provenance + source + média optionnel (schéma jour 1, capture d'image différée), révision FSRS, export Anki sans perte, expressions multi-mots.
- `lingua-data-packs` : format des packs de données par paire (L2→L1) — FST formes→lemmes, fréquences, gloses — versionnés, avec attributions de licences (AGID/wordfreq CC BY-SA/kaikki CC BY-SA) ; pipeline de construction hors-ligne reproductible.
- `lingua-browser-extension` : l'expérience navigateur — surlignage in-place, % par page, popup de mot, capture de sélection, side panel/drawer de révision, calibration, garde-fous (jamais de « lemme » à l'écran, dégradation douce, pages non-anglaises ignorées).
- `lingua-agent-capture` : l'ingestion des sessions d'agents IA — hook Claude Code, statusline, `/vocab`, MCP decks, trait `SessionSource`, local-only par défaut.

### Modified Capabilities
_Aucune. Ce change est local-first : il ne consomme ni ne modifie `id-*`/`platform-*` (la sync et l'audience `lingua` arriveront avec le change de sync backend)._

## Impact

- **Produits** : Lingua (nouveau — tout est nouveau) ; **Cymbra ID / Music / Live / back-office : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés). Ce change ne consomme le socle qu'à un endroit : la convention de coverage et la CI.
- **Arborescence** : `crates/lingua-core` (workspace Cargo racine), `apps/lingua-extension` (TS + wasm-pack, Yarn), `apps/lingua-agent` (binaire Rust + manifeste plugin Claude Code), `scripts/lingua-data/` (build des packs). `openspec/config.yaml` : ajout du domaine `lingua-*`.
- **CI** : lane Rust existante couvre `lingua-core` (llvm-cov ≥ 80 %) ; nouvelles lanes légères : build wasm-pack + vitest pour l'extension (mêmes conventions que `apps/back-office`).
- **Dépendances nouvelles** : `fst`, `unicode-segmentation`, `whichlang`, `fsrs` (Rust) ; wasm-bindgen/wasm-pack ; données AGID + wordfreq + kaikki (buildées hors-ligne, non commitées brutes).
- **Hors périmètre (changes ultérieurs déjà explorés)** : backend `lingua` + audience Cymbra ID + sync multi-device ; extension Safari + app conteneur iOS ; app browser mobile ; catalogue de livres/TextProfile communautaire ; OCR/capture d'image ; adaptateurs Codex/Aider ; packs langues romanes.
