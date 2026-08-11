# Design — add-lingua-mvp

## Context

Exploration produit complète menée en amont (2026-08-10) : étude concurrentielle (LingQ/Readlang/Migaku/Lute/jpdb), faisabilité par surface (7 + 4 + 2 agents de recherche), et un **preshot fonctionnel** (`~/workspace/lingua-preshot`, extension MV3 JS pur) qui a validé sur pièces : le rendu CSS Custom Highlight API sur page réelle, la boucle popup → statuts → repaint, la calibration par curseur, la capture de sélection, le side panel. Ce change industrialise ce preshot : le cœur passe en Rust, les données deviennent des packs licenciés proprement, et la capture Claude Code s'ajoute.

Contraintes héritées du monorepo : cœurs host-testables (`*_core.rs`), coverage ≥ 80 %, Yarn pour le JS, pas de logique métier dans les coquilles. Contraintes produit actées : jamais de silo de lecture, jamais « lemme » à l'écran, local-only par défaut (les transcripts Claude Code sont confidentiels), comptage honnête par lemme.

## Goals / Non-Goals

**Goals :**
- Un utilisateur (le fondateur d'abord) lit le web en anglais avec surlignage des mots inconnus, % par page honnête, decks et révision FSRS — **sans compte, sans réseau** (hors définitions du preshot, remplacées par les gloses du pack).
- Chaque réponse de Claude Code alimente automatiquement les compteurs d'exposition ; `/vocab` et la statusline exposent l'état.
- Un seul cerveau : la même analyse (même `analyzer_version`) produit les mêmes résultats dans l'extension (WASM) et le plugin (natif).

**Non-Goals :**
- Sync multi-device, comptes, backend, audience Cymbra ID (change suivant).
- Safari/Firefox/iOS/mobile (changes suivants ; l'architecture les anticipe : cible WASM propre, event-page-compatible).
- Catalogue de livres, TextProfile, OCR/images, langues romanes, adaptateurs d'autres agents (Codex/Aider), traduction automatique d'expressions (la carte d'expression stocke la phrase ; la MT viendra avec le seam `TranslatorPort`).

## Decisions

### D1 — Knowledge model : clé `(langue étudiée, lemme)`, sans POS, statuts à 4 états
- Clé = lemme texte normalisé par langue étudiée ; les expressions multi-mots sont des lemmes avec espaces. Le **POS ne fait pas partie de la clé** en v1 : l'ambiguïté (`can` nom/verbe) se résout en faveur de l'apprenant — connu si un candidat l'est — et le popup montre tous les sens. Alternative rejetée : clé (lemme, POS) — double la taille des tables, exige un tagger (qualité/poids), et le gain pédagogique est marginal au niveau visé.
- Statuts : `learning`, `known`, `ignored` explicites ; « nouveau » est l'absence d'entrée ; `known` est aussi **implicite** sous le seuil de calibration (rang de fréquence ≤ N). `ignored` compte comme connu dans le %.
- Le champ `known_source` (`manual | calibration | srs | import`) prépare l'inférence SRS façon Migaku sans la câbler.

### D2 — Lemmatisation anglaise : FST AGID + morphy en repli + repli pluriel hors-lexique
Pipeline : exceptions/irréguliers → lookup FST (AGID inversé formes→lemmes, ~0,5 Mo) → morphy (règles WordNet, crate `wordnet-lemmatizer` ou port interne) → repli pluriel simple pour les mots hors lexique (leçon du preshot : `endeavors`→`endeavor`, sinon le comptage ment). Alternatives rejetées : `rust-stemmers` (des stems, pas des lemmes — jamais montrables), `nlprule` (binaires LGPL lourds), spacy-lookups-data EN (couverture inférieure à AGID). **Déterminisme contractuel** : à `analyzer_version` égale, sortie identique octet pour octet — testé en CI (fixtures croisées natif/WASM).

### D3 — Packs de données par paire (L2→L1), format conteneur versionné
`pack.lingua` = magic + TOC : `meta` (paire, versions, licences), `forms.fst`, `lemmas.bin`, `freq.bin` (rangs quantisés wordfreq), `gloss.zst` (gloses FR extraites de kaikki, offset-indexées, top ~30 k lemmes), `NOTICE`. Construit hors-ligne par `scripts/lingua-data` (reproductible, données brutes non commitées) ; le pack lui-même n'est **pas commité** : il est reconstruit en CI (déterminisme testé) et mis en cache, le dev local le construit une fois via le script. Le profil utilisateur porte `native_language` (défaut : locale système ; plus tard locale Cymbra ID) — l'UI, les gloses et la future MT en dépendent. MVP : un seul pack (EN→FR), mais **tout le code est pair-keyed** — ajouter (ES→FR) = données, pas du code.

### D4 — Rendu : CSS Custom Highlight API, interaction par `caretRangeFromPoint`
Deux registres (`lingua-unknown`, `lingua-learning`), zéro mutation du DOM (pas de guerre avec React/hydration — validé par le preshot). Clic : `caretRangeFromPoint` → index de tokens trié (portable, Safari-compatible plus tard) ; `highlightsFromPoint` (Chromium 140+) en amélioration progressive. Repli `<span>` non implémenté en v1 (toutes les cibles MVP supportent l'API). MutationObserver débouncé avec re-scan **par sous-arbre muté** (le preshot re-scannait tout — suffisant pour juger, pas pour Gmail).

### D5 — Où vit le WASM : dans le content script
Le monde isolé de Chrome autorise `wasm-unsafe-eval` ; le module (code wasm ~1 Mo + pack EN ≤ 5 Mo) s'instancie par onglet, zéro IPC pour analyser. Le service worker ne garde que l'orchestration (badge, commandes, side panel). Les résultats par forme sont mémoïsés côté JS. Alternative rejetée : WASM dans le SW (aller-retours par page, SW tué à 30 s). L'analyse est exposée derrière un **`AnalyzerPort`** par messages malgré tout — c'est la couture qui permettra Firefox (WASM en event page) et Safari (nativeMessaging) sans toucher au content script.

### D6 — Permissions : `activeTab` + `optional_host_permissions <all_urls>`
Install sans avertissement effrayant ; « surligner cette page » marche immédiatement ; « toujours surligner » demande le grant une fois. Aligne Chrome sur le modèle imposé par Firefox/Safari et dé-risque la review du store. Le badge % et le side panel fonctionnent dans les deux modes.

### D7 — Révision : FSRS via le crate `fsrs`, deux surfaces d'UI
L'état FSRS (stabilité, difficulté, échéance) vit sur la carte, calculé dans `lingua-core` — le même planning partout. UI : side panel (Side Panel API — la page est poussée, survit aux navigations) par défaut ; drawer overlay injecté (shadow DOM fermé) en repli et pour les micro-révisions. Le badge de l'icône reste dédié au % de la page ; le compteur de cartes dues est visible dans le popup de l'icône et le side panel (pas de badge « dues » ni d'alarme en v1). Export Anki : CSV v1 avec **tous les champs peuplés** de la carte (mot, forme, phrase, glose, source, statut, échéance, état FSRS ; champ non peuplé = colonne vide) — jamais de perte silencieuse.

### D8 — Plugin Claude Code : un binaire, quatre branchements
`lingua` (binaire Rust, `lingua-core` natif) + manifeste plugin : hook `Stop` (lit `transcript_path`, extrait le texte assistant des JSONL, ingère les expositions), statusline (lit le même transcript, affiche `📖 96 % · 3 nouveaux`), skill `/vocab` (liste les inconnus de la session, gloses, ajout au deck), serveur MCP (`list_decks`, `add_words`, `due_cards`, `answer_card`). La révision côté plugin passe par l'agent lui-même (quiz conversationnel : `due_cards` → questions → `answer_card`) — pas de TUI dédiée en v1, les cartes du store plugin restent ainsi révisables et exportables. Les données vivent dans `~/.lingua/` (SQLite via `rusqlite`) — **jamais de contenu de transcript persisté, seulement lemmes + compteurs + phrases explicitement capturées par l'utilisateur**. Trait `SessionSource` : une impl `ClaudeCode` en v1, le trait est le contrat pour Codex/Aider ensuite.

### D9 — État extension : `chrome.storage.local`, schéma versionné
Statuts (map lemme→statut compact), cartes (JSON), calibration, préférences — sous une clé racine versionnée avec migration. Le pack est un asset de l'extension (pas dans storage). L'extension et le plugin Claude Code ont chacun leur store local en v1 ; la **réconciliation est le problème du change de sync**, pas de celui-ci (décision assumée : pas de bridge natif extension↔plugin en v1 — un faux sync local serait du travail jeté).

### D10 — Monorepo : `crates/lingua-core`, `apps/lingua-extension`, `apps/lingua-agent`
Le crate rejoint le workspace Cargo racine (couvert par la lane llvm-cov existante). L'extension : TS sans framework (le DOM injecté n'a pas besoin de Vue), wasm-pack `--target web`, vitest pour la logique TS, build Yarn. Pas de Flutter, pas de Tauri dans ce change.

## Risks / Trade-offs

- [Perf du re-scan sur SPA lourdes (Gmail, feeds virtualisés)] → re-scan par sous-arbre + IntersectionObserver (analyse du visible d'abord) + budget par frame ; page pathologique = dégradation douce (surlignage partiel), jamais de jank imputable à l'extension.
- [Taille WASM + pack (~6 Mo) instanciée par onglet] → mémoïsation par forme, instanciation lazy (au premier bloc anglais détecté), un seul module partagé par frame principale ; mesurer avant d'optimiser (cible < 50 ms d'init).
- [Qualité lemmatiseur v1 (AGID+morphy sans POS)] → suffisant pour le comptage (ambiguïté résolue pro-apprenant) ; les erreurs résiduelles sont le différenciateur du pack v2, pas un bloquant MVP. Fixtures de non-régression dès le jour 1.
- [Deux stores locaux (extension / plugin) non réconciliés] → assumé : le change de sync les fusionnera côté serveur ; les schémas partagent le même vocabulaire de types (`lingua-core`) pour que la fusion soit mécanique.
- [Licences données (CC BY-SA wordfreq/kaikki, notices AGID)] → `NOTICE` embarqué dans le pack + page attributions dans l'extension ; les tables dérivées sont publiées (share-alike satisfait) ; rien de GPL/NC n'entre dans le build (liste noire documentée dans `scripts/lingua-data`).
- [Store review Chrome (`<all_urls>` optionnel, lecture de texte de page)] → posture D6 + privacy policy « le texte lu ne quitte jamais l'appareil » (vraie par construction en v1).
- [Le crate `fsrs` évolue (paramètres par défaut)] → épingler la version ; les états FSRS stockent leurs paramètres.

## Migration Plan

Rien à migrer (nouveau produit, local-only). Rollback = désinstaller l'extension/le plugin. Le schéma de storage versionné (D9) prépare les migrations futures. Le preshot (`~/workspace/lingua-preshot`) reste un jouet jetable — aucune donnée à reprendre.

## Open Questions

- Distribution du plugin Claude Code (marketplace plugin vs repo git) — à trancher à la livraison, sans impact sur l'architecture.
- Publier l'extension sur le Chrome Web Store dès ce change (unlisted) ou rester en « load unpacked » jusqu'au change de sync — recommandation : unlisted dès que stable, pour roder la review.
- Le seuil exact de gloses embarquées (top 20 k vs 30 k lemmes) — mesurer la taille réelle du `gloss.zst` et arbitrer sous 5 Mo de pack total.
