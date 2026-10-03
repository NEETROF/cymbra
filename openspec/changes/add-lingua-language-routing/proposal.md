# add-lingua-language-routing — each page is read in its own language

## Why

A reader who studies English and Spanish reads pages in both. The reading session reads every page
in one language today, the reader's first (`add-lingua-studied-language-profile`). The engine keeps
only the blocks detected in that language (*Per-block language detection*), so a Spanish page read
as English is "not analysable", and nothing in it is highlighted.

The architecture the study settled says so (`docs/lingua/spanish-programme.md`): the language is
per document (a page, or an EPUB section), detected among the enabled languages. It comes back
beside the canonical `PageAnalysis` JSON, so English output does not move. Routing per block
comes later.

This is change 12 of the programme, in R3, a silent English release. Every reader accepts English
alone, so the session asks for no detection and nothing a reader sees changes.

## What Changes

- **The core chooses a document's language** among candidates
  (`detect_document_language`, `lingua-core`).
  - **The vote.** Each block long enough to be detected votes for the language whichlang finds,
    weighted by its length, when that language is a candidate.
  - **The hint.** The document's declared language (the page's `lang`) breaks a tie, and decides
    when no block votes.
  - **The fallback.** Without a hint, the first candidate wins.
  - **One candidate** is chosen without detection.
- **The engine and the port expose it**: `detectLanguage(blocks, candidates, hint)`, a whole-reader
  call that loads no pack.
- **The reading session reads each document in its language.** It reads the reader's accepted
  languages (`acceptedLanguages`, `add-lingua-language-sync-client`). With several, each repaint
  asks for the document's language among them, then reads in it: the analysis and every
  language-bound question about the document (glosses, cards, statuses, exposures). With one, it
  asks nothing.
- **The page's analysis does not change.** The chosen language travels beside it, in the session.
  Blocks in other languages stay excluded, as *Per-block language detection* says.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A document's language, chosen among the reader's*.
- `lingua-browser-extension`:
  - ADDED — *Each document is read in its own language*;
  - MODIFIED — *Each surface reads in the reader's language* (`add-lingua-studied-language-profile`,
    archived): a reading session asks about a document in that document's language. No open change
    holds this requirement.

## Impact

- **Products.** Cymbra Lingua's browser extension: `crates/lingua-core` (the vote), `crates/lingua-wasm`
  (one binding), the extension's port and reading session. The agent plugin and the Catalan/Galician
  guard (`add-lingua-spanish-detection-guard`) are later changes.
- **Release.** R3, silent: one accepted language, no detection, the same requests.
