# refine-lingua-fr-es-glosses — fr-es's glosses read as Spanish meanings, labelled, before Spanish speakers read French

## Why

Change 49 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-pack-fr-es`, implemented with #862) committed French glossed in Spanish at
**83.2 / 70.8 / 56.8 %** of the 5,000 / 10,000 / 20,000 commonest lemmas, above its floor of
81.4 / 68.8 / 54.5 %, and listed what still reads wrong (its design's and README's *Known data
defects*, its Open Questions 2–8). Changes 51 (`add-lingua-french-word-card`) and 52
(`enable-lingua-french`) added the card's view of the same rows. The owner settled on 2026-10-09
that fr-es's known defects are fixed before it ships — change 52 lists fr-es only once this change
has merged (52's Open Question 4) — and on 2026-10-10, in session, answered change 49's questions 2,
3, 4 and 8:

1. **A sense's register, outdated or regional label is shown**, in the card's language — Spanish
   words for a Spanish reader. « baiser » (rank 1,951) opens on « Coger (sexualmente) » with
   « Besar », which the Spanish Wiktionary labels outdated, near the end and nothing saying either.
2. **A word of the French Wiktionary's table is listed once** across the French word's parts of
   speech: « parti » (208) reads « Partido; Partido », « européen » (1,000) « Europeo; Europeo »,
   « extra » « Extra; Extra ».
3. **The inverted table's other-sense words are left out by a rule**, what is lost measured:
   « us » (1,330) reads « EEUU », « rap » (3,698) « Secuestro », « luc » « San Lucas ».
4. **The French words given as their own gloss are dropped**, the loanwords Spanish writes alike
   kept: « arnaque » (4,603) reads « Arnaque », « retraite » (965) « Retraite, jubilación, retiro »;
   « diaporama » and « raï » are Spanish too.

And the defects changes 51 and 52 list for fr-es: `être` (36) opening on the noun « Ser » before the
verb, `qui` (15) « Quién. (Pronombre nominativo.) », `rien` (64), a run with no part of speech
under `des` (6) — and those change 49's sample shows: « amie » « Amia o lamia », « ds » « Tiburón »,
« el » « Ella, ello o él », « hall » « Explanada », « extra » « Extra; Extra ».

Change 48's refinement (`refine-lingua-fr-en-glosses`, 48b, #861) settled the same family of
questions for fr-en, French's reference pair. This change says, for each of 48b's decisions, whether
it reaches fr-es and how — most of them through French's own tables, which 48b writes and fr-es
reads, so nothing of them is done twice (design D8).

## What Changes

Every rule is fr-es's own, in `reduce-fr-es.py`: no shared module and no edition module is edited,
so en-fr, es-fr, es-en, en-es and fr-en do not move (design D1).

- **Labels shown** (D2): a sense of the Spanish Wiktionary's French section opens on its labels in
  the edition's own words, as its categories file the sense — register (« coloquial »,
  « malsonante », « jergal »…), age (« anticuado », « obsoleto »), place (« Quebec », « Bélgica »…):
  « baiser » « (malsonante) Coger (sexualmente); …; (anticuado, Canadá, Bélgica) Besar; Beso,
  besuqueo u ósculo ». 202 rows (109 of the top 10,000). The order is not changed for a label —
  measured, it reads worse (« cul » would open on « Fundo (de un objeto) ») —; the senses the edition
  marks outdated stay last, as 24b's pre-pass writes them.
- **A word of the direct table listed once** (D3): a run whose words another run of the word shows
  goes, and a word still shown twice stays in the first run that shows it: « parti » « Partido »,
  « jeune » « Joven, chaval, muchacho », « russe » « Ruso, rusa ». 654 rows (222).
- **The inverted table's names and acronyms, and its other-sense words** (D4): read backwards, an
  acronym glosses only through itself (« US » « EEUU » out, « ONU » « ONU » kept), a one-word name
  through no saint's name (« Luc » « San Lucas »), an elided article read into a word (« lOrient »)
  and the language code « fr » are no French words; six other-sense words of the top 10,000 are left
  out by name, each with its reason (« rap » « Secuestro », « pilote » « Controlador », « ds »
  « Tiburón »). 19 lemmas lose their gloss (14 of the top 10,000), two of them right (« usa » and
  « éu » « EEUU »).
- **The French word is no Spanish gloss** (D5): a word of the direct table spelled as the French
  headword, a hundred times commoner in French than in Spanish (wordfreq), is dropped unless it is a
  loanword Spanish writes alike, from a reviewed list of twelve (« diaporama », « raï »): « retraite »
  « Jubilación, retiro, pensión », « mutuel » « Mutuo »; « arnaque » and 7 more lose a gloss that was
  French. 12 rows (4).
- **Change 51's and 52's defects** (D6): an infinitive's noun entry repeated by its verb goes after it
  (`être` « Ser; (être + participio) Haber; Estar; … » all a verb's, `devoir`); a contraction of a
  preposition is read as a preposition (`des`, `du`, `duquel`); a note naming the part of speech
  after the meaning goes (`qui` « Quién; Que », `quoi`); « etc. » keeps its period (`avec`,
  `cochon`). `rien` gains its labels; its « Pequeño cantidad de algo » is the source's (Open
  Question 2).
- **Measured and left** (D7, D8): the sample's « amie », « el », « hall » (the sources' own text,
  Open Question 2); `peu` and `autre` headed as pronouns; the notes before a meaning
  (« (être + participio) Haber »); a function word's row by the treebank (Open Question 4); French
  levels with no Spanish gloss (Open Question 3); labels on expressions (Open Question 1).
- **fr-es re-pinned at its snapshot** (D9): reduced again from `lingua-pack-sources-fr-es-2026.10.10`,
  its snapshot, studied record and sources byte for byte; 900 rows change (359 of the top 10,000),
  27 lemmas lose their gloss (16), none gains one; coverage **83.0 / 70.7 / 56.7 %**, held to the
  floor (+1.6 / +1.9 / +2.2).
- **Every moved row read** (D10, D11): no golden or snapshot reads fr-es's glosses yet; the owner reads
  every changed row of the top 10,000 and the two lists.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *fr-es's glosses show a sense's register, age and place in the Spanish
  Wiktionary's words*, *fr-es's definitions read as a French word's meanings*, *fr-es's
  translation-table glosses list each Spanish word once, and never the French word*, and *fr-es's
  inverted table glosses no French word through a name, an acronym or another sense*. *French is
  glossed in Spanish from the Spanish Wiktionary's French section and the French Wiktionary's
  translation tables* and *fr-es is held to a coverage floor fixed before its first committed
  measurement* (held by `add-lingua-pack-fr-es`, not archived) are read as written: these are more of
  fr-es's own rules, and the floor holds. Nothing modified; *A gloss is in the reader's language,
  written by a person* (archived) is what D5 applies to fr-es.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (`reduce-fr-es.py`, `test_reduce_fr_es.py`,
  `tables/fr-es/`, `SOURCES.md`). ID, Music, Live, the back office, the site, the backend, the engine,
  the extension's source and snapshots, the Apple host app and the agent plugin are untouched.
- **What moves.** fr-es's `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json` (`pack_version`),
  `pin.json` (rule digest, pack: 1,973,407 → 1,970,716 B in the prototype), `README.md`.
- **What does not move.** en-fr, es-fr, es-en, en-es and fr-en — tables, pins, packs, goldens — and
  `tables/fr/`: no file of their rule digests is edited. No golden or snapshot reads fr-es's glosses:
  `fr-en.golden` is an English-native reader's, change 51's `fr-es.golden` is not written yet, and
  the assertions `cross_native.rs` and `committed_tables.rs` make of fr-es (`maison` « Casa », `et`
  « Y, e », `quant` glossed and no dictionary word) hold.
- **Release.** Silent: no package lists fr-es before change 52, which waits for this change.
- **Order.** After 49 (on `main`); either side of 48b, whichever lands second re-records the other's
  fr-es pin; before 51's fr-es golden if possible, else 51's snapshots are re-blessed here; before 52.
- **Effort**: 1.5–2.5 ideal days (design, *Effort*).
