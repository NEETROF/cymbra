# Tasks

## 1. The packs and the languages followed

- [x] 1.1 `crates/lingua-core/src/packs/pack.rs`: `Pack::studied_in(bytes)`, the language a pack is for, read from its metadata and refused for `load`'s reasons, building no lexicon (design D1). Specs:
  - an English and a Spanish pack;
  - a pack of an unknown language and one of another analyser generation refused.
- [x] 1.2 `apps/lingua-agent/rust/src/engine.rs` (design D1, D2):
  - the packs installed: `*.lingua` in the data directory, in name order, the first of a language winning, a file that does not read skipped;
  - `$LINGUA_PACK` as the only pack when set;
  - the languages followed in tag order, one pack loaded on demand;
  - a reply's language by the core's vote, fenced code left out of it.

  Specs: two packs; a damaged one skipped; `$LINGUA_PACK`; a Spanish reply with code and an English one.

## 2. The store

- [x] 2.1 `apps/lingua-agent/rust/src/store.rs`, schema 2 (design D3):
  - `(language, lemma)` for exposures, statuses and cards, calibration per language;
  - the migration from schema 1 in one transaction, every row English;
  - a later schema left untouched, the plugin mute.

  Specs: a schema-1 store migrated, its rows English; a Spanish and an English row for one lemma; the knowledge state per language.

## 3. Reading the replies

- [x] 3.1 `ingest.rs`, `statusline.rs`, `vocab.rs`, `main.rs` (design D2, D4, D5):
  - each reply read and counted in its language;
  - the statusline's tag with several languages;
  - `/vocab` under a heading per language, `--add` with `--language` for a word listed in two.

  Specs: a Spanish reply counted in Spanish; « 📖 ES … » with two packs and the line unchanged with one; `/vocab` under two headings.

## 4. The MCP tools

- [x] 4.1 `apps/lingua-agent/rust/src/mcp.rs`: `language` on every tool, required with several languages for `add_words`, `due_cards` and `answer_card`, and `list_decks` per language (design D6). Specs:
  - one language followed, no parameter;
  - two followed: the refusal naming them, then the Spanish cards only with `language` `es`;
  - a language not followed refused.

## 5. The docs

- [x] 5.1 `apps/lingua-agent/commands/vocab.md` names no single language. `apps/lingua-agent/README.md`: installing another language's pack (`yarn gen:pack:real` in the extension, or `scripts/lingua-data/build.sh`), the statusline's tag, the tools' `language`.

## 6. Checks

- [x] 6.1 `cargo test -p lingua-core -p lingua-agent`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and line coverage ≥ 80 % (`cargo llvm-cov` with `.github/coverage-ignore-regex.txt`). The English baseline does not move.
- [x] 6.2 `openspec validate add-lingua-agent-languages --strict` passes.
- [x] 6.3 `docs/lingua/spanish-programme.md`: change 31 done.

## 7. Dogfood (owner)

- [ ] 7.1 With `es-fr.lingua` beside `pack.lingua`: a reply in Spanish shows « 📖 ES » in the statusline, `/vocab` lists its words under « Espagnol », and the agent reviews Spanish cards through the MCP tools.
