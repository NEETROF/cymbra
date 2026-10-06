# Design

## Context

- **The plugin** (`apps/lingua-agent`, archived change `add-lingua-agent`) is a `lingua` binary run
  by Claude Code:
  - `ingest`: the `Stop` hook counts each reply's lemmas;
  - `statusline`: the last reply's coverage;
  - `vocab`: the session's unknown words, and adding some to the deck;
  - `mcp`: an MCP server over the local deck.
- **English everywhere:**
  - one pack, `$LINGUA_PACK` or `~/.lingua/pack.lingua` (`engine.rs`);
  - `analyse_page(…, EN, …)` in `engine.rs` and `vocab.rs`;
  - the knowledge state filled for English (`store.rs`);
  - SQLite tables keyed by the lemma alone, schema 1, one calibration.
- **What the core already offers:**
  - `detect_document_language(blocks, candidates, hint)`, the extension's language vote, which
    needs no pack (`add-lingua-language-routing`);
  - `Pack::studied()`, the language a loaded pack is for.
- **Cost measured:** loading a pack natively takes about 25 ms (en-fr) and 28 ms (es-fr), release
  build. The hook and the statusline run after every reply.

## Goals / Non-Goals

**Goals:**
- Follow every language whose pack is installed, each reply in its own language.
- Counters, statuses, cards and calibration per language.
- The statusline, `/vocab` and the MCP tools per language, a review never mixing two.
- English alone unchanged.

**Non-Goals:**
- Syncing the plugin's store. It stays local (`add-lingua-connected-clients` D5).
- A language setting, or downloading packs: the reader copies or builds them, as today.
- Levels, the word card or translation in the plugin.

## Decisions

### D1 — The packs installed are the languages followed

- **The packs:** every regular file ending in `.lingua` directly in the data directory
  (`$LINGUA_HOME`, else `~/.lingua`), read in file-name order.
  - `pack.lingua` keeps being the English pack readers already have.
  - A file that is not a pack this core can read (damaged, unknown language, another analyser
    generation) is skipped.
  - The first pack of a language wins.
- **`$LINGUA_PACK`** keeps naming one pack, then the only one followed (tests, and a reader whose
  pack lives elsewhere).
- **The order:** the languages followed are taken in tag order (`en`, then `es`). The vote's ties go
  to the earlier one, so to English, as every reply went before.
- **No pack:** the plugin stays mute, as today.
- **Reading a pack's language costs no load.** The core gains `Pack::studied_in(bytes)`: the
  language a pack is for, read from its metadata, refused for the same reasons `load` refuses
  (unknown language, analyser generation). The plugin loads the one pack a reply needs.

### D2 — A reply is read in its language

- **The vote:** `detect_document_language` over the reply's lines, among the languages followed,
  with no hint. With one language there is no vote.
- **Code does not vote.** Lines inside fenced code blocks (between two « ``` ») are left out of the
  vote, so the code in a Spanish reply does not pull it to English. The analysis still reads every
  line it read before.
- **One language per reply:** its pack and its knowledge serve the counters, the statusline and
  `/vocab`.
- **A reply in a language not followed** (French) is voted into one of those followed and analyses
  as the extension analyses a page in another language: mostly not analysable.

### D3 — The store gains a language (schema 2)

- **Tables:** `exposures`, `statuses` and `cards` are keyed by `(language, lemma)`. Calibration is
  per language in `meta` (`calibration:<tag>`); the schema-1 key `calibration` is read as English.
- **Migration from schema 1**, in one transaction:
  - the new tables are created;
  - every row is copied with `language = 'en'`, since the plugin only ever stored English;
  - the old tables are dropped and `schema_version` becomes 2.
  Ingest offsets are per transcript and do not change.
- **A later schema** is left untouched, and the plugin stays mute rather than write into it.
- **The knowledge state** carries every language's calibration and statuses.

### D4 — The statusline names the language with several

- With two or more languages followed, the reply's tag in capitals comes first:
  « 📖 ES 91 % · 5 nouveaux », « 📖 EN 96 % · 3 nouveaux ».
- With one language, « 📖 96 % · 3 nouveaux », as before.

### D5 — `/vocab` per language

- **The listing:** each unknown word belongs to the language of the reply it was met in.
  - With several languages, the words are listed under a heading per language (« Anglais »,
    « Espagnol »), in the order of D1.
  - With one, the listing is unchanged.
- **Adding:** `--add w1,w2` adds each word in the language it was listed in. A word listed in two
  languages needs `--language <tag>`; without it, it is not added and the answer names it.
- **`commands/vocab.md`** stops saying « English ».

### D6 — The MCP tools take a `language`

- Every tool takes an optional `language`, an ISO 639-1 tag, described in its `inputSchema`.
- **One language followed:** without it, that language; any other tag is refused.
- **Several followed:**
  - `add_words`, `due_cards` and `answer_card` refuse a call without it, with a message naming the
    languages followed: the agent asks again with one;
  - `list_decks` without it answers the totals and `languages: [{language, cards, due}]`.
- **A language not followed** is refused.
- **A review never mixes languages:** `due_cards` lists one language's cards, as the extension's
  review does (`refine-lingua-review-language`).
- The server reads the languages followed when it starts (D1, no pack loaded).

### D7 — English alone

With `pack.lingua` alone:
- there is no vote;
- the statusline, `/vocab` and the tools answer as before, no tool asking for a language;
- the store migrates with every row English.

## Risks / Trade-offs

- **A reply misread.** Prose too short to vote, or English identifiers outside code fences, can put
  a Spanish reply in English. Its words then count in the wrong language: exposures only, no
  status changes.
- **A word in two decks** (« no » is English and Spanish): the tools and `/vocab` ask which
  language rather than guess.
- **Packs are not shipped with the plugin.** The README says how to build or copy one per language.
