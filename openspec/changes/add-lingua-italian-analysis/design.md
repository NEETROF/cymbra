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
- `document_names` (`engine.rs`) runs Spanish's base rule — a form the document never writes in
  lowercase and capitalises at least once in mid-sentence, not a dictionary word — in every language, with a French flag
  (a capital after an elided piece counts as mid-sentence; a hyphenated run as one form) and an
  English one (the pronoun *I*, a levelled lemma); change 5 gave Italian an early return.
- The closed classes are six sorted tables per language in `function_words.rs`, flagged by the
  cascade's lemma; English's, Spanish's and French's each chose their borderline words by
  measurement on a treebank.
- Italian's clitics: `mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`, `gli`, and the
  combined forms the first five make with the last six (`me lo` → `melo`, `te ne` → `tene`,
  `glie` + `lo`, `la`, `li`, `le`, `ne`). They attach to an infinitive, which drops its final `-e`, or its `-re` after an `-r`
  (`parlarmi`, `dirmelo`; `porlo` from `porre`), to a gerund (`parlandogli`), to an imperative
  (`parlami`, `prendetelo`), to `ecco` (`eccolo`), and to a past participle in literary use
  (`andatosene`). Five monosyllabic imperatives double the clitic's first consonant: `da'`, `fa'`,
  `va'`, `di'`, `sta'` give `dammi`, `fallo`, `vattene`, `dimmi`, `stacci` — not before `gli`
  (`digli`). No clitic moves a written accent.
- UD Italian-PUD (1,000 sentences) and MarkIT (CC BY 4.0) are the licence-clean treebanks; the
  gates French's cascade was held to on PUD — 98.5 % of words resolved, 93.5 % of content lemmas,
  97 % of auxiliary lemmas (`measure/fr-ud.sh`; French measured 99.12 / 96.48 / 99.90) — are
  measured with the tables, which change 9 commits.

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
  9 and added at a later bump if the treebanks warrant it. *Superseded at implementation: D2's
  candidate « a past participle base » was measured here (D6) and kept — see* Measured at
  implementation.
- A gender or number for a piece: the tables file `lo`, `la`, `li`, `le` under the article's lemma.

## Decisions

### D1 — NFC is a rule of Italian's pre-pass, not of its cascade alone

`nfc_for` answers NFC for Italian, as for Spanish and French: the elided forms, the articulated
prepositions and the truncations are compared composed, a run the pack lists is looked up
composed, and every token's text is composed, spans pointing into the source. The `tecnico` page's
NFD block then resolves to the pack's words; the corpus scenario still asserts the block is
committed decomposed.

### D2 — The cascade: the pack's forms, the enclitic rule, an unlisted plural, the form

*As implemented (see* Measured at implementation*): the pack's forms, the enclitic rule with a
regular past participle among its bases, the form — step 3 below was measured out. At review, the
imperative base was narrowed to what takes clitics (three letters or more, in `-o` only as a
gerund or a first-person plural: « Stone » is no `sto` + `ne`, « Sami » no `sa` + `mi`), the
undoubled monosyllables to `gli` and `glie-` alone (`falo`, `dami` read nothing), and the
participle base refused before a lone `ne` (`risultatone`).*

`analysis/italian.rs`, beside `spanish.rs` and `french.rs`; `lemmatize` dispatches to it. For each
token, lowercased (`’` read as `'`) and in NFC:

1. **the pack's forms** — the one lemma the forms tables chose (M8);
2. **the enclitic rule** — only when the whole form is unlisted. The form's end is matched against
   the clitics, longest first: the combined forms (`glielo`, `gliela`, `glieli`, `gliele`,
   `gliene`; `melo`, `mela`, `meli`, `mele`, `mene`, and likewise with `te`, `se`, `ce`, `ve`), then
   the singles (`gli`, `mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`). The rest is
   accepted as a base when the lexicon lists it shaped to take clitics:
   - an infinitive with its ending restored: a rest ending in `-r` is tried with `-e` restored
     (`parlar` → `parlare`, `dir` → `dire`) and, when the lexicon does not list that, with `-re`
     (`por` → `porre`, `condur` → `condurre`); the lemma is the base's;
   - a gerund: a rest ending in `-ando` or `-endo` the lexicon lists;
   - an imperative: a rest the lexicon files under a verb — a lemma ending in `-are`, `-ere`,
     `-ire` or `-rre` — (`parla`, `scrivi`, `prendete`, `andiamo`);
   - a monosyllabic imperative: a rest of `da`, `fa`, `va`, `di` or `sta` when the clitic's first
     consonant was doubled (`dammi` = `da` + `mmi`), the doubling undone and the base read as
     `dare`, `fare`, `andare`, `dire`, `stare` by a table of five — the lexicon lists `da` and `di`
     as prepositions, so the table, not the lexicon, names the verb; `digli`, `fagli`, `stagli` and `vagli`, without
     doubling, go the same way — `dagli` never reaches the cascade: change 6's pre-pass splits it
     as the articulated preposition `da` + `gli`, the imperative reading being the cost change 6
     named for `dai`;
   - `ecco`.
   Spanish's stress check has no counterpart: Italian spelling moves no accent when a clitic
   attaches. One split only: `dirmelo` is `dir` + `melo`, never `dirme` + `lo`.
3. **an unlisted plural** — only for a form written in lowercase, of four letters or more, that the
   pack does not hold and whose singular it does not hold either: `-chi` → `-co`, `-ghi` → `-go`,
   any other `-i` → `-o`; `-che` → `-ca`, `-ghe` → `-ga` (the plural inserts the `h`: `amica` →
   `amiche`, `bottega` → `botteghe`), any other `-e` → `-a`. Left alone: an elided or hyphenated word, and a form ending
   in `-ai`, `-ei` or `-ii` (verb endings);
4. **the form itself**.

The order is the determinism contract. **The tables decide**: step 2 never splits a listed word
(`cinema`, `sistema`, `vela` are not `cine` + `ma` or `ve` + `la` — they are listed), and step 3
never reads a form as a word the pack lists: `vite` (vines; lives) stays itself when the pack
lists `vita` — the rule would rename it, and the tables' choice stands.

*Measured at implementation, kept or rejected by the treebanks' figures (D6):* step 3's `-i` →
`-o` and `-e` → `-a` against the `-e` singulars (`cane` → `cani`, which `-i` → `-o` reads as `cano`;
`megalite` → `megaliti`, read as `megalito`), a past participle base, `-ci`/`-gi` plurals (`amici` → `amico`, `baci` → `bacio`:
the rule reads `baci` as `baco`; the tables list both). Each that moves tokens for the worse is
dropped from the cascade and named in `italian.rs`'s doc, as French's rejected rules are.

*Rejected — Spanish's accent retry.* Italian has no retired accented spellings; its accent
problem is the apostrophe (`perche'`), change 10's.

### D3 — Italian's closed classes

*As implemented (see* Measured at implementation*): `prima` and `salvo` are left out of the
prepositions by their counts, and `fino`, `nonostante`, `tramite`, `malgrado`, `eccetto` added to
them; `dove` joins the conjunctions, `qualsiasi` and `qualunque` the determiners, `colui`, `colei`,
`coloro`, `costui`, `costei`, `costoro` the pronouns; of the combined clitics, the pronouns hold
`glielo` … `gliene` alone, the others being nouns standing alone (`mela`, `vela`); `ad`, `ce`, `ve`
and the tables' inflected forms are added.*

Six sorted tables, flagged by the cascade's lemma: determiners (the articles `il`, `lo`, `la`,
`i`, `gli`, `le`, `un`, `uno`, `una`; the demonstratives `questo`, `quello`, `codesto`; the
possessives `mio`, `tuo`, `suo`, `nostro`, `vostro`, `loro`; `ogni`, `qualche`, `alcuno`,
`nessuno`, `ciascuno`, `tale`, `altro`, `stesso`; and the `di` family, `del`, `dello`, `della`,
`dei`, `degli`, `delle`, which the pre-pass keeps whole), pronouns (`io`, `tu`, `lui`, `lei`,
`egli`, `ella`, `esso`, `noi`, `voi`, `loro`, `me`, `te`, `sé`, the clitics and their combined
forms, `che`, `cui`, `chi`, `quale`, `ciò`, `qualcosa`, `qualcuno`, `ognuno`, `chiunque`,
`niente`, `nulla`), prepositions (`di`, `a`, `da`, `in`, `con`, `su`, `per`, `tra`, `fra`,
`senza`, `verso`, `contro`, `sopra`, `sotto`, `dentro`, `fuori`, `dopo`, `prima`, `durante`,
`tranne`, `presso`, `oltre`, `attraverso`, `mediante`, `circa`, `entro`),
conjunctions (`e`, `ed`, `o`, `od`, `ma`, `però`, `anzi`, `né`, `che`, `se`, `perché`, `poiché`,
`siccome`, `quando`, `mentre`, `come`, `dunque`, `quindi`, `perciò`, `infatti`, `cioè`, `sebbene`,
`benché`, `affinché`, `finché`, `purché`, `oppure`, `ovvero`, `tuttavia`, `eppure`), auxiliaries and
modals (`essere`, `avere`, `potere`, `dovere`), negation (`non`, `no`, `né`). The determiners
hold the quantifiers `molto`, `tanto`, `poco`, `tutto`, `troppo`, as Spanish's hold `mucho`,
`poco`, `tanto`, `todo` and French's `beaucoup`, `peu`, `tant`, `tout`. Left out, as words that
carry meaning: `volere` (a content verb, as French leaves `vouloir` and Spanish `querer` out),
`secondo` and `lungo` (everyday adjectives — « il secondo giorno », « un lungo viaggio » — which a
preposition's row would hide in every gloss, as French measured `certain` out), `mai`, `più`,
`neanche`, `nemmeno`, `neppure`, `anche`, `ancora`, and `stare` (the progressive auxiliary is also *to stay*, as
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
measures nothing). Until then, the candidate rules of D2 are measured on PUD and on MarkIT with a
pack built from a prototype of change 9's forms tables, as change 41 measured its cascade on
change 43's prototype tables: a lexicon of lemmas alone cannot measure this cascade — step 3's
guard would block every right plural (`libri` stays, `libro` being a lemma) and let only the
misreads fire, and a gerund or an imperative base is no lemma, so only the infinitive shape and
`ecco` would be exercised. The prototype's figures and the rules kept or dropped are recorded in
the pull request and in `italian.rs`'s doc; `prima` (preposition or adverb) and `stare` (D3) are
counted on MarkIT there. Where no prototype of change 9's tables is at hand, the candidate rules
are decided at change 9, and this change ships the enclitic rule and `-chi`/`-ghi`/`-i` → `-o`
alone.

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
  verbs so the rule has bases to accept; change 9's forms tables do the same for real, and replace the
  fixture in the golden at change 16.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian. Rollback is a revert
with the golden's.

## Measured at implementation

Task 8.1 built a prototype of change 9's forms tables from local copies (the English Wiktionary's
Italian section as kaikki.org extracts it; MarkIT's training and development sections at commit
`e210079d` for D5's counts; wordfreq 3.1.1's `it` for the ranks): 58,113 lemmas, 296,298 forms,
D3–D6 of change 9's design followed, without Morph-it! and with its names rule approximated. A
pack built from it was read by this change's cascade over UD Italian-PUD (`71d932aa`) and MarkIT's
test section, token by token with and without each candidate rule. **What « better » means was
written before any figure was read**: a token moves for the better when it takes the treebank's
lemma with the rule and not without, for the worse the other way, for nothing when it moves and is
wrong both ways; a rule is kept when it is right on a strict majority of the tokens it moves, on
the two treebanks pooled, and none of the three figures (resolved, content, auxiliary) falls; a
rule of D2's base cascade that moves nothing stays, a candidate addition that moves nothing is
not added.

| Rule | Better | Worse | For nothing | Decision |
|---|---|---|---|---|
| Step 3: `-i` → `-o` (with `-chi`, `-ghi`) | 1 | 6 | 3 | dropped; PUD content 90.43 → 90.36 % |
| Step 3: `-ci`/`-gi` read through `-i` → `-o` | 0 | 0 | 0 | dropped with the step it is a case of |
| Step 3: `-e` → `-a` (with `-che`, `-ghe`) | 0 | 14 | 7 | dropped; PUD content 90.49 → 90.36 % |
| Enclitic rule: a regular past participle base | 3 | 0 | 0 | kept (`Formatasi`, `sviluppatasi` ×2: 3 tokens, 2 word types, all with `si`) |
| Enclitic rule on the 133 verb + clitic words written as one (reported) | 91 | 0 | 1 | — |
| Enclitic rule on every other measured word it moves (reported) | 17 | 0 | 14 | — |

The unlisted-plural step goes whole: the tables list the regular plurals, so what is left unlisted
is mostly the plural of an `-e` noun or adjective (`thailandesi`, `viventi`), a loan, a compound
or a typo, which the rule misreads (`thailandeso`, `famigliara`, `folklora`). Its `-ci`/`-gi` case
moved nothing; dropping it with the step it is a case of departs from the written criterion, which
keeps a base rule that moves nothing. The participle base is kept on a thin sample — 3 tokens, 2
word types, every one with `si` —, which its figures say and no more. The enclitic rule's other
moves, on the words the treebanks write whole, are a plural participle or adjective the prototype
does not list read as an imperative and `ti`: right where the treebank has the verb (`inviati` →
`inviare`), for nothing where it has the noun or the adjective (`certificati` → `certificare`,
`impiegati` → `impiegare`, `porti` → `porre`, `fatti` → `fare`). The enclitic rule is
measured on the verb + clitic words the treebanks write as one and split (`dirmelo` = `dir` +
`me` + `lo`): 133 such words, 127 taking the treebank's verb with the decided cascade (the six
others: `riaccaparrarselo`, whose verb the prototype does not list; `congratularmi`, read as the
reflexive `congratularsi` the prototype files `congratulare` under; `andarsene` and `trovarci`,
which it lists whole; `evolutesi` and `Svoltesi`, irregular participles). With it, the
prototype gives PUD 98.49 % resolved, 90.56 % content, 93.81 % auxiliary lemmas and MarkIT's test
section 98.02 / 94.19 / 96.98 — a prototype's figures, under the gates change 9's tables are held
to.

D3's two counts, decided by rules written before counting: `stare` is an auxiliary in 13 of
MarkIT's 24, short of the two in three that would have put it in — it stays out, although pooled
with PUD (23 of 28) it is an auxiliary in 36 of 52, 69 %, above two thirds: the criterion named
MarkIT, and the owner may move it; `prima` is a preposition in none of MarkIT's 14 (8 adverbs,
« prima di » among them, 5 adjectives, « la prima volta »; PUD: none of 35) — it leaves D3's
prepositions; read as introducing a phrase (UPOS ADP/SCONJ/CCONJ, deprel `case`, `mark`, `cc`,
or a `fixed` « prima di »), it does so in 5 of MarkIT's 14 and 19 of 49 pooled, out either way.
Reported for the owner (no rule set before): `secondo` is ADP in 17 of its 22 occurrences
pooled, `circa` an adverb in 15 of 19, `oltre` in 12 of 15, `fuori` in 8 of 10.

At review, the closed-class words French or Spanish list but D3 left out without a reason were
decided by the same two thirds of MarkIT's occurrences, the function reading written before
counting (a preposition introducing a phrase as above; a determiner or pronoun as DET or PRON;
`dove` as ADV, SCONJ or PRON); with no MarkIT occurrence, the pooled figure; with none in either,
the dictionary's parts of speech, a word with no content one added:

| Word | MarkIT | PUD + MarkIT | Verdict |
|---|---|---|---|
| `dove` | 6 / 6 | 24 / 24 | added (conjunctions) |
| `fino` | 5 / 5 | 27 / 27 | added (prepositions); « fine, thin » never written |
| `nonostante` | 3 / 3 | 9 / 9 | added (prepositions) |
| `qualsiasi` | 2 / 2 | 4 / 4 | added (determiners) |
| `qualunque` | 1 / 1 | 2 / 3 | added (determiners); the other, « una persona qualunque » |
| `colui` | 1 / 1 | 1 / 1 | added (pronouns) |
| `coloro` | 1 / 1 | 2 / 3 | added (pronouns) |
| `tramite` | — | 6 / 6 | added (prepositions), by the pooled figure |
| `malgrado` | — | 1 / 1 | added (prepositions), by the pooled figure |
| `eccetto` | — | — | added (prepositions): a preposition alone in the dictionary |
| `colei`, `costui`, `costei`, `costoro` | — | — | added (pronouns): pronouns alone in the dictionary |
| `salvo` | 0 / 1 | 0 / 1 | left out: its one occurrence is the adjective « safe » |

**Other deviations, each a fact of the implementation:**

- *D2*: a verb is a lemma in `-are`, `-ere`, `-ire`, `-rre` or `-rsi` — the last for a
  reflexive-only verb, under which change 9's D3 files `pentite` and `pentendo` so that this rule
  reads `pentitevi`; and a form holding an apostrophe or a hyphen is never split, its pieces being
  the pre-pass's. A known cost: told by its ending, a noun or an adjective in `-are`, `-ere`,
  `-ire` passes as a verb (`mare`, `popolare`, `carattere`: an unlisted `marlo` reads as `mare`);
  telling them apart needs a part of speech the pack does not carry.
- *D2, at review*: the guards above (an imperative base of three letters or more, in `-o` only as
  `-ndo` or `-iamo`; the undoubled monosyllables before `gli` alone; no participle base before a
  lone `ne`) move no word of either treebank; they keep names (« Stone », « Sami », set aside as
  names at `0.2.0`) and augmentatives (`risultatone`, `vestitone`, `peccatone`, `gelatone`) from
  being read as verbs.
- *D3*: the pronouns hold the combined clitics written as one word before a verb, `glielo` …
  `gliene`, and not `melo` … `vene`, written apart before a verb and, standing alone, nouns
  (`mela`, `vela`, `tela`, `velo`, `cene`) a row would hide in every gloss; they hold `ce` and `ve`,
  the clitics' variants of « ce l'ho », « ve lo dico »; the prepositions hold `ad`, as the
  conjunctions `ed` and `od`; each table holds the inflected forms a pack may keep as lemmas
  (`questa`, `nessun`, `tutti`), as Spanish's and French's do.
- *The spec's scenarios*: *Each language keeps its own tables* reads `di`, not `il`, as English,
  Spanish and French — `il` is French's « he »; the names scenarios give the name a lemma of the
  pack that is no dictionary word, as French's do — a pack that does not list it sets it aside by
  the rule that belongs to no language, which would set « Ecco » aside too; *An unlisted plural*
  reads the plural as written.
- *D7*: the it-fr fixture's whole pack moves from 5,841 to 5,956 B (sha256 `bcc19522…`), which
  `lingua-pack`'s `pipeline_testdata.rs` records; no `pin.json` moves.

