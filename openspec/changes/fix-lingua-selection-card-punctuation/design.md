# fix-lingua-selection-card-punctuation — design

## Context

See proposal.md for the why. What exists:

- **The capture.** `captureFrom` (`src/reading/selection.ts`) clones the page's selection, snaps
  both ends of the copy outward to whole words (`WORD_CHAR`: letters, digits, `'`, `’`, `-`, the
  soft hyphen) and reads its text with soft hyphens dropped, whitespace collapsed and trimmed.
  `sentenceAndSelection` finds the sentence by position and tightens the span to the text, again
  of whitespace only. A comma, a full stop, a quote or a bracket the reader's selection took stays
  in `text`, in the span and in the range — the snap never adds one, but it never removes one
  either.
- **The cards.** `openForSelection` (`src/reading/selection-card.ts`) routes on that text:
  whitespace makes the whole-selection card (`openExpression`), headed and keyed by `sel.text`,
  with `surface: sel.text`; only when the pack's phrase gloss of the text holds an expression
  covering all its tokens (`wholeSelectionMatch`) is the card re-headed by the expression's key,
  with its gloss. The analyser tokenises by UAX #29 word boundaries, so the comma is not a token
  and does not by itself defeat that match: with today's pack, `four-poster bed,` finds
  `four-poster bed`. The dogfood card was captured before #726 (2026-10-06) added the entry: no
  match, so the raw text — comma included — became its lemma, with no gloss. Every selection the
  pack does not know as an expression is stored that way today, and the same raw text is sent as
  the translation's span, the online translation's request and the word looked up in the chapter
  recap; a single word with a comma and no page token opens the analyser's card under that text.
- **The card.** `addCard` (`crates/lingua-wasm`) stores the lemma and form as handed, the gloss
  labelled with the engine's native language, and marks the lemma `learning`.
- **The review.** `reviewCurrent` shows the card's own gloss when its gloss language is the
  engine's native one — `None` for a card captured without a gloss — and calls `readable_gloss`
  only for a card glossed in another language; `readable_gloss` reads the pack at `card.lemma` as
  stored (`pack.expression` for English, the French and Spanish key readings), deciding word or
  expression by `Card::is_expression` (a space in the lemma), and falls back to the card's gloss.
  The front (`src/review/view.ts`) marks the word in its sentence with `markWord(sentence,
  surface, headword)` — a whole-word, case-insensitive search of the text as stored — and the
  review's machine translation (`add-lingua-review-translation`) takes its span from that mark.

## Goals / Non-Goals

**Goals:**
- No new card is keyed, headed or translated with the punctuation at the ends of a selection.
- The cards already in a reader's deck are answered at review as if they had been captured
  trimmed, on every device, with nothing written.

**Non-Goals:**
- Rewriting, merging or deduplicating stored cards (D5); renaming a card by hand is
  `add-lingua-card-edit`.
- Punctuation inside a selection (`four-poster, bed`), letter case, and an expression card keyed
  by its inflected text (`gave up` captured before the table knew it): the pack is asked as today.
- The agent plugin's cards, which are created from dictionary forms, never from a selection.
- A pack change: the en-fr table already holds the expression.

## Decisions

### D1 — What an edge is: anything that is not a letter, a mark or a digit

A selection is trimmed, at each end, of the run of characters that are neither letters
(`\p{L}`), combining marks (`\p{M}`) nor digits (`\p{N}`): punctuation of every kind (commas,
full stops, quotes `" « » “ ” ‘ ’`, brackets, dashes, ellipses), symbols and whitespace. Nothing
inside the selection is touched, so an elision (`l’homme`, `d'abord`, `aujourd'hui`), a compound
(`four-poster`, `dit-il`, `a-t-il`) and an inner apostrophe (`rock 'n' roll`) are read as
written: each of their apostrophes and hyphens has a letter on both sides.

Apostrophes and hyphens at an edge are trimmed too. A closing quote is the same character as an
apostrophe (`‘four-poster bed,’` in a British novel) and an edge apostrophe is what the English
analyser already strips from a token (`dogs'` reads `dogs`); a hyphen at an edge is a suspended
compound (`pre-` of « pre- and post-war ») or a dash, never part of a key. The one word this
costs is a French elided piece selected alone (`l’`): within an analysed block it opens from its
page token, which the trim does not change (the piece is routed by the selection as the reader
made it, `selected`); outside one it is read as its letter, which the analyser glosses no worse.

A selection that is nothing but edge characters is no capture: no card opens, as for an empty
selection.

*Alternative.* Trimming Unicode punctuation only, keeping apostrophes: `‘four-poster bed,’` would
stop at the closing `’` and keep the comma behind it — the bug again, in the books that quote
with single marks. *Alternative.* Asking the analyser for the first and last token's span and
cutting there: exact, but an engine round trip before every card opens, for what a character
class answers.

### D2 — Trimmed once, where the capture is read

`captureFrom` trims its text, shrinks the span `sentenceAndSelection` returns by the same
characters, and moves the widened `range` inward past them, within the text node each end sits
in (as the snap does), so the word hit-test starts on a letter rather than on a leading « or (.
`selected`, the copy of the selection as the reader made it, is left alone. The trimmed text is
then classified (`bed,` is a word, `four-poster bed,` a phrase) and handed to the cards: the
headword, the form met, the phrase gloss, the grammar, the translation and online translation
requests, the chapter recap's lookup and the length bound all read it, and none of them changes.
The helper (`withoutEdgePunctuation`) sits beside `withoutSoftHyphens` and is exported for the
review's front (D4).

*Alternative.* Trimming in `openExpression` alone: the single-word path, the span, the
translation's mark and the hit-test would keep the punctuation, and each future reader of
`sel.text` would need to remember it.

### D3 — Review looks a card without a gloss up, at its trimmed lemma

`reviewCurrent` keeps a card's gloss only when the card holds one in the engine's native language;
a card holding none goes through `readable_gloss`, as a card glossed in another language already
does, and still falls back to its own (absent) gloss. `readable_gloss` reads the card at its lemma
without edge characters (D1's class, in Rust: `Card::lookup_lemma`, built on
`char::is_alphanumeric` and `unicode-normalization`'s combining-mark test, which the core already
depends on), and decides word or expression on that trimmed lemma — `bed ,` is a word. The French
and Spanish key readings (`add-lingua-french-expression-keys`, `add-lingua-spanish-expression-keys`,
`add-lingua-french-word-card`) receive the trimmed lemma unchanged otherwise. The view keeps its
keys; `headword` and `surface` are still the card's own.

Looking up a glossless card is the half of the fix that matters most: without it, even a card
whose lemma is already a clean key — an expression captured before the table knew it — never
shows the answer a later pack gained. With it, those cards are answered too, without capture
again.

*Alternative.* Trimming only, with the lookup still reserved to cards glossed in another
language: the dogfood card, glossed in the native language with no text, would stay
unanswerable. *Alternative.* Writing the pack's gloss into the card on review: a write per review,
a card operation to synchronise and a backup change for something the pack answers each time.

### D4 — The front marks the trimmed form

The front reads `headword` and `surface` through `withoutEdgePunctuation` before marking:
`four-poster bed` is underlined in its sentence, the comma left outside the mark; a card without a
sentence holding it is headed by the trimmed headword; the dictionary form is shown only when it
differs from the marked text once both are trimmed, so `four-poster bed,` is not offered as the
dictionary form of `four-poster bed`; read-aloud speaks the trimmed text; the no-translation note
tests the trimmed headword. The review's machine translation takes its span from the mark, so it
marks the expression without its comma.

*Alternative.* Marking the stored form: an underlined comma, and a front that contradicts the
answer it now reveals.

### D5 — No card is rewritten, merged or migrated

The `four-poster bed,` card keeps its lemma, form, schedule and history; the backup, the card
operations and synchronisation carry the same bytes, and a device on an older build reviews it as
before. If the reader later captures `four-poster bed` cleanly, it is another card under another
lemma: two cards for one expression, each answered. The lemma marked `learning` at that capture
(`four-poster bed,`) stays in the knowledge records; it matches no page token and is never shown.
Renaming or merging is the reader's act, in `add-lingua-card-edit`.

*Alternative.* A restore-time migration trimming every lemma: a key change that has to win
last-write-wins against every other device, collisions to merge (two schedules, two histories),
and a backup version — for cards the review now answers anyway.

## Risks / Trade-offs

- [A lemma whose ends are legitimately non-letters — `C++`, `.NET`, `#hashtag`] → read `C` and
  `NET`; none of them is a key of a pack, a word card from a page token is keyed by the token,
  never by the selection, and the review lookup only ever falls back to the card's own text.
- [The English baseline pins review views] → only cards without a gloss can change, and only to a
  pack gloss; the baseline is run and, if one of its cards gains a gloss, that is the fix and the
  pin is updated in the same pull request with the card named.
- [Two cards for one expression after a clean recapture] → both answered; merging waits for
  `add-lingua-card-edit` (D5).
- [The extension and the core trim differently] → one character class, written once in each
  language, pinned by the same table of cases in a TypeScript and a Rust test.

## Migration Plan

Extension and engine only, nothing stored: shipped with the next extension release (the Apple app
takes the same bundle). Rolling back restores today's capture and review over the same data; the
cards captured trimmed in between are ordinary cards.
