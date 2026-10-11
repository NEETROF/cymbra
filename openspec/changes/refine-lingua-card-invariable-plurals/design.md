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
| Change 51 (`add-lingua-french-word-card`, proposal merged as #838, implementation not started) | its D5 leaves every plural spelled like the dictionary form off a French card (1,072 forms), keyed by studied language in a `CARD_NAMES[studied]` table and applied in `composeLines` (its task 1.1), English and Spanish keeping theirs; its open question 3, settled by the owner on 2026-10-09: « aligned in a change of its own, proposed separately »; its D11 had handed « an invariable plural on Spanish and English cards » to change 56 |

A form spelled like its dictionary form gets a plural reading when the dictionary records the same
spelling in the plural: a true invariable (`crisis`, `lunes`, French `temps`), a homograph's plural
(`tu`), a lexicographer's entry read through another part of speech (`nadie`, `menos`), or a noun
used only in the plural (`gafas`, `afueras`, French `gens`, `environs`). The tags cannot tell the
first three apart — each reads the form in both numbers of one part of speech — but they tell the
last: no singular of its part of speech. English's table writes no noun's singular at all, so every
English plural spelled like its lemma reads as the last kind, `police`, `headquarters` and `fish`
alike.

The owner's decisions: 2026-10-09, English and Spanish cards aligned with French's, in a change of
its own (change 51's open question 3); 2026-10-09, the boundary: the line goes only where the card
also reads the form as a singular of that part of speech, a noun used only in the plural keeping it,
and English, whose table writes no singular, changing nothing.

**How it was measured.** A prototype, never committed, in a scratch copy of `origin/main` at
`c6a7699d`, run again in review at `f43c6035` (the core and the English and Spanish tables did not
move in between):
- a scratch test in `crates/lingua-wasm` builds en-fr, es-fr, es-en and en-es from the committed
  tables (`PackSource::Tables`), takes every form of `tables/en/forms.tsv` (75,315) and
  `tables/es/forms.tsv` (144,952), resolves its dictionary form as the page analysis does
  (`tokenize`, then `resolve_lemmas` on the first token, made public in the scratch copy — the
  page's own steps, without the block gates that a one-word block fails) and asks `word_grammar` —
  440,534 answers, equal to change 51's prototype's on every form but 20 contractions and to the
  goldens on every grammar probe they share;
- the extension's renderers, `main`'s and a copy with D2's rule, run under Node on every answer, the
  card opened as `wordpopup.ts` opens it (`surface` the first token, `written` the form), in each of
  the three interface languages; ranks are the dictionary form's in `tables/<studied>/freq.tsv`;
- French: the same on change 45's implementation (`claude/add-lingua-french-grammar-tables-impl` at
  `e58b14c1`, whose `tables/fr/grammar.tsv` holds French's readings), fr-en's pack, 124,096 forms,
  rendered by renderers given no French table — finite French forms unnamed on both sides, so only
  this rule's lines differ;
- the extension's whole suite on the scratch copy with the rule, and the card snapshots re-blessed
  there to read their diff.

## Goals / Non-Goals

**Goals:**
- The owner's decisions: a plural spelled like the card's own dictionary form and read beside a
  singular of its part of speech gives no line, on English, Spanish and French cards alike; a noun
  used only in the plural keeps its line.
- One rule, in the description, the same for every studied language and every renderer.
- Every line that moves measured and listed; everything else byte for byte.

**Non-Goals:**
- A noun used only in the plural: `gafas`, `afueras` and 89 more Spanish forms (`once` « eleven »
  among them, a table oddity), `gens`, `environs`, `plusieurs` and 62 more French ones, and English's
  194 (`head`, `young`, `police`, `fish`, `species`) keep « may also be the … plural of … ». Naming
  such a noun as a plural rather than « may also be » is gloss or sense-heading work.
- A plural of *another* dictionary form: `leaves` still says it may be the plural of `leaf`, `dos`
  the masculine plural of `do`, `gafas` the feminine plural of `gafa`, `fils` (French) the plural of
  `fil`. Measured, no card of the four pairs names a plural of another dictionary form spelled like
  the form itself.
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
which carries no degree, gives no line — whatever its gender — when the card's own readings also
read the form as a singular of that part of speech. It is left out where the dictionary form's own
reading is: from the readings the card names about that form. The form's other readings and its other
dictionary forms are said as before.

- **Nominal only.** A verb's plural spelled like its dictionary form is another form of the verb, not
  the word in the plural (*Non-Goals*: `circoncis`); a numeral is never named (`formKind`).
- **No degree.** A comparative or a superlative is named as such (change 51's D5): none sits on its
  own dictionary form's card in English or Spanish, nor in French on change 45's readings (measured:
  0).
- **Every gender, on either side.** `crisis` (feminine), `lunes` (masculine), `nadie` (both, « the
  masculine and feminine plural of nadie »); the singular's gender does not matter, as the owner named
  the part of speech: `paso`'s feminine plural goes beside its masculine singular, the one such card
  (Spanish; none in French).
- **Of that part of speech.** `menos` (Spanish) reads a noun in both numbers and a determiner in the
  plural only: the noun's plural goes, the determiner's stays — unnamed anyway, with no gender.
  `frais` (French) reads an adjective in both numbers and a noun in the plural only (« les frais »):
  the adjective's plural goes, the noun's stays, so its line reads as before. These are the only two
  such cards.
- **A plural alone keeps its line.** `gafas`, `afueras`, `gens`, `plusieurs`, and every English one
  (`police`, `fish`), whose table writes no singular reading (*Non-Goals*).
- **The other readings stay.** `paso` keeps « may also be the first-person singular present
  indicative of pasar », `clave` the subjunctive and imperative of `clavar`. Measured, no English or
  Spanish card held the plural on one line with another reading of the same dictionary form, so every
  card that moves loses one whole line; on change 45's French readings, `pop` and `antipersonnel`
  keep « may also be the feminine singular of pop » on theirs.

### D2 — In the description, keyed by nothing

`describeReadings` already sees every tag of the card: it notes the parts of speech read in the
singular, and leaves the plural out beside `isDictionaryForm` —
`if (same && (isDictionaryForm(tag) || isInvariablePlural(tag, tags))) continue;` — one exported
predicate over a tag and the card's tags, documented as the dictionary form's own card's second
omission. `FormDescription.own` and its comment say so; `formKind`, `nameReadings`, `composeLines` and
the three renderers do not change. Because `describeForm` takes no studied language, the rule holds
for English, Spanish and — once change 51 names its forms — French, in the French, English and
Spanish interfaces alike: *What each renderer names* keeps holding, every renderer leaving unnamed
what the French one leaves unnamed.

*Rejected — every plural spelled like the dictionary form* (change 51's D5 as merged, and this
proposal's first draft): it also took the line from nouns used only in the plural — 91 Spanish, 65
French and all 194 English — where it is the card's one sign of the noun's number (`gafas`, `gens`,
and `police`, which English agrees in the plural where French says « la police est »); the owner kept
it there on 2026-10-09. *Rejected — a singular of the same gender too*: it would keep `paso`'s
feminine plural alone; the owner named the part of speech. *Rejected — a key per studied language*
(change 51's `CARD_NAMES[studied]` entry): a value every studied language shares is not keyed; a key
would invite a fourth language to differ silently. *Rejected — in `composeLines`, dropping the line*:
the line may hold another reading (`pop`'s feminine singular), which would go with it. *Rejected — in
a renderer*: three copies of one decision, which change 18 moved into the description. *Rejected — in
the core or the tables*: the reading is true (`les temps`), the tables are required to carry it, and
removing it would move every golden, pack byte and pin for a wording decision. *Rejected — only true
invariables, beyond the tags* (`crisis` apart from `tu`, `nadie`): no tag tells them apart; both read
the form in both numbers.

### D3 — The order with change 51

The owner's boundary of 2026-10-09 is French's too: change 51's D5, merged before it, leaves out
every plural spelled like the dictionary form (1,072 French forms); with this boundary it leaves out
1,007, and `gens`, `environs`, `plusieurs`, `frais` and 61 more keep their line. Either order then
ends with the same code and lines; **this change first is recommended**: it waits on nothing, where
change 51 is implemented once changes 44, 45, 48 and 49 are.

- **This change first.** The rule is in `describeReadings` for every studied language, and change
  51's merged documents, written before this change, are read so by its implementation, whose pull
  request says so, pointing here:
  - its task 1.1: `CARD_NAMES` holds the infinitive, the present participle, the gerund and the moods'
    merge, not « a plural spelled like the dictionary form »; its last clause (`composeLines` leaving
    the plural out) is met by the description, with no arm;
  - its task 1.3: the description's French decisions are tested without a plural key; « temps », « un »
    and « fils » keep their cases, which pass on this rule, and « gens » gains one, keeping its line;
  - its D1 and D5: the plural is not keyed by studied language, and D5's first row reads this
    boundary — 1,007 French forms, 68 / 200 of the 1,000 / 5,000 commonest, 979 then with no grammar
    line, 26 keeping another word's line, `pop` and `antipersonnel` their own feminine singular; its
    *Non-Goals*, D11 and *Known data defects* handing « an invariable plural on Spanish and English
    cards » to change 56: settled here. Corrected in its design by its implementation, whose task 4.5
    rewrites *Known data defects* and D5's counts;
  - its D10 and task 5.1: the four pairs' 440,534 forms compared with a `main` holding this change,
    byte for byte;
  - its spec: « a plural spelled like that form SHALL give no line » is read as this boundary (D6),
    and « as before » against that `main`; its scenario (« temps », « un », « fils ») holds as written.
- **Change 51 first.** If its implementation took this boundary, French leaves the plural out through
  `CARD_NAMES.fr`; this change removes that entry, that arm and their tests (its task 1.3), and puts
  the rule in `describeReadings` — French's 1,007 cards byte for byte unless that arm dropped a whole
  line holding another reading (then `pop` and `antipersonnel` get their feminine singular back, D1),
  the 576 Spanish ones moving as D4 says. If it took its D5 as merged, this change also gives French's
  65 their line back — fr-en and fr-es, which no reader studies before change 52 — and re-blesses
  `word-card-fr-en.txt` and `word-card-fr-es.txt` where a probe moves, saying why.

Change 51 is not in `archiveAfter`: this change modifies a requirement change 51 does not hold, and
reads none of its own; change 51's *A French card names its forms in the interface language's
grammar* states French's rule — « a plural spelled like that form SHALL give no line », read as this
boundary — and « the cards of English and Spanish words SHALL read as before », which in either
archive order reads « as before that requirement » (D6).

### D4 — What moves, and what does not

Measured on every form of the four committed pairs (*How it was measured*), the same forms in each of
the three interface languages:

| Studied | Pairs | Forms | Cards losing the line | Dictionary form in the 1,000 / 5,000 / 10,000 / 20,000 commonest | Left with no grammar line | Keeping another line |
|---|---|---|---|---|---|---|
| English | en-fr, en-es | 75,315 | 0 | — | — | — |
| Spanish | es-fr, es-en | 144,952 | 576 | 13 / 56 / 128 / 237 | 542 | 34 |
| French (change 45's implementation, for change 51) | fr-en (fr-es reads the same readings) | 124,096 | 1,007 | 68 / 200 / 319 / 554 | 979 | 28 |

- The commonest: Spanish `tu` (31), `menos` (74), `nadie` (114), `paso` (218), `ex` (488), `crisis`
  (556), `análisis` (612), `post` (625), `viernes` (749), `café` (883), `lunes` (894), `clave` (923),
  `pro` (932), then `jueves`, `pa`, `rosa`, `cumpleaños`, `martes`, `miércoles`; French `un` (7),
  `pas` (9), `plus` (21), `si` (32), `nous` (34), `non`, `temps`, `fois`, `moins`.
- Keeping another line: Spanish's 34 another reading or dictionary form (`paso` → *pasar*, `clave` →
  *clavar*, `pa` → *para*, `rosa` → *roso*); French's 28: 26 another word's line (`fils` → *fil*),
  `pop` and `antipersonnel` their own feminine singular (D1).
- Keeping their line: Spanish's 91 nouns used only in the plural, French's 65, English's 194 — the
  cards change 51's D5 as merged, and this proposal's first draft, would have moved.
- The lines lost, as es-en words them: « the masculine plural of » 326, « the feminine plural of »
  138, « the masculine and feminine plural of » 80, « the plural of » 22, « the plural and masculine
  and feminine plural of » 10.
- Forms whose card shows a grammar line: Spanish 88,247 → 87,705; English 35,033, unchanged.
- **Committed files.** The extension's whole suite passes but `word-card-es-en.spec.ts`, whose
  snapshot loses one line — the probe `menos menos`, « grammar: may also be the masculine plural of
  menos », `menos` being read as a masculine noun in both numbers (`word-card-es-en.txt`, 468 → 467
  lines); its gloss pages and row are unchanged. `word-card-en-es.txt`, `selection-rows-fr.txt`,
  `voice-ranking.txt`, `test/word-grammar.spec.ts` (the French renderer's 108 assertions),
  `word-grammar-en.spec.ts`, `-es.spec.ts`, `grammar-description.spec.ts` and `i18n.spec.ts` pass
  unchanged. (Two specs that need the generated gRPC clients fail on a scratch copy without them,
  with and without the rule.)
- **Nothing of the engine.** The line is computed in the extension only. The goldens record readings
  (`"readings":[{"pos":"DET","features":{"Number":"Plur"}},{"pos":"NOUN","features":{"Gender":"Masc","Number":"Plur"}},…]`
  for `menos menos`), so `en-fr.golden`, `es-fr.golden`, `es-en.golden`, `en-es.golden` and
  `fr-en.golden` do not move; no table, pack byte, pin, `pack_version` or analyser version moves; no
  Rust file changes.
- **Bundles.** The content script and the reader grow by the predicate, a few dozen bytes; measured
  in the pull request.

### D5 — es-fr moves, approved by the owner

The programme's first rule is « en-fr and es-fr output does not move, nor the French interface ».
This change departs from it for es-fr, by the owner's decisions of 2026-10-09: a
French-speaking reader of Spanish loses the line on 576 cards; en-fr does not move (0 cards). No
committed artefact pins those French lines — the es-fr golden records readings, and the only
French-interface card tests are hand-written — so they are measured instead: the pull request lists
every one of the 576 cards with its line before and after, in French, ordered by rank, beside the
es-en snapshot's line. The owner approved that re-bless under this boundary on 2026-10-09 (task 4.1),
as for change 41b's; the extension's release is the owner's (task 4.2).

### D6 — The specification, and wording held elsewhere

`lingua-browser-extension`'s *The word card says what the form is* is held by no open change: it is
MODIFIED, its « What the form is » saying which readings are left out of the statement on a form
spelled like its dictionary form, its text and scenarios kept, three scenarios added (*A plural read
in both numbers*, *A noun used only in the plural*, *Nothing else on the card moves*).

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
  modifies; « A card of an English word SHALL read as before » holds: no English card moves.
- *The card of English-native readers of Spanish is pinned on the real pack* (change 23): its
  snapshot moves and is re-blessed in this pull request, saying why, as it asks; its golden does not
  move. *The card of Spanish-native readers of English is pinned on the real pack* (change 24): its
  snapshot does not move. Their *French readers unchanged* scenarios read « after this change » as
  theirs.
- Change 51's *A French card names its forms in the interface language's grammar*: « On a card
  opened on its own dictionary form, a plural spelled like that form SHALL give no line » is read as
  this requirement's boundary — the plural of a form also read as its singular, as its scenario's
  « temps » and « un » are —, not as « gens »; if it archives first, « The cards of English and
  Spanish words SHALL read as before, in every interface language » reads « as before that
  requirement ».

These are best reworded when changes 18 and 51 archive (open question 2). `archiveAfter` names
changes 18, 23 and 24, whose requirements this change reads.

### D7 — Tests

- `test/grammar-description.spec.ts`, *the description of a form names no language*: the rule without
  words — on a form spelled like its dictionary form, a noun's plural of each gender, an adjective's,
  a pronoun's and a determiner's plural beside a singular of their part of speech are left out of
  `own`, a feminine plural beside a masculine singular too; a plural with no singular of its part of
  speech kept (a noun's plural alone; a noun's plural beside an adjective's singular only, as
  `frais`); kept when the form differs (`rápidas`, `casas`); kept in `others` (`leaves` → `leaf`); a
  comparative plural kept; a verb reading beside the plural kept (`paso`). *What each renderer names*:
  a plural beside its singular gets no line from the French, English or Spanish renderer, a plural
  alone keeps it, for English and for Spanish studied.
- `test/word-grammar-en.spec.ts` and `-es.spec.ts`: « crisis » gives no line; "police" and « gafas »
  keep theirs, « gafas » also its line about `gafa`; « paso » keeps its line about `pasar`; "leaves"
  still says the plural of "leaf".
- `test/word-grammar.spec.ts` unchanged: the French renderer's own assertions all pass, and the
  description's spec runs it on the rule.
- `word-card-es-en.txt` re-blessed (`yarn vitest run test/word-card-es-en.spec.ts -u`, the flag after
  the file): one line out, nothing in.

### D8 — What later changes take from here

| Change | Takes |
|---|---|
| 51 `add-lingua-french-word-card` | the rule and its boundary, in the description (D3): no `CARD_NAMES` entry for the plural, its D5 narrowed to 1,007 forms; or, implemented first, the entry this change removes |
| 52 `enable-lingua-french` | nothing: French's cards already leave the plural out where it is read beside its singular |
| 56 `refine-lingua-matrix-wording` | of change 51's shared wording, the bare plural beside gendered ones only; the invariable plural is settled here |

## Risks / Trade-offs

- **A true invariable loses an informative line** (`crisis`, `análisis`, `lunes`, `virus`, `tesis`,
  `cumpleaños`; French `temps`, `pays`) → the owner's decision, aligned across languages; the gloss
  and the sense headings still say what the word is, and on the commonest words the line was mostly
  noise (`tu`, `menos`, `nadie`).
- **English keeps what Spanish drops** (`fish`, `sheep`, `species` say « may also be the plural of »,
  `crisis` does not) → one rule over different tables: English's writes no noun's singular. A change
  that writes it moves those 194 cards, measured in its pull request.
- **es-fr moves unpinned** (D5) → every moved line is listed in the pull request, approved by the
  owner; the description's spec pins the rule in all three renderers.
- **Change 51 implemented with its own key, or with its D5 as merged** → D3: its pull request reads
  this design; the entry is removed, and French's 65 get their line back, by whichever change lands
  second.
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
   SHALL name its gender and number », and change 51's « a plural spelled like that form » and « as
   before », are read as written; best reworded by the change that touches each requirement next,
   once 18 and 51 archive.
3. **A noun used only in the plural** (D1, D2) — **settled by the owner on 2026-10-09: it keeps its
   line.** The line goes only where the card also reads the form as a singular of that part of speech
   (`crisis`, `lunes`, `temps`); `gafas`, `gens` and every English plural (`police`, `headquarters`,
   `percent`) keep it: 576 Spanish cards move, 1,007 French ones for change 51, no English one.
