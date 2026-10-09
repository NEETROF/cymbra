# refine-lingua-card-invariable-plurals — a word read in both numbers no longer says it may be its own plural

## Why

Change 51 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-french-word-card`, #838) leaves one line off a French card: on a form spelled like its
dictionary form, the plural spelled the same way (its D5: `temps`, `pays`, `un`). Its open question 3
asked whether English and Spanish cards should do the same. The owner settled it on 2026-10-10 —
**aligned with French, in a change of its own** — and its boundary on 2026-10-11: **the line goes
only for a true invariable**, a form the card also reads as a singular of that part of speech
(`crisis`, `lunes`, `temps`); a noun used only in the plural keeps it (`gafas`, `gens`). This is that
change (row 51b, outside the 57).

Today **Spanish** cards say it of **576 such forms**, 13 of the 1,000 commonest lemmas: `tu`
« may also be the masculine plural of tu » (a homograph's), `menos`, `nadie`, `crisis`, `lunes`.
**English** has none: its table writes no noun's singular, so `police` keeps its line.

## What Changes

- **A plural read beside a singular of the same part of speech gives no line on that form's own
  card, in every studied language** (design D1): on a card whose form is spelled like its dictionary
  form, a noun's, proper noun's, adjective's, determiner's or pronoun's plural reading without a
  degree, whatever its gender, is left out with the dictionary form's own reading when the card also
  reads the form as a singular of that part of speech. A plural with no such singular keeps its line
  (`gafas`, `afueras`, French `gens`; every English one, `police`, `fish`). The form's other readings
  (`paso` still « may also be the first-person singular present indicative of pasar ») and other
  dictionary forms (`dos` still « may also be the masculine plural of do », `gafas` of `gafa`) are
  said as before. A comparative or superlative is still named.
- **The rule lives in the language-neutral description** (D2): `describeReadings` in
  `src/reading/grammar-description.ts` leaves the plural out beside `isDictionaryForm`, keyed by
  nothing, so the French, English and Spanish renderers say the same of every studied language — and
  of French once change 51 names its forms. No renderer changes.
- **Either order with change 51; this one first is recommended** (D3): it waits on nothing, where
  change 51 waits on the implementations of changes 44, 45, 48 and 49. Change 51's D5 takes this
  boundary in either order: measured on change 45's implementation, French leaves the line out on
  1,007 forms, not 1,072 — `gens`, `environs`, `plusieurs` and 62 more keep theirs. Implemented first,
  change 51 needs no `CARD_NAMES` entry and no `composeLines` arm for the plural; implemented after,
  this change removes them.
- **What moves** (D4): the grammar line of 576 Spanish forms, in every interface language, and nothing
  else of the 440,534 forms of en-fr, es-fr, es-en and en-es; 542 of those cards then show no grammar
  line, 34 keep another line. No English card moves. One committed line:
  `test/baseline/word-card-es-en.txt` loses « may also be the masculine plural of menos ». The line is
  computed in the extension only: no golden of `crates/lingua-wasm` moves (they carry readings), no
  pack byte, pin, `pack_version` or analyser version.
- **es-fr moves, approved by the owner** (D5): the programme's rule « en-fr and es-fr output does not
  move, nor the French interface » is departed from for es-fr's 576 cards in French, by the owner's
  decision; the owner approved the re-bless on 2026-10-11 (task 4.1). en-fr does not move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED — *The word card says what the form is*, held by no open
  change: its « What the form is » leaves out, on a form spelled like its dictionary form, a plural
  read beside a singular of the same part of speech; its text and scenarios kept, three scenarios
  added. Read as written, and not modified while their changes hold them (design D6): *The word card
  describes a form once, and says it in the interface language* (change 18: « the French rendering
  byte for byte what the card said before » is its own refactor's guarantee; every renderer still
  names what the French one names), *A Spanish card names its forms as French schools do* (change 18:
  « a noun's … form SHALL name its gender and number » of a form the card names), *The card of
  English-native readers of Spanish is pinned on the real pack* (change 23: its snapshot re-blessed in
  the pull request that moves it, as it asks), *The card of Spanish-native readers of English is
  pinned on the real pack* (change 24: its snapshot does not move) and change 51's *A French card
  names its forms in the interface language's grammar* (« a plural spelled like that form » read as
  this boundary).

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
  readers of Spanish see 576 cards without the line; readers of English see no change. No copy, store
  listing or setting moves.
- **Compatibility.** No stored format, wire field, table, pin or pack moves.
- **Order.** Independent of every open change: before change 51's implementation (recommended) or
  after it (D3); in either order with `fix-lingua-lemma-lookup` (41b), which moves neither
  `word-card-es-en.txt` nor any of these forms (they are lemmas read as themselves). Change 56
  (`refine-lingua-matrix-wording`) keeps only change 51's other shared wording, a bare plural beside
  gendered ones.
- **Effort**: 0.5–1 ideal day (design *Effort*).
