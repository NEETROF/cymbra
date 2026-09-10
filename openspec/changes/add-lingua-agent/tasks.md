# Tasks — add-lingua-agent

## 1. Plugin Claude Code (spec lingua-agent-capture)

- [ ] 1.1 Binaire `apps/lingua-agent` : `lingua-core` natif + store SQLite `~/.lingua/` (schéma versionné) ; sous-commandes `ingest`, `statusline`, `vocab`, `mcp`
- [ ] 1.2 Trait `SessionSource` + impl Claude Code (parse JSONL, extraction texte assistant) ; tests sur transcripts factices ; invariant testé : aucune phrase persistée, aucune connexion réseau
- [ ] 1.3 Hook `Stop` (manifeste plugin) → `lingua ingest --transcript <path>` ; idempotence par offset de transcript
- [ ] 1.4 Statusline : % du dernier message + nouveaux de la session ; dégradation silencieuse
- [ ] 1.5 Skill `/vocab` : liste des inconnus de session avec gloses, ajout au deck avec consentement (phrase d'origine incluse à ce moment-là seulement)
- [ ] 1.6 Serveur MCP (`list_decks`, `add_words`, `due_cards`, `answer_card`) avec validation d'entrées ; test d'intégration bout-en-bout incluant la révision conversationnelle
- [ ] 1.7 Manifeste plugin Claude Code (hooks + statusline + skill + MCP) + doc d'installation ; documenter la configuration manuelle de la statusline si le manifeste ne peut pas l'installer

## 2. Gates et finitions

- [ ] 2.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` ; `apps/lingua-agent` ajouté au filtre `ci-units` ; lint « pas de “lemme” » étendu aux sorties utilisateur du plugin (statusline, `/vocab`, MCP)
- [ ] 2.2 `openspec validate add-lingua-agent --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
