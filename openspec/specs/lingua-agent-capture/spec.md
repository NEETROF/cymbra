# lingua-agent-capture Specification

## Purpose
TBD - created by archiving change add-lingua-agent. Update Purpose after archive.
## Requirements
### Requirement: Claude Code transcript ingestion, local by construction
A `Stop` hook SHALL invoke the `lingua` binary with the transcript path; the binary SHALL extract the assistant messages' text from the JSONL, analyse it (natively, at the same `analyzer_version` as the extension) and update the per-lemma exposure counters in the local store (`~/.lingua/`). Transcript content (sentences, code) SHALL NOT be persisted nor sent over the network — only lemmas, counters and timestamps are, plus the sentences the user explicitly captures through `/vocab`.

#### Scenario: End of a Claude Code turn
- **WHEN** Claude Code finishes a reply and the `Stop` hook runs
- **THEN** the lemmas of the assistant text have their exposure counters incremented and no sentence from the transcript appears in the store

#### Scenario: No network egress
- **WHEN** ingestion runs
- **THEN** the binary opens no network connection

### Requirement: Coverage statusline
The statusline SHALL show, for the current session, the percentage of known words in the last assistant message and the number of new words met during the session, degrading silently (no error output) when the transcript cannot be read.

#### Scenario: Display after a reply
- **WHEN** the statusline runs after a reply containing 3 unknown lemmas
- **THEN** it shows the known percentage and « 3 nouveaux » (the plugin ships French UI copy)

### Requirement: The /vocab command
A `/vocab` skill SHALL list the current session's unknown words with their dictionary form, their gloss and their rarity, and SHALL let the user add a selection of those words to the deck (cards carrying the source sentence from the transcript, added only with the user's explicit consent).

#### Scenario: Reviewing a session
- **WHEN** the user invokes `/vocab` after a session containing 4 unknown words
- **THEN** the 4 words are listed with their glosses and the user can add any or all of them to the deck

### Requirement: MCP server for deck operations
The plugin SHALL expose an MCP server offering at least `list_decks`, `add_words`, `due_cards` and `answer_card` (FSRS grading `again/hard/good/easy`), operating on the local store. Inputs SHALL be validated (normalised lemmas, bounded sizes). Reviewing the cards in the plugin store SHALL be possible through the agent (a conversational quiz, `due_cards` → `answer_card`).

#### Scenario: The agent adds words
- **WHEN** the user asks Claude to add three words to their deck and Claude calls `add_words`
- **THEN** three cards are created in the local store and `due_cards` reflects them

#### Scenario: Conversational review
- **WHEN** the agent quizzes the user and grades a card through `answer_card` with `good`
- **THEN** the card's FSRS state and due date are updated in the local store

### Requirement: Extensible session sources
Ingestion SHALL go through a `SessionSource` abstraction (locating sessions, extracting the assistant text), of which Claude Code is the first implementation, so that a new agent is added without touching the analysis pipeline.

#### Scenario: Adding a source
- **WHEN** a test `SessionSource` implementation supplies a synthetic transcript
- **THEN** the ingestion pipeline produces the same counters as it would for an equivalent Claude Code transcript

