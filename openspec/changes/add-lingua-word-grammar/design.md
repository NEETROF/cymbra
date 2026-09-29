## Context

The pack maps a written form to one dictionary form through the `forms` FST, and each
dictionary form to one flat gloss of at most three senses (`gloss.zst`). Nothing in it says
what a form *is*. This is not because the sources are silent: `reduce-en-fr.py` throws the
information away.

- **ESDB** (`scowl.txt`, pinned at `rel-2026.02.25`) lists a word's derived forms in fixed
  slots, which the reducer's own fixtures show:
  - `go <v>: went, gone, going, goes`
  - `lie <v> {fib}: lied, lying, lies`: the past participle slot is folded into the past slot
    when the two are spelled the same;
  - `be <v>: (was | @: wast), were, been, being, am, (are | @: art), is, are`;
  - `datum <n>: data`.

  `parse_esdb_relations` keeps only a coarse kind per form (`N`, `V` or `A`), and
  `resolve_forms` then keeps one lemma per form.
- **kaikki** gives every entry a `pos` (`noun`, `verb`, `adj`, `adv`, `name`, `prep`,
  `conj`, `det`, `pron`, `intj`, `particle`, `character`, `symbol`, affixes…).
  `_join_senses` picks one sense per entry in turn, so a verb meaning appears beside the noun,
  and then drops the label.

Measured with the current reduction rules on the 2026-09-21 kaikki snapshot:

- 5 218 of 24 420 glossed words mix senses of several parts of speech;
- 465 picked senses hold a `;` of their own, the character that already separates senses.

The PR gives the exact figures on the pinned snapshot.

The engine answers the card through `AnalyzerPort`. Where the engine runs depends on the target:

- in the content script on Chromium, synchronously;
- in the event page on Firefox and Safari, through a message round trip;
- in the event page on Chromium too, as a fallback, when a page's CSP refuses WASM.

The word card (`reading/selection-card.ts`, `reading/wordpopup.ts`) lays out, top to bottom:
the headword, the form seen, the rarity, the listen button, the gloss, the translation, and the
actions. A highlighted word's card opens complete from its page token. A known or ignored word's
card opens pending and asks `gloss(lemma)`, under the wait-and-fallback rule
`add-lingua-phrase-gloss` introduced: a pending card offers no action, and completes exactly
once.

The next packs are Romance languages. The exploration recorded their sources: Morphalou for
fr, morph-it! for it, MorphoBr for pt, kaikki for es. Every one of them already attaches full
morphology to each form.

## Goals / Non-Goals

**Goals:**

- The word card says what the form is, lists the other dictionary forms it may be, and shows
  the pieces of a split word.
- The word card shows the gloss under the part of speech of its senses, with a noun's gender
  where a language has one.
- One grammatical vocabulary that a Romance pack fills without any change to the container,
  the core API or the card's code, needing only new label wording.
- Stay additive:
  - no move of `ANALYZER_VERSION`, and `analyse_page` output unchanged byte for byte;
  - no change to the card model, the review, the sync or the `.proto`;
  - no new source, licence or permission.

**Non-Goals:**

- Choosing a reading from context. That needs a part-of-speech tagger, which is a separate
  change. The vocabulary here is the one such a tagger would emit (D1).
- Grammar as something to learn: grammar points, exposure counts per tense, review.
- Opening another reading's card from the card, and highlighting or counting by reading.
- Grammar on the review card or in the word-by-word rows.
- Tense names for a studied language other than English (D7).
- Fixing the senses a gloss picks. The measurement shows `a` and `i` glossed by their letter
  senses. Grouping makes this visible, but it is a reducer-quality change of its own.

## Decisions

### D1 — The vocabulary: Universal Dependencies tags, as text, in a closed subset

A **tag** is a Universal Dependencies part of speech (UPOS), optionally followed by features in
UD's `FEATS` notation, sorted by name as UD requires:

```text
VERB|Mood=Ind|Tense=Past|VerbForm=Fin
NOUN|Number=Plur
NOUN|Gender=Fem
PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs
```

The pack stores each distinct tag once, as text in a pool (D3). Records point into the pool by
index.

The vocabulary is the 17 UPOS tags plus this closed subset of features. This change reads only
these:

| Feature | Values |
|---|---|
| `Gender` | `Masc`, `Fem`, `Neut`, `Com` |
| `Number` | `Sing`, `Plur` |
| `Person` | `1`, `2`, `3` |
| `Tense` | `Pres`, `Past`, `Imp`, `Fut`, `Pqp` |
| `Mood` | `Ind`, `Sub`, `Cnd`, `Imp` |
| `VerbForm` | `Fin`, `Inf`, `Part`, `Ger` |
| `Degree` | `Pos`, `Cmp`, `Sup` |
| `Case` | `Nom`, `Acc`, `Dat`, `Com` |
| `Reflex` | `Yes` |
| `PronType` | `Prs`, `Art`, `Dem`, `Int`, `Rel`, `Ind` |
| `Definite` | `Def`, `Ind` |

The vocabulary covers the Romance categories. This is the hard constraint, and it is met by the
vocabulary, not by English:

| What the learner meets | Tag (features) |
|---|---|
| passé simple, pretérito indefinido, passato remoto | `Mood=Ind\|Tense=Past\|VerbForm=Fin` |
| imparfait, imperfecto, imperfetto | `Mood=Ind\|Tense=Imp\|VerbForm=Fin` |
| subjonctif présent | `Mood=Sub\|Tense=Pres` |
| subjonctif imparfait (*dijéramos*) | `Mood=Sub\|Tense=Imp` |
| futur du subjonctif (pt *quando eu for*) | `Mood=Sub\|Tense=Fut` |
| plus-que-parfait synthétique (pt *fizera*) | `Mood=Ind\|Tense=Pqp` |
| conditionnel | `Mood=Cnd` |
| impératif | `Mood=Imp` |
| infinitif personnel (pt *fazermos*) | `VerbForm=Inf\|Number=Plur\|Person=1` |
| gérondif, gerundio | `VerbForm=Ger` |
| agreed participle (*escrita*) | `Gender=Fem\|Number=Sing\|Tense=Past\|VerbForm=Part` |
| gender unlike French (*la leche*) | `NOUN\|Gender=Fem` |
| clitic pronoun (*me*, *lo*, *se*, *gli*) | `PRON\|Case=…\|Person=…\|PronType=Prs` (+ `Reflex=Yes`) |
| contracted article (*del*, *au*, *do*) | pieces from the pre-pass (D5): `ADP` + `DET\|Definite=Def\|PronType=Art` |

These follow the features UD's French, Spanish, Italian and Portuguese treebanks use, checked on
2026-09-27 against the UD documentation and the feature statistics of fr_gsd, es_gsd, it_isdt and
pt_bosque (the sources, row by row, are in `SOURCES.md`). The check added `Case=Com` (Spanish
`conmigo`). It also found es_gsd giving `me`, `te`, `nos` and `os` the multi-value `Case=Acc,Dat`:
the parser reads one value per feature, so a Spanish pack writes such a form as two readings, or
the parser learns UD's comma-separated values then. Neither is a container change.

**How new and unknown features are handled.**

- **Adding a feature.** The container never enumerates features, so adding one is a change to
  the core's list and to the labels, never to the container. `Polite` is a candidate, for
  *usted* and *Lei*.
- **In the core.** The core parses a tag into a typed value when it looks one up, and skips a
  feature it does not know. A pack from a later vocabulary still loads and shows what it can.
- **In the builder.** The builder runs the same parser in strict mode and fails on an unknown
  part of speech, feature or value, naming it. A typo in the reducer therefore never ships.

_Why UD._ It is the one tagset that every planned Romance source already has a mapping to: the
UD treebanks for these languages were converted from the same traditions. A later contextual
tagger trained on UD treebanks would emit exactly these tags, so the pack and the tagger would
speak one language with no translation table between them. Only the tag names come from UD, and
no data.

_Alternatives considered:_

- **An English-shaped enum** (the reducer's `N`/`V`/`A`, or slot names such as `PAST`, `PP`,
  `ING`, `S`). Rejected: the first Romance pack would change the format, which is the very thing
  the constraint forbids.
- **Penn Treebank tags** (`VBD`, `VBN`, `NNS`). Rejected: they are English-only, with no mood
  and no gender.
- **Each source's own tags** (morph-it!'s `VER:ind+pres+1+s`, Morphalou's attributes).
  Rejected: the core and the card would have to learn one tagset per language.
- **Binary feature codes.** Rejected: they save a few hundred bytes, but a feature added later
  would change the layout.

### D2 — Where the English readings come from

The reducer keeps the slot of every ESDB derived form. English tags follow UD English
conventions:

| ESDB | Slot | Tag |
|---|---|---|
| `<v>`, 4 entries | past | `VERB\|Mood=Ind\|Tense=Past\|VerbForm=Fin` |
| | past participle | `VERB\|Tense=Past\|VerbForm=Part` |
| | -ing | `VERB\|VerbForm=Ger` |
| | -s | `VERB\|Mood=Ind\|Number=Sing\|Person=3\|Tense=Pres\|VerbForm=Fin` |
| `<v>`, 3 entries | past and past participle, then -ing, then -s | the two past tags on the first entry |
| modal `<v>` (`could, -, can`) | past only: no -ing, so no participle; its -s slot, spelled like the modal, is no reading | `VERB\|Mood=Ind\|Tense=Past\|VerbForm=Fin` |
| `<m>` (a noun/verb ESDB could not tell apart) | read as `<v>` | the verb tags |
| `<n>` | plural | `NOUN\|Number=Plur` |
| `<n_v>` | the verb's slots, then possessives (dropped); its -s | both the plural and the -s verb tag, as `_esdb_kinds` already says |
| `<aj>`, `<av>`, `<d>` | comparative, superlative | `ADJ`/`ADV`/`DET` with `Degree=Cmp`, then `Degree=Sup` |
| `be <v>` | eight slots | named by an explicit table, since only `be` has them |

Other rules for the readings:

- **One tag for -ing.** The pack cannot tell a gerund from a present participle outside a
  sentence, so an -ing form carries a single reading, labelled « forme en -ing ».
- **Spelling alternatives.** Alternatives inside one slot (`(A B: focused | AV Bv: focussed)`)
  share that slot's tag. Lesser variants stay excluded, as they are today.
- **Forms added from kaikki.** Kaikki's form-of links add regular inflections only. The tag of
  such a form follows the ending that `regular_inflection` already checks: -ing, -ed or -d (both
  past tags), -s (verb or noun, by the relation's kind), -er or -est. The French wording of the
  link (« Prétérit de … ») is not parsed: the shape is enough for a regular form, and one source
  of truth is enough.
- **Forms spelled like their dictionary form.** `put` as its own past, or `sheep` as its own
  plural, carry readings although the reducer skips them as pairs today.

**Which relations become readings.** Every relation that survives the reducer's existing
filters becomes a reading of its own dictionary form: lesser, archaic, doubtful and
questionable variants stay out, and so does a possessive. A relation is named as *another*
dictionary form of a written form only when the reducer finds it **believable**, by the
`believable` test `own_words` already applies:

- the base has the right part of speech, and
- the form is its regular inflection, or Wiktionary names the base as what the form is a form
  of.

This keeps `uses` from naming `us`, and lets `leaves` name `leaf`, `lives` name `live`, and
`bored` name `bore`. The reading the analysis itself chose is always shown, believable or not.
It is what the page already counted.

### D3 — Three additive sections, keyed by dictionary form, no second form index

| Section | Holds |
|---|---|
| `tags` | the tag pool: UTF-8, one tag per line, id = line index |
| `paradigms.zst` | zstd blob in the offset-indexed layout of `gloss.zst`, keyed by lemma id. Per dictionary form, a list of **readings** (the form, a tag id) and **also** entries (a form the analysis resolves to this dictionary form, and the id of another dictionary form it is also a reading of). |
| `senses.zst` | zstd blob in the same layout, keyed by lemma id: the runs of the gloss (tag id, number of senses), in gloss order |

**How a form is stored.** A form is written as an **edit of its dictionary form**: the number of
characters (Unicode scalar values) to strip from its end, and the suffix to append. For example,
`hablar` → `hablábamos` is strip 2, append `ábamos`, and `go` → `went` is strip 2, append
`went`.

**How the builder fills `paradigms`.** The builder resolves every form through
`lingua_core::analysis::lemmatize` against the lexicon it has just assembled, which is how the
expression table is keyed. It then files the "also" entries under the dictionary form the
analysis will actually put on the card. The reducer never needs to know how the analysis
resolves a form.

**Why keyed by dictionary form.** The obvious alternative is a second FST keyed by form, as the
expression table does, which gives an O(1) lookup by form. It duplicates the form index, though:
the `forms` FST already holds every form. In a Romance pack, forms dominate the size (about fifty
per verb), and a second FST would roughly double the largest section.

Keyed by dictionary form, each form costs a short suffix. The suffixes repeat across a
conjugation class, which zstd compresses well. An English irregular form costs its full spelling,
which is what it is. Every dictionary form also gets its paradigm, which a later conjugation table
can show with no format change.

The lookup always has the card's dictionary form in hand, and a paradigm holds tens of entries at
most, so a scan by form is cheap.

_Alternative considered:_ packing a reading into the `forms` FST's value (lemma id plus tag).
Rejected: it changes what an older core reads from a required section, so it is not additive.

**Memory.** A section is decompressed once at load and kept as bytes with its index. Records are
decoded per lookup, never all at load. The analyser runs inside each Chromium tab's content
script, so decoding every paradigm up front would cost every tab. The build is deterministic,
like the other sections: the order is the lemma id, then the form, then the tag.

### D4 — The reducer: senses grouped, one separator, two new tables

`_join_senses` receives each sense with the part of speech of its entry. The selection keeps the
same round-robin; how many senses it keeps and how they are cut change (see "Cuts that end on a
word" and "Eight whole senses, paged").

1. After picking, the senses are grouped by part of speech, stably, in the order each part of
   speech first appears.
2. A `;` inside a sense becomes `,`.
3. The gloss is joined and cut as today.
4. The runs are counted from the senses that survived the cut.

**Acronyms stay out of a common word's gloss.** The reducer lowercases every headword, so the
entries of an acronym merged into the common word spelled like it. Dogfooding in Safari showed the
card of `and` reading « *verbe* Faire le ET de »: the French Wiktionary's `AND`, the logic
operator, is a noun and a verb. The flat gloss already said « Et; ET; Faire le ET de »; grouping by
part of speech made it glaring. It was not isolated: 246 glossed words mixed an all-capitals
headword with their own, among them `for` « Franco wagon », `be` « Meilleure estimation », `me`
« Médecin légiste », `who` « OMS », `us` « États-Unis » and `if` « Indice de citations ».

So an entry whose headword is written all in capitals (two letters or more) is left out of a word
that has an entry of its own, and kept when it is the only one (`NATO` still glosses `nato`). A
capitalised headword is no acronym: `He` still glosses `he`. The cost is the acronym read in
context: `US` in "the US economy" opens the card of `us`, which no longer says « États-Unis ». That
is the same card as before, and such a reading is far rarer than the pronoun. `wiktionary_signals`
is left as it is, so the inflection rules, `forms.tsv` and `freq.tsv` do not move.

**Cuts that end on a word.** Dogfooding in Safari showed `has` read « Avoir. Auxiliaire utilisé
pour former l'as » and `is` « …le passif ave »: every sense was cut at 42 characters and the gloss at
80, mid-word. That was already so in the flat gloss; one line per part of speech made it plain.
4 405 of the 24 420 glosses held a cut sense. Measured on real packs:

| Limits (sense / gloss) | Pack | Glosses with a cut sense |
|---|---|---|
| 42 / 80, before | 1 703 229 B | 4 405 |
| 60 / 120 | 1 728 725 B | 2 443 |
| 80 / 160, chosen | 1 741 714 B | 1 105 |

A word's gloss now takes 80 characters per sense and 160 in all (mean length 31 → 37). What is
still too long is cut on a word boundary with an ellipsis (`cut_at_word`). The next sense is kept
whole while it fits; it is cut when at least 20 characters of room remain, and left out below that.
Expressions keep 42 / 80 and the plain cut.

**Eight whole senses, paged.** A second round of dogfooding asked for the rest of a long gloss
rather than a better cut: « plutôt que coupé, un bouton suivant ». The pack now keeps up to eight
senses of a word, each within 300 characters and all within 800, and the card pages them (D6).
Most words have fewer senses than that, so the cost is small. Measured on real packs:

| Senses × sense / gloss | Pack | Glosses with a cut sense |
|---|---|---|
| 3 × 80 / 160, before | 1 739 035 B | 1 208 |
| 6 × 200 / 400 | 1 812 923 B | 165 |
| 8 × 300 / 800, chosen | 1 830 370 B | 88 |
| 12 × 400 / 2 000 | 1 842 943 B | 81 |

Word-by-word rows keep one sense and cut it at 80 characters on a whole word, as the pack did
before (`rowGloss`), and a card created from a word's card stores the first page, not eight senses.

**The Wiktionary's notes to its readers.** The same dogfooding showed `there` reading « Y avoir.
→ voir there be »: a link on the wiki, dead text on a card. 20 word glosses and 9 expressions held
such a pointer, and 164 word glosses and 46 expressions the placeholder of an unfinished page,
« Définition manquante ou à compléter. (Ajouter) », often after a real translation (`pig`
« Vivre dans la promiscuité et la saleté. Définition manquante… »). `strip_wiki_notes` takes out a
pointer (`→ voir`, `→ Comparer`, with or without brackets) up to the end of its sense, a
placeholder wherever it sits, and the etymology placeholder; the rest of the sense is kept, and a
sense left empty is dropped. An arrow that is no pointer (« hard → hardest ») stays. 13 words had
nothing else and lose their gloss: their card says the pack has no translation, which is true.
It applies to expressions too, so `mwe.tsv` moves for those 55 entries and no others.

`forms.tsv` and `freq.tsv` are asserted byte-identical. `gloss.tsv` moves, and the pull request's
report counts the reordered glosses and the changed separators.

**kaikki `pos` → UPOS.**

| kaikki `pos` | UPOS |
|---|---|
| `noun` | `NOUN` |
| `verb` | `VERB` |
| `adj` | `ADJ` |
| `adv` | `ADV` |
| `name` | `PROPN` |
| `pron` | `PRON` |
| `prep`, `postp` | `ADP` |
| `conj` | `CCONJ` for the seven coordinators (`and`, `or`, `but`, `nor`, `yet`, `so`, `for`), `SCONJ` otherwise |
| `det`, `article` | `DET` |
| `particle` | `PART` |
| `intj`, `onomatopoeia` | `INTJ` |
| `num` | `NUM` |
| `character`, `symbol` | `SYM` |
| anything else (affixes, `phrase`, `typographic variant`) | `X` |

**The committed tables.**

- `grammar.tsv`: `form<TAB>lemma<TAB>tag`, one line per reading, sorted. It also carries the
  believability mark of D2 as a fourth column (`other` when the relation may be named as
  another dictionary form).
- `senses.tsv`: `lemma<TAB>tag:count[<TAB>tag:count…]`, one line per glossed lemma, runs in
  gloss order. For example: `can<TAB>NOUN:1<TAB>VERB:2`.

A tag never holds `:`, a tab or a newline, so the lines split unambiguously. Both tables derive
from ESDB and kaikki and carry their licences (CC BY-SA 4.0 for the Wiktionary part), exactly as
`forms.tsv` does. The README and `SOURCES.md` list them.

### D5 — The core: `word_grammar(written, lemma)`

```rust
pub struct WordGrammar {
    pub gloss: Option<String>,        // Pack::gloss(lemma), the flat text, unchanged
    pub senses: Vec<SenseGroup>,      // { tag: Option<Tag>, text: String }
    pub readings: Vec<Tag>,           // the piece's readings as `lemma`
    pub others: Vec<OtherReading>,    // { lemma: String, readings: Vec<Tag> }
    pub pieces: Vec<String>,          // pre-pass surfaces, only when there are 2 or more
}
pub fn word_grammar(written: &str, lemma: &str, studied: StudiedLanguage, pack: &Pack) -> WordGrammar
```

1. **Pieces.** `written` is the word as it stands on the page. It goes through `tokenize`, with
   the studied language's pre-pass, and `resolve_lemmas`, as in the page analysis.
2. **The piece.** The piece whose dictionary form is `lemma` is taken, or the first piece if
   none matches. Its readings come from `lemma`'s paradigm: the entries whose form equals the
   piece, lowercased.
3. **Other dictionary forms.** They come from `lemma`'s "also" entries for that piece, each
   read from the other dictionary form's own paradigm. A word the pre-pass split names none: the
   split has settled what the piece is. Without this rule, `does` in `doesn't` would be offered
   as the plural of `doe`, as the real pack showed.
4. **Senses.** They come from splitting the flat gloss on `; ` and applying the runs. With no
   runs, or runs that disagree with the gloss (which the build forbids), the answer is one group
   with no tag and the whole gloss. The same holds when the pack has no `senses` section.

The JSON serialises a tag as `{ "pos": "VERB", "features": { "Tense": "Past", … } }` from
ordered maps, so it is deterministic. Unknown features are skipped (D1).

It is exposed in three places:

- `lingua-wasm` as `wordGrammar(written, lemma)`;
- `AnalyzerPort` as `wordGrammar(written, lemma): Promise<WordGrammar>`, through
  `WasmAnalyzerPort`, `MessagingLinguaPort` and `rpc-host`;
- `analyzer/types.ts`, which mirrors the types.

`gloss(lemma)` stays for `cardGloss` and the other callers. `analyse_page` does not call any of
this, so `golden.json` does not move. A new `grammar_golden.json` pins the answers for the parity
test.

_Alternative considered:_ attach the grammar to every page token. Rejected: it grows every page's
payload for cards that are mostly never opened, and it changes `analyse_page`'s output, which
means `ANALYZER_VERSION`.

### D6 — The card: one answer, a short bound, nothing moves

**Which call a card makes.** Every word card asks `wordGrammar` once:

| Card | Call |
|---|---|
| Highlighted word | the new call |
| Known or ignored word | replaces the `gloss(lemma)` call; the answer carries the gloss |
| Word outside the page analysis | after `phraseGloss` has resolved it |

**What `written` is.** For a page token, `PageHit` gains `written`, the text of the token's
source span. The two halves of a contraction share that span, so `do` in `don't` asks about
`don't`. For a selection, `written` is the selected text.

**Timing.** The layout puts the actions last. If the grammar lines and the grouped gloss were
inserted late, they would push the actions down while the reader aims at them, which the
engine-waiting rule forbids. So a card that already holds its gloss (a highlighted word) waits
for the answer up to `GRAMMAR_WAIT_MS = 250`, and **is never drawn pending**:

- It is drawn once, complete, when the answer arrives. On Chromium's in-process engine that is
  the next microtask.
- **Past the bound**, it is drawn with the page token's gloss and no grammar, which is exactly
  today's card.
- **A late answer** lands nowhere: the card compares the view's generation with the one it asked
  under, as `request()` does, so a card shown, hidden or replaced meanwhile is never written over.

A first implementation drew the card pending when the answer missed the next frame. It was dropped:
the pending card's waiting line replaced a gloss the card already held, for a wait of at most
250 ms. Cards that hold no gloss keep their existing wait and fallback.

_Alternatives considered:_

- **Show at once, then insert the grammar.** Rejected: the actions move.
- **Reserve the space.** Rejected: the height depends on the number of readings and groups.
- **Prefetch the grammar of a page's highlighted words after the analysis.** This is static pack
  data, so a cache would never go stale. Kept as the follow-up if the Safari measurement (see
  Risks) shows the bound is often missed on a cold engine.

**Rendering (`wordpopup.ts`).**

- **A grammar block** sits between the listen row and the gloss, one line per statement. It is
  placed there because a pending card offers the listen buttons: had the grammar gone into the
  `.seen` line above them, a known word's card completing its answer would push those buttons
  down. The `.seen` line keeps `forme vue : « went »` as today. Everything is set through text
  nodes, the studied language's words in `<em>`, never markup.

  | Case | Line |
  |---|---|
  | Readings | `prétérit de go`, or `prétérit et participe passé de walk` |
  | Same spelling, other reading | `peut aussi être le prétérit et le participe passé de put` |
  | Other dictionary form | `peut aussi être le pluriel de leaf` (text only) |
  | Pieces | `« doesn't » = does + not` |

- **The gloss block** renders one line per group: the heading (the part of speech, plus the
  gender when there is one) in italics, then the group's senses. A group tagged `SYM`, `X`,
  `PUNCT` or with no tag has no heading.
- **Pages** (`reading/gloss-pages.ts`). A gloss longer than `PAGE_CHARS = 160` — what a card
  showed before — is cut into pages of whole senses, at least one per page, and a group that runs
  across two pages is headed again on the second. A control under the gloss reads « ‹ 1/3 Suivant › »;
  it is absent when the gloss fits. Its buttons keep the reader's selection, as the listen
  buttons do, and a page change redraws only the gloss and re-anchors the card. Every `show`
  starts on the first page, which is also what a created card stores (`pageText`), so the deck
  never holds eight senses and does not depend on the page the reader was on.
  Dogfooding showed a short last page shrinking the card under the pointer: `refer`'s « Se référer
  à » alone narrowed it until « Je connais » wrapped, and moved « ‹ » up. So a paged card is
  measured on every page when shown and keeps the widest width and the tallest page height
  (`holdPageSize`); nothing moves while paging.

### D7 — Labels: generic names here, tense names per studied language

A new module `reading/grammar-labels.ts`, in the `COPY` style of `reader/copy.ts`, turns a tag
into French words:

- **Parts of speech:** nom, verbe, auxiliaire, adjectif, adverbe, préposition, conjonction
  (both `CCONJ` and `SCONJ`), pronom, déterminant, interjection, nombre, particule, nom propre.
- **Gender:** masculin, féminin, neutre, commun.
- **Number and person:** « pluriel », « 3e personne du singulier ».
- **Degree:** comparatif, superlatif.
- **Verb forms**, for English: prétérit (`Tense=Past|VerbForm=Fin`), participe passé, forme en
  -ing, and « 3e personne du singulier du présent ».

The same code names a different tense per language (`Tense=Past` is the prétérit in English and
the passé simple in Spanish), so the verb-form names are keyed by studied language, and only the
English table ships now. A tag the labeller cannot name is not shown. A card never shows a code,
and a test asserts it alongside the existing "lemme" lint.

### D8 — What stays flat, and `pack_version`

**What stays flat.** None of these move:

- the gloss stored on a card, and hence the review, the sync ops and seeded cards;
- the word-by-word rows' first-sense rule. Grouping is stable by first appearance, so a gloss's
  first sense stays first.

New cards store the regrouped text, and existing cards keep theirs. There is no migration.

**`pack_version`.** The grammar tables come from a re-reduction of the pinned snapshot, and
`build.sh --reduce` stamps `pack_version` with the *source* snapshot (`2026.09.26`). The ESDB
switch (#560) re-reduced the same way and kept that version. Yet two requirements call for a new
version whenever the tables change:

- "A pack says which dictionary it is" (an updated dictionary reports a different
  `pack_version`);
- the container requirement (adding an additive table bumps it).

This change makes the practice meet the requirements. A re-reduction stamps
`<snapshot>+<first 7 hex digits of the reducer's sha256>`, taken from `pin.json`, which already
records it. An update from live sources keeps the bare snapshot. No counter needs storing, and
nothing reads `pack_version` except reports and humans. _Alternative:_ keep the #560 practice and
reword both requirements. Rejected: a version that does not change when the dictionary does
identifies nothing.

### D9 — Archive order

`add-lingua-phrase-gloss`, then `add-lingua-expression-table`, then this change. The MODIFIED
container and size-budget requirements here are written over the wording
`add-lingua-expression-table` gives them. Archived first, this change would restore the older
text under the same headers, and the expression-table archive would then overwrite it.

## Risks / Trade-offs

- **[A cold engine on Safari misses the 250 ms bound, so the first cards show no grammar]** →
  The card is then exactly today's. Measure on device (iPhone and Mac Safari, engine suspended
  and awake). If the bound is missed on most first opens, add the page-level prefetch of D6.
- **[Readings expose the pack's single-lemma choices]** (`leaves` → `leave` in a sentence about
  trees) → This is the honest view of what the page counted. The "peut aussi être" line tells
  the reader. Choosing by context is the tagger's change, not this one.
- **[Grouping makes weak glosses visible]** (`a`, `i` glossed by letter senses, under no
  heading) → Recorded as a follow-up for the reducer. Grouping does not make them worse.
- **[Size]** → Estimate for English: about 45 000 readings, roughly a few hundred KB compressed,
  against 1.54 MB and a budget of 5 MiB. The build measures it, and the budget's arbitration
  (readings of the rarest dictionary forms, after expressions) is enforced by the builder, with
  a test that drives it over.
- **[A Romance pack is larger by nature]** → The layout (D3) is chosen for it: suffix edits under
  the dictionary form rather than a second form index. The real figure comes with the first
  Romance pack, whose budget is its own.
- **[UD's Romance conventions differ from the D1 table on some point]** → The task checks each
  row against the UD documentation. A difference changes a mapping row, never the container.

## Migration Plan

Pure addition:

- The pack gains three optional sections and takes a new `pack_version`.
- The extension ships the core, the WASM and the card together, as every release does.
- Rollback is a revert: an older extension ignores the sections.

No reader data moves. Statuses, cards and decks keep their keys and shapes, so no report of
statuses is needed. Lingua has no readers yet (confirmed on 2026-09-26); that is worth checking
again before merge, but nothing here depends on it.

## Open Questions

- The exact bound (250 ms) is a first value. The Safari measurement decides whether to keep it,
  or whether prefetch replaces the bound.
- Whether the modal pasts (`could`, `would`, `should`, `might`) should read as pasts of their
  modal at all. They are taught as words of their own, and `own_words` already keeps them as
  dictionary forms. As readings they would add « peut aussi être le prétérit de can ». The
  first implementation keeps them, and the dogfooding decides.
