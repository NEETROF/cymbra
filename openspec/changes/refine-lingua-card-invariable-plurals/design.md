# Design — refine-lingua-card-invariable-plurals

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/lingua-extension/src/reading/grammar-description.ts` (change 18) | the description: `describeForm` sets `sameAsHeadword` when the form, letter case aside, is spelled like the card's dictionary form, and `describeReadings(tags, same)` merges the readings by tag, leaving out on that form's own card every tag `isDictionaryForm` accepts — an infinitive, a noun's singular, an adjective's masculine singular; `formKind` names a nominal plural (« pluriel », "plural", « plural ») with its gender; `composeLines` writes « peut aussi être … de X » / "may also be the … of X" / « también puede ser … de X » for a form spelled like its dictionary form, then a line per other dictionary form |
| `src/i18n/{fr,en,es}/grammar.ts` | the three renderers; none decides what is named |
| `src/reading/wordpopup.ts` | the only caller of `grammarLines`: the word card, on pages and in the book reader |
| The engine | `lingua-core` `engine::word_grammar` answers a form's readings as the card's dictionary form, unmerged and unfiltered; `crates/lingua-wasm/tests/baseline/*.golden` record that answer as JSON — readings, never lines |
| Snapshots | `test/baseline/word-card-es-en.txt` (change 23) and `word-card-en-es.txt` (change 24) render the goldens' grammar probes into lines; `selection-rows-fr.txt` pins rows, which read glosses only |
| Change 51 (`add-lingua-french-word-card`, proposal merged as #838, implementation not started) | its D5 leaves the plural off a French card, keyed by studied language in a `CARD_NAMES[studied]` table and applied in `composeLines` (its task 1.1), English and Spanish keeping theirs; its open question 3, settled by the owner on 2026-10-10: « aligned in a change of its own, proposed separately »; its D11 had handed « an invariable plural on Spanish and English cards » to change 56 |

A form spelled like its dictionary form gets a plural reading when the dictionary records the same
spelling in the plural: a true invariable plural (`sheep`, `species`, `crisis`, `lunes`), an adjective
used as a plural noun (`the young`, `the dead`), a homograph's plural (`tu`, the Greek letter), a
lexicographer's entry read through another part of speech (`nadie`, `menos`), or a noun used only in
the plural (`gafas`, `afueras`, French `gens`, `environs`). The card cannot tell the first four apart:
all are `NOUN|…|Number=Plur`, `ADJ|…|Number=Plur` or the like on a form equal to its lemma, beside a
singular reading in Spanish and French. The last has no singular reading in Spanish (91 of the 667
forms, `once` « eleven » among them, a table oddity) and French (64 of the 1,072); English's table
writes no noun's singular at all, so `police`, `headquarters` and `fish` read alike (open question 3).

**How it was measured.** A prototype, never committed, in a scratch copy of `origin/main` at
`c6a7699d`:
- a scratch test in `crates/lingua-wasm` builds en-fr, es-fr, es-en and en-es from the committed
  tables (`PackSource::Tables`), takes every form of `tables/en/forms.tsv` (75,315) and
  `tables/es/forms.tsv` (144,952), resolves its dictionary form as the page analysis does
  (`tokenize`, then `resolve_lemmas` on the first token, made public in the scratch copy — the
  page's own steps, without the block gates that a one-word block fails) and asks `word_grammar` — 440,534 answers;
- the extension's renderers, `main`'s and a copy with D2's rule, run under Node on every answer, the
  card opened as `wordpopup.ts` opens it (`surface` the first token, `written` the form), in each of
  the three interface languages; ranks are the dictionary form's in `tables/<studied>/freq.tsv`;
- French: the same on change 45's implementation (`claude/add-lingua-french-grammar-tables-impl` at
  `e58b14c1`, whose `tables/fr/grammar.tsv` holds French's readings), fr-en's pack, 124,096 forms,
  rendered in English and Spanish by renderers given no French table — finite French forms unnamed
  on both sides, so only this rule's lines differ;
- the extension's whole suite on the scratch copy with the rule, and the card snapshots re-blessed
  there to read their diff.

## Goals / Non-Goals

**Goals:**
- The owner's decision: a plural spelled like the card's own dictionary form gives no line, on
  English and Spanish cards as on French ones.
- One rule, in the description, the same for every studied language and every renderer.
- Every line that moves measured and listed; everything else byte for byte.

**Non-Goals:**
- A plural of *another* dictionary form: `leaves` still says it may be the plural of `leaf`, `dos`
  the masculine plural of `do`, `fils` (French) the plural of `fil`. Measured, no card of the four
  pairs names a plural of another dictionary form spelled like the form itself.
- A feminine singular spelled like the dictionary form: one Spanish form today (`paseante`, « may also
  be the feminine singular of paseante »), eight French ones on change 45's readings (`amateur`,
  `bateau`, `latino`, `top`, `schnock`, `sodique`, and `pop` and `antipersonnel`, which also carry the
  plural). The owner's decision names the plural; the tables already write an adjective of one form
  for both genders without a gender (`grande`, `feliz`: `ADJ|Number=Sing`), so no line.
- A plural past participle spelled like its dictionary form: none in English or Spanish; two French
  forms on change 45's readings (`circoncis`, `retransmis`) keep « masculine plural past participle »,
  a verb form and not the word in the plural.
- The pack's readings: the tables and `word_grammar` keep the plural (`les temps`, `las crisis` are
  plurals; *Grammar tables* asks the table to carry the reading); only the card's line goes.
- The wording change 56 (`refine-lingua-matrix-wording`) keeps: a bare plural beside gendered ones
  (change 51's D6).

## Decisions

### D1 — The rule

On the card of a form spelled like its dictionary form, letter case aside, a reading whose part of
speech is a noun, a proper noun, an adjective, a determiner or a pronoun, whose number is plural and
which carries no degree, gives no line — whatever its gender. It is left out where the dictionary
form's own reading is: from the readings the card names about that form. The form's other readings
and its other dictionary forms are said as before.

- **Nominal only.** A verb's plural spelled like its dictionary form is another form of the verb, not
  the word in the plural (*Non-Goals*: `circoncis`); a numeral is never named (`formKind`).
- **No degree.** A comparative or a superlative is named as such (change 51's D5): none sits on its
  own dictionary form's card in English or Spanish, nor in French on change 45's readings (measured:
  0).
- **Every gender.** `crisis` (feminine), `lunes` (masculine), `nadie` (both, « the masculine and
  feminine plural of nadie »), `police` (none).
- **The other readings stay.** `paso` keeps « may also be the first-person singular present
  indicative of pasar », `clave` the subjunctive and imperative of `clavar`. Measured, no English or
  Spanish card held the plural on one line with another reading of the same dictionary form, so every
  card that moves loses one whole line; on change 45's French readings, `pop` and `antipersonnel`
  keep « may also be the feminine singular of pop » on theirs.

### D2 — In the description, keyed by nothing

`describeReadings` leaves the plural out beside `isDictionaryForm`:
`if (same && (isDictionaryForm(tag) || <a nominal plural>(tag))) continue;` — one exported predicate
over a tag, documented as the dictionary form's own card's second omission. `FormDescription.own` and
its comment say so; `formKind`, `nameReadings`, `composeLines` and the three renderers do not change.
Because `describeForm` takes no studied language, the rule holds for English, Spanish and — once
change 51 names its forms — French, in the French, English and Spanish interfaces alike: *What each
renderer names* keeps holding, every renderer leaving unnamed what the French one leaves unnamed.

*Rejected — a key per studied language* (change 51's `CARD_NAMES[studied]` entry): a value every
studied language shares is not keyed; a key would invite a fourth language to differ silently.
*Rejected — in `composeLines`, dropping the line*: the line may hold another reading (`pop`'s feminine
singular), which would go with it. *Rejected — in a renderer*: three copies of one decision, which
change 18 moved into the description. *Rejected — in the core or the tables*: the reading is true
(`les temps`), the tables are required to carry it, and removing it would move every golden, pack
byte and pin for a wording decision. *Rejected — only true invariable plurals* (`sheep`, `crisis`):
no tag tells them from `tu`, `nadie`, `young` or `dead`; the decision aligns with French, which makes
no such distinction either. *Rejected — keeping the plural of a noun with no singular reading*
(`gafas`, `afueras`, `gens`): it keeps 91 Spanish and 64 French lines, but every one of English's 194,
whose table writes no noun's singular (`police` and `fish` alike), so English would not move at all;
it departs from change 51's D5 (1,072 French forms → 1,008); and « may also be the feminine plural of
gafas » misreads a noun that is only plural anyway (open question 3).

### D3 — The order with change 51

Either order ends with the same code and lines; **this change first is recommended**: it waits
on nothing, where change 51 is implemented once changes 44, 45, 48 and 49 are.

- **This change first.** The rule is in `describeReadings` for every studied language, and change
  51's merged documents, written before this change, are read so by its implementation, whose pull
  request says so, pointing here:
  - its task 1.1: `CARD_NAMES` holds the infinitive, the present participle, the gerund and the moods'
    merge, not « a plural spelled like the dictionary form »; its last clause (`composeLines` leaving
    the plural out) is met by the description, with no arm;
  - its task 1.3: the description's French decisions are tested without a plural key; « temps », « un »
    and « fils » keep their cases, which pass on this change's rule;
  - its D1 and D5 (the plural « keyed by studied language », English and Spanish keeping « today's
    values »), and its *Non-Goals*, D11 and *Known data defects* handing « an invariable plural on
    Spanish and English cards » to change 56: settled here, and corrected in its design by its
    implementation, whose task 4.5 rewrites *Known data defects* and D5's counts;
  - its D10 and task 5.1: the four pairs' 440,534 forms compared with a `main` holding this change,
    byte for byte;
  - its spec, unchanged: « a plural spelled like that form SHALL give no line » holds through the
    description, and « as before » against that `main`.

  Its D5 holds as measured: on change 45's implementation the description's rule leaves out 1,072
  French forms, 71 / 209 of the 1,000 / 5,000 commonest, change 51's own figures — the same readings
  its prototype's filter left out (`NUM` included and the degree unchecked: neither sits on a form's
  own card in the three languages; 0 of the 564,630 cards of the four pairs and fr-en differ).
- **Change 51 first.** French leaves the plural out through `CARD_NAMES.fr`; this change removes that
  entry, that arm and their tests (its task 1.3), and puts the rule in `describeReadings` — French's
  1,072 cards still without the plural, byte for byte unless that arm dropped a whole line holding
  another reading (then `pop` and `antipersonnel` get their feminine singular back, D1), the 861
  English and Spanish ones moving as D4 says.

Change 51 is not in `archiveAfter`: this change modifies a requirement change 51 does not hold, and
reads none of its own; change 51's *A French card names its forms in the interface language's
grammar* states French's rule and « the cards of English and Spanish words SHALL read as before »,
which in either archive order reads « as before that requirement » (D6).

### D4 — What moves, and what does not

Measured on every form of the four committed pairs (*How it was measured*), the same forms in each of
the three interface languages:

| Studied | Pairs | Forms | Cards losing the line | Dictionary form in the 1,000 / 5,000 / 10,000 / 20,000 commonest | Left with no grammar line | Keeping another line |
|---|---|---|---|---|---|---|
| English | en-fr, en-es | 75,315 | 194 | 7 / 42 / 72 / 132 | 194 | 0 |
| Spanish | es-fr, es-en | 144,952 | 667 | 13 / 62 / 141 / 275 | 600 | 67 |
| French (change 45's implementation, for change 51) | fr-en (fr-es reads the same readings) | 124,096 | 1,072 | 71 / 209 / 334 / 584 | 1,025 | 47 |

- The commonest: English `head` (240), `young` (296), `police` (383), `military` (515), `dead` (528),
  `box` (774), `percent` (785), then `grand`, `fish`, `religious`, `species`, `score`, `stone`;
  Spanish `tu` (31), `menos` (74), `nadie` (114), `paso` (218), `ex` (488), `crisis` (556), `análisis`
  (612), `post` (625), `viernes` (749), `café` (883), `lunes` (894), `clave` (923), `pro` (932), then
  `jueves`, `pa`, `rosa`, `cumpleaños`, `martes`, `miércoles`.
- French's 47: 45 keep another word's line (`fils` → *fil*), `pop` and `antipersonnel` their own
  feminine singular (D1). Change 51's D5 gives 1,007 and 65 on its prototype readings; its task 4.5
  measures them again.
- The lines lost, as es-en words them: « the masculine plural of » 384, « the feminine plural of »
  171, « the masculine and feminine plural of » 80, « the plural of » 22, « the plural and masculine
  and feminine plural of » 10; every English one « the plural of ».
- Forms whose card shows a grammar line: Spanish 88,247 → 87,647, English 35,033 → 34,839.
- **Committed files.** The extension's whole suite passes but `word-card-es-en.spec.ts`, whose
  snapshot loses one line — the probe `menos menos`, « grammar: may also be the masculine plural of
  menos » (`word-card-es-en.txt`, 468 → 467 lines); its gloss pages and row are unchanged.
  `word-card-en-es.txt`, `selection-rows-fr.txt`, `voice-ranking.txt`, `test/word-grammar.spec.ts`
  (the French renderer's 108 assertions), `word-grammar-en.spec.ts`, `-es.spec.ts`,
  `grammar-description.spec.ts` and `i18n.spec.ts` pass unchanged. (Two specs that need the generated
  gRPC clients fail on the scratch copy with and without the rule.)
- **Nothing of the engine.** The line is computed in the extension only. The goldens record readings
  (`"readings":[{"pos":"DET","features":{"Number":"Plur"}},{"pos":"NOUN","features":{"Gender":"Masc","Number":"Plur"}},…]`
  for `menos menos`), so `en-fr.golden`, `es-fr.golden`, `es-en.golden`, `en-es.golden` and
  `fr-en.golden` do not move; no table, pack byte, pin, `pack_version` or analyser version moves; no
  Rust file changes.
- **Bundles.** The content script and the reader grow by the predicate, a few dozen bytes; measured
  in the pull request.

### D5 — en-fr and es-fr move, with the owner's approval

The programme's first rule is « en-fr and es-fr output does not move, nor the French interface ».
This change departs from it, by the owner's decision of 2026-10-10: a French-speaking reader of
English loses the line on 194 cards, of Spanish on 667. No committed artefact pins those French lines
— S0 and the es-fr golden record readings, and the only French-interface card tests are hand-written —
so they are measured instead: the pull request lists every one of the 861 cards with its line before
and after, in French, grouped by studied language and ordered by rank, beside the es-en snapshot's
line. The owner approves that list before the merge (task 4.1), as the owner approved change 41b's
re-bless; the extension's release is the owner's (task 4.2).

### D6 — The specification, and wording held elsewhere

`lingua-browser-extension`'s *The word card says what the form is* is held by no open change: it is
MODIFIED, its « What the form is » saying which readings are left out of the statement on a form spelled
like its dictionary form, its text and scenarios kept, two scenarios added (*A plural spelled like
its dictionary form*, *Nothing else on the card moves*).

Read as written, never modified while their changes hold them:
- *The word card describes a form once, and says it in the interface language* (change 18, ADDED):
  « A renderer SHALL name what the French card names today … The French rendering SHALL be byte for
  byte what the card said before », and its scenario *Every reader today*. Read as change 18's own
  guarantee — its refactor moved no French byte, and its tests still pass — not as a freeze on later
  requirements: the rule here is the description's, so every renderer still names what the French one
  names (*What each renderer names* passes unchanged).
- *A Spanish card names its forms as French schools do* (change 18, MODIFIED): « A noun's,
  adjective's, determiner's or pronoun's form SHALL name its gender and number » is read of a form the
  card names, as `sus`' plural without a gender is already unnamed; « A reading that merely says what
  the card's dictionary form is SHALL give no line » is what this change widens, in the requirement it
  modifies; « A card of an English word SHALL read as before » reads as its scenario words it, « as
  before this requirement ».
- *The card of English-native readers of Spanish is pinned on the real pack* (change 23): its
  snapshot moves and is re-blessed in this pull request, saying why, as it asks; its golden does not
  move. *The card of Spanish-native readers of English is pinned on the real pack* (change 24): its
  snapshot does not move. Their *French readers unchanged* scenarios read « after this change » as
  theirs.
- Change 51's *A French card names its forms in the interface language's grammar*, if it archives
  first: « The cards of English and Spanish words SHALL read as before, in every interface language »
  reads « as before that requirement ».

These are best reworded when changes 18 and 51 archive (open question 2). `archiveAfter` names
changes 18, 23 and 24, whose requirements this change reads.

### D7 — Tests

- `test/grammar-description.spec.ts`, *the description of a form names no language*: the rule without
  words — on a form spelled like its dictionary form, a noun's plural of each gender, an adjective's,
  a pronoun's and a determiner's plural are left out of `own`; kept when the form differs (`rápidas`,
  `casas`); kept in `others` (`leaves` → `leaf`); a comparative plural kept; a verb reading beside
  the plural kept (`paso`). *What each renderer names*: a plural spelled like its dictionary form gets
  no line from the French, English or Spanish renderer, for English and for Spanish studied.
- `test/word-grammar-en.spec.ts` and `-es.spec.ts`: « crisis » and "police" give no line; « paso »
  keeps its line about `pasar`, « gafas » its line about `gafa` (open question 3); "leaves" still says
  the plural of "leaf".
- `test/word-grammar.spec.ts` unchanged: the French renderer's own assertions all pass, and the
  description's spec runs it on the rule.
- `word-card-es-en.txt` re-blessed (`yarn vitest run test/word-card-es-en.spec.ts -u`, the flag after
  the file): one line out, nothing in.

### D8 — What later changes take from here

| Change | Takes |
|---|---|
| 51 `add-lingua-french-word-card` | the rule, in the description (D3): no `CARD_NAMES` entry for the plural, its D5 met; or, implemented first, the entry this change removes |
| 52 `enable-lingua-french` | nothing: French's cards already leave the plural out |
| 56 `refine-lingua-matrix-wording` | of change 51's shared wording, the bare plural beside gendered ones only; the invariable plural is settled here |

## Risks / Trade-offs

- **A true invariable plural loses an informative line** (`sheep`, `species`, `aircraft`, `fish`,
  `crisis`, `análisis`, `lunes`, `virus`, `tesis`, `cumpleaños`) → the owner's decision, aligned with
  French; the gloss and the sense headings still say what the word is, and on the commonest words the
  line was mostly noise (`tu`, `menos`, `nadie`, `young`, `dead`).
- **A noun used only in the plural loses the card's one sign of its number** (`gafas`, `afueras`,
  `víveres`; `headquarters`, and `police`, which English agrees in the plural where French says « la
  police est ») → the same decision; its line read « may also be the plural of » anyway, and the
  gloss (« eyeglasses », « outskirts ») says it; open question 3.
- **en-fr and es-fr move unpinned** (D5) → every moved line is listed in the pull request and
  approved by the owner; the description's spec pins the rule in all three renderers.
- **Change 51 implemented with its own key anyway** → D3: its pull request reads this design; a key
  whose three values agree is removed by whichever change lands second.
- **A future table writes a verb's or a numeral's plural on its own lemma** → not left out (D1), and
  the snapshots of each pair show it.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves. Rollback is a revert.

## Effort

0.5–1 ideal day: the predicate and its comments 0.1; the tests and the snapshot 0.25–0.4; the
measurement re-run on the implementation and the pull request's list 0.15–0.4; the programme's row
0.05.

## Open Questions

For the owner, none blocking:
1. **A feminine singular spelled like the dictionary form** (non-goals): `paseante` today, eight
   French forms once change 45 lands (`amateur`, `top`, `pop`), keep « may also be the feminine
   singular of … ». Left as they are: the decision names the plural, and the tables write most
   adjectives of one form without a gender.
2. **Held wording** (D6): change 18's « the French rendering byte for byte » and « a noun's … form
   SHALL name its gender and number », and change 51's « as before », are read as written; best
   reworded by the change that touches each requirement next, once 18 and 51 archive.
3. **A noun used only in the plural** (D2): `gafas`, `afueras` and 89 more Spanish forms, `gens`,
   `environs` and 62 more French ones, lose the line saying they are plural (`gafas` keeps « may also
   be the feminine plural of gafa »), as `police` and `headquarters` do in English. Left out, as
   proposed: keeping them would leave English's 194 cards as they are (its table writes no noun's
   singular) and move change 51's French figure; naming such a noun as plural is gloss or
   sense-heading work, not this line's.
