# Design

## Context

See proposal.md — Why. What constrains the shape:

- Change 5's Italian arm at each seam is the baseline's: `nfc_for` returns the text as it came,
  `lemmatize` takes the pack's lemma for the lowercased form else the form, `is_function_word`
  answers false, `document_names` returns early with none, `expression_piece` returns the lemma.
  Change 6 adds the pre-pass (elision pieces, articulated prepositions, truncations) at `0.2.0`,
  comparing written forms as they came.
- Spanish's cascade (`analysis/spanish.rs`): the pack's forms → old accented spellings → the
  enclitic rule → a plural fallback → the form. Its enclitic rule is a rule, not a table: at most
  two clitics off the end of an unlisted form, a base the lexicon lists shaped to take them
  (infinitive, gerund, a listed monosyllabic imperative, another verb form), the written accent
  the clitics moved checked against the base's stressed vowel. French's cascade
  (`analysis/french.rs`): the pack's forms → an unlisted lowercase plural whose singular is
  unlisted too → the form; an acronym step before it (change 43c); no enclitics (French's are
  hyphenated and split by the pre-pass).
- `document_names` (`engine.rs`) runs Spanish's base rule — a capitalised form appearing only
  mid-sentence, never in lowercase, not a dictionary word — in every language, with a French flag
  (a capital after an elided piece counts as mid-sentence; a hyphenated run as one form) and an
  English one (the pronoun *I*, a levelled lemma); change 5 gave Italian an early return.
- The closed classes are six sorted tables per language in `function_words.rs`, flagged by the
  cascade's lemma; English's, Spanish's and French's each chose their borderline words by
  measurement on a treebank.
- Italian's clitics: `mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`, `gli`, and the
  combined forms the first five make with the last six (`me lo` → `melo`, `te ne` → `tene`,
  `glie` + `lo`, `la`, `li`, `le`, `ne`). They attach to an infinitive, which drops its final `-e`
  (`parlarmi`, `dirmelo`, `porlo` from `porre`), to a gerund (`parlandogli`), to an imperative
  (`parlami`, `prendetelo`), to `ecco` (`eccolo`), and to a past participle in literary use
  (`andatosene`). Five monosyllabic imperatives double the clitic's first consonant: `da'`, `fa'`,
  `va'`, `di'`, `sta'` give `dammi`, `fallo`, `vattene`, `dimmi`, `stacci` — not before `gli`
  (`dagli`). No clitic moves a written accent.
- UD Italian-PUD (1,000 sentences) and MarkIT (CC BY 4.0) are the licence-clean treebanks; the
  gates French's cascade was held to (99.0 % resolved, 95.9 % content lemmas, 99.9 % AUX on PUD)
  are measured with the tables, which change 9 commits.

## Goals / Non-Goals

**Goals:**

- A verb carrying a clitic resolves to its verb; a phrase gloss stops glossing `di`, `il` and
  `non`; a decomposed accent is the same word; a name is set aside as in Spanish and French.
- The tables decide every lemma they list; a rule reaches only what they do not.
- Nothing of English, Spanish or French moves.

**Non-Goals:**

- The detection guard (change 8); the forms, grammar and level tables (9, 13, 14); the
  apostrophe-accent spellings (10); acronyms in capitals and names out of the words to learn (11);
  the expression keys (12).
- A past participle carrying clitics (`andatosene`): literary, rare in prose; measured at change
  9 and added at a later bump if the treebanks warrant it.
- A gender or number for a piece: the tables file `lo`, `la`, `li`, `le` under the article's lemma.

## Decisions

### D1 — NFC is a rule of Italian's pre-pass, not of its cascade alone

`nfc_for` answers NFC for Italian, as for Spanish and French: the elided forms, the articulated
prepositions and the truncations are compared composed, a run the pack lists is looked up
composed, and every token's text is composed, spans pointing into the source. The `tecnico` page's
NFD block then resolves to the pack's words; the corpus scenario still asserts the block is
committed decomposed.

### D2 — The cascade: the pack's forms, the enclitic rule, an unlisted plural, the form

`analysis/italian.rs`, beside `spanish.rs` and `french.rs`; `lemmatize` dispatches to it. For each
token, lowercased (`’` read as `'`) and in NFC:

1. **the pack's forms** — the one lemma the forms tables chose (M8);
2. **the enclitic rule** — only when the whole form is unlisted. The form's end is matched against
   the clitics, longest first: the combined forms (`glielo`, `gliela`, `glieli`, `gliele`,
   `gliene`; `melo`, `mela`, `meli`, `mele`, `mene`, and likewise with `te`, `se`, `ce`, `ve`), then
   the singles (`gli`, `mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`). The rest is
   accepted as a base when the lexicon lists it shaped to take clitics:
   - an infinitive with its final `-e` restored: a rest ending in `-r` is tried as `-re`
     (`parlar` → `parlare`, `dir` → `dire`), a rest ending in `-rr` as `-rre` (`por` → `porre`,
     `condur` → `condurre`); the lemma is the base's;
   - a gerund: a rest ending in `-ando` or `-endo` the lexicon lists;
   - an imperative: a rest the lexicon files under a verb — a lemma ending in `-are`, `-ere`,
     `-ire` or `-rre` — (`parla`, `scrivi`, `prendete`, `andiamo`);
   - a monosyllabic imperative: a rest of `da`, `fa`, `va`, `di` or `sta` when the clitic's first
     consonant was doubled (`dammi` = `da` + `mmi`), the doubling undone and the base read as
     `dare`, `fare`, `andare`, `dire`, `stare` by a table of five — the lexicon lists `da` and `di`
     as prepositions, so the table, not the lexicon, names the verb; `dagli` and `digli`, without
     doubling, go the same way;
   - `ecco`.
   Spanish's stress check has no counterpart: Italian spelling moves no accent when a clitic
   attaches. One split only: `dirmelo` is `dir` + `melo`, never `dirme` + `lo`.
3. **an unlisted plural** — only for a form written in lowercase, of four letters or more, that the
   pack does not hold and whose singular it does not hold either: `-chi` → `-co`, `-ghi` → `-go`,
   any other `-i` → `-o`, `-e` → `-a`. Left alone: an elided or hyphenated word, and a form ending
   in `-ai`, `-ei` or `-ii` (verb endings);
4. **the form itself**.

The order is the determinism contract. **The tables decide**: step 2 never splits a listed word
(`cinema`, `sistema`, `vela` are not `cine` + `ma` or `ve` + `la` — they are listed), and step 3
never reads a form as a word the pack lists: `vite` (vines; lives) stays itself when the pack
lists `vita` — the rule would rename it, and the tables' choice stands.

*Measured at implementation, kept or rejected by the treebanks' figures (D6):* step 3's `-e` →
`-a` (feminine plurals against `-e` singulars such as `cane` → `cani`, which `-i` → `-o` would
also mis-read), a past participle base, `-ci`/`-gi` plurals (`amici` → `amico`, `baci` → `bacio`:
the rule reads `baci` as `baco`; the tables list both). Each that moves tokens for the worse is
dropped from the cascade and named in `italian.rs`'s doc, as French's rejected rules are.

*Rejected — Spanish's accent retry.* Italian has no retired accented spellings; its accent
problem is the apostrophe (`perche'`), change 10's.

### D3 — Italian's closed classes

Six sorted tables, flagged by the cascade's lemma: determiners (the articles `il`, `lo`, `la`,
`i`, `gli`, `le`, `un`, `uno`, `una`; the demonstratives `questo`, `quello`, `codesto`; the
possessives `mio`, `tuo`, `suo`, `nostro`, `vostro`, `loro`; `ogni`, `qualche`, `alcuno`,
`nessuno`, `ciascuno`, `tale`, `altro`, `stesso`; and the `di` family, `del`, `dello`, `della`,
`dei`, `degli`, `delle`, which the pre-pass keeps whole), pronouns (`io`, `tu`, `lui`, `lei`,
`egli`, `ella`, `esso`, `noi`, `voi`, `loro`, `me`, `te`, `sé`, the clitics and their combined
forms, `che`, `cui`, `chi`, `quale`, `ciò`, `qualcosa`, `qualcuno`, `ognuno`, `chiunque`,
`niente`, `nulla`), prepositions (`di`, `a`, `da`, `in`, `con`, `su`, `per`, `tra`, `fra`,
`senza`, `verso`, `contro`, `sopra`, `sotto`, `dentro`, `fuori`, `dopo`, `prima`, `durante`,
`secondo`, `tranne`, `presso`, `oltre`, `attraverso`, `mediante`, `circa`, `lungo`, `entro`),
conjunctions (`e`, `ed`, `o`, `od`, `ma`, `però`, `anzi`, `né`, `che`, `se`, `perché`, `poiché`,
`siccome`, `quando`, `mentre`, `come`, `dunque`, `quindi`, `perciò`, `infatti`, `cioè`, `sebbene`,
`benché`, `affinché`, `finché`, `purché`, `oppure`, `ovvero`, `tuttavia`, `eppure`), auxiliaries and
modals (`essere`, `avere`, `potere`, `dovere`,
`volere`), negation (`non`, `no`, `né`). Left out, as words that carry meaning: `mai`, `più`,
`neanche`, `nemmeno`, `neppure`, `anche`, `ancora`, `molto`, `tanto`, `poco`, `tutto` (a
quantifier the reader learns), and `stare` (the progressive auxiliary is also *to stay*, as
Spanish's table keeps `estar` but French's leaves `aller` out — measured on MarkIT at
implementation, kept out unless the treebank files `stare` as AUX in the clear majority of its
occurrences). The owner reads the tables (task 9.1).

### D4 — An Italian document's names: Spanish's rule, after an elision, over a hyphenated run

`document_names` gains an Italian arm beside the French one: Spanish's base rule, a capital right
after an elided piece counting as mid-sentence (`L'Italia`, `dell'Europa`, `un'Olanda`), and a
hyphenated run the pack does not list read as one form. The early return change 5 added goes.
Change 11 later takes the names out of the words to learn (the Spanish and English rule).

### D5 — Italian's analyser version leaves the baseline's

`ITALIAN_ANALYZER_VERSION = "1.0.0"`; the spec says « at or above `1.0.0` » so that change 8's
guard (`1.1.0`) and later bumps contradict nothing. The fixture's manifest follows; the extension's
constant map reads it.

### D6 — The measurement at implementation

`scripts/lingua-data/measure/it-ud.sh`, after `fr-ud.sh`: the cascade's lemmas against UD
Italian-PUD and MarkIT's test section, once change 9's tables exist (the cascade over the fixture
measures nothing). Until then, the candidate rules of D2 are measured on the raw corpus and on
PUD with a stand-in lexicon built from PUD's own lemmas, as change 41 measured its rejected rules
before change 43's tables; the figures and the rules kept or dropped are recorded in the pull
request and in `italian.rs`'s doc.

### D7 — What the Italian golden shows, and what cannot move

The re-bless of `it-fr.golden` is this change's diff: the `clitici` page's forms resolve to their
verbs (the fixture lists `dire`, `dare`, `andare`, `fare`, `parlare`, `scrivere`, `ecco` and their
imperative and gerund forms), the `tecnico` page's NFD block to the pack's words (`città` glossed
« ville » appears), the phrase probes drop `non`, `di`, `il`, `che`; the `nomi` page's names are
set aside; the `omografi` page does not move (the tables decide). The six goldens, S0, every pin
and the English, Spanish and French arms are byte for byte.

### D8 — OpenSpec: the baseline's requirements rewritten, not handed on

As change 41 did for French: *Italian is a studied language served by the baseline analysis* is
REMOVED and *Italian is a studied language served by its own analysis* ADDED with the scenarios
that still hold, because a MODIFIED block cannot drop *No Italian word is a function word yet*
nor *An enclitic form is one token*, both false now; the baseline requirement is MODIFIED (its
*What the baseline shows today* names Italian's own version). Both are change 5's, modified by
change 6: this change archives after them.

## Risks / Trade-offs

- [A clitic read off a word that is not a verb] → the rule runs on unlisted forms only, and
  accepts only a base the lexicon lists shaped to take clitics; the fixture's and the tables'
  listed words are never split.
- [`-e` → `-a` and `-i` → `-o` mis-read a plural] → both guarded by « neither the form nor the
  singular listed »; measured on the treebanks (D6) and dropped if they move tokens for the worse.
- [`stare` as an auxiliary] → measured; kept out unless the treebank says otherwise, and the
  owner reads the tables.
- [A name after `dell'`] → French's reading, tested on `dell'Europa`.
- [The fixture's imperative forms] → the fixture files `parla`, `scrivi`, `prendete` under their
  verbs so the rule has bases to accept; change 16's tables do the same for real.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian. Rollback is a revert
with the golden's.
