# Tasks — add-lingua-mvp

## 1. Fondations monorepo

- [ ] 1.1 Déclarer le domaine `lingua-*` dans `openspec/config.yaml` (table des préfixes + règles par artefact si besoin)
- [ ] 1.2 Créer `crates/lingua-core` (lib) dans le workspace Cargo racine ; dépendances `unicode-segmentation`, `fst`, `whichlang`, `fsrs`, `serde` ; module layout `analysis/`, `knowledge/`, `decks/`, `packs/`
- [ ] 1.3 Brancher le crate sur la lane CI Rust (fmt/clippy/llvm-cov) et ajouter l'exclusion du glue wasm-bindgen au `--ignore-filename-regex` (lane CI **et** commande documentée dans CLAUDE.md) — la logique reste host-testée, seuls les bindings sont exclus

## 2. lingua-core — analyse (spec lingua-analysis)

_Les sections 2-4 testent sur des mini-fixtures synthétiques (FST/fréquences de test) ; le vrai pack arrive en section 5._

- [ ] 2.1 Tokenisation UAX #29 + pré-passe anglaise (contractions, apostrophes de bord, mots d'une lettre du lexique) avec tests
- [ ] 2.2 Format FST formes→lemmes : lecture depuis un slice (`include_bytes!`-compatible), API de lookup, mini-FST de test
- [ ] 2.3 Cascade de lemmatisation : irréguliers → FST → repli morphy → repli pluriel hors-lexique → identité ; fixtures de non-régression (min. 100 cas, incluant `endeavors→endeavor`, `bigger→big`, `went→go`)
- [ ] 2.4 Détection de langue par bloc (`whichlang`) + règle « page non analysable » ; tests FR/EN mélangés
- [ ] 2.5 Calcul du % de tokens connus à partir de classifications fournies en entrée (occurrences, ignorés=connus, en-cours=inconnus, noms propres hors-lexique exclus) — l'intégration avec les statuts réels arrive en 3.x
- [ ] 2.6 `analyzer_version` exposée + test de déterminisme sur corpus de fixtures

## 3. lingua-core — knowledge model (spec lingua-knowledge-model)

- [ ] 3.1 Types statuts (`learning/known/ignored` + provenance `manual/calibration/srs/import`) et résolution connu-implicite par rang ≤ calibration ; le statut explicite prime
- [ ] 3.2 Résolution multi-candidats (connu si un candidat l'est) avec tests
- [ ] 3.3 Profil L1/L2 : `native_language` + langues étudiées ; toutes les API clées par paire ; tests avec paire factice
- [ ] 3.4 Import LingQ (CSV) → statuts `known` provenance `import`, entrées lemmatisées ; test sur échantillon réel anonymisé
- [ ] 3.5 Compteurs d'exposition par (langue, lemme) : incréments à l'ingestion, source + horodatage, sans effet sur les statuts ; tests

## 4. lingua-core — decks et révision (spec lingua-decks-review)

- [ ] 4.1 Schéma de carte (lemme, forme, phrase, source, glose, `media` optionnel avec `source`/`sync_policy`, état FSRS) sérialisable versionné
- [ ] 4.2 Intégration FSRS : notation `again/hard/good/easy`, échéances, compteur de dues ; version du crate épinglée, paramètres stockés sur l'état
- [ ] 4.3 « Je connais » en révision → statut `known` provenance `srs`, carte conservée hors file ; tests
- [ ] 4.4 Export Anki CSV (une colonne par champ du schéma, colonnes vides pour les champs non peuplés) avec tests de fidélité des champs

## 5. Packs de données (spec lingua-data-packs)

- [ ] 5.1 `scripts/lingua-data/` : pipeline reproductible (sources datées, données brutes non commitées) ; téléchargement AGID + export wordfreq + extrait kaikki fr-glosses
- [ ] 5.2 Construction du FST (AGID inversé) + table de fréquence (rangs quantisés) + `gloss.zst` offset-indexé (top lemmes, budget)
- [ ] 5.3 Format conteneur `pack.lingua` (magic, TOC, meta avec `pack_version`/`analyzer_version`/licences, NOTICE) + lecteur dans `lingua-core` avec refus des versions incompatibles
- [ ] 5.4 Garde-fous licences : liste noire GPL/AGPL/NC documentée + vérification du NOTICE au build ; échec de build si pack > 5 Mo
- [ ] 5.5 Test de reproductibilité (double build identique) ; le pack (en→fr) est construit en CI et mis en cache (jamais commité), build local documenté

## 6. Cible WASM

- [ ] 6.1 Crate/feature `lingua-wasm` : bindings wasm-bindgen (analyse par lot de blocs → tokens classés, statuts, %, gloses) ; build wasm-pack `--target web`
- [ ] 6.2 Test de parité natif/WASM sur les fixtures (même `analyzer_version` ⇒ sorties identiques)
- [ ] 6.3 Lane CI : build wasm + exécution des tests de parité

## 7. Extension navigateur (spec lingua-browser-extension)

- [ ] 7.1 Scaffold `apps/lingua-extension` : MV3, TS sans framework, Yarn, vitest, esbuild/vite build ; manifest `activeTab` + `optional_host_permissions <all_urls>` + `storage` + `sidePanel` + commandes clavier
- [ ] 7.2 `AnalyzerPort` (messages) + instanciation lazy du WASM dans le content script ; mémoïsation par forme
- [ ] 7.3 Moteur de surlignage : TreeWalker → Ranges → deux registres Highlight ; styles `::highlight()` ; exclusions (script/style/éditable/hôtes d'UI)
- [ ] 7.4 Re-scan par sous-arbre muté (MutationObserver débouncé) + IntersectionObserver pour prioriser le visible ; test manuel sur 5 SPA lourdes documenté
- [ ] 7.5 Popup de mot (shadow DOM fermé) : forme du dictionnaire, forme vue, glose du pack, rareté vulgarisée, actions statuts ; propagation cross-onglets via `storage.onChanged` ; AUCUNE occurrence du mot « lemme » (test de lint des chaînes UI)
- [ ] 7.6 Capture de sélection au raccourci : extraction de la phrase d'origine, carte d'expression
- [ ] 7.7 Badge % par onglet + popup d'icône (stats, compteur de cartes dues, curseur de calibration, reset)
- [ ] 7.8 Side panel (deck, session de révision FSRS, réponse masquée/révélée) + drawer injecté replié partageant la même logique
- [ ] 7.9 Storage versionné avec migrations + réinitialisation ; export Anki depuis le side panel
- [ ] 7.10 Page attributions (NOTICE du pack) + privacy note « rien ne quitte l'appareil »
- [ ] 7.11 Vérification manuelle : load unpacked, parcours complet (calibration → lecture → +Deck → révision → export) sur 5 sites réels

## 8. Plugin Claude Code (spec lingua-agent-capture)

- [ ] 8.1 Binaire `apps/lingua-agent` : `lingua-core` natif + store SQLite `~/.lingua/` (schéma versionné) ; sous-commandes `ingest`, `statusline`, `vocab`, `mcp`
- [ ] 8.2 Trait `SessionSource` + impl Claude Code (parse JSONL, extraction texte assistant) ; tests sur transcripts factices ; invariant testé : aucune phrase persistée, aucune connexion réseau
- [ ] 8.3 Hook `Stop` (manifeste plugin) → `lingua ingest --transcript <path>` ; idempotence par offset de transcript
- [ ] 8.4 Statusline : % du dernier message + nouveaux de la session ; dégradation silencieuse
- [ ] 8.5 Skill `/vocab` : liste des inconnus de session avec gloses, ajout au deck avec consentement (phrase d'origine incluse à ce moment-là seulement)
- [ ] 8.6 Serveur MCP (`list_decks`, `add_words`, `due_cards`, `answer_card`) avec validation d'entrées ; test d'intégration bout-en-bout incluant la révision conversationnelle
- [ ] 8.7 Manifeste plugin Claude Code (hooks + statusline + skill + MCP) + doc d'installation ; documenter la configuration manuelle de la statusline si le manifeste ne peut pas l'installer

## 9. Gates et finitions

- [ ] 9.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (avec le `--ignore-filename-regex` mis à jour en 1.3)
- [ ] 9.2 vitest vert sur `apps/lingua-extension` ; lint « pas de “lemme” » sur les chaînes UI de l'extension ET les sorties utilisateur du plugin (statusline, `/vocab`, MCP)
- [ ] 9.3 README produit (installation extension + plugin, philosophie local-first, limites connues)
- [ ] 9.4 `openspec validate add-lingua-mvp --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
