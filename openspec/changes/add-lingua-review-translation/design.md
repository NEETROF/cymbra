# add-lingua-review-translation — design

## Context

See proposal.md for the why. What exists:

- **The translator.** `createTranslatorPort()` (`src/translate/create-port.ts`) gives a source,
  `language → TranslatorPort | null`: null unless the reader turned « Traduction étendue » on for
  this device and every model of that language's route is on it (`languageReady`). The port is
  only ever the messaging port, sending the request to the engine's host off every thread that
  paints; it is wrapped to keep the engine loaded between requests (`keepalive.ts`) and to answer
  a request it has already answered from the page's memory (`answer-memory.ts`: the same
  language, sentence and span; 32 answers; never stored). `warm(language)` has the engine load a
  route ahead of a request. `test/lint-translator-placement.spec.ts` walks the import graph of
  every painting surface — the side panel included — and fails if one reaches `src/translate/host`.
- **Who asks today.** Only the reading session creates a source (`reading/session.ts`) and hands
  it to the word and selection cards (`reading/selection-card.ts`). An expression card always asks;
  a single word asks only when the pack has no gloss for it and it is not a proper noun outside the
  lexicon, in its sentence with the word marked, never alone. The answer is awaited for at most
  `TRANSLATION_WAIT_MS` (15 s). The host drops footnote calls, marks by position, marks only the
  languages of `MARKED_LANGUAGES`, and translates Spanish through English.
- **How the card shows it.** `renderTranslation` (`reading/wordpopup.ts`): the label « Dans votre
  phrase — traduction automatique », then the sentence with each mark as a `<mark>`, built from text
  nodes, a span out of range skipped.
- **The review**, after `refine-lingua-review-session`: `mountReview` (`review/review-page.ts`)
  drives a `ReviewController` and repaints through `renderReview` (`review/view.ts`); the answer
  space is reserved before the reveal (D6 there) and holds the dictionary form and the gloss, or
  « Pas de traduction pour ce mot. » / « … pour cette expression. ». A card carries its headword,
  the form met, its sentence, its gloss and its language. The drawer and Chrome's side panel host
  the page; the drawer is created by the reading session, the side panel has no session.
- **`lingua-translation` holds « A machine translation is never stored »**: computed for display
  and discarded, never on a card, never a gloss, never sent.

## Goals / Non-Goals

**Goals:**
- A card the pack cannot answer is answered at review wherever a model is ready, with the old
  cards of a deck included.
- On e-ink, the translation is part of the answer at the reveal, so the reveal stays one repaint.

**Non-Goals:**
- Translating cards that have a gloss (D1).
- Keeping a translation on a card: `add-lingua-card-sentence-translation`, next.
- Preparing the next card's translation before it comes up (Risks).
- Any engine, model, catalogue or route change: the routes by pair of the language matrix
  programme reach the review through the same source as the word card.

## Decisions

### D1 — The cards translated are those the word card translates

An expression, or a single word without a gloss, is translated; a card with a gloss shows its gloss
alone and costs the engine nothing. The review answers where the word card answered at capture,
and `add-lingua-card-sentence-translation` will keep exactly those translations, so the two changes
cover the same cards — this one for the cards already in the deck, that one from capture on.

*Alternative.* A sentence translation under every gloss: an engine call per card on the e-ink
reader, and a second answer competing with the gloss the reader is testing. Worth reconsidering
only if the dogfood asks for it.

### D2 — Asked when the card comes up, warmed when the review starts, never waited for

When a card without a gloss becomes the current card, the page asks for its translation at once,
before the reveal; the reader spends that time recalling. When a review starts, the page warms the
route of its language, so the first card does not pay for a cold engine. The request is bounded by
`TRANSLATION_WAIT_MS`, as the cards' are, and its answer is tied to the card it was asked for: an
answer that lands after that card was answered is dropped.

The reveal never waits. If the answer is not there yet, the answer space says « Traduction en
cours… », as the word card does, and the translation replaces that line when it lands; when none
comes, the no-translation note is shown, as today.

*Alternative.* Asking at the reveal: a wait or a second repaint on every card instead of on the
rare late one.

### D3 — The request is the card's sentence, the word marked where the front marks it

The span is where the review's front marks the word in its sentence (`markWord`: the form met,
else the dictionary form, a whole word, letter case aside), in UTF-16 offsets, which the host marks
by position. A card whose sentence does not hold its word:
- an expression is sent alone, with no span, and shown under « Traduction automatique » rather
  than « Dans votre phrase — … »: such cards were captured before the sentence was taken from the
  selection's position (#523), and an expression translated alone still says what it means;
- a single word is not sent: a word translated alone is the guess the word card never shows.

### D4 — The translator comes from the host

`mountReview` takes an optional `TranslatorSource`; without one the review is today's. The drawer
receives the reading session's source and hands it on, so the drawer's review shares the session's
engine host and answer memory with the word cards. Chrome's side panel calls `createTranslatorPort()`
itself: the messaging port is all it reaches, which the placement lint already checks. The EPUB
reader reviews through its drawer.

*Alternative.* `mountReview` creating its own source: a second answer memory and setting watcher
in the drawer's frame, and no seam for tests.

### D5 — The translation sits in the answer space, labelled as the word card labels it

Once revealed, the answer space shows the dictionary form and the gloss as today, then the label
and the translated sentence, each mark a `<mark>`, all of it text nodes — the text came from a page
through the engine. Without a gloss, the translation takes the place of the no-translation note.
The review's read-aloud does not read it: its voices speak the studied language. The block takes
its colours from `tokens.css` and its sizes from `--cymbra-lingua-ui-scale`, with no transition.

### D6 — Nothing is kept

The translation lives in the page as long as its card: no card field, no backup, no sync. The
answer memory is the page's and goes with it. « A machine translation is never stored » stays true
as written. When `add-lingua-card-sentence-translation` lands, a card that keeps a translation shows
it and the engine is not asked; that change, which stores and synchronises a translation, will have
to modify that requirement.

## Risks / Trade-offs

- [On e-ink, a translation that lands after the reveal repaints the answer space a second time]
  → asked on arrival, route warmed at the start; checked in the e-ink dogfood. Preparing the next
  card ahead would need an engine binding that names the upcoming card, whose answer can still move
  when a missed card comes back: a follow-up if the dogfood shows late translations.
- [The engine stays loaded while the reader reviews] → the same keepalive as reading
  (`ENGINE_IDLE_MS`), released after.
- [An expression translated alone loses its sentence's context] → only for cards whose sentence
  does not hold the expression, and labelled as such.
- [Spanish goes through English, two models] → the warm loads the whole route; the word card
  already does the same.

## Migration Plan

Extension only, nothing stored: shipped with the next extension release. Rolling back restores
today's review over the same data.
