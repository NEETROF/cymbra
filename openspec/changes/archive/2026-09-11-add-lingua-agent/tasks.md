# Tasks — add-lingua-agent

## 1. Claude Code plugin (spec lingua-agent-capture)

- [x] 1.1 `apps/lingua-agent` binary: native `lingua-core` + a SQLite store in `~/.lingua/` (versioned schema); `ingest`, `statusline`, `vocab` and `mcp` subcommands
- [x] 1.2 `SessionSource` trait + the Claude Code impl (JSONL parsing, assistant-text extraction); tests over synthetic transcripts; invariants under test: no sentence persisted, no network connection
- [x] 1.3 `Stop` hook (plugin manifest) → `lingua ingest --transcript <path>`; idempotence keyed on the transcript offset
- [x] 1.4 Statusline: the last message's percentage + the session's new words; silent degradation
- [x] 1.5 `/vocab` skill: list the session's unknown words with glosses, add to the deck with consent (the source sentence is included only at that point)
- [x] 1.6 MCP server (`list_decks`, `add_words`, `due_cards`, `answer_card`) with input validation; an end-to-end integration test covering conversational review
- [x] 1.7 Claude Code plugin manifest (hooks + statusline + skill + MCP) + install docs; document the manual statusline configuration if the manifest cannot install it

## 2. Gates and finishing

- [x] 2.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80`; `apps/lingua-agent` added to the `ci-units` filter; the "no 'lemma' in UI copy" lint extended to the plugin's user-facing output (statusline, `/vocab`, MCP)
- [x] 2.2 Final `openspec validate add-lingua-agent --strict` + spec updates if the implementation moved a contract
