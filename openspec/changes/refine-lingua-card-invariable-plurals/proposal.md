# refine-lingua-card-invariable-plurals — no card says a word may be its own plural

## Why

Change 51 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-french-word-card`, PR #838) leaves one line off a French card: on a card opened on a
form spelled like its own dictionary form, a plural spelled the same way gives no line (its D5:
`temps`, `pays`, `un`, 1,072 French forms). Its open question 3 asked whether English and Spanish
cards should do the same, since they say it today. The owner settled it on 2026-10-10: **aligned
with French, in a change of its own.** This is that change, outside the programme's 57, row 51b.

Today, measured over every form of the committed packs (design *How it was measured*):
- **Spanish** cards say it of **667 forms**, 13 of them forms of the 1,000 commonest lemmas: `tu`
  « may also be the masculine plural of tu », `menos`, `nadie` (« the masculine and feminine plural
  of nadie »), `paso`, `ex`, `crisis`, `análisis`, `post`, `viernes`, `café`, `lunes`, `clave`,
  `pro`;
- **English** cards say it of **194 forms**, 7 of the 1,000 commonest: `head`, `young`, `police`,
  `military`, `dead`, `box`, `percent` (« peut aussi être le pluriel de police »).

On the commonest words the line reads a homograph's or a lexicographer's plural as the word met
(`tu`, `nadie`, `young`, `dead`), which is change 51's reason for leaving it off French cards; on the
others it says that the word looks the same in the plural, or that a noun used only in the plural is
one (`gafas`, `police`: design open question 3). The same form read in three studied languages
should not be described by three rules.

## What Changes

- **A plural spelled like the card's dictionary form gives no line on that form's own card, in
  every studied language** (design D1): on a card whose form is spelled like its dictionary form,
  a noun's, proper noun's, adjective's, determiner's or pronoun's plural reading, whatever its
  gender, is left out with the dictionary form's own reading. The form's other readings (`paso` still « may also be
  the first-person singular present indicative of pasar ») and other dictionary forms (`dos` still
  « may also be the masculine plural of do », `antes` of `ante`) are said as before. A
  comparative or superlative is still named.
- **The rule lives in the language-neutral description** (D2): `describeReadings` in
  `src/reading/grammar-description.ts` leaves the plural out beside `isDictionaryForm`, keyed by
  nothing, so the French, English and Spanish renderers say the same of every studied language — and
  of French once change 51 names its forms. No renderer changes.
- **Either order with change 51; this one first is recommended** (D3): it waits on nothing, where
  change 51 waits on the implementations of changes 44, 45, 48 and 49. Implemented first, change 51
  needs no `CARD_NAMES` entry and no `composeLines` arm for the plural; implemented after, this
  change removes them. Measured on change 45's implementation, the description's rule leaves out
  exactly change 51's 1,072 French forms.
- **What moves** (D4): the grammar line of 194 English and 667 Spanish forms, in every interface
  language, and nothing else of the 440,534 forms of en-fr, es-fr, es-en and en-es; 794 of those
  cards then show no grammar line, 67 Spanish ones keep another word's line. One committed line:
  `test/baseline/word-card-es-en.txt` loses « may also be the masculine plural of menos ». The line is
  computed in the extension only: no golden of `crates/lingua-wasm` moves (they carry readings), no
  pack byte, pin, `pack_version` or analyser version.
- **en-fr and es-fr move, with the owner's approval** (D5): the programme's rule « en-fr and es-fr
  output does not move, nor the French interface » is departed from, by the owner's decision; the
  owner approves the moved lines — 194 en-fr and 667 es-fr cards in French, and es-en's snapshot —
  before the merge.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED — *The word card says what the form is*, held by no open
  change: its « What the form is » leaves out, on a form spelled like its dictionary form, a plural
  spelled like it; its text and scenarios kept, two scenarios added. Read as written, and not
  modified while their changes hold them (design D6): *The word card describes a form once, and says
  it in the interface language* (change 18: « the French rendering byte for byte what the card said
  before » is its own refactor's guarantee; every renderer still names what the French one names),
  *A Spanish card names its forms as French schools do* (change 18: « a noun's … form SHALL name its
  gender and number » of a form the card names), *The card of English-native readers of Spanish is
  pinned on the real pack* (change 23: its snapshot re-blessed in the pull request that moves it, as
  it asks) and *The card of Spanish-native readers of English is pinned on the real pack* (change 24:
  its snapshot does not move).

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension` — *changed*: `src/reading/grammar-description.ts` (`describeReadings`,
    one predicate beside `isDictionaryForm`); *tests*: `test/grammar-description.spec.ts` (the rule
    without words, and through the three renderers), `test/word-grammar-en.spec.ts` and
    `-es.spec.ts` (a case each); *re-blessed*: `test/baseline/word-card-es-en.txt` (one line);
    *unchanged*: `test/word-grammar.spec.ts`, `word-card-en-es.txt`, `selection-rows-fr.txt`,
    `voice-ranking.txt`, every renderer; *consumed*: the description and the renderers (change 18),
    the card snapshots and their harness (changes 23, 24).

  ID, Music, Live, the back office, the site, the backend, `crates/lingua-core`, `crates/lingua-wasm`
  and its goldens, `crates/lingua-pack`, the tables and packs, the Apple host app and the agent
  plugin are untouched.
- **Release.** The extension's next release (Chromium, Firefox, Safari's host app): French-speaking
  readers of English and Spanish see 861 cards without the line. No copy, store listing or setting
  moves.
- **Compatibility.** No stored format, wire field, table, pin or pack moves.
- **Order.** Independent of every open change: before change 51's implementation (recommended) or
  after it (D3); in either order with `fix-lingua-lemma-lookup` (41b), which moves neither
  `word-card-es-en.txt` nor any of these forms (they are lemmas read as themselves). Change 56
  (`refine-lingua-matrix-wording`) keeps only change 51's other shared wording, a bare plural beside
  gendered ones.
- **Effort**: 0.5–1 ideal day (design *Effort*).
