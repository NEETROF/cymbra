# refine-lingua-it-en-glosses — it-en's glosses read as Italian meanings, in the order Italian uses them, before English speakers read Italian

## Why

Change 23 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 22
(`add-lingua-pack-it-en`, #1063) committed it-en's glosses on 2026-10-25 — 33,199 rows, coverage
**91.3 / 84.2 / 75.8 %** against the floor 89.3 / 82.2 / 73.7 — and listed what still reads wrong
(its design's *Known data defects* and *Left for change 23*, its README's *Known defects*) for this
change, « measured on the committed tables ». Change 25 (`enable-lingua-italian-for-english-speakers`)
lists it-en only once this change has merged (its D2), as fr-en shipped after its own refinement.

Measured on the committed tables and on it-en's pinned sources (design, *Measured*; criteria written
before the figures):
- **The wrong sense first.** The shared round-robin opens a row on the page's first entry, in the
  order of its etymologies: `auto` (rank 391) « auto (genre of dramatic literature) », its « car »
  behind a pointer that lends nothing; `testo` « an earthenware lid », `polizia` « polity »,
  `università` « totality, entirety, whole », `bene` « upper-class, posh », `gente` « noble », `forte`
  « fort, fortress », `presto` « ready », `morire` « death », `prego` « prayer », `più` « anymore »;
  `piano` loses « plan » to the eight-sense cap. About 29 of the 1,000 commonest rows; 173 ranked rows
  open on a sense the page itself marks archaic, obsolete, dated or historical.
- **A note or a pointer's wording shown as the gloss.** `lo` (21) opens on a 300-character usage note
  ending in « the », and every `l'` card shows it; `de` (94) « Marche, Milan, Rome, and Venice form of
  di », `el` « archaic, regional, and Rome form of il », `che` (3) « proscribed spelling of ché »,
  `aprile` « compound of apri…, with le », `minimo` « superlative degree of piccolo » (31 rows nest
  their meanings under a degree's wording), `ce` « reinforcing particle, “do” » alone.
- **Pointers that lend nothing.** 106 ranked rows hold a « female equivalent of X » that lends nothing
  beside another sense — `amica` (882) « girlfriend », `bambina` « babe », `compagna` « wife, spouse »;
  `obbiettivo` is « a surname »; `guai` « woe! » without « trouble ».
- **Names after a common word's senses.** 1,143 rows of the 20,000 commonest hold a surname's note
  (`volta`, `guerra`, `marzo`, `musica` « …; a surname »), 597 a given name's (`delfino` « dolphin;
  …; a male given name »), and common words keep a town (`noto` « …; a town in Syracuse, Sicily »,
  `grado`, `cambio`).
- **Smaller items.** « etc » without its period (71 rows, 13 expressions), `mar` « (“sea”) (used in
  poetry…) », `itria` « # alternative form of lontra », the Italian Wiktionary's jocular « anshent »
  in `all'antica`.

## What Changes

Every rule is it-en's own, in `reduce_italian_en.py` and `reduce-it-en.py`, which no other pair loads;
fr-en's treebank rule and MarkIT's counts are imported unchanged, as it-fr imports them (design D1).

- **The order of a row** (D2): a function word's row opens on the part of speech UD Italian-MarkIT
  reads it as — it-fr's rule, fr-en's constants — (16 rows, all of the 1,000: `su` « on », `solo`
  « only », `spesso` « often », `verso` « toward »); a row that would open on a sense the page marks
  archaic, obsolete, dated or historical opens on its first sense that is not (173 rows: `polizia`
  « police », `università` « university », `ufficio` « office », `prodotto` « product »); a reviewed
  table, keyed by the sense a row opens on, orders 40 rows the rules miss or misread (`bene` « well »,
  `gente` « people », `testo` « text », `piano` « floor, storey » with « plan » back, `come` held at
  « how ») — the 19 the owner approved (task 0.1), and six added after his answer on the review of
  this proposal, approved with the gloss diff (`dì` « day », `contrastare` « to oppose », `contea`
  « county », `pro` « for », `parlamento` « parliament », `ce` « us »), and `parmigiano` « parmesan
  cheese », added on the implementation's gloss diff, and fourteen more on an Italian specialist's review
  of it (`tanto`, `costa`, `mica`, `manco` …), all approved on 2026-10-25.
- **Pointers read** (D3, D4): pointer wordings the English edition does not know (« … form of »,
  « proscribed spelling of », « compound of … with », « Old Italian form of ») are pointers; a pointer to a
  word of one or two letters lends that word's senses after the row's own, within the eight-sense bound
  (`de` → *di*, `el` « he; the », `ce` → *ci*, `se` gaining *si*'s reflexive); a meaning
  nested under a degree's wording, a degree's carried meaning, a clipping's and, beside another part of
  speech, a plural's are read (`minimo` « minimum, (the) least… », `più` « more », `auto` « car; bus »,
  `guai` « woe!; trouble »).
- **Female equivalents** (D5): the rule that would lend the masculine's senses is measured and rejected
  (`signora` « lord (ruler) », `bambina` « boy, young boy »); four rows corrected by name, the owner's
  answer (task 0.3): `amica` « friend », `bambina` « girl, little girl », `compagna` « companion,
  partner », `spagnola` « Spaniard ».
- **Names' notes** (D6), the owner's answer (task 0.2): a surname's note goes from a row whose word holds
  a common meaning, a given name's where it follows a common word's senses, a town's, a comune's or a
  village's from a common word unless the row opens on the town — cities, capitals and provincial
  capitals kept, `massa` by a reviewed list, `andria` by the rule. 1,260 rows; a row holding only names
  keeps them (`garibaldi`, `clinton`), and `francesco`, `alice`, `diego`, which open on the name, keep it.
- **The page's notes and typography** (D7): `lo` « the; him; it… », « etc. », a list marker, a quoted
  opener.
- **`CORRECTIONS`** (D8): it-fr's reviewed shape for it-en, correcting and adding — `all'antica` without
  « anshent », and the four rows of D5 —, each reported or suggested upstream.
- **Measured and left** (D9): bracketed constructions (« to ask [with di ‘about (someone)’] »), kept as
  the preposition a verb takes; `USA`'s order, the shared acronym table's; offensive senses unlabelled;
  names opening a row.
- **One re-reduction** (D10): a dispatch (`lingua-pack-update`, mode `reduce`) at it-en's pin, to which
  the implementation's first commit adds MarkIT's two sections by hand (D1); together the rules change
  **1,578 rows** (132 / 300 / 601 / 545 by band), 341 in their first sense, no lemma gained or lost,
  coverage **91.3 / 84.2 / 75.8 %** as now; Italian's studied tables, the seven other pairs and the PUD
  gates byte for byte; `it-en.golden` and the English card re-blessed once by the update workflow, their
  moved lines listed for approval (D11).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *it-en's glosses read an Italian word's meanings in the order Italian uses them*, *it-en
    reads a pointer as its word's meaning where the section names one*, *it-en's glosses hold no
    name's note or page note a common word does not need* and *it-en corrects, by name, the slips its
    sources write*.
  - MODIFIED, copied whole from the open change that holds it, the clauses this change edits named in
    the pull request: *Italian is glossed in English from the English Wiktionary's Italian section and
    the Italian Wiktionary's English translations* (`add-lingua-pack-it-en`: a degree's carried meaning
    read wherever it stands and a plural's beside another part of speech; a name beside a common word's
    senses kept but for the notes the new requirement leaves out).
  - This change archives after `add-lingua-pack-it-en` (`archiveAfter`). Change 25 renames change 22's
    other requirement, which this change does not touch.
- `lingua-analysis`: none. The analysis reads it-en's glosses as written.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce_italian_en.py` (the pre-pass, `FIRST_RUN`,
    `TOWNS_KEPT`), `reduce-it-en.py` (the post-passes, `CORRECTIONS`, MarkIT read from the work
    folder), `test_reduce_it_en.py`, `test_reduce_editions.py`; `pack_sources.py` (`PINNED["it-en"]`,
    MarkIT's two sections, as fr-es's refinement added UD French-GSD) and `test_pack_sources.py`;
    `tables/it-en/` (`pin.json`, MarkIT's two sources by hand, then `gloss.tsv`, `senses.tsv`, `mwe.tsv`,
    `manifest.json`, `README.md` by the dispatch),
    `SOURCES.md`; *consumed*: `reduce_common.py`, `reduce_edition_en.py`, `reduce_italian_acronyms.py`,
    `reduce_french_treebank.py`, `reduce_italian_treebank.py`, `gloss_coverage.py`, unchanged.
  - `crates/lingua-wasm/tests/baseline/it-en.golden` re-blessed; `crates/lingua-pack/tests/committed_tables.rs`
    against the new pin, its list of it-en's source keys naming MarkIT's two.
  - `apps/lingua-extension/test/baseline/word-card-it-en.txt` re-blessed once.
  - ID, Music, Live, the back office, the site, the backend, lingua-core and the engine, the
    extension's source, the Apple host app and the agent plugin are untouched.
- **What does not move.** `tables/it/` and the seven other pairs — tables, pins, packs, goldens and
  snapshots — byte for byte; the English edition's rules and the shared ones; Italian's analyser
  version; UD Italian-PUD's figures.
- **Release.** Silent: no package lists it-en before change 25.
- **Order.** After change 22 (on `main`, its requirement still open), whose requirement this change
  modifies; before change 25.
- **The owner's**: three questions, answered on 2026-10-25 as recommended (tasks 0.1–0.3); the six
  table rows added after that answer and the golden's and card's moved lines, approved on their listing
  (tasks 4.1, 4.2).
- **Effort**: 2.5–3.5 ideal days (design, *Effort*), against the programme's 2–3.5.
