# read-lingua-card-in-another-studied-language — a word the page's language does not know is read in the reader's other language

## Why

Dogfooding change 34's implementation (#810) on 2026-10-11, the owner read a **French page
explaining Spanish conjugation** with an engine holding es-en and fr-en — an English speaker who
studies Spanish and French, as change 52 (`enable-lingua-french`) will allow. The Spanish word
« hablaba » was highlighted, and its card showed **no gloss, no grammar line, and « Rare — beyond
the 20,000 most frequent words »**. The engine was right by its own rule: a document is read in one
language (`lingua-browser-extension`, *Each document is read in its own language*; the session
chooses it once per document, `src/reading/session.ts` `repaint`), so « hablaba » was a French word
the French pack does not know — its lemma « hablaba », its gloss null, its rank none
(`src/analyzer/engine.ts` `wordGrammar`), and the card paths (`src/reading/selection-card.ts`
`openForToken`, `openLooseWord`) drew exactly that. The same engine's es-en pack reads it at once:
*hablar*, imperfect, "to talk, to speak", rank 181.

**The owner's decision (2026-10-11): when a word is unknown in the document's language but known in
another language the reader studies, the card reads it in that language** — « hablaba — imperfect of
hablar — to talk, to speak ». This is that change (row 52b of the
[language matrix programme](../../../docs/lingua/language-matrix-programme.md), outside the 57).

It is not only a stage-3 case. A French speaker who studies English and Spanish meets it today on an
English page about Spanish (`Spanish verbs` on the English Wikipedia: **451 highlighted tokens**
the English pack does not know and the Spanish pack glosses, 2 of them wrong). On a French page —
their own language — they keep today's card (the owner's answer of 2026-10-11, below).

**The owner's answers (2026-10-11)** to the design's three questions: **(1)** a word marked from a
re-read card loses its highlight on the page where it was marked; on another page it is highlighted
again until its card is opened (D6 as proposed); **(2)** one word only — a selection of several words
keeps its card (D8 as proposed); **(3)** no re-read on a page written in the reader's native
language — a French speaker selecting a word on a French page gets today's card, never a Spanish
false friend (« demande » → *demandar* « poursuivre en justice »); the re-read applies only on a page
read in a language the reader studies (D1).

## What Changes

- **The card reads a word the document's language does not know in the reader's other studied
  language** (design D1, D2): on a word card — a click on a highlighted word, a one-word selection
  over one, or a one-word selection no page token covers — whose word the document's pack does not
  know at all (an unknown word with no gloss, no grammar reading, no other dictionary form and no
  frequency rank), the card asks the reader's other accepted languages, in the profile's order, and
  the first whose pack glosses the word as one token answers. A word the reader marked in the
  document's language, a name, a word the document's pack ranks or reads without a gloss, and a
  selection of several words keep their card (D1, D8). **Only on a page read in a language the
  reader studies** (D1, the owner's answer 3): a page with no text in a studied language, or one
  that declares the reader's native language, keeps today's cards. A reader who studies one
  language sees nothing new.
- **The card says what it is** (D4): headed by that language's dictionary form, with its gloss, its
  grammar named as that language's forms (the renderers changes 18, 23, 24 and 51 wrote), its
  frequency from that pack, and one line naming the language — « Mot espagnol », "Spanish word",
  « Palabra española », from the labels module. The word's listen buttons use that language's
  voice; the sentence keeps the document's.
- **Within the card's bound** (D3): the re-read is asked only once the document's answer is empty,
  answered under one more `GRAMMAR_WAIT_MS` (250 ms); past it the card is the one it is today, and
  the re-read replaces it if the card is still shown. No new engine call: the other language's
  `phraseGloss`, `wordGrammar` and rank, through the existing per-language view, its pack loaded the
  first time it is asked (28–38 ms to add natively, measured). Measured cost of one re-read: 8–12 µs
  natively; asked only on a click, never at paint.
- **Recorded in the language it was read in** (D5): the card's gesture carries that language, so
  « Je connais », « + Deck » and the day's learnt-word count land under Spanish, the card storing the
  es-en gloss it showed. Statuses and cards already sync with their language (`lingua-analysis`
  *Sync records carry their own language*, `lingua-sync`): no server, wire, setting or stored format
  changes, and another device follows the languages it accepts, as it does today.
- **The page stays read in its language** (D6): its analysis, highlights, figures and exposures are
  unchanged — no golden and no snapshot moves. A word re-read on the open document is remembered for
  that document and painted by its status in the language it was read in, so « Je connais » takes its
  highlight off at once; leaving the document forgets it (the owner's answer 1).
- **Never into a language whose guard keeps the page's declared language out** (D7): a document
  that declares Catalan, Galician or Occitan gets no Spanish re-read, and one that declares Catalan,
  Occitan or Romanian no French one, the guards' own lists (changes 42 and 42b). Measured on the
  Catalan, Galician and Occitan Wikipedias, a selection there would otherwise read 5.3 %, 22 % and
  4.7 % of the words as the other language.
- **Measured** (D10): over 18 page × reader runs on Wikipedia articles and the three baseline
  corpora (124,458 analysed tokens), the re-read reads up to 451 tokens of a page that quotes the
  other language (61 on the French *Grammaire de la langue espagnole*, 133 on *Espagnol*, 218 and
  451 on the English *Spanish grammar* and *Spanish verbs*), judged wrong for 1 to 4 of them a page,
  each a cognate or a Basque word (*dialectal*, *chapela*); on long pages that quote nothing in the other language it reads
  1 to 12 tokens in 12,000–17,000 (French words the French pack lacks: *bicaméral*, *discordantes*;
  French quotations in a Spanish text read as English: *tiers* → *tier*), and none on the baseline
  corpora.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`:
  - MODIFIED — *Each document is read in its own language*, held by no open change: its
    language-bound questions stay the document's, but for a word card the document's language does
    not know, which the new requirement reads in another studied language; its text and scenarios
    kept, one scenario added.
  - ADDED — *A word the document's language does not know is read in another studied language*,
    *A word read in another language is recorded and painted in that language*, and *No word is
    read in a language whose guard keeps the document's language out*.
  - Read as written, not modified: *The word card says how common its word is* (the band of the
    card's own dictionary form, here the other pack's), *A reader is served the pairs of their
    native language* (the other language is one of the reader's served pairs), *A voice per studied
    language*, and change 52's *French is studied by readers of English and Spanish*.

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension` — *changed*: `src/reading/selection-card.ts` (the re-read in
    `openForToken`, `openLooseWord` and the completion of a card holding its gloss),
    `src/reading/session.ts` (the other languages and their ports handed to the cards, the guard
    table, the gesture written under the card's language, the document's re-read words painted by
    their status), `src/reading/wordpopup.ts` (the language line, the gesture's language, the word's
    voice), `src/reading/speech.ts` (a speaker per language asked), `src/i18n/{fr,en,es}/selection.ts`
    (one line each); *tests*: `test/selection-card.spec.ts`, `test/session*.spec.ts`,
    `test/wordpopup*.spec.ts`, the catalogue's alignment and labels lints.
  - ID, Music, Live, the back office, the site, the backend, `crates/lingua-core`,
    `crates/lingua-wasm` and its six goldens, `crates/lingua-pack`, the tables and packs, the
    extension's committed snapshots, the Apple host app and the agent plugin are untouched.
- **Release.** The extension's next release after changes 34, 35 and 52 (Chromium, Firefox, the
  Safari host app). French-speaking readers of English and Spanish benefit from it as soon as it
  ships; English and Spanish speakers with change 52.
- **Compatibility.** No stored format, wire field, table, pin, pack, analyser version or setting
  moves. A status or card it records is an ordinary record of its language.
- **Order.** Implemented on `main` once the native-language stack has merged — #810 (change 34),
  #814 (change 35) and #880 (change 52), which touch the same session, cards and studied-language
  type, and never stacked on them; archived after those three and after the two guards whose lists
  it reads (changes 42 and 42b) (design D9).
- **Effort**: 2–3.5 ideal days (design *Effort*).
