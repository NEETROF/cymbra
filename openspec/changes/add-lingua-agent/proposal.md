# add-lingua-agent — Cymbra Lingua : plugin Claude Code (capture des sessions d'agents)

## Why

Le troisième gap de marché confirmé par l'étude concurrentielle — l'ingestion du corpus quotidien du développeur (sessions d'agents IA) qu'aucun produit n'exploite — n'est couvert par aucune étape précédente de la pile : le cœur (`lingua-core`) et l'extension navigateur savent analyser et réviser, mais chaque réponse de Claude Code reste invisible des compteurs d'exposition. Ce change livre le plugin Claude Code complet, **local-only par construction** (les transcripts sont du code employeur, confidentiels) : le même cerveau d'analyse (même `analyzer_version`) que l'extension, branché sur les sessions de l'agent.

**Position dans la pile** (12 changes) : **10e** — après `add-lingua-apple`, avant `add-lingua-backend`. **Prérequis explicites : `add-lingua-decks-review`** (et transitivement `add-lingua-analysis` + `add-lingua-knowledge-model` : cascade de lemmatisation, statuts, cartes FSRS) ainsi que `add-lingua-data-pack` (pack EN→FR pour les gloses). **Parallélisable avec la branche extension** (`add-lingua-extension-reading` → `add-lingua-firefox` → `add-lingua-apple`) : aucune dépendance dans un sens ni dans l'autre.

## What Changes

- **Nouveau plugin Claude Code** (`apps/lingua-agent`) : binaire Rust `lingua` + hook `Stop` (ingestion des transcripts JSONL), statusline « % connus · N nouveaux », skill `/vocab`, serveur MCP pour les opérations de deck. Ingestion **local-only par construction** (les transcripts sont confidentiels). Architecture `SessionSource` extensible aux autres agents (Codex, Aider — hors périmètre de ce change).
- **Store local `~/.lingua/`** (SQLite, schéma versionné) : lemmes, compteurs, cartes — jamais de contenu de transcript persisté, seulement les phrases explicitement capturées par l'utilisateur via `/vocab`.
- Vocabulaire UI : le mot « lemme » n'apparaît **jamais** dans les sorties utilisateur du plugin (statusline, `/vocab`, MCP) — « forme du dictionnaire », « mots différents ».

## Capabilities

### New Capabilities
- `lingua-agent-capture` : l'ingestion des sessions d'agents IA — hook Claude Code, statusline, `/vocab`, MCP decks, trait `SessionSource`, local-only par défaut.

### Modified Capabilities
_Aucune. Le plugin consomme `lingua-analysis`, `lingua-knowledge-model` et `lingua-decks-review` (livrées par les changes précédents de la pile) via `lingua-core` en natif — il ne les redéclare pas. Aucun socle `id-*`/`platform-*` touché : pas de compte, pas de réseau._

## Impact

- **Produits** : Lingua (nouveau livrable plugin) ; **Cymbra ID / Music / Live / back-office : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés).
- **Arborescence** : `apps/lingua-agent` (binaire Rust + manifeste plugin Claude Code : hooks, statusline, skill, MCP).
- **CI** : la lane Rust existante (`cargo --workspace`) couvre le binaire (fmt/clippy/llvm-cov ≥ 80 %, logique host-testée) ; `apps/lingua-agent` est ajouté au filtre `ci-units`. Le lint « pas de “lemme” » s'étend aux sorties utilisateur du plugin.
- **Dépendances nouvelles** : `rusqlite` (store local) ; `lingua-core` déjà dans le workspace.
- **Hors périmètre** : adaptateurs Codex/Aider/Gemini (le trait `SessionSource` est le contrat, pas les impls), TUI de révision dédiée (la révision passe par l'agent), toute synchronisation réseau du store `~/.lingua/` (les changes `add-lingua-backend`/`add-lingua-connected-clients` synchronisent l'extension et l'app — **pas le plugin**, réaffirmé là-bas).
