# lingua-agent-capture Specification

## Purpose
TBD - created by archiving change add-lingua-agent. Update Purpose after archive.
## Requirements
### Requirement: Claude Code transcript ingestion, local by construction
A `Stop` hook SHALL invoke the `lingua` binary with the transcript path; the binary SHALL extract the assistant messages' text from the JSONL, read each reply in the language the engine finds in it among the languages the plugin follows, analyse it with that language's pack (natively, at the same analyser version as the extension) and update that language's per-lemma exposure counters in the local store (`~/.lingua/`). Transcript content (sentences, code) SHALL NOT be persisted nor sent over the network — only lemmas, counters and timestamps are, plus the sentences the user explicitly captures through `/vocab`. A store written before languages were stored SHALL be kept whole, as English.

#### Scenario: End of a Claude Code turn
- **WHEN** Claude Code finishes a reply and the `Stop` hook runs
- **THEN** the lemmas of the assistant text have their exposure counters incremented and no sentence from the transcript appears in the store

#### Scenario: No network egress
- **WHEN** ingestion runs
- **THEN** the binary opens no network connection

#### Scenario: A reply in Spanish
- **WHEN** the plugin follows English and Spanish and Claude Code finishes a reply written in Spanish, with a block of code in it
- **THEN** the reply's lemmas are counted as Spanish, from the Spanish pack, and no English counter moves

#### Scenario: A store kept by an earlier version
- **WHEN** the binary opens a store written before languages were stored
- **THEN** every counter, status, card and calibration in it is kept, as English

### Requirement: Coverage statusline
The statusline SHALL show, for the current session, the percentage of known words in the last assistant message and the number of new words met during the session, in the language that message is read in, degrading silently (no error output) when the transcript cannot be read. When the plugin follows several languages, the statusline SHALL name that language before the percentage.

#### Scenario: Display after a reply
- **WHEN** the statusline runs after a reply containing 3 unknown lemmas
- **THEN** it shows the known percentage and « 3 nouveaux » (the plugin ships French UI copy)

#### Scenario: Two languages followed
- **WHEN** the plugin follows English and Spanish and the last reply is in Spanish
- **THEN** the statusline starts « 📖 ES », then the Spanish percentage and new words

### Requirement: The /vocab command
A `/vocab` skill SHALL list the current session's unknown words with their dictionary form, their gloss and their rarity, each in the language of the reply it was met in, and SHALL let the user add a selection of those words to the deck of that language (cards carrying the source sentence from the transcript, added only with the user's explicit consent). When the plugin follows several languages, the words SHALL be listed under their language, and a word listed under two SHALL be added only in the language the user names.

#### Scenario: Reviewing a session
- **WHEN** the user invokes `/vocab` after a session containing 4 unknown words
- **THEN** the 4 words are listed with their glosses and the user can add any or all of them to the deck

#### Scenario: A session in two languages
- **WHEN** the user invokes `/vocab` after a session with replies in English and in Spanish
- **THEN** the unknown words are listed under « Anglais » and « Espagnol », and a word added goes to the deck of the language it was listed under

### Requirement: MCP server for deck operations
The plugin SHALL expose an MCP server offering at least `list_decks`, `add_words`, `due_cards` and `answer_card` (FSRS grading `again/hard/good/easy`), operating on the local store, each taking an optional `language` among the languages the plugin follows. Inputs SHALL be validated (normalised lemmas, bounded sizes, a language the plugin follows). Reviewing the cards in the plugin store SHALL be possible through the agent (a conversational quiz, `due_cards` → `answer_card`), in one language at a time. When the plugin follows several languages, `add_words`, `due_cards` and `answer_card` SHALL refuse a call without `language`, naming the languages followed, and `list_decks` SHALL count each language.

#### Scenario: The agent adds words
- **WHEN** the user asks Claude to add three words to their deck and Claude calls `add_words`
- **THEN** three cards are created in the local store and `due_cards` reflects them

#### Scenario: Conversational review
- **WHEN** the agent quizzes the user and grades a card through `answer_card` with `good`
- **THEN** the card's FSRS state and due date are updated in the local store

#### Scenario: Two languages followed
- **WHEN** the plugin follows English and Spanish and the agent calls `due_cards` without a language
- **THEN** the call is refused with a message naming English and Spanish, and `due_cards` with `language` `es` lists the Spanish cards only

### Requirement: Extensible session sources
Ingestion SHALL go through a `SessionSource` abstraction (locating sessions, extracting the assistant text), of which Claude Code is the first implementation, so that a new agent is added without touching the analysis pipeline.

#### Scenario: Adding a source
- **WHEN** a test `SessionSource` implementation supplies a synthetic transcript
- **THEN** the ingestion pipeline produces the same counters as it would for an equivalent Claude Code transcript

### Requirement: The languages followed are the packs installed
The plugin SHALL follow every studied language whose pack is installed in its data directory, the English pack staying at `pack.lingua`, with no other setting. A pack that cannot be loaded SHALL be skipped without disrupting the agent. A plugin with only one pack installed SHALL behave as it did before it followed several languages.

#### Scenario: Installing the Spanish pack
- **WHEN** the user copies `es-fr.lingua` beside `pack.lingua`
- **THEN** the plugin follows English and Spanish from the next reply, with no setting changed

#### Scenario: A pack that does not load
- **WHEN** a pack in the data directory is damaged or built for another analyser generation
- **THEN** it is skipped and the other packs are followed

#### Scenario: English alone
- **WHEN** only `pack.lingua` is installed
- **THEN** the statusline, `/vocab` and the MCP tools answer as before, and no tool asks for a language

