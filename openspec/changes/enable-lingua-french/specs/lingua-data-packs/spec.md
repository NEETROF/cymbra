## MODIFIED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair glossed in a native language gives that native language's default studied language, and SHALL build each listed pair's pack from that pair's committed tables, checked against that pair's own recorded sha256. It SHALL refuse a pack whose analyser version is not the version of the pack's own studied language. Every package of a release SHALL carry exactly the listed packs, each byte-identical across Chromium, Firefox and Safari. The list SHALL hold en-fr, es-fr, es-en, en-es then fr-en, followed by fr-es once fr-es's tables are committed at or above its coverage floor, and a package whose list is anything else SHALL be refused. A listed pair held to a coverage floor SHALL measure at or above it on its committed tables.

#### Scenario: Building the listed packs
- **WHEN** a release builds its packs with the list holding en-fr, es-fr, es-en, en-es, fr-en then fr-es
- **THEN** it builds each pack from its own tables, each sha256 is the one that pair's pin records, and the package carries those packs and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pack's analyser version differs from the core's version for that pack's studied language
- **THEN** the build fails with a message naming the pack, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list other than en-fr, es-fr, es-en, en-es then fr-en, with fr-es after them or not as its floor allows
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack is built from its tables after this change
- **THEN** it is byte-for-byte the pack built before, and English stays a French-native reader's default studied language

#### Scenario: English speakers learning Spanish
- **WHEN** a reader whose browser is in English installs a package built with this list
- **THEN** the native-language choice is offered, preset to English, and the reader studies Spanish through es-en

#### Scenario: Spanish speakers learning English
- **WHEN** a reader whose browser is in Spanish installs a package built with this list
- **THEN** the native-language choice is offered, preset to Spanish, and the reader studies English through en-es

#### Scenario: English speakers learning French
- **WHEN** a reader whose native language is English uses a package built with this list
- **THEN** Spanish stays their default studied language, French is offered beside it, and a French page is read through fr-en

#### Scenario: Spanish speakers learning French
- **WHEN** fr-es is listed and a reader whose native language is Spanish uses a package built with this list
- **THEN** English stays their default studied language, French is offered beside it, and a French page is read through fr-es

#### Scenario: French for English speakers alone
- **WHEN** fr-es's first committed measurement fell under its floor, so that no fr-es table is committed
- **THEN** the list ends with fr-en, no package carries fr-es, and a reader whose native language is Spanish is offered English alone

#### Scenario: No French for French speakers
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** their pairs are en-fr and es-fr as before, French is never offered, and neither French pack is fetched

#### Scenario: A listed pair under its floor
- **WHEN** the list names a pair held to a coverage floor whose committed tables measure under it at one of the three tops
- **THEN** the coverage tests fail, naming the pair, the top and the figure

### Requirement: French's forms and frequencies
French's studied tables SHALL be reduced by its reference pair, fr-en, from the English Wiktionary's French section as derived from the English edition's dump, and committed once, in `tables/fr/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, nor from a gender or number marker an entry lists among its forms, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech, the chain following that part of speech past a lemma of another part of speech the candidate is spelt like. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in UD French-GSD's training and development sections, then by the form's own entry, then by the lemma's frequency, then alphabetically; a form that is a name and another word's SHALL keep the commoner reading. The tables SHALL serve French's tokenisation: every word the French pre-pass writes for an elided piece, a split contraction or an inverted pronoun SHALL be a form; each elided piece — `l'`, `d'`, `j'`, `m'`, `t'`, `s'`, `n'`, `c'`, `ç'`, `qu'`, `jusqu'`, `lorsqu'`, `puisqu'`, `quoiqu'` — SHALL itself be a form of the word the pre-pass reads it as outside its special cases, by a reviewed table; no word without a hyphen that begins with an elided piece SHALL be a form; a hyphenated noun, adjective, adverb, pronoun or preposition of the dictionary whose last piece is a pronoun SHALL be a form, so that the inversion rule never splits it; `au` and `aux` SHALL be neither a form nor a ranked lemma; `du` and `des` SHALL each be a lemma of its own; and a verb form joined to clitic pronouns by hyphens SHALL be no form. A word the dictionary gives only as another word's ASCII spelling of a ligature or post-1990 spelling SHALL be a form of that word. The tables SHALL keep the 60,000 commonest French lemmas by wordfreq — none of wordfreq's bare elision stems, and a hyphenated word only when GSD's training sections attest it, ranked at the lower of wordfreq's estimate and GSD's own frequency, or when it is one of the nouns ending in a pronoun above, ranked last — with their forms attested in wordfreq. Every ranked lemma SHALL be the lemma of its own form, so that a pack finds each rank on its own lemma. `tables/fr/` SHALL name fr-en as its reference, and SHALL hold an empty tag pool and no dictionary word until fr-en's readings and glosses are reduced. fr-en's pack SHALL be built from the committed tables and checked against its pin in the extension's checks, and SHALL be carried by a package only once the list of shipped pairs names it.

#### Scenario: A participle's agreement
- **WHEN** the tables are reduced and `dirigée` is listed only as the feminine of the participle `dirigé`, itself listed only as the past participle of *diriger*
- **THEN** `dirigée` maps to *diriger*

#### Scenario: A participle filed under a noun's spelling
- **WHEN** the tables are reduced, `citée` is listed only as the feminine of the participle `cité`, and `cité` is both the past participle of *citer* and the noun *cité* (city), a ranked lemma
- **THEN** `citée` maps to *citer*, and `cité` itself keeps the noun

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
- **THEN** `le`, `si`, `moi`, `toi`, `jusque`, `à`, `il`, `on` and `vous` are each a form of `tables/fr/forms.tsv`

#### Scenario: A noun ending in a pronoun stays whole
- **WHEN** the tables are reduced
- **THEN** `rendez-vous` and `qu'en-dira-t-on`, nouns of the dictionary, are forms, and `est-il` and `allez-y`, a verb with its pronoun, are not

#### Scenario: A word no form reaches
- **WHEN** the tables are reduced, the first choice keeps the noun *tenue*, and every form of it, `tenue` and `tenues`, maps to *tenir* once its forms of forms are followed
- **THEN** *tenue* is not a ranked lemma, and its rank goes to the next word

#### Scenario: A ranked word's own form reads as itself
- **WHEN** the tables are reduced, `donnée` maps to *donner* by GSD's counts, and `données` alone still reaches the noun *donnée*
- **THEN** *donnée* is not a ranked lemma, and the pack built from the tables holds every rank of `freq.tsv` on that very lemma

#### Scenario: A gender marker is no form
- **WHEN** the tables are reduced and the French section lists `m`, the gender of *Paris*, among its forms
- **THEN** `m` is not a form of *paris*

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
- **THEN** fr-en's pack is built from `tables/fr/` and `tables/fr-en/` and checked against its pinned sha256, and the extension's list of shipped pairs names it

#### Scenario: An update of fr-en
- **WHEN** `lingua-pack-update` reads today's sources for fr-en
- **THEN** it fetches the English edition's dump alone, publishes `kaikki-French.jsonl` under `lingua-pack-sources-fr-en-<snapshot>`, records GSD's two sections at their commit, and keeps no dump

#### Scenario: Nothing else moves
- **WHEN** French's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing

#### Scenario: The French baseline keeps its fixture
- **WHEN** French's tables are committed while fr-en glosses nothing
- **THEN** the French invariance baseline still runs over its fixture pack, and its golden does not move

### Requirement: fr-en is committed at its studied tables' snapshot, and the French baseline runs on it
fr-en's native side SHALL be reduced from the pin French's studied tables were reduced from, fetching no source the pin does not record, and committed beside them, every studied table but the dictionary words byte for byte; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, SHALL stay under the size budget, and SHALL be carried by a package only once the list of shipped pairs names it; and from the pull request that commits fr-en's glosses on, the French invariance baseline SHALL run over the pack built from the committed French tables.
fr-en's folder holds its glosses, sense runs and expressions, its notice and manifest — which credit
the English Wiktionary's French section for the glosses too — its pin and its README; the pin keeps
its snapshot and its sources, and names the new rule digest and pack. The French golden SHALL be
re-blessed once, in that pull request, which changes no French rule and says, probe kind by probe
kind, what moved and why; the hand-written fr-en fixture stays for the tests that build it, its
manifest following French's analyser version. From then on a change to `tables/fr/` or
`tables/fr-en/` that moves the French golden re-blesses it in its own pull request and says so, as a
dictionary update does the other baselines. en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and goldens SHALL
NOT move.

#### Scenario: The same snapshot
- **WHEN** fr-en is reduced from its pin with its native side
- **THEN** its pin keeps its snapshot and its sources, every table of `tables/fr/` but `lexical.tsv` is byte for byte as committed, and `tables/fr-en/` holds `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`

#### Scenario: The pack is built, not shipped
- **WHEN** a pull request runs the extension's checks while the list of shipped pairs does not name fr-en
- **THEN** fr-en's pack is built from `tables/fr/` and `tables/fr-en/`, has the sha256 its pin records and weighs less than 5 MiB, and no package carries it

#### Scenario: The French baseline on the committed pack
- **WHEN** fr-en's glosses are committed
- **THEN** the French invariance baseline builds fr-en's pack from the committed tables, beside es-en's, its golden is re-blessed once in that pull request, its pack line names fr-en's `pack_version`, and its `beside es-en` line does not move

#### Scenario: The French tables move after the hand-over
- **WHEN** a later pull request changes `tables/fr/` or `tables/fr-en/`, and the French baseline's output moves with it
- **THEN** that pull request re-blesses the French golden and names the tables' change as the reason

#### Scenario: The fixture stays
- **WHEN** a test builds the hand-written fr-en fixture after the hand-over
- **THEN** it builds, and loads at French's analyser version

#### Scenario: Nothing else moves
- **WHEN** fr-en's native side is committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines and the extension's snapshots pass without re-blessing

### Requirement: French is glossed in Spanish from the Spanish Wiktionary's French section and the French Wiktionary's translation tables
fr-es's glosses, sense runs and expressions SHALL be reduced from the Spanish Wiktionary's French section with the Spanish edition's rules, then, for what that section leaves out, from the Spanish translations the French Wiktionary's French entries list, in the table's order, then from the Spanish entries of the Spanish Wiktionary whose translations list the French word, the commonest Spanish word first — words people wrote in Spanish, with no pivot and no machine translation —, and fr-es SHALL read French's committed studied tables as a pair that is not its studied language's reference does, loading no other pair's reducer.
Before the shared rules read the section, its letters are left out; its senses are read as the
Spanish edition reads en-es's — the senses it marks obsolete or outdated after the others of their
entry, one typography —; the sense of a capitalised proper noun that only says a word is a surname or
a given name glosses no word that has an entry of its own in lower case, other than a proper noun's,
holding a meaning; and an adjective section the edition tags possessive or demonstrative, or whose
every sense is a form of such a word, is read as a determiner. A typographic apostrophe in a French
headword, or in a French word of either table, is read as `'`. A definition whose every sense is the
French headword itself, as written up to case, SHALL give way to the French Wiktionary's Spanish
translations of the word when they hold no word spelled as the headword. In both tables a letter
SHALL gloss no word, and a Spanish word that several parts of speech of one Spanish entry list for a
French word SHALL be listed once, under the first of them French's readings name, or else the first
listed. A pack built from fr-es's tables carries French's dictionary words, fr-en's glossed lemmas:
a lemma fr-es alone glosses is none. The share of the glossed lemmas a definition glosses — of all
of them, and of the 10,000 commonest — SHALL be measured and shown with the tables, and stored in no
pack. The Spanish edition's rules SHALL be read by en-es and fr-es and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Spanish Wiktionary's French section defines `maison` « Casa. »
- **THEN** the gloss of `maison` is « Casa », as a noun

#### Scenario: A direct-table gloss
- **WHEN** the section has no entry for `intérêt`, and the French Wiktionary's `intérêt` lists « interés » among its Spanish translations
- **THEN** the gloss of `intérêt` is « Interés », at most three Spanish words per part of speech

#### Scenario: An inverted-table gloss
- **WHEN** neither the section nor the French Wiktionary's table glosses `travers`, and the Spanish Wiktionary's `través` lists `travers` as its French translation
- **THEN** the gloss of `travers` is « Través »

#### Scenario: The studied word is no gloss
- **WHEN** the section defines the conjunction `et` « Et. » and the French Wiktionary translates `et` « y » and « e »
- **THEN** the gloss of `et` is « Y, e »

#### Scenario: A cognate keeps its definition
- **WHEN** the section defines `venir` « Venir. » and the French Wiktionary's table lists « venir »
- **THEN** the gloss of `venir` is « Venir »

#### Scenario: A name on a common word's card
- **WHEN** the section has `pierre`, a noun defined « Piedra. », and `Pierre`, a proper noun defined « Nombre de pila de varón, equivalente del español Pedro. »
- **THEN** the gloss of `pierre` is « Piedra »

#### Scenario: A name's own row
- **WHEN** the section has `François`, a proper noun defined « Nombre de pila de varón, equivalente del español Francisco », and no `françois` in lower case
- **THEN** the gloss of `françois` is that note, as before

#### Scenario: Possessives and their forms
- **WHEN** the section has `mon`, an adjective section tagged possessive defined « Mi. », and `mes`, an adjective section whose every sense is a form of `mon`
- **THEN** `mon` and `mes` are glossed « Mi » as determiners

#### Scenario: A letter glosses no word
- **WHEN** French's tables rank `h` and `x` among the 500 commonest lemmas, and the only Spanish word a translation table gives each is the letter itself
- **THEN** `h` and `x` have no gloss in fr-es, and `à` and `y` keep the section's definitions

#### Scenario: A Spanish word listed once
- **WHEN** the Spanish Wiktionary's `este` lists `cet` among its French translations as an adjective and as a pronoun
- **THEN** the gloss of `cet` is « Este », once

#### Scenario: A typographic apostrophe
- **WHEN** the French Wiktionary's `main-d’œuvre` lists « mano de obra » among its Spanish translations
- **THEN** the committed lemma `main-d'œuvre` is glossed « Mano de obra »

#### Scenario: French's dictionary words
- **WHEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`
- **THEN** it carries a lexical table listing exactly the lemmas `tables/fr/lexical.tsv` lists, and a lemma fr-es glosses and fr-en does not (`quant` in the prototype) is no dictionary word

#### Scenario: The pack is built, not shipped
- **WHEN** a pull request runs the extension's checks while the list of shipped pairs does not name fr-es
- **THEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`, has the sha256 its pin records and weighs less than 5 MiB, and no package carries it

#### Scenario: A rule of the Spanish edition
- **WHEN** `reduce_edition_es.py` changes
- **THEN** en-es's and fr-es's rule digests move, and en-fr's, es-fr's, es-en's and fr-en's do not

#### Scenario: Nothing else moves
- **WHEN** fr-es's native side is committed
- **THEN** en-fr's, es-fr's, es-en's, en-es's and fr-en's tables, pins and packs and every table of `tables/fr/` are byte for byte as before, and the invariance baselines and the extension's snapshots pass without re-blessing
