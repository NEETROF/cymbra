# fix-lingua-lemma-lookup — a dictionary form is read as itself, never as the word it is a form of

## Why

The reviews of changes 45 (`add-lingua-french-grammar-tables`, its D12 and open question 4) and 46
(`add-lingua-french-levels`, its D3) of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
found the same defect from both ends: **the pack finds a lemma by looking its string up as a
form**. `FstLexicon::id_of` maps a form to its lemma's id; `Pack::gloss`, `readings`,
`other_readings` and `sense_runs` call it with a lemma. When that lemma string is itself a form of
another word, the card reads the other word's entry — and its readings apply the other word's
inflection edits to the wrong base. The builder has the twin: it files ranks, glosses, levels,
dictionary words, readings and sense runs under `id_of(lemma)`.

It is reached today, on the released packs, through the English and Spanish plural fallbacks, which
return a singular the lexicon holds only as a form (measured, design *Measured*):

- **Spanish**: 32,372 words read as such a singular; 2,103 of them carry another word's readings —
  `ablativas` → *ablativa* reads « masculine plural » off *ablativo*'s paradigm, 434 `-as` plurals
  read masculine only, 306 carry a verb's reading (300 of them no word wordfreq lists) — and
  29,830 another word's gloss in es-fr (`cuentos` « Compter », `tomos` « Prendre »).
- **English**: 15,215 words; 12,129 carry another word's readings, 14,231 another word's gloss in
  en-fr. Most are no words (`abandoneds`), but 625 have a frequency in wordfreq: `buildings` reads
  as *build* — « Construire, édifier », a plural noun and a present-tense third person singular —,
  `wounds` as *wind* (« Vent »), `settings` as *set* (« Prêt »), `thoughts` as *think*.
- **The goldens already pin it**: `en-fr.golden` and `en-es.golden` record `saw` asked as `saw` with
  *see*'s gloss and *see*'s past tense as its reading, `lay`, `thought` and `more` alike; the
  Spanish card renders it « también puede ser el pasado simple de saw ».

French's own analysis (change 41, merged) never asks so — its plural rule requires the singular to
be unknown too — but its golden's `été` probe does, and its `porte` probe will once change 48 builds
the golden from the tables: change 45 measured `porte` read off *porter*'s paradigm there. The
builder's twin has hit no committed table, and change 43 (merged, #829) forbids it in French's; its
earlier prototype gave *venir* the rank and level of `venue`. This is a core fix outside the
programme's 57 changes, to land before French ships (change 52) and before changes 45, 46 and 48
rely on lemma lookups.

## What Changes

- **A lemma is found among the lemmas** (D1): `FstLexicon::lemma_id` searches the lemma pool itself
  (binary search; the builder writes it sorted, and the core refuses a pack whose pool is not, as it
  refuses a form pointing outside it). `id_of` stays the form lookup and says so.
- **What a card describes is the dictionary form's own** (D2): `Pack::gloss`, `readings`,
  `other_readings` and `sense_runs` read by `lemma_id`; a string the pool holds as no lemma reads
  nothing. The gloss is decided explicitly: its own, or none — never the gloss of the word its
  spelling is a form of. A card left without a gloss is translated in its sentence where the reader
  has translation, as any word the pack does not gloss is.
- **What estimates familiarity keeps reading through the spelling** (D3): `rank`, `level` and
  `is_dictionary_word` answer, for a string, the lemma the forms read it as — no token's class, no
  count, no percentage and no Spanish name moves.
- **The builder files by the lemma's own place, and refuses an ambiguous lexicon** (D4): every
  lemma-keyed section is keyed by `lemma_id`; a forms table listing a form with two lemmas, or a
  lemma whose own spelling it reads as another, fails the build, named.
- **`build_lexicon_blobs` honours its comment** (D5): a form the pairs map wins over a lemma's
  identity entry spelled the same; today the byte-wise smaller lemma id wins, whichever it is.
- **Nothing else moves** (D6): no pack byte, no pin, no fixture pack, no analyser version, no
  `pack_version`, no token's lemma or class. The five invariance goldens and the en-es word card's
  snapshot are re-blessed on the lines that read another word's entry, each listed in the pull
  request; en-fr's and es-fr's move with the owner's approval (the programme's rule).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED *A dictionary form is read as itself*. *A word's grammar, from the pack*
  (archived) is read as written: « the dictionary form its card is keyed by » is that dictionary
  form, and a string that is no dictionary form of the pack has none of what it lists. *A French
  invariance baseline runs beside the English and Spanish ones* (added by
  `add-lingua-french-baseline`, modified by changes 40 and 41, none archived) is read as written:
  this change moves five of its probes and re-blesses them, saying why, as that requirement asks of
  any pull request that moves its golden. Its reasons (a French rule that bumps French's analyser
  version, the fixture replaced by the committed tables, the beside pack's update) do not name a fix
  of how the core reads every pack, which bumps no version (design D6): a departure from that list,
  stated here and in the pull request, which this change cannot modify while those changes hold it;
  its scenario *Nothing of French changed* reads this change as one of the rules French runs.
- `lingua-data-packs`: ADDED *A pack's lexicon reads every lemma as itself*. *Versioned pack
  container, keyed by language pair* (« compressed glosses indexed by lemma ») and *Reproducible
  offline build* hold as written; every committed pair already meets the new rule, byte for byte.
  *A level reaches the lemma it is written for* (held by `add-lingua-french-levels`, not archived)
  holds; its scenario *A level written for a lemma whose form reads as another* names `donner`,
  which after this change keeps its own level — the check still fails, naming `donnée`, whose level
  no lemma carries (design D7; that change's to word when it lands).

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-core` (`analysis/lexicon.rs`, `packs/pack.rs`),
  `crates/lingua-pack` (`lib.rs`, `lexical.rs`), their tests,
  `crates/lingua-wasm/tests/baseline/*.golden`,
  `apps/lingua-extension/test/baseline/word-card-en-es.txt`. The extension (Chromium, Firefox,
  Safari's host app) and the agent plugin take it with their next release. ID, Music, Live, the
  back office and the site are untouched.
- **What moves** (measured on a prototype over `main`, design *Measured*): `en-fr.golden` and
  `en-es.golden` 10 lines each — the `word-grammar` probes `more more`, `thought thought`, `lay lay`,
  `saw saw`, the `gloss` probes `more`, `are`, `has`, and the new reader's `news`, `academic` and
  `register` pages, where `lowers`, `findings` and `strangers` lose a gloss; `es-fr.golden` and
  `es-en.golden` 3 — `gloss cuenta`, `gloss llama`, the `quijote` page (`quebrantos`); `fr-en.golden`
  5 — `gloss vis`, `as`, `été`, `est` and `word-grammar été été`; `word-card-en-es.txt` the four
  probes (21 lines out, 4 in). Nothing else: 31 of 31 changed golden lines are these.
- **What does not move**: the five committed packs (sha256 = pin), fr-en's since #829 included, the
  testdata and fixture packs, every token's lemma, class and count, every percentage,
  `word-card-es-en.txt`, `selection-rows-fr.txt`, `voice-ranking.txt`; 122 of the extension's 123
  test files and every Rust test pass unchanged.
- **Order.** After changes 41 (merged, #832: the same 5 French lines move on it) and 43 (merged,
  #829: its tables pass D4). Before the implementations of changes 45, 46 and 48 and before change
  52; in either order with 42 and 44 (measured on their branches: the same 31 lines, theirs
  elsewhere in `fr-en.golden`, design D7); independent of 47, 49 and 50. Row 41b, outside the 57,
  like rows 23b and 24b.
- **Owner.** Approved on 2026-10-10: the re-bless of en-fr's and es-fr's lines (the programme's rule
  « en-fr and es-fr output does not move »), and of es-en's, en-es's, fr-en's and the Spanish card's;
  Q1 settled the same day (no gloss), Q2 left open as a change of its own. Remains: the extension's
  release; the agent plugin, outside the programme (M17), takes the fix with its next release.
