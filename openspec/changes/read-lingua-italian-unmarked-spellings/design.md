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
