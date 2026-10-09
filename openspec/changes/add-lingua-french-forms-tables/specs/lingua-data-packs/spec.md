## ADDED Requirements

### Requirement: French's forms and frequencies
French's studied tables SHALL be reduced by its reference pair, fr-en, from the English Wiktionary's French section as derived from the English edition's dump, and committed once, in `tables/fr/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in UD French-GSD's training and development sections, then by the form's own entry, then by the lemma's frequency, then alphabetically; a form that is a name and another word's SHALL keep the commoner reading. The tables SHALL serve French's tokenisation: every word the French pre-pass writes for an elided piece, a split contraction or an inverted pronoun SHALL be a form; each elided piece — `l'`, `d'`, `j'`, `m'`, `t'`, `s'`, `n'`, `c'`, `ç'`, `qu'`, `jusqu'`, `lorsqu'`, `puisqu'`, `quoiqu'` — SHALL itself be a form of the first word the pre-pass reads it as, by a reviewed table; no word without a hyphen that begins with an elided piece SHALL be a form; a hyphenated noun, adjective, adverb, pronoun or preposition of the dictionary whose last piece is a pronoun SHALL be a form, so that the inversion rule never splits it; `au` and `aux` SHALL be neither a form nor a ranked lemma; `du` and `des` SHALL each be a lemma of its own; and a verb form joined to clitic pronouns by hyphens SHALL be no form. A word the dictionary gives only as another word's ASCII spelling of a ligature or post-1990 spelling SHALL be a form of that word. The tables SHALL keep the 60,000 commonest French lemmas by wordfreq — none of wordfreq's bare elision stems, and a hyphenated word only when GSD's training sections attest it, ranked at the lower of wordfreq's estimate and GSD's own frequency — with their forms attested in wordfreq. `tables/fr/` SHALL name fr-en as its reference, and SHALL hold an empty tag pool and no dictionary word until fr-en's readings and glosses are reduced. fr-en's pack SHALL be built from the committed tables and checked against its pin in the extension's checks, and no package SHALL list it.

#### Scenario: A participle's agreement
- **WHEN** the tables are reduced and `dirigée` is listed only as the feminine of the participle `dirigé`, itself listed only as the past participle of *diriger*
- **THEN** `dirigée` maps to *diriger*

#### Scenario: A noun's plural is not its homograph's verb
- **WHEN** the tables are reduced, `étés` is listed only as the plural of the noun `été`, and `été` maps to *être*
- **THEN** `étés` is not in the forms table

#### Scenario: A noun and a verb share a form
- **WHEN** the tables are reduced and GSD counts `porte` 39 times under *porter* and 23 times under the noun *porte*, with no override for it
- **THEN** `porte` maps to *porter*, and *porte* is no lemma of the tables

#### Scenario: The elided pieces
- **WHEN** the tables are reduced
- **THEN** `l'` maps to *le*, `qu'` to *que*, `s'` to *se* and `jusqu'` to *jusque*, `c'est`, `d'abord`, `l'on` and `jusqu'à` are not in the forms table, and the hyphenated `c'est-à-dire`, which GSD attests, is

#### Scenario: The contracted articles
- **WHEN** the tables are reduced
- **THEN** `au` and `aux` are neither forms nor ranked lemmas, `à`, `le` and `les` are forms, and `du` and `des` map to themselves

#### Scenario: Every word the pre-pass writes is a form
- **WHEN** the French pre-pass reads an elided piece as `le`, `si`, `moi`, `toi` or `jusque`, splits `au` into `à` and `le`, or splits an inversion into its verb and `il`, `on` or `vous`
- **THEN** each word it writes is a form of `tables/fr/forms.tsv`

#### Scenario: A noun ending in a pronoun stays whole
- **WHEN** the tables are reduced
- **THEN** `rendez-vous` and `qu'en-dira-t-on`, nouns of the dictionary, are forms, and `est-il` and `allez-y`, a verb with its pronoun, are not

#### Scenario: A hyphenated word by evidence
- **WHEN** the tables are reduced
- **THEN** `peut-être`, which GSD attests, is a ranked lemma, and the inversion `est-il`, which it does not, is neither a ranked lemma nor a form

#### Scenario: wordfreq's elision stems are no words
- **WHEN** the tables are reduced
- **THEN** `l`, `d` and `qu`, which wordfreq counts for `l'`, `d'` and `qu'`, are not ranked

#### Scenario: A spelling variant
- **WHEN** the tables are reduced
- **THEN** `coeur` maps to *cœur* and `connait` to *connaître*

#### Scenario: The reference pair writes French's folder
- **WHEN** fr-en is reduced again from its pinned sources
- **THEN** it writes `tables/fr/` and `tables/fr-en/` byte for byte as committed and leaves its pin unchanged, `tables/fr/studied.json` names fr-en, `tables/fr/tags.tsv` and `tables/fr/lexical.tsv` are empty, and `tables/fr-en/gloss.tsv` is empty

#### Scenario: The pack builds where the others' do
- **WHEN** a pull request runs the extension's checks
- **THEN** fr-en's pack is built from `tables/fr/` and `tables/fr-en/` and checked against its pinned sha256, and the extension's list of shipped pairs does not name it

#### Scenario: An update of fr-en
- **WHEN** `lingua-pack-update` reads today's sources for fr-en
- **THEN** it fetches the English edition's dump alone, publishes `kaikki-French.jsonl` under `lingua-pack-sources-fr-en-<snapshot>`, records GSD's two sections at their commit, and keeps no dump

#### Scenario: Nothing else moves
- **WHEN** French's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing

#### Scenario: The French baseline keeps its fixture
- **WHEN** French's tables are committed while fr-en glosses nothing
- **THEN** the French invariance baseline still runs over its fixture pack, and its golden does not move

### Requirement: French forms are measured on held-out treebanks
The pipeline SHALL measure the fr-en pack, built from the committed tables, with the real analyser on UD French-PUD and on UD French-GSD's test section — neither of which the reduction reads —, each fetched at a pinned commit, checked by sha256, and never committed. The measurement SHALL count punctuation, numbers, symbols, foreign words and proper nouns out; it SHALL fail when fewer than 98.5 % of PUD's words resolve in the lexicon, fewer than 93.5 % of its content words (nouns, verbs, adjectives, adverbs) take PUD's lemma, or fewer than 97 % of its auxiliaries do; and it SHALL report GSD's test-section figures beside PUD's without failing on them.

#### Scenario: The committed tables pass the gates
- **WHEN** the harness runs over UD French-PUD with the pack built from the committed tables
- **THEN** it reports at least 98.5 % of words resolved, 93.5 % of content lemmas and 97 % of auxiliaries, and succeeds

#### Scenario: GSD's test section is reported
- **WHEN** the harness runs
- **THEN** it prints GSD's test-section figures beside PUD's, and they do not decide its exit status

#### Scenario: The reduction never reads them
- **WHEN** fr-en is reduced
- **THEN** neither treebank file is among its inputs, and its pin records GSD's training and development sections alone
