# add-lingua-phrase-gloss — answer a selection from the pack, honestly

## Why

Selecting several words in the Lingua extension opens a card that says "Pas de traduction
dans le pack" — always, because a phrase is never looked up. The reader gets nothing at the
moment they ask for help, on every browser. Two nearby defects have the same cause, a
selection being handled as raw text instead of going through the analyser:

- a single word selected in a block the page analysis skipped is keyed by the form as
  written, so "+ Deck" on `endeavors` creates a card and a status the page classifier never
  looks up, and the word stays highlighted;
- a word the reader already knows shows no translation at all, because the page analysis
  withholds the gloss of known words and the popup never asks for it.

This is the first step of multi-word translation, and the only one that needs no model, no
download and no new permission: it works on the five targets (Chromium, Firefox desktop and
Android, Safari macOS and iOS) and stays the floor under every later layer. Machine
translation comes in later changes and never replaces this answer when it is unavailable.

**Products.** Cymbra Lingua only: `crates/lingua-core`, `crates/lingua-wasm` and
`apps/lingua-extension` (all three variants; the Safari variant ships inside
`apps/lingua-apple`, whose native code does not change). Everything here is new to Lingua
and nothing is consumed from the platform. Cymbra ID, Music, Live, the back office, the site
and the backend are untouched: no `.proto`, no migration, no flag.

## What Changes

- **A phrase gloss in the core.** `lingua-core` gains a deterministic function that reads a
  short text the way the page analysis does — same tokeniser, same lemma cascade, same
  classification — but without the two gates meant for pages (language detection and the
  minimum token count), and returns for **every** token its dictionary form, its status
  class, its pack gloss whatever that class, and whether it is a function word; a hyphenated
  compound the lexicon does not list also carries its parts, each described the same way.
  It is exposed by `lingua-wasm` and added to the extension's `AnalyzerPort`.
- **A selected word is a dictionary form wherever it was selected.** A selection holding no
  whitespace — a hyphen no longer sets it apart — opens the word card: from the page token
  when the page analysis covered it, exactly as today, and from the analyser otherwise, so
  the card and the status are keyed by the dictionary form there too. A name the analyser
  takes for a proper noun keeps the card it has today, under the text as written.
- **Word by word is a last resort, and says so.** A selection of several words — or a
  compound the lexicon does not list — with no better answer shows the pack glosses of the
  words the reader does not know, function words left out, a few rows at most, under a line
  stating that this is not a translation of the expression. When nothing is worth showing,
  the card says so plainly. These rows are a reading aid: they are never stored on a card.
- **A known or ignored word shows its gloss** when its card is opened.
- **A card that waits for the engine opens at once, and offers its actions with the
  answer.** On Firefox and Safari the engine lives in the event page, which Safari suspends
  constantly. The card appears pending, then becomes complete exactly once — by the answer,
  or by a fallback when the engine fails or stays silent — so nothing the reader can press
  moves or changes meaning; an answer meant for a card the reader has left, closed or seen
  completed is dropped. The click that ends a mouse selection no longer closes the card it
  opened, and still never follows the link of an untreated word.

Not in this change: the multi-word expression table in the pack
(`add-lingua-expression-table`), any translation engine, the opt-in translation setting,
contexts on the review card, and the source sentence found by text search (`put` matching
`input`), which is its own fix. No manifest, permission, store listing or privacy text
changes, the card model and the sync contract do not move, and `ANALYZER_VERSION` does not
move — the page analysis output is byte-identical.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-analysis`: a new requirement, the phrase gloss — reading a short text without the
  page gates, with a gloss for every token, under the same native/WASM parity contract.
- `lingua-browser-extension`: new requirements for a selected word resolving to its
  dictionary form, for word-by-word as a labelled last resort that is never stored, for the
  gloss of a known word, and for a card that waits for the engine.

Every delta is an ADDED requirement. "Word popup on click" and "Selection capture on any
pointer" keep their text and stay true: a selection of one word still opens that word's
popup — this change makes it do so in the blocks where it did not.

## Impact

- **Rust**: `crates/lingua-core` (`engine.rs` gains the phrase gloss and shares the token
  classification with the page analysis; `analysis/pipeline.rs` makes the lemma resolution
  visible inside the crate; a new closed-class word list for English), `crates/lingua-wasm`
  (`phraseGloss` binding; a second parity golden, the existing one untouched as the proof
  that the page analysis did not move). Host-tested, inside the Rust 80% gate.
  `apps/lingua-agent` links the core natively and is not affected by an added function.
- **Extension**: `src/analyzer/` (`port.ts`, `engine.ts`, `messaging-port.ts`, `types.ts`),
  `src/reading/selection.ts` (whitespace decides what a phrase is), `src/reading/wordpopup.ts`
  (pending state, rows, the label, a generation the card view owns), a new
  `src/reading/selection-card.ts` that owns every decision — kept out of the coverage exclude
  list and tested — and `src/content.ts` as the thin caller. The build guard already refuses
  a bundle whose wasm glue lacks a declared method, so a stale `src/wasm/pkg` fails the
  build instead of the reader.
- **Prerequisite**: pull request #518 merged. It added `packGlossKey`, a hyphenated
  selection looked up whole; this change removes it, since the word card covers that case
  and the unlisted compounds it missed.
- **Dogfooding**: the rows are only meaningful with the real pack (`yarn gen:pack:real`);
  the test pack glosses a handful of words.
- **Release**: an ordinary extension release on the two stores, then a dispatch of
  `lingua-apple-release` with `deliver` — an extension-only change never moves the
  `lingua-apple` tag, so Safari readers stay behind unless someone sends them the build.
  Nothing for a store reviewer to re-read: no new origin, no new permission.
