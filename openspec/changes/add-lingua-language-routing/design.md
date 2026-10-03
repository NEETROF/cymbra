# Design — add-lingua-language-routing

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `crates/lingua-core/src/analysis/language.rs` | `block_is_studied(text, studied)`: a block of at least `MIN_BLOCK_BYTES` (12) whose whichlang language is the studied one. `analyse_document` keeps those blocks, and a document under `MIN_ANALYSABLE_TOKENS` counted tokens is not analysable. |
| `crates/lingua-core/src/engine.rs` | `analyse_page` returns the canonical `PageAnalysis`; the S0 baseline pins its English JSON. |
| `apps/lingua-extension/src/reading/session.ts` | `this.language` is the reading language (`readingLanguage`), read at start and after an external restore; `repaint()` analyses the document's blocks through `this.lang`, the view bound to it. |
| `src/analyzer/pairs.ts` | `acceptedLanguages(port)`: the reader's studied languages that the package ships (`add-lingua-language-sync-client`). |

## Goals / Non-Goals

**Goals:**
- A page read in its own language among the reader's, with the page analysis unchanged.
- English unchanged: one accepted language, no detection.

**Non-Goals:**
- Routing per block (a page that mixes languages paragraph by paragraph). The vote picks one, and
  the other blocks stay excluded.
- The Catalan/Galician guard (`add-lingua-spanish-detection-guard`): whichlang has no class for
  either, and reads them as Spanish.
- A book's own language (`add-lingua-reader-language`); until then a section is a document like a
  page.

## Decisions

### D1 — A length-weighted vote, in the core

`detect_document_language(blocks, candidates, hint)` lives next to `block_is_studied` and uses the
same rule for a block that can vote: at least `MIN_BLOCK_BYTES` once trimmed. Each such block adds
its length to the language whichlang finds, when that language is a candidate. The most weight
wins. Ties go to the hint if it is one of the tied, else to the earlier candidate (the reader's
order). With no vote at all, the hint wins if it is a candidate, else the first candidate. A single
candidate returns at once.

Weighting by length keeps a page's short chrome (menus, buttons, captions) from outvoting its text,
and is what the analysis itself would count.

*Rejected — one vote per block.* A navigation bar of twenty short items would outvote an article's
five long paragraphs.

### D2 — A whole-reader call, beside the analysis

The engine binding is `detectLanguage(blocks, candidates, hint): string`. It refuses an unknown or
empty candidate list and ignores an unknown hint: a page may declare `fr`. It needs no pack, so it is
a root call of `LinguaPort` and loads nothing (`package-lingua-packs-per-pair`). The session keeps the
language it got next to the analysis: the `PageAnalysis` JSON does not change, which is the
architecture rule.

### D3 — The session asks per repaint, only with several languages

The session keeps `languages` (`acceptedLanguages`), read where it read the reading language. With
more than one, `repaint()` asks for the language of the document's blocks among them, with the
document's `lang` (primary subtag) as hint, before it analyses. It then sets `this.language`, so the
analysis, glosses, cards, statuses, exposures and the speaker all follow. Each repaint is already a
whole-document analysis, so the vote sees the whole document, and grows with it on a page that loads
more text. With one language, it asks nothing (English invariance).

## Risks / Trade-offs

- **A page whose language flips between repaints** (more text loaded) → its later analyses are in
  the new language. The vote weighs the whole document, so a flip needs the new language to outweigh
  the old.
- **A section of a book read in another language than the book** → the vote per section; the book's
  own language arrives with `add-lingua-reader-language`.
- **Catalan or Galician read as Spanish** → the guard is G1's (`add-lingua-spanish-detection-guard`);
  no reader accepts Spanish before then.
