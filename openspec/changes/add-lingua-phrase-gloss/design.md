## Context

A selection reaches `content.ts` through `SelectionWatcher`, which classifies it with a
regular expression — a space or a hyphen makes it a `phrase`, anything else a `word` — and
hands it to `onCapture`. From there:

- a `word` is hit-tested against the tokens of the page analysis (`hitAt` at the range
  start, first token covering the caret), which gives a lemma-correct, status-aware word
  card; on a miss — the block was never analysed — the card is built from the raw text:
  `port.gloss(text.toLowerCase())`, headword = the text, and the popup's gesture uses the
  headword as the key, so the card and the status are created under the form as written;
- a `phrase` gets `gloss = null` and the card reads "Pas de traduction dans le pack" — since
  pull request #518, a prerequisite of this change, unless it holds no whitespace, in which
  case `packGlossKey` looks the hyphenated text up whole. D4 replaces that shortcut.

The page analysis cannot answer a selection. `analyse_document` drops a block shorter than
12 bytes or not detected as the studied language, and reports fewer than 10 tokens as not
analysable (`analysis/language.rs`, `analysis/pipeline.rs`); `analyse_page` fills `gloss`
only for unknown and learning tokens (`engine.rs`), and its `AnalyzedToken` drops the parts
the pipeline found in a hyphenated compound. The WASM API offers `analyse(blocks)` and
`gloss(form)`, nothing that lemmatises a text.

The engine does not live in one place: Chromium runs it in the content script, with the
service worker as the fallback on pages whose CSP blocks WASM; Firefox and Safari run it in
the event page, reached by one `runtime.sendMessage` per call, and Safari suspends that page
constantly. Any answer to a selection is therefore asynchronous, can be slow to start, and
can fail.

Two facts about today's flow shape the design. `onClick` hides a visible card whenever the
click does not land on a painted word, and a mouse selection ends with a `click` dispatched
after the `mouseup` that flushes the capture; the card survives today only because a
`[-\s]` guard returns early for phrases and because the raw-text card is shown after an
awaited call. And the card's layout puts the answer above the actions, so content that
arrives late pushes the buttons down — or flips the whole card above the selection.

The founder's rule for this layer: a word-by-word gloss is a last resort, shown only when
there is no better answer, and it must tell the reader what it is.

## Goals / Non-Goals

**Goals:**

- Answer a multi-word selection from the pack on the five targets, with no model, no
  network, no permission and no store text to change.
- Make a selected word a dictionary form wherever it was selected, without changing what a
  selection opens where the page analysis already covers it.
- Keep the page analysis output byte-identical: no `ANALYZER_VERSION` change, no pack
  change, the existing parity golden untouched.
- Put every decision in a tested module, because `content.ts` is excluded from coverage and
  the translation layers that follow will reuse the same card behaviour.

**Non-Goals:**

- Recognising expressions (`give up`, `in spite of`). That is the pack's expression table,
  the next change; until then a phrase the reader's words do not explain gets an honest
  "no translation".
- Any translation of the selection or of its sentence. No engine, no download.
- Storing anything new on a card. The card model and the sync contract do not move.
- The source sentence found by text search (`put` matching `input`): a separate fix, which
  touches neither the analyser nor the pack. The offsets of a selection inside its sentence
  come with the change that consumes them.
- Re-keying cards created under a surface form before this change, and the status a reader
  can already put on a text the analyser finds no token in (`B2B`).

## Decisions

### D1 — The phrase gloss is a core function, not a mapping done in the extension

`engine::gloss_phrase(text, studied, pack, knowledge) -> PhraseGloss`, serialised by
`gloss_phrase_json` like `analyse_page_json`, bound in `lingua-wasm` as
`phraseGloss(text) -> String` and added to `AnalyzerPort` as
`phraseGloss(text): Promise<PhraseGloss>`.

It runs `tokenize` and the lemma resolution of the pipeline on the text as one block, with
no gate, and classifies each token exactly as `analyse_page` does. The classification block
of `analyse_page` (plain token, listed compound, unlisted compound, out-of-lexicon proper
noun) moves into one private function both callers use, so the two can never disagree about
a token; `pipeline::resolve_lemmas` becomes `pub(crate)`.

```
PhraseGloss { tokens: [PhraseToken] }
PhraseToken { surface, lemma, class: TokenClass, gloss: Option<String>,
              function_word: bool, parts: [PhrasePart] }   // parts omitted when empty
PhrasePart  { lemma, class: TokenClass, gloss: Option<String>, function_word: bool }
```

A part is described like a token — classified on its own with `knowledge.classify` —
because the row rule (D5) has to filter parts exactly as it filters words: `opt-in` must not
show `in`, and `error-prone` must not show an `error` the reader knows.

_Alternative considered_ — map the selection onto the tokens the content script already
holds and call `gloss(lemma)` per token. Rejected: it answers nothing in a block the page
analysis skipped, which is where the surface-keyed card bug lives; it has no gloss for known
words, no compound parts and no proper-noun verdict; and it would put lemma logic in
TypeScript, outside the parity contract.

_Alternative considered_ — relax the gates of `analyse` behind a flag, or add parts and
every gloss to `AnalyzedToken`. Rejected: `analyse` is the parity surface and the golden
pins its bytes; it would also grow every page analysis by the glosses of the words the
reader knows, which is most of them, to serve a card opened now and then.

### D2 — No gate, on purpose

The language gate needs 12 bytes and whichlang's verdict; the token gate needs 10 tokens.
Both protect the page percentage from noise. A selection is the opposite case: the reader
points at a few words and asks. A text in another language simply comes back unglossed.

### D3 — Function words are a closed list in the core, per studied language

A new `analysis/function_words.rs` holds the English closed classes as dictionary forms, in
the six classes the spec names — articles and other determiners (possessives and
demonstratives among them), pronouns, prepositions and particles, conjunctions, auxiliaries
and modals (`be`, `have`, `do`, `will`, `can`…), negation — and
`is_function_word(lemma, studied)`. It sits beside the other English tables the core already
carries (irregular forms, contraction pre-pass). `have` and `do` are on the list: glossing
them inside a phrase tells the reader nothing.

_Alternative considered_ — a frequency-rank cut. Rejected: the hundred most frequent lemmas
hold `time`, `people`, `say`, `know`, which are exactly the words worth a row.

_Alternative considered_ — part-of-speech tags in the pack. Rejected for this change: it
alters the pack format and the reducer for a filter a short list does well enough.

### D4 — Routing a selection

Whitespace decides what a phrase is, which is also what the core means by an expression
(`Card::is_expression` tests for a space). `classifySelection` drops its hyphen rule.

1. **The selection holds whitespace** → the expression card, keyed by its lowercased text
   as today, `expression: true` ("+ Deck" and "Ignorer"). It opens pending and asks
   `phraseGloss(text)` (D5, D6).
2. **No whitespace, a page token at the range start** → the word card of that token.
   `hitAt` is unchanged — first token covering the caret — so `don't` still opens `do`, and
   a hyphenated compound, which is one page token, now opens its word card instead of the
   phrase card. A token that needs nothing more — a highlighted word, whose gloss came with
   the page analysis — opens complete, exactly as today. What a page token can lack is
   fetched by one follow-up, and that card opens pending (D6): for a hyphenated token with
   no gloss of its own, `phraseGloss(surface)`, whose single token brings its gloss whatever
   its class and the parts of a compound the lexicon does not list; otherwise, for a known
   or ignored word, `port.gloss(lemma)`.
3. **No whitespace, no page token** → a pending card, then `phraseGloss(text)`. The first
   token decides, as `hitAt` would have: headword = its dictionary form, form seen = its
   surface, status and rarity from its class, gloss and parts from the token. If that token
   is a proper noun outside the lexicon — its dictionary form is an artefact of the plural
   fallback (`Jenkins` → `jenkin`) — or if there is no token at all, the card is the
   raw-text card such a selection opens today: headed and keyed by the text as written,
   `expression: false`, the "Sélection." line, the three actions. It is not the expression
   card of step 1.

Proper nouns are never page-token hits (they are not clickable), so every selected name
takes step 3, on analysed pages too; that is why the rule is stated there.

`onClick` does two jobs, and only one of them changes. It still cancels a click that lands
on a painted, untreated word inside a link, exactly as today. What it stops doing is opening
or hiding a card with the click that ends a selection gesture: the capture flushes on
`mouseup`, the `click` follows, and `onClick` hides any card when that click does not land on
a painted word — so the pending card of steps 1 and 3, opened synchronously, would be closed
by the gesture that asked for it. The rule is about the gesture, not about the selection: a
card opened by a capture since the last pointer-down makes the following click leave cards
alone. Testing for a live selection instead would be wrong — a click on a link, a button or
inside the selection does not collapse it in Chromium, and the reader never clears the page
selection, so such a guard would swallow real clicks long after the gesture. The existing
`[-\s]` guard stays as it is.

### D5 — The rows, and what they are not

The candidates are the tokens of the answer. A compound the lexicon does not list is
replaced by its parts, alone or inside a phrase — unless its own class is known or ignored:
`classify_compound` lets a status the reader put on the compound win over its parts, and a
compound the reader has dealt with gives no row. A candidate makes a row when its class is
unknown or learning, it is not a function word, and the pack glosses it; one row per
dictionary form, in reading order, at most **6**. Above them, one line in the reader's language — « Mot à mot — ce n'est pas
une traduction de l'expression. » — and when no row qualifies, « Pas de traduction dans le
pack pour cette expression. ». The copy avoids the word "lemme" (the `lint-lemma` test) and
the rows use token colours only (`lint-hex`).

Known words are left out because a row for a word the reader knows is noise, and because on
a phrasal verb it is worse than noise: `put up with` would show `put → mettre`. With `put`
known and `up`, `with` function words, the card says it has no translation — which is true
until the expression table lands.

"Only when there is no better answer": in this change nothing is better, so the rule is
unconditional. It is written as a rule so that the expression table and the translator can
each add their answer ahead of it without rewriting it.

The rows are never stored, and that is decided in code that is tested, not left to what the
pack happens to hold: `Gesture` gains `expression`, and the card's gloss is chosen by one
function — nothing for an expression, `port.gloss(lemma)` for a word. Today the same result
falls out of `port.gloss("put up with")` finding no entry, a property of the pack the next
change exists to alter.

### D6 — One tested module owns the decisions; a card is pending, then complete

`src/reading/selection-card.ts` holds, with injected ports and no DOM: the routing of D4,
the follow-up a page-token card needs, the row rule of D5, the choice of a card's gloss, the
click rule of D4, and the two class mappers a word card is built from — `statusOfClass` and
`rarityText` move there from `content.ts`, since step 3 builds a word card for a token that
never was a page token. `content.ts` stays the thin caller it is.

A card that needs the engine has two states and one transition. **Pending**: headword as
selected, the kind line, a quiet waiting line, the close button — and no action.
**Complete**: an ordinary `show` of the final content, which may differ from the pending
one in everything — headword, form seen, status, actions — and is positioned afresh. There
is no in-place patch: the only thing that ever changes a card is `show`, so the card's
gesture key is always the one on screen.

A request is **settled exactly once**, by whichever comes first: the engine's answer, its
rejection, or a **3-second** timer. The fallback states that the pack has no translation and
offers actions only when their key does not depend on the answer: the expression card's
(keyed by the text), and a page token's (keyed by its dictionary form, offered by its
status). The pending card of step 3 has no key without the answer, so its fallback offers
nothing to press — better than recreating the surface-keyed card this change removes. An
answer that arrives after the fallback is dropped.

Staleness belongs to the card view, not to captures. `CardView` keeps a generation that
every `show` and every `hide` bumps — including its own close button and the hide that
follows an action, which never pass through `WordPopup` — and exposes it; the module
completes a request only if the generation is still the one its pending `show` returned. A
card opened for something else, a scroll, Escape, the close button or the reader being
switched off all make the answer land nowhere.

Waiting costs the reader one engine call before "+ Deck" on the cards that need one — a few
milliseconds where the engine is local, the wake of the event page elsewhere — and buys a
card whose buttons never move or change meaning under a finger. The common card, a
highlighted word whose gloss came with the page analysis, needs no answer and opens
complete, as today.

### D7 — Test data

Core host tests build their lexicon and pack inline, as `engine.rs` and `pipeline.rs` already
do, holding exactly the forms each scenario names. The parity proof is a second committed
golden in `crates/lingua-wasm/tests/fixtures/`, for selections made of the fixture pack's own
words. It is written under its own variable, `LINGUA_UPDATE_PHRASE_GOLDEN`: the existing
helper returns without asserting when `LINGUA_UPDATE_GOLDEN` is set, so reusing that variable
would rewrite `golden.json` in the same run and absorb silently the drift this change
promises not to cause. `golden.json` matching with no diff is the proof that the page
analysis did not move.

## Risks / Trade-offs

- [Word-by-word glosses are weak: pack glosses ignore part of speech and some read like
  definitions] → the label says what they are, known words and function words are left out,
  the rows are bounded and never stored; the copy is settled on real pages with the real
  pack before release. Removing the rows later would be a spec change of its own.
- [Actions wait for one engine call on every card that needs an answer] → it is the price of
  buttons that do not move; the common card — a highlighted word, whose gloss came with the
  page analysis — needs no answer and is as immediate as today.
- [On a slow wake, a word selected outside the page analysis gets a card with nothing to
  press] → it says so and the reader selects again; the alternative is a status under a key
  nothing ever looks up.
- [A stale `src/wasm/pkg` lacks `phraseGloss`] → `assertWasmMatchesEngine` already fails the
  build on a method declared on `WasmEngine` and absent from the glue.
- [The function-word list is a judgement] → it is lemmas of closed classes only, pinned by
  tests per class; adding a studied language means adding its list, like its tokeniser
  pre-pass.
- [Cards created under a surface form before this change stay as they are] → not migrated;
  they are few (single words selected in unanalysed blocks) and re-keying a card would have
  to merge review histories.
- [The extension has no coverage threshold, so "tested" is a discipline, not a gate] → the
  new module stays out of the exclude list and each scenario names its test; adding a
  threshold is a change to the check lane, not to this feature.

## Migration Plan

None. No stored data, contract, manifest or pack changes. Rollback is a revert.

## Open Questions

- The exact French copy of the two lines in D5, to be read on a phone before release.
- Whether 6 rows is the right bound once seen on real pages.
