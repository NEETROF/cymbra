# Design — add-lingua-spanish-names

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `percent.rs` `is_out_of_lexicon_proper_noun` | A capitalised surface whose lowercase form and lemma the lexicon does not know is `ProperNounOutOfLexicon`. |
| `engine.rs` `classify_token` | That rule, else the knowledge model. Shared by the page analysis and the phrase gloss. |
| `engine.rs` `analyse_page` | Classifies every token of a document's blocks, all at once. |
| The extension | Paints neither `ProperNounOutOfLexicon` nor excluded tokens, and `Coverage` leaves them out of the percentage. |
| The EPUB reader | Analyses a book one section (chapter) at a time. |

## Goals / Non-Goals

**Goals:**
- A novel's characters and places stop showing as unknown words with no gloss.
- No word the reader could learn is hidden.

**Non-Goals:**
- English. Its names are rarer in the lexicon, and changing its analysis would move its baseline and
  its pack, which only a deliberate English change may do. It can follow with its own version bump.
- A whole book's names at once. Each section is judged on its own text. A name in most chapters is set
  aside in each of them.
- The phrase gloss of a selection. Without the document, a selected name keeps the word's card.

## Decisions

### D1 — The evidence: a capital in mid-sentence, never a lowercase

Within the document, a form is a name when:
- it is never written in lowercase;
- at least one occurrence is capitalised in mid-sentence, right after a letter, a digit, a comma or a
  semicolon;
- its dictionary form has no gloss in the pack.

At the head of a block, after a full stop, a question or exclamation mark, a colon, a quotation mark or
a dialogue dash (`—Augusto`), any word takes a capital, so those occurrences prove nothing. Once a form
is a name, all its occurrences are set aside, the sentence-initial ones included.

The position is read from the token's byte offset in its block. An offset that falls inside a character
counts as no evidence.

*Rejected — a share of capitals (90 %).* A single lowercase occurrence already says the form is also a
word. The gloss condition keeps the cost of a miss low.

### D2 — Only words without a gloss

A capitalised word the pack glosses keeps its card: `Dios` is « Dieu », and a reader may want to learn
it. The rule only removes cards that had nothing to say.

*Rejected — every capitalised form.* `Dios`, `Señor` or `Virgen` would vanish from a religious text.

### D3 — A name joins the out-of-lexicon proper nouns

The token gets `ProperNounOutOfLexicon`, which the extension and the percentage already set aside. No
new class means no change to the JSON, the extension or the sync. The override applies only to a token
the knowledge model reads `Unknown`: a word the reader marked keeps its status.

### D4 — Spanish only, version `1.2.0`

The classification is part of the analysis output, so Spanish's analyser version moves from `1.1.0` to
`1.2.0`. The es-fr pack is built with it, and `Pack::load` refuses a pack of another version. The
English analyser, its version and the English baseline do not move: `analyse_page` computes names only
for Spanish.

## Risks / Trade-offs

- **A word without a gloss that a text always capitalises** (a title, an institution) is set aside. It
  had no gloss to show anyway, and it no longer counts against the reader.
- **A minor name met once, at the head of a sentence**, stays a word: there is no evidence.
- **The `testdata/es-fr` pack of `enable-lingua-spanish`** carries the old version. That branch has to
  rebuild it after this change merges.
