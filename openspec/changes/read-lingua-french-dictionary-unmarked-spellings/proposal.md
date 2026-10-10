# read-lingua-french-dictionary-unmarked-spellings — the dictionary's accent-less spellings read as their word

## Why

Row 43d of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes: the second follow-up of change 43b (`read-lingua-french-ca-as-ca-cedilla`,
implementation #884), its open question 2, settled by the owner on 2026-10-10.

The English Wiktionary's French section gives some words only as another word written without its
accents, cedilla or diaeresis: `etre` « obsolete spelling of être », `etat` « alternative spelling of
État », `siecle` « obsolete form of siècle », `repondre` « obsolete spelling of répondre ». Today each
is a French word of its own: `etat` is ranked 489th, a dictionary word glossed « state, condition… »
with no level; `etait` (rank 1,995) and `etats` (867) are words with no gloss at all; a reader of
« merci bcp pour hier soir, c'etait vraiment genial !! » (change 39's own corpus) is shown `etait`,
unknown and unglossed, where the page says *être*. In fr-es none of them has a Spanish gloss. And
none meets its word's expressions: « il etait une fois » is not `il était une fois`, « un coup
d'Etat » not `coup d'état`.

The owner decided (2026-10-10): read them as their accented word — **only when the accented word is
the commoner one** by wordfreq, so that `diesel` « alternative form of diésel », 78 times commoner
than `diésel`, stays itself.

Measured on the pinned section (design, *Measured*): the « about 186 » are 141 accent-less spellings
and 45 ASCII spellings of a ligature (`coeur`, `oeuvre`, `soeur`) that change 43's spelling rule
already reads as their word; 51 of the 141 are ranked. By the owner's rule 81 are read as their word
— 33 of the 51 ranked among them (`etat`, `etre`, `etait`, `arreter`, `siecle`, `repondre`) — and 56
keep their own reading because the spelling is the commoner (`diesel`, `ego`, `egypte`, `etienne`:
18 ranked).

## What Changes

- **Change 43's spelling rule reads a third kind** (design D1): a word the section gives only as one
  marked word written without its marks — every sense of every entry of it an alternative, obsolete,
  archaic, nonstandard or misspelt spelling or form of that word — is a form of that word, as an
  ASCII spelling of a ligature (`coeur` → *cœur*) and a post-1990 spelling (`connait` → *connaître*)
  already are. Its inflections and the entries pointing at it follow it (`repond`, `reponds`,
  `repondu` → *répondre*; `siecles` → *siècle*), and its readings are filed under the marked word.
  Measured against 43b's table mechanism with rows generated from the same tags, which drops a
  spelling's entries whole: 20 of their forms would become unglossed ranked words of their own
  (`egyptiens` at rank 15,256, `repond`, `siecles`) instead of reading as their word.
- **The frequency condition** (D2): wordfreq (3.1.1, the pin's) must rate the marked word strictly
  commoner than the spelling **and than each of the spelling's forms**. The second half keeps one
  spelling out: `teter` « alternative form of téter », whose listed form `tete` (Zipf 3.83) is how the
  web writes *tête*, would read « head » as « to suckle ». A word with an entry of its own beside the
  spelling (`surs`, the plural of *sur* « sour »; `entrainant`, the participle of *entrainer*; `colon`)
  is not read by it (D1).
- **What a read spelling's own entry becomes: nothing of its own** (D3). No rank, no level, no
  dictionary word, no gloss in fr-en or fr-es; a reader sees its word's card. Its marked word keeps its
  rank, level, readings and glosses. fr-en loses no gloss a form had (`etat`'s « state, condition… » is
  *état*'s); fr-es loses one (`uberisation` « Uberización »: *ubérisation* has no Spanish gloss).
- **What moves, measured on a prototype that reproduces the committed tables byte for byte** (D4):
  - `tables/fr/forms.tsv` 124,101 → 124,184 forms: 100 forms now read as a marked word (69 that read
    as one of the 33 spellings, 30 that were no form, and `rebellions`, *rebeller*'s form, now
    *rébellion*'s), 53 forms of the 33 lemmas entering the cut; `freq.tsv` the 33 spellings out,
    `dagon` … `eamon` in at 59,965–59,997, 59,476 ranks up by 1–33; `level.tsv` `benoit` (B2) out,
    `patriotique` C1 → B2, `opter` C2 → C1, `digestif` into C2; `lexical.tsv` 26,484 → 26,471 (25 out,
    12 in); `grammar.tsv` 125,177 → 125,303; `tags.tsv` byte for byte;
  - fr-en: 29 glosses out (the 25 dictionary words and 4 names), 12 in (the entering lemmas),
    `mwe.tsv` byte for byte; 34 forms gain a gloss; coverage 93.6 / 86.9 / 76.3 % unchanged
    (4,679 / 8,688 / 15,259 → 4,681 / 8,688 / 15,254);
  - fr-es: re-reduced on the new `tables/fr/`, every other row byte for byte, `uberisation` out and
    four entering lemmas in; 85 forms gain a Spanish gloss (`etat` « Estado », `etre` « Ser… »);
    coverage 83.0 / 70.7 / 56.7 → 83.1 / 70.7 / 56.7 % (4,153 / 7,071 / 11,347), floor 81.4 / 68.8 /
    54.5;
  - UD French-PUD 99.13 / 96.41 / 99.90 → 99.13 / 96.36 / 99.90 % (PUD lemmatises « Etats-Unis » as
    written, four times), GSD's test section 98.89 / 95.86 / 99.72 → 98.91 / 95.94 / 99.72 %.
- **The goldens** (D5): `fr-en.golden` and `fr-es.golden` move 6 of their 221 probes each — the pack
  line, « c'etait » on the `informel` page read as *être*, the vocabulary universe 26,484 → 26,471 and
  the reader's estimate 2,185 → 2,183, and the deck the levels seed (`opter` in) —; four phrase probes
  are added: « il etait une fois » and « un coup d'Etat » meeting `il était une fois` and `coup
  d'état`, « Il faut etre patient. » read as *être*, and « un moteur diesel », which the frequency rule
  keeps. The word-card snapshots gain those four cards and move nothing else.
- **Not here** (D6): the web's unglossed spellings the dictionary does not name (`tete`, `francais`,
  `tres`, `deja`) — the reviewed list by name the owner settled as a change of its own, which `surs`,
  `sures`, `sure` and `entrainant` are handed to —; 43b's reviewed table (`ca`, `age`, `forcement`),
  which this rule neither adds to nor reads.
- **en-fr, es-fr, es-en and en-es cannot move**: the rule lives in `reduce-fr-en.py`, which only
  fr-en's rule digest names; no shared or edition module, no table of theirs, no core code changes;
  their goldens and the cross-native, parity, card-language and deck tests passed unmoved over the
  prototype.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A French spelling the dictionary gives only as a marked word reads as
  that word when the marked word is the commoner* (the third kind of change 43's spelling rule; the
  frequency condition over the spelling and its forms; the spelling's own entries giving no lemma,
  rank, level or gloss; a word with an entry of its own, 43b's table and every other pair untouched).

No requirement is modified. The new one widens the last sentence of change 43's *French's forms and
frequencies* (held by change 52 too) as 43b's narrowed it, and sits beside 43b's *A French spelling
without its marks reads as the word a reviewed table names*, whose « only when a reviewed table names
it » speaks of the spellings no entry links to their word, as it already did of change 43's post-1990
spellings. It moves what changes 45 (readings), 46 (*French's estimated levels*: its scenario's `etre`
still has no level, as no lemma), 48 and 48b (fr-en's glosses, dictionary words) and 49 and 49b
(fr-es) write from the forms — all open — so it says so rather than MODIFY them; they, change 41
(whose cascade reads every token lowercased) and change 44 (whose keys a spelling meets) are in
`archiveAfter`.

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the pre-pass naming the spellings, the third
    kind of `spelling_target`, read by `Lexicon.read` and `read_readings`), `test_reduce_fr_en.py`,
    `test_reduce_editions.py`, `tables/fr/` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`,
    `lexical.tsv`), `tables/fr-en/` (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`,
    `README.md`), `tables/fr-es/` (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`,
    `README.md`), `SOURCES.md`; *consumed*: `reduce_common.py`, `reduce_edition_en.py`,
    `reduce-fr-es.py`, unchanged.
  - `crates/lingua-pack` — `tests/committed_tables.rs`: the spellings read as their words in both
    French packs, `diesel` and `teter` as before; the levels test's spelling example `etre` → `parceque`.
  - `crates/lingua-wasm` — four phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed.
  - `apps/lingua-extension` — `word-card-fr-en.txt` and `word-card-fr-es.txt` gain four cards, their
    specs' count 42 → 46; no code.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host app
  and the agent plugin are untouched; French's analyser version does not move.
- **Release.** Silent: no package lists a French pair. It follows 43b, which ships only with or after
  the acronym change (43c, `keep-lingua-french-acronyms-in-capitals`), and lands before change 52
  lists French.
- **Compatibility.** No reader holds a French pack, so no status, card or level names a spelling;
  nothing to migrate.
- **Effort, against no programme estimate (outside the 57)**: 1–1.5 ideal days (design, *Effort*).
