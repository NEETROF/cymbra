# Design

## Context

See proposal.md — Why. What constrains the shape:

- Change 6's words seam: `push_italian_elisions` splits the elided pieces off the front of a written
  word; the rest is read as a truncation when the whole lowercased rest with its apostrophe is
  listed (`po'`), else by the rules every language shares, UAX #29 leaving a trailing apostrophe
  outside the word. French's seam runs `apostrophe_ending_a_word` on the rest after its elisions;
  Italian's re-attaches only a listed word, and never after an opening quote (the character before
  the written word is `‘` or `'`). So `perche'` is `perche`, `e'` is `e`, `c'e'` is `ci` + `e`,
  `E'` is `E` at `1.1.0`.
- Change 9's tables list Italian's forms with their accents (`perché`, `più`, `città`, `è`, `può`,
  `così`); the section marks `perchè`, `nè` and `caffé` as misspellings of *perché*, *né* and *caffè*
  (43d's kind, the mark being another accent, not a missing one) and gives `perche` (*perca*'s
  plural), `cosi` (*coso*'s plural; *cosare*), `citta` (*citto*'s feminine) and `caffe` (*caffo*'s
  feminine plural, *caffa*'s plural) as words of their own; `piu`, `puo`, `gia`, `sara` have no
  entry.
- wordfreq 3.1.1 strips an edge apostrophe: `perche'` is `perche`, `e'` is `e`, `c'e'` is `c` + `e`
  (checked with `wordfreq.tokenize`). Zipf figures (Italian; English, French, Spanish where they
  matter): `perche` 4.95 / `perché` 6.18; `perchè` 5.65; `piu` 5.22 / `più` 6.63; `citta` 4.16 /
  `città` 5.77; `cosi` 5.12 / `così` 6.14; `puo` 4.61 / `può` 6.01; `gia` 4.59 / `già` 5.95; `sara`
  4.53 / `sarà` 5.73 (English 3.96, Spanish 4.13); `caffe` 3.75 / `caffè` 4.83; `caffé` 3.40; `cafe`
  3.33 (English 4.09, Spanish 3.81); `papa` 5.15 / `papà` 4.66; `eta` 3.73 (Spanish 4.10) / `età`
  5.11; `pero` 4.42 (Spanish 6.57) / `però` 5.70; `nè` 4.55 / `né` 5.21; `poiche` 3.22 / `poiché`
  4.95; `cioe` 3.86 / `cioè` 5.28; `verita` 3.68 / `verità` 5.16; `lunedi` 3.85 / `lunedì` 4.67;
  `sopratutto` 4.21 / `soprattutto` 5.43; `e` 7.46 / `è` 7.10; `da` 6.84 / `dà` 4.94; `si` 6.88 /
  `sì` 5.58; `se` 6.61 / `sé` 4.98; `ne` 6.10 / `né` 5.21; `la` 7.30 / `là` 5.03; `li` 5.80 / `lì`
  5.28; `fa` 6.09 / `fà` 3.72; `sta` 5.87 / `stà` 3.51; `va` 5.73 / `và` 3.39; `po` 5.75 / `pò` 4.47;
  `su` 6.38 / `sù` 3.16.
- French's two precedents: matrix change 43d reads a spelling the section gives only as a marked
  word when the word is the commoner — a rule over the tables, nothing of the spelling's own left,
  with the condition that the spelling be the word with its marks dropped (misaccented spellings
  went to 43f's list); change 43e reads the web's spellings by a reviewed list of 468 rows kept by
  four conditions (Zipf ≥ 2.5, no meaning of its own, a dictionary word's, less than half as frequent,
  English not writing it more), with the owner's exception `sure` → *sûr* outside the method, a
  boundary check at each reduction (`web_boundary`: a dictionary update naming a row stops for a
  person), entries pointing at a row following it, and the two-words and doubtful groups left out.
- The Italian golden runs over the it-fr fixture (`scripts/lingua-data/testdata/it-fr/`) until change
  16 (change 9's D9); the fixture lists `è` and `città` and not `perché`, `più`, `può`, `così`;
  `analyzer_compatible` is an exact match, so its manifest moves with every version.
- The Italian corpus's `informale` page holds `perche'`, `e'`, `E'`, `piu'`, `citta'`, `perche`,
  `cosi`.
- Change 6's eight truncations: `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'`, `be'`. The
  literary apocopes `ne'` (*nei*), `de'`, `co'`, `pe'`, `a'` (*ai*), `su'` (*sui*) and the Tuscan
  `e'` (*egli*) are frequent in public-domain Italian, which the reader's EPUBs hold.

## Goals / Non-Goals

**Goals:**

- The three ways the web writes a final stressed vowel resolve to the accented word, with its
  gloss, rank and readings, wherever the reading is safe.
- A spelling that is also a word keeps its own reading: Italian recall of *e*, *da*, *si* comes
  first.

**Non-Goals:**

- Capitals without accents in running text other than `E'`.
- A spelling of a word the tables do not rank.
- The apocopes `ne'`, `de'`, `co'`, `pe'`, `a'`, `su'`: read as change 6 reads an unlisted trailing
  apostrophe (outside the word) — a rule of their own if the reader's books ask for it.

## Decisions

### D1 — The apostrophe that stands for an accent: a rule over the lexicon, on the rest after the elisions

In Italian's words seam, **after** `push_italian_elisions` has split the elided pieces off — as
French's seam runs `apostrophe_ending_a_word` on its rest —, the rest ending in a vowel followed by
an apostrophe (straight or typographic) with neither a letter nor a digit after it is a candidate,
unless an opening quote (`‘` or `'`) stands before the whole written word (an elision's own
apostrophe is never one). The rest with its final vowel accented — `a` → `à`, `e` → `è` or `é`,
`i` → `ì`, `o` → `ò`, `u` → `ù` — is looked up in the lexicon; when an accented form is listed and
the rest as written (with its apostrophe) is not, the token is that form, spanning the rest and
its apostrophe, its written capital kept (`E'` → `È`). So `perche'` → `perché`, `e'` → `è`,
`c'e'` → `ci` + `è`, `dov'e'` → `dove` + `è`, `com'e'` → `come` + `è`, `l'e'` → `lo` + `è`.

**`è` or `é`.** Among the listed candidates, the one the pack files under itself (`lemma_of(c) ==
c`, as `listed_whole` reads) wins over one filed under another word; when neither or both are so
filed, `è` before `é`. The tables decide, never a frequency: wordfreq folds every apostrophe
spelling into the accent-less one. So with D2's `perchè` a form of *perché* beside `perché`,
« perche' » reads `perché`; « caffe' » reads `caffè` with `caffé` filed under it; « e' » reads `è`
(no `é`).

**Change 6's listed words come first**: the eight truncations keep their reading (`po'` → `poco`,
`da'` → `dai`, `fa'` → `fai`, `va'` → `vai`, `sta'` → `stai`, `di'` → `dici`, `mo'` → `modo`, `be'` →
`bene`); `pò`, `dà`, `fà`, `và`, `stà`, `dì` are not tried. The cost, named: `da'` written for *dà*
reads `dai` — the same lemma *dare*, the card right and the token text not —, `fa'`, `sta'`, `va'`
likewise; `di'` written for *dì* (the day) reads *dire*, rare. **The apocopes `ne'`, `de'`, `co'`,
`pe'`, `a'`, `su'` are excluded** from the rule by a closed list beside it: `ne'` would read `né`,
`su'` `sù`, `a'` nothing (no `à`), wrong every time in a book; they stay as change 6 reads an
unlisted trailing apostrophe. The Tuscan `e'` (*egli*) reads `è`: the cost accepted, « E' vero »
being the rule.

**`is_elided`'s invariant widens**: a token read by this rule spans an apostrophe, so the phrase
gloss's `elided` flag (change 44c's reading, which change 12 carries to Italian) says yes for it;
an expression written with `perché` is not met when the page writes `perche'` — named for change
12's measurement, which may read the flag by the token's text.

*Rejected — a reviewed list of apostrophe spellings.* They are the accented words' tail, one for
one; the lexicon is the list, and change 9's tables update it.

### D2 — The dictionary's own accent-less spellings: change 43d's rule, without the marks condition

`reduce-it-fr.py` reads a form the section gives only as a misspelling, alternative, obsolete or
nonstandard spelling of a marked word — every sense of every entry of it such a spelling — as a
form of that word, when wordfreq rates the marked word strictly commoner than the spelling and than
each of the spelling's forms; its inflections and the entries pointing at it follow; nothing of its
own remains. A spelling with a sense of its own beside the marking keeps its entry. Unlike 43d, the
rule does not require the spelling to be the word with its marks dropped — on purpose: Italian's
dictionary spellings are mostly another accent (`perchè`, `nè`, `caffé`, `fà`, `stà`, `và`, `sù`,
`pò`) or a consonant (`sopratutto`), 43f's kind for French, read here by one rule. D3's candidate
generation then yields `fa`, `sta`, `va`, `su`, `po` from those rows — words of their own, excluded
by D3. Measured at task 2.2, before the owner's read: the spellings read, the ones refused and why,
as 43d listed them.

### D3 — The web's accent-less spellings: change 43e's method, Italian's list, the owner's exceptions

Candidates: every form of the tables ending in an accented vowel, its accent dropped. Kept when:
wordfreq attests the spelling at Zipf ≥ 2.5; the section gives it no meaning of its own (a form of a
word counts) and does not give it as a spelling of its word (D2's kind); its accented word's lemma
has a sense of its own in the section that is no name's — change 9's own-entry rule, standing in
for « an Italian dictionary word » until change 16 writes `lexical.tsv`, which then re-checks every
row —; the spelling is less than half as frequent as the word (a Zipf gap of 0.30 or more); and
neither English, French nor Spanish (wordfreq's lists; 43e read English alone, 43f seven languages)
writes it more often than Italian does. As 43e: the boundary check at each reduction
(`web_boundary`: a dictionary update that names a row stops for a person), an entry pointing at a
row follows it, the two-words and doubtful groups left out.

**The owner's exception rows**, outside the method and the boundary check, as 43e's `sure` → *sûr*:
`perche` → *perché* (the section: *perca*'s plural, the perch; 4.95 against 6.18), `cosi` → *così*
(*coso*'s plural, *thingy*, and *cosare*; 5.12 against 6.14), `citta` → *città* (*citto*'s feminine,
archaic; 4.16 against 5.77), `caffe` → *caffè* (*caffo*'s feminine plural, *caffa*'s plural; 3.75
against 4.83) — each the web's commonest accent-less spelling against a homograph nobody writes,
with its reason and its corpus counts (task 2.2), settled by the owner (task 0.1). `piu`, `puo`,
`gia`, `verita`, `liberta`, `universita`, `attivita`, `lunedi`, `poiche`, `cioe` pass the method;
`sara` too (4.53 against 5.73; a name capitalised, as 43e's Q3 kept names).

**Left out, by group**, the first failing test named: the words of their own — `e` (*and*, 7.46
against `è` 7.10), `da`, `si`, `se`, `ne`, `la`, `li` —; `papa`, commoner than *papà* (5.15 against
4.66); `eta` (the Greek letter) and `pero` (the pear tree), words of their own first and Spanish's
besides (4.10 against 3.73; 6.57 against 4.42); `cafe`, English's
(4.09 against 3.33); D2's spellings; hyphenated words wordfreq cannot count. The rows join
`reduce-it-fr.py`'s table as one literal of `spelling → word` with one shared reason and the
exception rows with theirs, each a form of its word.

### D4 — Nothing of a spelling's own; the readings follow the word

A read spelling has no rank, level, gloss or dictionary-word bit; the pack keys everything by the
accented lemma, and the card shows it. Once change 13 gives Italian its readings, a spelling carries
its word's, as a form of it (no readings exist before; `tags.tsv` holds `NOUN` alone).

### D5 — Version, tables, the fixture, the golden

- `ITALIAN_ANALYZER_VERSION` `1.1.0` → `1.2.0` — or the next minor, whichever of change 8 and this
  change lands second (D1 is a tokenisation rule).
- it-fr re-reduced (D2 and D3 are rows of `tables/it/forms.tsv`), `pin.json`, `manifest.json`,
  `README.md` moved; `it-ud.sh`'s figures re-measured and recorded.
- The fixture (`scripts/lingua-data/testdata/it-fr/`), which the golden runs over until change 16:
  `manifest.json` at the new version; `forms.tsv` gains `perché`, `più`, `può`, `così` (D1's
  targets the `informale` page needs; `è` and `città` are there) and two spelling rows standing in
  for D2 and D3 — `perche` → `perché`, `cosi` → `così` —; `gloss.tsv` their glosses.
- The Italian golden re-blessed on the `informale` page — `perche'`, `e'`, `E'`, `piu'`, `citta'`
  resolve by D1 through the fixture's accented words, `perche` and `cosi` through the fixture rows —
  and the version line; change 16 re-blesses on the tables. Nothing else moves; the six goldens and
  pins do not move.

## Measured

- Before proposing: the Zipf figures of the Context (wordfreq 3.1.1, 2026-10-22) and the section's
  entries for the candidates (`perche`, `cosi`, `citta`, `caffe`, `perchè`, `nè`, `caffé`, `e'`,
  `c'è`, `dov'è`; `piu`, `puo`, `gia`, `sara` absent).
- At task 2.2, before the owner's read (0.1 and 4.1): D2's spellings read and refused; D3's list
  with each row's figures and the groups left out; D1's cost on the corpus and Tatoeba (`e'` for
  *e*, the truncations written for *dà*, *fà*, *dì*).

## Risks / Trade-offs

- [`e'` read as *è* where a writer meant *e* with a stray apostrophe, or the Tuscan *egli*] →
  rare against « E' vero »; measured on the corpus and Tatoeba.
- [A truncation written for an accented word] → `da'`, `fa'`, `sta'`, `va'` keep their lemma (the
  card right); `di'` for *dì* reads *dire* (rare); named in D1.
- [The apocopes of books] → excluded from D1 (`ne'` never `né`); a rule of their own later.
- [A spelling that is a word] → the list excludes every form the tables list as a word; the rule
  of D1 never applies to a word the tables list as written; the four exceptions are the owner's.
- [The accent spelling read as an elided piece by the phrase gloss] → named for change 12.
- [A quoted word ending in a vowel] → no re-attachment after an opening quote (change 6's D5).

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. Rollback is a revert with the
tables', the pin's, the fixture's and the golden's.

## Implementation — task 2.2's measurement and what departs from the decisions

Measured on 2026-10-23, before the owner's read (tasks 0.1 and 4.1), by criteria written before any
figure was read, on it-fr's pinned sources: the Italian section of the release
`lingua-pack-sources-it-fr-2026.10.23` (sha256 `ae7de484…`), MarkIT's training and development
sections and Morph-it! 0.4.8 (`b137343d…`), from which `reduce-it-fr.py` at change 9's head writes
change 9's committed tables byte for byte (128,429 forms); wordfreq 3.1.1 installed by hash from
`requirements-reduce.txt`. A first pass on kaikki.org's own extract of the section (`cfb9e823…`) gave
the same rows, the same apocopes and figures within a few spellings. The running text is change 8's Italian
corpus but its held-out parts — the `pages-it` baseline, the two Gutenberg novels, its Italian Wikipedia
sample, its 20,000-sentence Tatoeba sample (`it-guard-tatoeba-ids.txt`) and MarkIT's training and
development sections, 990,502 words —, and the whole Tatoeba export (989,119 sentences) beside it.
UD Italian-PUD and MarkIT's test section were read by nothing D1, D2 or D3 decides.

### D1 as built

- **Change 6's listed words, not the pack's apostrophe forms, come first.** Change 9's tables list 87
  forms ending in an apostrophe — the pieces, and `e'` (the Tuscan *ei*), `i'` (*io*), `de'`, `ca'`,
  `vo'`, `pro'`, `citta'`, `liberta'`, the imperatives `rifa'`, `disfa'` … from the section's entries
  and Morph-it!. Read as D1 writes it (« the rest as written is not listed »), « E' vero » would read
  as before, against D1's own example. The rule consults the pack for the accented form alone; what
  keeps its apostrophe's reading is what Italian's pre-pass lists (the truncations, an elided form
  written alone). The spec's ADDED requirement says so.
- **`è` or `é`: a form the pack files under the other is the other's spelling.** The tables rank the
  misspelling `é` as a word of its own (wordfreq 5.17) while `è` is a form of *essere*: « the one the
  pack files under itself wins » read every « E' » as `É`. The rule drops a candidate the pack files
  under the other candidate (`perchè` under `perché`, `caffé` under `caffè`), then tries `è` before `é`;
  every outcome D1 names is kept, « E' » is `È`.
- **The closed list grows from six to sixteen**, by the measurement: ten literary apocopes and
  truncations the rule read as no word of theirs, each a form the tables list with the accent and that
  Italian does not write so — `que'` (*quei*, 267 times in *I promessi sposi*, read `qué`), `ca'`
  (« Ca' Foscari », 17, `cà`), `ve'` (*vedi*, 10, `vè`), `mi'` (*mio*, 4, `mì`), `fra'` (*frate*, 4,
  `frà`), `to'` (*togli*, 3, `tò`), `i'` (*io*, 3, `ì`), `me'` (*meglio*, 2, `mè`), `vo'` (*voglio*,
  2, `vò`), `fe'` (*fece*, 1, `fè`). Each reads as change 6 read it, its apostrophe outside the word.
- **The quote** is change 6's own scan (`inside_a_quote`): no quote opened before the word still open,
  so « ‘Ma e' vero’ » keeps `e`.

**D1's cost, measured.** In the corpus, with the 16 excluded, the rule reads 34 tokens: `E'`/`e'` 20
times — 18 *è* (MarkIT's 14 « E' un tipo… », Tatoeba's 1, the `informale` page's 3), 2 the Tuscan
*egli* in Manzoni (« e' poteva ben aspettare »), D1's accepted cost —; the `informale` page's `perche'`,
`citta'`, `piu'`; `pie'` → `piè` and `pa'` → `pà` (*piede*, *papà*, right); and 9 wrong or neutral:
`se'` (*sei*, Dante and Collodi) → `sè` 3, `so'` (dialect *sono*) → `sò` 3 (whose word is *essere*
anyway), `cchiu'` (Neapolitan) → `cchiù` 2, `che'` (*che i*, Dante) → `ché` 1. The whole Tatoeba export:
46 tokens, `E'`/`e'` 43 — 41 *è*, 2 the Tuscan proverb « Batti il buono, e' megliora » —, `finche'` →
`finché`, `qua'` → `quà` (read as *qua*), `Dato'` (a Malaysian title) → `datò`. Left outside as before:
`de'` 613, `que'` 267, `ne'` 106, `a'` 99, `co'` 81, `ca'` 17, `su'` 13, `ve'` 10, `i'` 5, `fra'` 4, `mi'`
4, `vo'` 3, `pe'` 3, `to'` 3, `me'` 2. Change 6's truncations (583: `po'` 480, `da'` 68 — Manzoni's
*dai*, « da' primi suoi anni » —, `di'` 18, `va'` 13, `fa'` 4, `sta'` 2) keep their reading: none of them
is written for *dà*, *fà* or *dì* in the corpus.

### D2 as built

- **The rule reads 3,206 spellings** (3,209 before the owner kept three apart), not the dozen the
  proposal names: every word the section gives only as an alternative form or spelling (2,196), an
  obsolete, archaic or dated one (823), a misspelling (168) or a nonstandard one (12) of a commoner word,
  7 mixed. 771 become forms of the tables (their word ranked); **261 were ranked lemmas** and leave the
  ranks — `perchè` 124, `ve` 1,050 (→ *vi*),
  `et` 1,057 (→ *e*), `sino` 1,477 (→ *fino*), `finchè` 2,666, `sopratutto` 3,177, `oramai` 3,517,
  `poichè` 3,645, `alcool` 3,664, `bo` 4,037 (→ *boh*), `core` 4,045 (→ *cuore*), … (the PR lists them
  all).
- **Kept apart**, as the report names them: 1,739 whose word is not the commoner (99 ranked: `the`,
  `anti`, `figo`, `puzza`, `click`, `zombie`, `obbedire` …), 144 with a form as common as the word (the
  forms guard: `altiero`, `canaria`), 15 whose senses spell two words (`nè`: *né* and *neh*; `je`), 332
  with no word form (`nov`, `mag`, `dic`: the months' abbreviations; `daccordo`, « alternative spelling
  of d'accordo », a word beginning with an elided piece), 1 a reviewed table's row (`giu`, D3's), and the
  owner's 3 (`SPELLINGS_KEPT_APART`, below).
- **Clarified while measuring**, each by D2's own terms: a pointer at several words spells no word
  (`vabbè`, « alternative form of va bene », would have read as `va`; `menomale`, `daccordo`); an
  apocope, an abbreviation, a clipping, a contraction or a pronunciation spelling is no spelling (`un`,
  `ca'`, `bici`: change 9 reads the apocopes); a pointer at one of change 6's pieces is the piece's,
  so `fà` (« misspelling of fa » and « of fa' ») spells *fa* — `nè` stays apart, against the proposal,
  its second entry spelling *neh*.
- **Departure — a spelling's forms follow it only where no other word claims them.** Read as D2 writes
  it (« its inflections and the entries pointing at it follow »), an archaic verb's paradigm carried 43
  forms of other words to the commoner verb by D5's frequency choice: `sguardi` (*sguardo*'s plural, a
  row of `sguardare`) → *guardare*, `annate` and `annessi` → *andare*, `salse` → *salire*, `carcasse` →
  *caricare*, `reduci` → *ridurre*, `acconti` → *raccontare*. The link from such a form to the spelling
  is dropped once the section is read (`Lexicon.drop_spelling_rows`, 1,069 forms); a form no other word
  claims follows (`alcoolici` → *alcolico*, `annunziato` …). One move is left, a form of a form:
  `accise` (*accisa*'s plural, and *acciso*'s, the participle of the archaic `accidere`) → *uccidere*.
- **Its costs:** 12 forms lose their reading and read as unknown words, none as another word: the
  spellings `toh` (Zipf 3.30, rank 13,309, → `to'`), `tiè` (→ `tie'`), `quer` (→ the apocope `quel`),
  `velia`, `postproduzione` (→ the hyphenated `post-produzione`), and forms whose chain now ends on an
  unranked word — `accellera`, `querci`, `soprese`, `velie`, `ziani`, and `citto` and `liberte`, rows of
  the exceptions' own entries (all under Zipf 2.1). Readings a reader may doubt, the dictionary's:
  `core` → *cuore* (also English's « core »), `deo` → *dio*, `jo` → *io*, `lugo` → *luogo*, `bueno` →
  *buono*, `hora` → *ora*, `seoul` → *seul*, `et` → *e*; `casinò`, `omg` and `cè` the owner keeps apart.

### D3 as built

**The candidates** (the tables after D2): every form ending in an accented vowel, the accent dropped —
4,902 spellings. Tests in order; a group is named by its first failing test.

| Group | Spellings |
|---|---|
| Below Zipf 2.5 | 3,711 |
| Named by the section — a word of its own | 649 (`di`, `e`, `che`, `la`, `si`, `da`, `se`, `ne`, `li`, `su`, `fa`, `te`, `re`, `papa`, `eta`, `pero`, `citta`, `meta`, `faro`, `passera` …) |
| — a form of a word | 336 (`cosi`, `perche`, `caffe`, `liberta` (*liberto*'s feminine), `unita`, `necessita`, `stabilita`, `capacita`, `felicita`, `finche`, the passato remoto's `-o` spellings that are the present's first person: `domando`, `comincio`, `dichiaro` …) |
| — an inflection a word's entry lists | 4 (`gia`: `gìa`, *gire*'s imperfect; `dira`, `umidita`, `tome`) |
| — the dictionary's spelling, D2's | 6 (`ve`, `bo`, `anziche`, `pieta`, `pote`, `benche`: read by D2) |
| — its word's spelling | 1 (`aho`) |
| Its word a name or no word of the section | 60 (`to`, `co`, `pre`, `que`, `canada`, `jose`, `cafe` (and English 4.09 against 3.33), `peru`, `forli`, `nicolo` …) |
| Not half as frequent as its word | 25 (`po`, `the`, `be`, `como`, `ultra`, `toto`, `ventitre`, `trentatre` …) |
| Spanish writes it more | 5 (`dara`, `potro`, `carita`, `ando`, `tribu`) |
| French writes it more | 5 (`verra`, `arrivera`, `agora`, `finira`, `portera`) |
| **Rows** | **100**, and the owner's 6 exceptions |

**The rows**, by band — the spelling, its accented form (and its word when another), the spelling's
Zipf / the accented form's:

| Zipf | Rows | Spellings (spelling's Zipf / the accented form's) |
|---|---|---|
| 4.00– | 5 | `piu` → più 5.22/6.63, `puo` → può, *potere* 4.61/6.01, `sara` → sarà, *essere* 4.53/5.73, `cio` → ciò 4.14/5.70, `giu` → giù 4.08/4.80 |
| 3.50–4.00 | 17 | `societa` → società 3.97/5.56, `realta` → realtà 3.92/5.40, `venerdi` → venerdì 3.90/4.79, `attivita` → attività 3.86/5.36, `cioe` → cioè 3.86/5.28, `lunedi` → lunedì 3.85/4.67, `vabbe` → vabbè 3.80/4.40, `possibilita` → possibilità 3.75/5.25, `qualita` → qualità 3.74/5.18, `giovedi` → giovedì 3.72/4.51, `verita` → verità 3.68/5.16, `martedi` → martedì 3.67/4.44, `universita` → università 3.64/5.11, `fara` → farà, *fare* 3.61/4.89, `mercoledi` → mercoledì 3.56/4.42, `chissa` → chissà 3.53/4.64, `responsabilita` → responsabilità 3.51/4.95 |
| 3.00–3.50 | 28 | `proprieta` → proprietà 3.46/5.04, `difficolta` → difficoltà 3.44/5.01, `pubblicita` → pubblicità 3.43/4.79, `saro` → sarò, *essere* 3.41/4.65, `comunita` → comunità 3.40/5.08, `avra` → avrà, *avere* 3.38/4.97, `umanita` → umanità 3.37/4.44, `gesu` → gesù 3.34/4.85, `novita` → novità 3.31/4.67, `sanita` → sanità 3.28/4.59, `potra` → potrà, *potere* 3.27/4.89, `autorita` → autorità 3.26/4.92, `quantita` → quantità 3.26/4.86, `modalita` → modalità 3.24/4.73, `volonta` → volontà 3.24/4.80, `velocita` → velocità 3.23/4.83, `poiche` → poiché 3.22/4.95, `opportunita` → opportunità 3.19/4.76, `andra` → andrà, *andare* 3.18/4.64, `dignita` → dignità 3.18/4.58, `curiosita` → curiosità 3.15/4.52, `facolta` → facoltà 3.13/4.64, `identita` → identità 3.12/4.71, `avro` → avrò, *avere* 3.10/4.38, `nonche` → nonché 3.08/4.73, `dovra` → dovrà, *dovere* 3.07/4.72, `percio` → perciò 3.04/4.78, `solidarieta` → solidarietà 3.00/4.52 |
| 2.50–3.00 | 50 | `bensi` → bensì 2.94/4.31, `riusci` → riuscì, *riuscire* 2.94/4.09, `schiavitu` → schiavitù 2.92/4.04, `personalita` → personalità 2.91/4.60, `disponibilita` → disponibilità 2.89/4.29, `priorita` → priorità 2.87/4.40, `probabilita` → probabilità 2.87/4.45, `civilta` → civiltà 2.86/4.44, `andro` → andrò, *andare* 2.85/4.17, `virtu` → virtù 2.84/4.36, `poverta` → povertà 2.79/4.39, `mentalita` → mentalità 2.77/4.16, `criminalita` → criminalità 2.76/4.11, `affinche` → affinché 2.75/4.41, `parita` → parità 2.74/4.27, `varieta` → varietà 2.73/4.57, `entita` → entità 2.69/4.38, `specialita` → specialità 2.69/4.05, `nullita` → nullità 2.68/3.52, `contabilita` → contabilità 2.67/3.75, `dovro` → dovrò, *dovere* 2.66/4.14, `vedra` → vedrà, *vedere* 2.66/4.17, `purche` → purché 2.65/3.93, `localita` → località 2.64/4.55, `tabu` → tabù 2.64/3.69, `trinita` → trinità 2.64/3.66, `celebrita` → celebrità 2.63/3.99, `popolarita` → popolarità 2.63/4.08, `professionalita` → professionalità 2.63/3.91, `legalita` → legalità 2.62/4.11, `succedera` → succederà, *succedere* 2.61/4.16, `vedro` → vedrò, *vedere* 2.61/3.83, `maturita` → maturità 2.60/4.14, `provera` → proverà, *provare* 2.60/3.31, `daro` → darò, *dare* 2.59/3.86, `visibilita` → visibilità 2.57/4.24, `rimarra` → rimarrà, *rimanere* 2.56/4.17, `sensibilita` → sensibilità 2.56/4.30, `altresi` → altresì 2.55/4.07, `attualita` → attualità 2.55/4.09, `serenita` → serenità 2.55/3.97, `sovranita` → sovranità 2.54/4.22, `granche` → granché 2.53/3.68, `serieta` → serietà 2.53/3.93, `funzionalita` → funzionalità 2.52/4.02, `ahime` → ahimè 2.51/3.66, `bonta` → bontà 2.51/4.02, `tranquillita` → tranquillità 2.51/4.00, `indennita` → indennità 2.50/3.82, `validita` → validità 2.50/4.02 |

No spelling two words share: a row's second accented form is a misspelling the tables rank as a word
of its own (`piú` 4.49, `puó`, `ció`, `giú`, `saró`, `ahimé`, `vabbé` …), a list of 43f's kind for
Italian; each row reads as the commoner, the accented word.

**The owner's exception rows**, outside the method and the boundary (task 0.1: the four D3 named, and `gia` and `liberta`, settled on 2026-10-23):

| Spelling | The section | Zipf it / en / fr / es | The accented word | In the corpus and Tatoeba |
|---|---|---|---|---|
| `perche` | *perca*'s plural (the perch); `perché`'s entry lists it as an obsolete alternative | 4.95 / 1.68 / 3.59 / 1.74 | `perché` 6.18 | 1 (`informale`: *perché*); no *perca*; `perché` 19,881 |
| `cosi` | *coso*'s plural (*thingy*), *cosare*'s present | 5.12 / 2.21 / 2.39 / 2.20 | `così` 6.14 | 4: 3 *così* (MarkIT « ottenibili cosi facilmente », `informale`, Tatoeba), 1 the surname « Francesca Cosi »; `così` 12,625 |
| `citta` | *citto*'s feminine (Tuscan, archaic: a girl) | 4.16 / 2.04 / 1.84 / 1.66 | `città` 5.77 | 2, Tatoeba's « sarò in citta »: *città*; `città` 4,004 |
| `caffe` | *caffo*'s feminine plural, *caffa*'s plural (odd, of a number) | 3.75 / 2.32 / 1.75 / 1.75 | `caffè` 4.83 | 0; `caffè` 2,895, `caffé` 9 |
| `gia` (the owner's, 2026-10-23) | `gìa`, *gire*'s archaic imperfect, listed by *gire* and *ire*: the tables read `gia` as *ire* | 4.59 / 3.07 / 2.86 / 2.81 | `già` 5.95 | 1, the syllable « -gia- » in a Wikipedia article; `già` 5,996 |
| `liberta` (the owner's, 2026-10-23) | *liberto*'s feminine (a freedwoman): the tables read it as *liberto* | 3.73 / 1.27 / 1.93 / 2.59 | `libertà` 5.16 | 0; `libertà` 563 |

Each would pass every test but the section's rare homograph. D3's text named `gia` and `liberta`
among the rows the method passes: the section's entries say otherwise, and the owner made them
exception rows (below). The method leaves others of that kind, which read as the tables read them:
`umidita` (2.75 against 4.06), `felicita`, `capacita`, `gravita`, `utilita` (verb forms of *felicitare*,
*capacitare*, *gravitare* …), `finche` (3.55 against `finché` 4.85).

**Clarified at the boundary**, as 43e reads it: a name's entry says nothing of the word (`sara`,
`gia`'s `Gia`: D3 named `sara` a row); a spelling of another word D2 does not read passes (`giu`,
« alternative form of giu. », the abbreviation of *giugno*); an inflection a source marks doubtful is
no inflection (`perché` lists `perche` as « alternative, obsolete »). **Added**: the reduction also
stops when a row's word is no ranked lemma (`unread_web_spellings`), a row following the tables'
choices.

**The rows in running text** (the corpus and the whole Tatoeba): 54 occurrences — 17 the accented word
(`cosi` 3, `difficolta` 2, `venerdi` 2, `citta` 2, `perche`, `fara` « Domani fara bello? », `cio`
« cio è », `responsabilita`, `riusci`, `lunedi`, `martedi`, `realta`), 34 names (`Sara` 30 — mid-
sentence, the names rule sets it aside; at a sentence's start it reads as *essere* —, the surnames
`Cosi` and `Fara`, `Avro`, the acronym `CIO`), 3 other (`giu.`, the syllable `-giu-`, Sicilian `chissa`).

### D5 as built

- `ITALIAN_ANALYZER_VERSION` is `1.2.0`; `packs.spec.ts` already matches Italian's version by its
  pattern (change 7), and does not move.
- **The fixture gains the two spelling rows alone.** It lists `perché`, `più`, `può` and `così` since
  changes 7 and 8; glossing them, as D5 asks, would move every page that writes `più` (`manzoni`,
  `narrativa`, `nomi`, `tecnico`, `grammatica`) and the probes that count glossed words (the universe
  111 → 114, the estimate 83 → 85, A2's typical vocabulary), against D5's « nothing else moves ».
- The golden moves by the `informale` page (8 tokens: `È`, `perché`, `città`, `è`, `più`, `perche` →
  *perché*, `cosi` → *così*, `è`), its phrase probe (5 tokens), the version on every page and the pack's
  line (5,956 → 5,963 B); nothing else. The fixture's whole pack is 5,963 B
  (`pipeline_testdata.rs` records its sha256).

### What the dispatch will move (local preview)

The reduction of the pinned sources with D2 and D3, run locally (task 2.3 makes the real one, and
should find the same bytes), with the owner's decisions: `forms.tsv` 128,429 → 129,736 rows (1,319 in:
509 spellings, 810 forms of the lemmas entering; 12 out); `freq.tsv` 362 lemmas out, all spellings (2 in
the top 1,000, 18 in the top 5,000, 49 in the top 10,000), 362 in at ranks 9,776 to 60,000 — names that
were forms of a spelling's paradigm (`uffizi`, `annunziato`) and the cut's next words. The whole pack
1,287,014 → 1,295,051 B. Measured as `it-ud.sh` measures (the gate, not a choice: nothing was tuned on it), with
this change's core on both: UD Italian-PUD 99.26 / 91.38 / 99.90 % (D1 moves nothing there) → 99.26 /
**91.31** / 99.90 % of 19,155 words, 8,909 content words and 985 auxiliaries, against the gates 98.5 /
90.5 / 97; MarkIT's test section 99.43 / 94.68 / 98.87 → 99.43 / 94.65 / 98.87 %.

## Owner's decisions, 2026-10-23

Guillaume settled tasks 0.1 and 4.1 on this section's figures:

- **Task 0.1 — the exception rows**: the four D3 named, `perche` → *perché*, `cosi` → *così*, `citta` →
  *città*, `caffe` → *caffè*, and two more the measurement found, `gia` → *già* (the tables read it as
  the archaic *gire*'s imperfect, *ire*) and `liberta` → *libertà* (read as *liberto*'s feminine), each
  with its reason and figures in `OWNER_EXCEPTIONS`. `finche`, `felicita`, `umidita` and the method's
  other homographs are no rows: they read as the tables read them.
- **Task 4.1 — D2's report and D3's list**: D3's 100 rows as listed. Three of D2's readings are wrong and
  kept apart by name, each with its reason, in `SPELLINGS_KEPT_APART`: `casinò` (the gambling house is
  not *il casino*), `omg` (the web's « oh my god », not *ogm*) and `cè` (a mistyped *c'è*, not *cioè*);
  they read as before, words of their own. `core` → *cuore* (poetic, Romanesco) and `deo` → *dio* stay.
  The 12 forms that lose their reading are accepted as recorded costs: none is misread — each reads as
  an unknown word —, and the most frequent, the interjection `toh`, is Zipf 3.30 (rank 13,309).
