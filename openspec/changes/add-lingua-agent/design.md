# Design — add-lingua-agent

## Context

Les briques amont de la pile fournissent tout le nécessaire : la cascade de lemmatisation et l'`analyzer_version` contractuelle (`add-lingua-analysis`), les statuts et compteurs d'exposition (`add-lingua-knowledge-model`), les cartes et la planification FSRS (`add-lingua-decks-review`), le pack EN→FR et ses gloses (`add-lingua-data-pack`). Ce change ne fait que brancher ce cerveau, compilé en natif, sur les sessions Claude Code. L'exploration amont a établi les contraintes : le surlignage in-place du TUI est **impossible** (aucun post-processing du rendu) ; ce qui est viable est le hook `Stop` (reçoit `transcript_path` JSONL), la statusline, une skill et un serveur MCP. Les transcripts sont confidentiels (code employeur) → local-only par défaut, non négociable.

Décisions héritées non rediscutées ici : clé `(langue étudiée, lemme)` sans POS et compteurs d'exposition (`add-lingua-knowledge-model`), FSRS via le crate `fsrs` épinglé (`add-lingua-decks-review`), déterminisme à `analyzer_version` donnée (`add-lingua-analysis`).

## Goals / Non-Goals

**Goals :**
- Chaque réponse de Claude Code alimente automatiquement les compteurs d'exposition ; `/vocab` et la statusline exposent l'état.
- Un seul cerveau : la même analyse (même `analyzer_version`) produit les mêmes résultats dans l'extension (WASM) et le plugin (natif).
- Les cartes du store plugin restent révisables (via l'agent) et exportables.

**Non-Goals :**
- Sync du store `~/.lingua/` (les transcripts sont confidentiels ; la sync des autres surfaces est `add-lingua-backend`/`add-lingua-connected-clients`, le plugin y est explicitement exclu).
- Adaptateurs d'autres agents (Codex, Aider, Gemini) — le trait `SessionSource` est le contrat, les impls viendront.
- TUI de révision dédiée, surlignage dans le terminal.

## Decisions

### D1 — Plugin Claude Code : un binaire, quatre branchements
`lingua` (binaire Rust, `lingua-core` natif) + manifeste plugin : hook `Stop` (lit `transcript_path`, extrait le texte assistant des JSONL, ingère les expositions), statusline (lit le même transcript, affiche `📖 96 % · 3 nouveaux`), skill `/vocab` (liste les inconnus de la session, gloses, ajout au deck), serveur MCP (`list_decks`, `add_words`, `due_cards`, `answer_card`). La révision côté plugin passe par l'agent lui-même (quiz conversationnel : `due_cards` → questions → `answer_card`) — pas de TUI dédiée en v1, les cartes du store plugin restent ainsi révisables et exportables.

### D2 — Local-only par construction
Les données vivent dans `~/.lingua/` (SQLite via `rusqlite`, schéma versionné) — **jamais de contenu de transcript persisté, seulement lemmes + compteurs + phrases explicitement capturées par l'utilisateur**. Le binaire n'ouvre aucune connexion réseau ; l'invariant est testé (et le restera lors des changes de sync : le plugin y est hors périmètre). Le store plugin et le store extension ne sont **pas réconciliés** dans la pile locale (décision héritée du découpage : un faux sync local serait du travail jeté) — les schémas partagent les types de `lingua-core` pour que la fusion soit mécanique le jour venu.

### D3 — Trait `SessionSource` : Claude Code est la première impl, pas la seule
L'ingestion passe par une abstraction `SessionSource` (localisation des sessions, extraction du texte assistant) dont Claude Code est la première implémentation. Un nouvel agent (Codex, Aider) s'ajoute par une impl du trait sans modifier le pipeline d'analyse. La couche interactive (decks, révision) est le serveur MCP — la seule intégration standardisée, portable sur tous les agents ; l'affichage riche (statusline live) reste Claude-Code-only.

## Risks / Trade-offs

- [Store plugin et store extension divergent (deux états locaux)] → assumé et hérité de la pile locale ; la fusion est le problème des changes de sync (qui excluent le plugin — divergence documentée dans leur UI de stats : « hors sessions d'agents »).
- [Formats de transcript Claude Code non contractuels (JSONL interne)] → parsing défensif, dégradation silencieuse (statusline muette, ingestion sautée), idempotence par offset de transcript ; fixtures factices en tests plutôt que transcripts réels.
- [Le manifeste plugin ne peut peut-être pas installer la statusline] → documenter la configuration manuelle (tâche dédiée).

## Migration Plan

Rien à migrer (nouveau livrable, local-only). Rollback = désinstaller le plugin ; le store `~/.lingua/` reste sur disque, supprimable par l'utilisateur. Le schéma SQLite versionné prépare les migrations futures.

## Open Questions

- Distribution du plugin Claude Code (marketplace plugin vs repo git) — à trancher à la livraison, sans impact sur l'architecture.
