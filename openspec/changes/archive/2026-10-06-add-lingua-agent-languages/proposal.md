# add-lingua-agent-languages — the agent plugin follows Spanish too

## Why

The Claude Code plugin (`apps/lingua-agent`, capability `lingua-agent-capture`) reads every
assistant reply as English:
- it loads one pack, `~/.lingua/pack.lingua`;
- its analysis is pinned to English (`engine.rs`, `vocab.rs`);
- its store has no language column (`store.rs`): exposures, statuses, cards and calibration are
  keyed by the word alone.

A reader who studies Spanish, and talks with their agent in Spanish, gets nothing from it. Decision
D6 of the Spanish programme keeps the plugin in scope, last: change 31.

## What Changes

- **The languages it follows are the packs installed.** Every pack in `~/.lingua/` is a language
  the plugin follows; `pack.lingua` stays the English one. Copying `es-fr.lingua` beside it is all a
  reader does: no setting.
- **Each reply is read in its own language**, the one the engine finds in it among the languages
  followed (the extension's language vote), with that language's pack.
- **The store gains a language** (schema 2): exposures, statuses, cards and calibration are per
  language. Everything a schema-1 store holds becomes English.
- **The statusline names the language** when several are followed: « 📖 ES 91 % · 5 nouveaux ».
  With one language it is unchanged.
- **`/vocab`** lists the session's words under their language when several are followed, and adds
  a word to the deck of the language it was met in.
- **The MCP tools take a `language`:**
  - optional when one language is followed;
  - with several, `add_words`, `due_cards` and `answer_card` ask for it, answering with the
    languages followed when it is missing. A review is in one language, never mixed, as the
    extension's is (`refine-lingua-review-language`);
  - `list_decks` counts each language.
- **English alone: unchanged.** One pack, no vote, the same statusline and tool answers; a schema-1
  store migrates with every row English.
- **Local by construction, unchanged:** nothing leaves the machine.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-agent-capture`:
  - « Claude Code transcript ingestion, local by construction »: each reply is read and counted in
    its language;
  - « Coverage statusline »: the language's code when several are followed;
  - « The /vocab command »: the words under their language;
  - « MCP server for deck operations »: the `language` parameter;
  - added: « The languages followed are the packs installed ».

  No open change holds a requirement of `lingua-agent-capture`.

## Impact

- **Products:**
  - Cymbra Lingua: the Claude Code plugin only. The extension, the Apple app and the server are
    unchanged;
  - nothing is consumed from ID or the platform.
- **Code:**
  - `crates/lingua-core/src/packs/pack.rs`: `Pack::studied_in(bytes)`, a pack's language read from
    its metadata;
  - `apps/lingua-agent/rust/src/`: `engine.rs`, `ingest.rs`, `store.rs`, `statusline.rs`,
    `vocab.rs`, `mcp.rs`, `main.rs`, and their tests (a Spanish fixture pack);
  - `apps/lingua-agent/commands/vocab.md`, `apps/lingua-agent/README.md`.
- **Data:** `~/.lingua/lingua.db` migrates to schema 2 on first open, in one transaction, every row
  English. Packs are not shipped with the plugin: the reader copies or builds them, as today.
- **English alone:** unchanged, and the extension's English baseline does not move.
