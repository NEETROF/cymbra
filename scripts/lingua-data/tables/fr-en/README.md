# The fr→en dictionary tables

The reduced tables of Cymbra Lingua's French→English data pack, committed so that every build makes
the same pack with no download, and so that a change to the dictionary is a pull request whose diff
shows it (add-lingua-french-forms-tables, change 43 of `docs/lingua/language-matrix-programme.md`).
No extension package carries the pack: `packs.json` does not list it, and no reader holds a French
pack before `enable-lingua-french` (change 52).

They are in two folders (split-lingua-pack-tables-by-language). fr-en is French's reference pair
(`../fr/studied.json`): its reduction writes French's own tables in `../fr/`, which every pair
studying French reads as committed (fr-es, change 49), and this folder's `pin.json` records the
sources of both. fr-en reduces French's forms, ranks, readings and estimated levels, and its own
native side — the English glosses of French words and expressions (add-lingua-pack-fr-en, change 48,
*The glosses* below) — from the same section; its glossed lemmas, less those it glosses by a proper
noun's senses alone, are French's dictionary words (refine-lingua-fr-en-glosses).

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → English gloss: up to eight whole senses, grouped by part of speech (*The glosses*) | kaikki.org, the English Wiktionary's French section (CC BY-SA 4.0 + GFDL), read by the English edition's rules |
| `senses.tsv` | lemma → its gloss's runs: each part of speech and how many senses it holds | the same entries |
| `mwe.tsv` | expression → English gloss: the section's headwords with a space, and the single words French's tokenisation splits (`d'abord`) | the same section |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata: French glossed in English, French's analyser version (`FRENCH_ANALYZER_VERSION`, read from lingua-core), `levels_estimated` (the levels are estimated, not a CEFR list's), and `pack_version` (the snapshot, and the rules that reduced it) | — |
| `pin.json` | the raw sources these tables and `../fr/` came from, and the pack they build | — |

In `../fr/`, French's tables, written by fr-en's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org, the English Wiktionary's French section (CC BY-SA 4.0 + GFDL), with UD French-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0); a hyphenated word by GSD's own frequency too |
| `grammar.tsv` | form → its readings, as Universal Dependencies tags (`form<TAB>lemma<TAB>tag<TAB>other\|-`) | kaikki.org, the same section (add-lingua-french-grammar-tables, change 45) |
| `level.tsv` | lemma → estimated CEFR level (*The levels* below) | derived from `freq.tsv`, the English Wiktionary's French section and French's dictionary words (no source of its own) |
| `lexical.tsv` | French's dictionary words: the lemmas fr-en glosses less those it glosses by a proper noun's senses alone (26,486; *The glosses*) | derived from `gloss.tsv` and `senses.tsv` by `reduce-fr-en.py` (`dictionary_words`), filed by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | French's pinned tag pool: the 79 tags its readings carry, in byte order; written by no reducer | committed by hand, once, from the first reduction's readings |
| `studied.json` | the pair whose reduction writes `../fr/`: fr-en | committed by hand |

## The sources

- **kaikki.org, the English Wiktionary's French section** (`kaikki-French.jsonl`, 403,269 entries,
  510,058,226 B), derived from the English edition's dump (`pack_sources.py EDITIONS`, `DUMPS["fr-en"]`):
  the dump regenerated on 2026-10-03 08:24, read on 2026-10-09, decompressed sha256 `93b79aac…`.
  Its inflections are tagged, a lemma's table lists them and a form's own entry points at what it is
  a form of. A verb's table lists the masculine singular past participle (`dirigé`) and none of its
  agreed forms: those hang under the participle's own entry. Its senses, written in English for
  French words, are fr-en's glosses too: the one source of the native side, no translation table.
- **UD French-GSD**, its training and development sections at `94d5b68e185fc22a9ef292040e84f476d36d9b0e`
  (`pack_sources.py PINNED["fr-en"]`), read for how often a form stands for each lemma and how often
  a hyphenated lemma occurs (390,368 words) — and, for the glosses, how often it reads each word
  under each part of speech (refine-lingua-fr-en-glosses D5). Its test section is never read: the
  measurement holds it out.
- **wordfreq `fr` 3.1.1**, pinned by version and by `../../requirements-reduce.txt`'s hashes.

## The rules (`reduce-fr-en.py`)

- **A form** is a French word, lowercased, in NFC, its typographic apostrophe read as `'`: letters,
  words joined by hyphens or an inner apostrophe (`aujourd'hui`, `presqu'île`), and the elided
  pieces' final apostrophe. Its candidates are the inflections a lemma's entry lists — never kaikki's
  bookkeeping, a multi-word construction (`avoir dirigé`, and `ayant`, which kaikki tags as one), nor
  an inflection tagged alternative, obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard,
  proscribed, abbreviated, clipped or a pronunciation spelling, nor a gender or number marker a head
  left among the forms (`m` under *Paris*, which read « M. » as Paris; 7 rows in the section) — and
  the first word of each form-of target of its own entry (`bel` → *beau*).
- **A form of a form, along one part of speech.** A candidate is followed along the part of speech
  of the entry linking the form to it: along it, a word reads as the first choice among itself —
  only when one of its lemma entries is of that part of speech — and the words its entries of that
  part of speech link it to, never a word whose entries are all of other parts of speech; a
  candidate it reads as another word stands for that word. `dirigée`, the feminine of the
  participle `dirigé`, reaches *diriger*; `citée`, the feminine of the participle `cité`, reaches
  *citer*, not the noun *cité* (the city) the participle is spelt like; `tues` reaches *tuer* and
  *taire* (through the participle `tu`), not the pronoun; `quise` reaches *quérir* through `quis`,
  whose verb entry's « masculine plural of qui » is no verb; `étés`, the plural of the noun `été`,
  does not follow `été` to *être* through a verb entry, and stays out — an unknown word rather than
  the verb. Without the rule the tables fail the resolution gate (97.61 % on PUD). The first
  implementation followed a candidate's overall first choice, which stopped at the noun: 157 forms
  only verb entries link read as a noun, a name or a pronoun (`citée` → *cité*, `tues` → *tu*,
  `marchée` → *marché*); one remains (`shaka` → *shaker*, whose verb has no entry in the section).
- **French's tokenisation (M21)**, as add-lingua-french-tokenisation's pre-pass reads it:
  - every word the pre-pass writes is a form: the words the elided pieces stand for, `à`, `le`,
    `les` and the inversion's pronouns (32 words; the rarest, `quoique`, is rank 3,319);
  - the fourteen elided pieces are forms of the word the pre-pass reads them as outside its special
    cases, by a reviewed table (`ELISIONS`: `l'` → *le*, `s'` → *se*, `qu'` → *que*, `jusqu'` →
    *jusque*, …): the treebanks write them as words, and the dictionary cannot decide them;
  - no plain word beginning with a piece is a form (`c'est`, `d'abord`, `l'on`, `jusqu'à`): the
    pre-pass splits it whatever the pack lists. A hyphenated run is read whole first, so it may be one
    (`c'est-à-dire`, rank 460);
  - the dictionary's hyphenated nouns, adjectives, adverbs, pronouns and prepositions ending in a
    pronoun are listed whole, or the inversion rule would split them: `rendez-vous` (rank 1,586),
    and `qu'en-dira-t-on`, `malgré-nous` and `non-moi`, which GSD never meets, at the cut's last
    ranks. A reduction that leaves one out fails. The verbs, phrases and interjections made of a verb
    and its pronouns (`est-il`, `a-t-il`, `allez-y`, `excusez-moi`) stay out: the split reads them;
  - `au` and `aux` are neither forms nor ranks: the pre-pass always splits them;
  - `du` and `des` are words of their own (ranks 10 and 6), `des` against GSD's counts, which read
    it as *un* 1,730 times out of 1,736;
  - a verb form joined to its pronouns by hyphens (`souviens-toi`, `allons-nous-en`, `sois-t'en`) is
    no form: 459 of them, which the inversion rule reads as the verb and its pronouns.
- **A spelling variant reads as the word it spells**: an entry whose every sense is an ASCII
  spelling of a ligature (`coeur` → *cœur*) or a post-1990 spelling (`connait` → *connaître*,
  `évènement`, `chaine`), with its own inflections (`coeurs` → *cœur*). A word of its own as well
  (`clef`) keeps its entry.
- **One lemma per form (M8).** A form that is a proper name and a commoner word's keeps the word
  (`cette` → *ce*; `paris` stays the city), then the first rule that decides wins:
  1. `OVERRIDES` in `reduce-fr-en.py`, each row with its reason — no homograph; three rows correct
     the source's copy errors: `fatiguée` (« feminine singular of parlé ») → *fatiguer*, `bridée`
     (a form-of target « female slant ») → *bridé*, `quis` (« masculine plural of qui » in a verb
     entry) → *quérir*. A row holds wherever a chain passes too (`fatiguées` follows `fatiguée`);
  2. GSD's counts of the form under each lemma (`porte` → *porter*, 39 against 23);
  3. the form's own entry;
  4. the commoner lemma;
  5. the alphabet.
- **The ranks**: wordfreq's order over its words that are not only inflected forms, the 60,000 first
  kept. Not ranked:
  - wordfreq's elision stems, which carry the pieces' frequency (`l`, `d`, `qu`, `jusqu`, `ç`, …);
  - a word whose own form reads as another word once the forms of forms are followed: a pack finds a
    lemma by its own form — the builder keys its rank, and later its gloss and level, by looking the
    lemma up as a form —, so such a word would lend its rank to the other. Every form of the noun
    *tenue* reads as *tenir*; `donnée` reads as *donner*, and the noun *donnée*, which `données`
    alone still reached, leaves the pack with it, as M8's nouns do (`données` then reads as *donner*).
    27 words give their rank to the next this way (`tenue`, `allée`, `destinée`, `levée`, `donnée`,
    `venue`, `revenue`, `saisie`, `tranchée`, `bordée`, `retombée`, `subordonnée`, …);
  - a hyphenated word GSD's training sections do not attest. wordfreq splits at the hyphen and
    estimates a compound from its parts, so `est-il` would be French's 17th word. A compound GSD
    attests is ranked at the lower of wordfreq's estimate and GSD's own frequency, after wordfreq's
    words of the same frequency, compounds alphabetically: `lui-même` 277, `celui-ci` 336,
    `peut-être` 941, `au-delà` 1,010, `week-end` 1,672, `après-midi` 1,919, `en-cas` 10,659.
- **The forms**: those of a kept lemma that wordfreq attests, each kept lemma's own form, and the
  elided pieces.

## What is in them

On the 2026-10-09 tables: **60,000 lemmas** and **124,096 forms** (`forms.tsv` 2,257,098 B,
`freq.tsv` 844,897 B). 3,593 forms keep more than one ranked candidate: GSD's counts decide 1,382,
the form's own entry 1,312, frequency or the alphabet 899. 7,156 forms take their lemma through a
form of a form. 474 hyphenated words are ranked — 471 by GSD's evidence and the 3 nouns ending in a
pronoun it never meets — 85 of them among the 5,000 first ranks, and 140 words with an inner
apostrophe (`aujourd'hui` 136, `quelqu'un` 204). Every ranked lemma's own form reads as itself
(`crates/lingua-pack/tests/committed_tables.rs` checks every pair's pack holds each rank on its own
lemma).

The pack these two tables built — no gloss, reading or level — was 1,241,733 B; Spanish's same two
tables build 1,308,123 B. With the levels it is 1,302,031 B (*The levels*), and with the readings
too 1,460,253 B (*The readings*). With the glosses, their runs and the expressions it was 2,527,222 B,
and with fr-en's own rules and French's lexical table it is **2,537,587 B** (*The glosses*), under
the builder's 5 MiB.

## Measured

`scripts/lingua-data/measure/fr-ud.sh` builds the pack from these tables and runs the real analyser
(`lingua-pack-measure`, French at `1.0.0`, change 41's cascade) over UD French-PUD at
`db260db10fe728853c549760801229ef4e7b16e1` — gated, with Spanish's thresholds — and over GSD's test
section, reported. The reduction reads neither. Punctuation, numbers, symbols, foreign words and
proper nouns are left out.

| | Words | Resolved | Content words | Auxiliaries |
|---|---|---|---|---|
| UD French-PUD (gate) | 20,232 | **99.13 %** (98.5 %) | **96.41 %** of 9,573 (93.5 %) | **99.90 %** of 1,030 (97 %) |
| GSD test (reported) | 8,049 | 98.89 % | 95.86 % of 3,791 | 99.72 % of 359 |

At `0.2.0`, the pack's lookup alone, the same tables read 99.13 / 96.30 / 99.90 on PUD and 98.89 /
95.70 / 99.72 on GSD's test section: change 41's cascade (an unlisted plural read as its singular)
adds 0.11 and 0.16 points of content lemmas. The figures below are the lookup's, at `0.2.0`.

The design's prototype measured 99.12 / 96.38 / 99.90 and 98.91 / 95.65 / 99.72; the same binary
reads its tables alike at `0.1.0` and `0.2.0`. Two rules of the implementation move them: a ranked
word whose own form reads as another gives its rank (PUD's content words 96.38 → 96.30, `données`
now *donner*; GSD's 95.65 → 95.67), and a gender marker is no form (GSD's resolved words 98.91 →
98.89: its `m` no longer reads as Paris). The chain along one part of speech moves them by a word:
PUD's resolved words 99.12 → 99.13 (`maitrisés`), its content words even — six participles take
PUD's verb (`traitée`, `coupée`, `revenue`), four adjectives and a noun lose PUD's adjective
(`morte` → *mourir*, where PUD writes *mort*), and `entrainé` reads as *entraîner*, the spelling it
stands for, where PUD writes *entrainer* —; GSD's content words 95.67 → 95.70. A participle's agreed
form that is an adjective's too (`morte`, `sacrée`, `ravie`, 506 forms) now reaches the verb beside
the adjective, and the one-lemma rule decides between them: GSD's counts first (`morte` 13 times
*mourir*, once *mort*), else the commoner lemma (`crues` → *croire*, `rangées` → *ranger*). 713
forms change lemma, 98 come in and 52 leave; none is among wordfreq's 1,800 commonest words.

The copy errors were looked for by the forms whose lemma begins with another letter (106): all but
three are a suppletive verb (`sont` → *être*, `va` → *aller*), a ligature's ASCII spelling (`oeuvre`
→ *œuvre*), `eux` → *ils*, `yeux` → *œil*, a pair the dictionary itself links (`marraine` →
*parrain*, `moindre` → *petit*), an Old French form (`fust` → *estre*) or an inclusive pronoun
(`ellui` → *iel*). `fatiguée` and `bridée` are overridden; `créditiste` (Zipf 1.2) reads as
*ralliement*, the first word of its target « Ralliement créditiste », which no row can mend since
*créditiste* is no ranked lemma.

The harness reads UD's words as UD writes them: an elided piece is `l'`, which `ELISIONS` resolves;
an inverted pronoun is `-il` or `-t-il`, which stays unresolved where the pre-pass hands `il` to the
lookup (18 of PUD's words, 17 of GSD's).

The cut, measured on the prototype's tables by the same lookup in Python when the design was
written (the binary reads 0.09 points less on PUD, the inverted pronouns):

| Cut | Forms | `forms.tsv` | PUD resolved | content | GSD test resolved | content |
|---|---|---|---|---|---|---|
| 20,000 lemmas | 62,013 | 1.15 MB | 98.24 % | 95.65 % | 97.70 % | 94.14 % |
| 40,000 lemmas | 97,037 | 1.78 MB | 99.05 % | 96.29 % | 98.93 % | 95.41 % |
| **60,000 lemmas** | **124,040** | **2.26 MB** | **99.21 %** | **96.40 %** | **99.12 %** | **95.67 %** |
| 80,000 lemmas | 147,773 | 2.66 MB | 99.32 % | 96.46 % | 99.23 % | 95.75 % |
| 60,000, every form | 212,718 | 4.15 MB | 99.23 % | 96.44 % | 99.12 % | 95.67 % |

## What one lemma per form costs (M8)

A lemma is keyed by its own form, so a dictionary noun whose form GSD reads as a verb's leaves the
pack: « la porte » shows *porter*'s card. Among wordfreq's 1,000, 2,000 and 5,000 commonest words,
**30, 55 and 135** dictionary nouns read as a verb and are no ranked lemma — Spanish's committed
tables, counted the same way, have 30, 48 and 95 (`cuenta` → *contar*). Among the commonest French
loses what Spanish does; further down it loses more, its past participles being nouns too. An
override row keeps one noun and reads every verb form it shares as the noun; the owner may name
rows, each with its reason, at any reduction. The 135, with their wordfreq rank and GSD's counts
(of the form under the verb / under itself; `données` is the noun *donnée*'s plural, which GSD
counts 40 times under *donnée*):

| Rank | Noun | Read as | GSD |
|---|---|---|---|
| 8 | est | être | 5,150 / 66 |
| 14 | a | avoir | 2,112 / 43 |
| 49 | fait | faire | 428 / 125 |
| 65 | été | être | 830 / 51 |
| 84 | va | aller | 78 / 0 |
| 92 | dit | dire | 99 / 0 |
| 205 | reste | rester | 88 / 37 |
| 249 | demande | demander | 37 / 36 |
| 256 | passe | passer | 53 / 7 |
| 284 | porte | porter | 39 / 23 |
| 318 | passé | passer | 38 / 12 |
| 326 | donne | donner | 62 / 4 |
| 364 | aura | avoir | 32 / 0 |
| 384 | ayant | avoir | 84 / 0 |
| 422 | étant | être | 113 / 0 |
| 500 | écrit | écrire | 66 / 1 |
| 555 | laisse | laisser | 25 / 0 |
| 559 | produit | produire | 48 / 22 |
| 620 | présente | présenter | 57 / 0 |
| 633 | offre | offrir | 23 / 15 |
| 684 | données | donner | 1 / 0 |
| 685 | dû | devoir | 37 / 0 |
| 689 | joue | jouer | 49 / 2 |
| 691 | montre | montrer | 24 / 3 |
| 718 | connu | connaître | 75 / 0 |
| 732 | annonce | annoncer | 22 / 5 |
| 747 | sort | sortir | 17 / 9 |
| 789 | fini | finir | 14 / 3 |
| 813 | reçu | recevoir | 40 / 0 |
| 827 | permis | permettre | 30 / 5 |
| 1,018 | arrêté | arrêter | 15 / 13 |
| 1,042 | rendu | rendre | 24 / 4 |
| 1,055 | change | changer | 10 / 3 |
| 1,076 | suivi | suivre | 34 / 7 |
| 1,097 | signe | signer | 25 / 14 |
| 1,147 | pose | poser | 12 / 5 |
| 1,213 | touche | toucher | 7 / 6 |
| 1,240 | tombe | tomber | 13 / 7 |
| 1,282 | vit | vivre | 23 / 0 |
| 1,411 | envoyé | envoyer | 14 / 1 |
| 1,441 | lance | lancer | 18 / 7 |
| 1,450 | partage | partager | 11 / 10 |
| 1,523 | figure | figurer | 13 / 5 |
| 1,549 | élu | élire | 43 / 2 |
| 1,585 | chargé | charger | 23 / 0 |
| 1,590 | découvert | découvrir | 14 / 1 |
| 1,671 | passant | passer | 17 / 0 |
| 1,686 | tente | tenter | 23 / 1 |
| 1,688 | vivant | vivre | 10 / 7 |
| 1,748 | oublie | oublier | 2 / 0 |
| 1,794 | vécu | vivre | 12 / 0 |
| 1,796 | élève | élever | 14 / 13 |
| 1,859 | conduit | conduire | 15 / 1 |
| 1,952 | estime | estimer | 17 / 4 |
| 1,991 | demeure | demeurer | 18 / 6 |
| 2,019 | tenue | tenir | 9 / 5 |
| 2,034 | disparu | disparaître | 17 / 4 |
| 2,084 | invité | inviter | 7 / 2 |
| 2,113 | blessé | blesser | 11 / 2 |
| 2,166 | commande | commander | 6 / 5 |
| 2,185 | monte | monter | 9 / 2 |
| 2,219 | composé | composer | 30 / 3 |
| 2,336 | réduit | réduire | 8 / 1 |
| 2,346 | vendu | vendre | 18 / 0 |
| 2,383 | remarque | remarquer | 5 / 3 |
| 2,397 | venue | venir | 6 / 3 |
| 2,403 | affiche | afficher | 7 / 5 |
| 2,409 | cache | cacher | 6 / 0 |
| 2,416 | condamné | condamner | 14 / 4 |
| 2,454 | tire | tirer | 5 / 0 |
| 2,464 | cesse | cesser | 7 / 3 |
| 2,478 | frappe | frapper | 9 / 0 |
| 2,514 | donnée | donner | 15 / 4 |
| 2,548 | tiré | tirer | 11 / 0 |
| 2,563 | commis | commettre | 5 / 2 |
| 2,601 | touché | toucher | 7 / 0 |
| 2,605 | acquis | acquérir | 12 / 1 |
| 2,613 | extrait | extraire | 6 / 2 |
| 2,645 | soumis | soumettre | 11 / 0 |
| 2,656 | allant | aller | 17 / 0 |
| 2,713 | employé | employer | 7 / 1 |
| 2,747 | venant | venir | 8 / 0 |
| 2,793 | pousse | pousser | 8 / 0 |
| 2,795 | relève | relever | 3 / 1 |
| 2,805 | établi | établir | 21 / 0 |
| 2,808 | associé | associer | 11 / 2 |
| 2,857 | traite | traiter | 8 / 5 |
| 3,028 | paye | payer | 3 / 0 |
| 3,120 | accusé | accuser | 13 / 0 |
| 3,121 | adopté | adopter | 10 / 0 |
| 3,147 | lis | lire | 1 / 0 |
| 3,149 | lâche | lâcher | 3 / 0 |
| 3,155 | participe | participer | 50 / 0 |
| 3,168 | sachant | savoir | 5 / 0 |
| 3,181 | assuré | assurer | 11 / 0 |
| 3,217 | raté | rater | 1 / 0 |
| 3,236 | applique | appliquer | 6 / 0 |
| 3,261 | frappé | frapper | 5 / 0 |
| 3,309 | abandonné | abandonner | 14 / 0 |
| 3,322 | convaincu | convaincre | 6 / 0 |
| 3,345 | paie | payer | 4 / 0 |
| 3,374 | voyant | voir | 8 / 0 |
| 3,407 | intéressé | intéresser | 4 / 1 |
| 3,458 | gagnant | gagner | 3 / 1 |
| 3,478 | opposé | opposer | 9 / 2 |
| 3,499 | remonte | remonter | 18 / 0 |
| 3,527 | destinée | destiner | 10 / 0 |
| 3,601 | contenant | contenir | 16 / 0 |
| 3,738 | bouge | bouger | 1 / 0 |
| 3,754 | enseigne | enseigner | 6 / 0 |
| 3,766 | jugé | juger | 10 / 0 |
| 3,772 | limitée | limiter | 3 / 0 |
| 3,836 | devoirs | devoir | 2 / 0 |
| 3,917 | finance | financer | 2 / 1 |
| 3,937 | marié | marier | 7 / 0 |
| 3,975 | adapté | adapter | 9 / 3 |
| 3,976 | allée | aller | 3 / 0 |
| 4,089 | poussé | pousser | 2 / 0 |
| 4,180 | restant | rester | 10 / 4 |
| 4,262 | partant | partir | 8 / 0 |
| 4,291 | tenant | tenir | 6 / 2 |
| 4,292 | tombée | tomber | 6 / 0 |
| 4,308 | attaché | attacher | 9 / 0 |
| 4,344 | joint | joindre | 4 / 0 |
| 4,387 | traverse | traverser | 16 / 0 |
| 4,488 | exposé | exposer | 5 / 2 |
| 4,561 | conserve | conserver | 12 / 2 |
| 4,636 | épargne | épargner | 2 / 0 |
| 4,669 | failli | faillir | 3 / 0 |
| 4,727 | aie | avoir | 1 / 0 |
| 4,771 | jeté | jeter | 2 / 0 |
| 4,794 | prévenu | prévenir | 4 / 2 |
| 4,869 | lancée | lancer | 11 / 0 |
| 4,952 | levée | lever | 3 / 2 |
| 4,980 | reproche | reprocher | 4 / 0 |

The rank is wordfreq's own, before the tables skip its stems and inflected forms. Most are a verb
form first in GSD by far; the closest are `demande` (37 / 36), `partage` (11 / 10), `arrêté`
(15 / 13), `élève` (14 / 13), `touche` (7 / 6) and `porte` (39 / 23).

## Function words ranked on their own

Twelve determiners and pronouns are ranked lemmas of their own though another determiner's or
pronoun's table lists them as its feminine or plural: `des` (M21's word of its own), `vous` and
`nous` (listed as `tu`'s and `je`'s plurals), `ils`, `ma`, `mes`, `elles`, `ta`, `tes`, `vôtres`,
`iels`, `celleux`; `cet` is a headword no table points from `ce`. The dictionary gives each an entry
of its own, and GSD's lemmas never name the dictionary's head — it lemmatises every possessive as
*son* (`ma` 54 times, `mon` 66) and `ils` as *lui* —, so the counts cannot decide and the form's own
entry does (D5's third rule); `ma` is also a Louisiana preposition's headword, and two `Ma` proper
nouns in GSD count for it. Spanish's tables merge them (`mis` → *mi*, `ellos` → *él*), English's
keep a pronoun's case forms apart (`them`, `their`). No rule merges them here: a paradigm rule would
read `nous` as *je* and `vous` as *tu*. The rows that would — `ma` → *mon*, `mes` → *mon*, `ta` →
*ton*, `tes` → *ton*, and, as a convention, `ils` → *il*, `elles` → *elle* — are the owner's to name
in `OVERRIDES`, each with its reason (task 5.1); `cet` → *ce* needs a candidate the dictionary does
not give, left to the analysis (change 41) or the grammar (change 45).

## The levels (`../fr/level.tsv`)

**Estimated, not a CEFR list's** (add-lingua-french-levels, the programme's M7). No French CEFR list
can be shipped: FLELex (CEFRLex) is CC BY-NC-SA 4.0, which the builder's licence guard refuses, and
no openly licensed French CEFR word list was found. French's levels are estimated from frequency, as
Spanish's are, and the manifest says `levels_estimated`: the extension labels them estimated and the
ladder borrows English's typical vocabularies, saying so — no code of their own. No FLELex data is
read, measured against or committed.

**The derivation.** The lemmas of `../fr/freq.tsv`, in rank order, take English's level sizes —
1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2: 8,302 lemmas — skipping those a CEFR list
would leave out. The sizes are constants of `reduce-fr-en.py` (`ENGLISH_BANDS`), equal to es-fr's by
a test and never read from English's tables, so an English update cannot move French's levels
unannounced. Where the truth is known, giving English's 8,302 CEFR lemmas these sizes in their own
rank order agrees with the lists for 39.8 % of them and within one level for 82.6 %; each estimated
level's mean true level rises from 1.67 at A1 to 5.03 at C2, so French keeps six levels.

**Which lemmas take one** is read from the English Wiktionary's French section — the source of the
forms, never a pair's glosses — and, since refine-lingua-fr-en-glosses (D3), from French's
dictionary words: a lemma they do not list takes no level, so that a card seeded from a level always
carries a gloss. The table moves with fr-en's glosses only when they add or remove a dictionary word.
Within the levelled span (ranks 1–10,820), 2,518 ranked lemmas take no level:

| Rule | Left out | For example |
|---|---|---|
| 1. the section gives it no sense that is not a form of another word… | 1,395 | `the`, `etc`, `in`, `km`, `http` |
| …or only a name's | 1,028 | `france`, `paris`, `québec`, `facebook` |
| 2. a single character the section gives no word's sense — a letter's name, a symbol, an abbreviation | 16 | `p`, `i`, `h`, `e`, `b`; `à` (a preposition), `y` (a pronoun), `x` (a stool, X-rated) and `ô` (a vocative) keep their place |
| 3. every sense it is given, a name's aside, only spells another word — an obsolete, archaic, rare, dated or alternative spelling, a letter-case form, a misspelling, a pronunciation spelling | 33 | `etat`, `etre`, `etait`, `parceque`, `orient`, `zombie`, `lys` |
| 4. its own form reads as another lemma in `forms.tsv` | 0 | none: the ranks keep no such lemma; the rule stays as a guard, since the builder keys a level by looking the lemma up as a form (`donnée`, read as *donner*, would give *donner* its level) |
| 5. French's dictionary words do not list it — fr-en glosses it not at all, or by a name's senses alone (read last; refine-lingua-fr-en-glosses D3) | 46 | `parce` (met only in « parce que »), `quant`, `x`, `pme`, `stp`, `expliquez`; `coran`, `satan`, `bcp`, `jo` |

`du` and `des`, words of their own (M21), are A1 whatever their senses say; `au` and `aux` are no
words. Every word the pre-pass writes has a level: the elided pieces' words, `à`, `le` and the
inversion's pronouns are A1, `jusque` A2 and `quoique` B1. M8's cost carries over: `porte` is no
lemma, so the door has no level and « il porte » reads as *porter* (A1).

| Level | Lemmas | Ranks | First words |
|---|---|---|---|
| A1 | 1,020 | 1–1,085 | de, le, et, à, en, des, un, que |
| A2 | 1,158 | 1,086–2,385 | croissance, célèbre, davantage, distance, entier, exposition |
| B1 | 2,015 | 2,386–4,832 | communiste, concentration, couteau, doucement, ed, fi |
| B2 | 2,347 | 4,833–8,139 | australien, boue, ciné, configuration, cuire, célébration |
| C1 | 886 | 8,140–9,474 | naïveté, parano, patriotique, pitoyable, plasma, postal |
| C2 | 876 | 9,475–10,820 | morphologie, nantais, opter, orphelin, panda, parachute |

**Levels given to dictionary words alone** (refine-lingua-fr-en-glosses D3): against change 46's
table, 45 lemmas lose their level — 36 fr-en does not gloss (`parce`, `x`, `quant`, `to`, `mm`,
`for`, `pp`, `tom`, `com`, `inter`, `rio`, `av`, `encontre`, `fur`, `pme`, `po`, `am`, `ong`,
`instar`, `ken`, `ep`, `bo`, `app`, `cie`, `pass`, `stp`, `caf`, `expliquez`, `sp`, `rc`, `ht`,
`rsa`, `tnt`, `rip`, `tpe`, `nc`) and 9 it glosses as names alone (`pq`, `jo`, `coran`, `bcp`,
`satan`, `vo`, `cb`, `cac`, `mao`); rule 5 also names `asm`, past the old span —, 45 gain one at C2's end (ranks 10,763–10,820:
`sous-titre`, `sèche-cheveux`, `tire-bouchon`…), and 122 move one band up (5 A2 → A1, 10 B1 → A2,
23 B2 → B1, 40 C1 → B2, 44 C2 → C1): 212 rows, 8,302 levels, as many at each level as before. The
four levelled lemmas of change 48's 40 that fr-en now glosses (`french`, `burger`, `dev`, `ès`, by a
pointer that carries its meaning) keep their level. The figures below are change 46's, measured on
its table.

**Measured with open data only** (design D4), Spanish's committed estimate the control:

| | French | Spanish |
|---|---|---|
| A levelled lemma's translation — the first word of its English Wiktionary glosses that English's CEFR lists level — and that word's level: exact / within one | 28.3 % / 66.7 % of 5,949 (71.7 % covered) | 29.4 % / 65.9 % of 6,140 (74.0 %) |
| Mean English level of the translations, A1 → C2 | 1.80, 2.40, 2.75, 3.06, 3.12, 3.06 | 1.83, 2.37, 2.74, 3.04, 3.04, 3.08 |
| A1 words whose translation is English A1 / A1–A2 | 51.6 % / 76.5 % | 49.6 % / 73.8 % |
| UD PUD's words at A1, A2, B1, B2, C1, C2, none (gold lemmas; names, numbers, symbols, punctuation left out) | 78.4, 7.8, 5.7, 2.7, 0.6, 0.6, 4.3 % | 76.6, 8.1, 6.0, 2.6, 0.8, 0.4, 5.5 % |
| PUD's words presumed known when declaring A2, B1, B2, C1, C2 | 78.4, 86.2, 91.9, 94.6, 95.2 % | 76.6, 84.7, 90.7, 93.3, 94.1 % |

English's lists on the same PUD sentences: 65.8, 12.2, 9.0, 3.9, 0.5, 0.2, 8.3 %, a B1 reader
presumed to know 78.0 % of the words. The estimate presumes more below each level than English's
lists, in French as in Spanish. The translation proxy rises from A1 to B2 and cannot tell B2, C1 and
C2 apart in either language: a rare word's translation is often a common English word.

**Against the design's figures**, measured on change 43's prototype ranks: every proxy holds to the
tenth, and the spans end a few ranks earlier — A2 at 2,374 (2,376), B1 at 4,809 (4,814), B2 at
8,081 (8,086), C1 at 9,412 (9,418), C2 at 10,762 (10,768). Seven lemmas the prototype ranked within
the span are no ranks of these tables: `venue`, `donnée`, `saisie`, `tranchée` and `revenue`, whose
own form reads as a verb, and `ç` and `jusqu`, wordfreq's elision stems. So 2,460 lemmas are left out
instead of 2,466 — rule 4 finds none of its four, the letters lose `ç`, the unknown words `jusqu` —,
the translation proxy covers 5,949 lemmas instead of 5,950, and four levels differ from the
prototype's table: `revenue` (B2) is no lemma, so `confidentialité` moves up to B2, `cheveu` to C1,
and `sous-préfecture` takes C2's last place. On the prototype's ranks the reduction gives the
design's table byte for byte.

**Departures from Spanish's outcome.** Rules 2 and 3 are French's: Spanish's rule (a French gloss
that is not only a proper noun's) levels 20 single characters, 15 of them letters rather than words
(`b`, `d`, `h`, `k` A1, `w` A2, `z` B1), which « Renforcer un niveau » can seed. Without them French's
A1 would hold 11 letters and `etat`, `etats`, `etre`. Most of the 33 spellings spell a word with a
level of its own (`etre` → *être*, `hazard` → *hasard*); some spell none the table levels (`orient`,
`méditerranée`, `zombie`, `ndlr`, `latino-américain`) and lose a level with nothing in its place.
Spanish's table does not move; aligning it, or narrowing rule 3 to a spelling of a commoner word, is
the owner's question (task 5.2).

**Known weaknesses**, as for Spanish: numbers written as words are levelled by frequency (`onze`,
`treize`, `soixante`, `dix-sept` B1, `vingt-quatre` B2, `dix-neuf`, `soixante-dix` C2, `trente-deux`
none), where English's lists put them at A1; a word ranked by a name's frequency keeps its rank
(`jean` A1, `twitter` A2); a noun said mostly in the plural ranks by its singular (`cheveu` C1, `œil`
A2). On change 46's table C2 ended inside a block of 326 ranked lemmas sharing Zipf 3.41, most of
them compounds GSD meets once: 209 of C2's 876 lemmas were in it, 125 of them compounds, and its 27
lemmas past the edge had no level. C2 now ends at rank 10,820. A C2 word is presumed known by no declared level, so only the C2 ladder and C2 seeding see
it.

**If a licence is granted** (design D8): the owner asks UCLouvain's CENTAL for FLELex, with ELELex,
for deriving a lemma → level table, committing it in this public repository and shipping it in the
packages for commercial use. Then `reduce-fr-en.py` reads FLELex at a pinned version — each lemma the
first level at which the list attests it, the lowest on collisions, joined to French's lemmas through
`forms.tsv` —, the manifest drops `levels_estimated`, the NOTICE credits the list under the granted
terms, the licence guard admits the grant as a category of its own, and fr-en and fr-es are reduced
again in one pull request. This estimate is measured against FLELex then, and the figures kept here.

The levels section is one byte per lemma of the pack's pool: the pack grows from 1,241,733 B to
1,302,031 B — 60,035 B the section and the manifest's flag, 263 B the NOTICE's sentence.

## The readings (`../fr/grammar.tsv`)

French's word grammar (add-lingua-french-grammar-tables, change 45): each form's readings, in the
vocabulary the word card reads (add-lingua-word-grammar), written by the same pass over the French
section as the forms, once they are chosen — the readings choose no form and no rank. Only the forms
`forms.tsv` holds carry readings, and only under the lemmas `freq.tsv` ranks: a card opens on a form
the analysis resolved through the table.

**The rules** (`reduce-fr-en.py`, after the forms):
- **Verbs.** kaikki's tags as UD's: the indicative's present, imperfect and future; the passé simple
  (`historic past`) as `Tense=Past`; the present and imperfect subjunctive; the conditional and the
  imperative with **no tense**, as Spanish's tables write them (UD French writes `Tense=Pres`); the
  infinitive; the present participle as `VerbForm=Part|Tense=Pres`, as UD French writes it; the past
  participle with its gender and number — a verb's table lists it bare, the masculine singular, and
  the agreed forms hang under the participle's own entry (`dirigé`: `dirigée`, `dirigés`,
  `dirigées`). Every verb is `VERB`. A sense merging persons or moods (`parle`: « first/third-person
  singular present indicative/subjunctive ») reads as each of them; a compound tense
  (`avoir + past participle`) gives none. A pronominal verb's rows read without their pronoun
  (`s'évanouit` → `évanouit`, `nous évanouissions` → `évanouissions`, `évanouis-toi` →
  `évanouis`), as French's pre-pass leaves the bare form: 12,023 rows.
- **Nouns, adjectives, determiners (articles among them), pronouns, numerals.** A noun's gender from
  `fr-noun`'s head, else its senses, on its own form and on its plural, a reading per gender for a
  noun of both (`enfant`); both numbers on a noun the dictionary gives one form for (`temps`, `bras`,
  `vis`: 549 readings); no reading from a feminine noun's masculine row (`déesse` → `dieu`). An
  adjective's own form is masculine singular when it has a feminine of its own (`grand`), singular
  when one form serves both (`rapide`), plural when every sense is (`plusieurs`); `bel` is a
  masculine singular. A determiner, pronoun or numeral reads its gender and number (`la` is *le*'s
  as the article and as the pronoun), a pronoun's row only with a gender (`ils` is no plural of
  *il*), and a plural-headed determiner's or pronoun's table gives none (`tes` lists `ton`). A
  determiner's, pronoun's or numeral's dictionary form names nothing on its own card.
- **Where a reading comes from.** A lemma's table first, a form's own entry for the pairs no table
  lists. A form of a form along one part of speech reads as the word the form is a form of reads,
  with its own agreement (`dirigée` → `dirigé` → *diriger*'s feminine past participle; `faites`
  adds *faire*'s feminine plural participle), through the form's own lemma too, unless the form
  already reads as that word in that part of speech and verb form (`les` is no feminine through
  `la`). A spelling variant reads as the word it spells (`coeurs` → *cœur*).
- **A reading's part of speech is one its lemma's entries hold**, when the dictionary holds the lemma
  as a noun, verb, adjective, determiner, pronoun or numeral: a participle filed under a noun's or
  an adjective's spelling reads no verb form of it and names its verb (`cités`, the plural of the
  noun *cité*, names *citer*; `privée` *priver*), and `venait` names no *came*, an English gloss its
  entry lists that the section holds as a noun.
- **Never a reading** from a row or sense change 43 leaves out of the forms (alternative, obsolete,
  archaic, rare, dated, misspelt, nonstandard, proscribed, abbreviated, a multi-word construction…),
  from a form's own sense marked as a region's or a register's (`été`, Louisiana's past participle of
  *aller*), from a capitalised headword (`CE`, `LE`: 3,592 entries), from an entry whose every sense
  is an alternative form or a neologism (`estre`, archaic spelling of *être*, whose table would make
  `est` its form; `lea`: 2,548 entries), from a letter's name toward its plural (`elle`, the letter
  L), or through a link an override row of change 43 sets aside as a copy error: `fatiguée`'s verb
  entry reads « feminine singular of parlé », and no reading of it names *parler* (an overridden
  form's own entry linking it to another word than the row's, which no lemma's table lists).
- **`other` marks (M8).** A form's readings of its own lemma are its own; a reading of another
  ranked lemma is marked `other`, and the card names that word without counting it (`fils` names
  *fil*, `couvent` *couver*, `vis` *vivre* and *voir*), only toward an entry of the dictionary that
  is not only regional: `irait` names no *would*, `va` no *vader* (Louisiana's, Switzerland's),
  `entrainait` (*entraîner*'s) no *entrainer*, a spelling the section holds no entry of.

**What they hold**, on the 2026-10-09 tables: **125,177 readings of 88,666 forms**, in 79 tags —
116,089 under the form's own lemma, which 88,518 forms carry, and **9,088 `other` marks** on 4,832
forms. 116,561 come from a lemma's table, 484 from a form's own entry, 8,132 from a form of a form.
By part of speech: 70,566 verb readings, 34,958 noun, 19,570 adjective, 39 determiner, 30 pronoun,
14 numeral. Own readings per form:

| Readings | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| Forms | 69,640 | 15,010 | 1,475 | 101 | 2,152 | 140 |

The 35,578 forms without a reading of their own are, for 35,308 of them, forms of a lemma the
dictionary holds as none of the parts of speech read (adverbs, prepositions, conjunctions,
interjections, names); the rest are a determiner's, pronoun's or numeral's dictionary form (`le`,
`je`, `deux`), old spellings the dictionary marks doubtful, and 18 forms whose readings the
part-of-speech rule takes off their lemma (`croisée`, filed under the noun *croisé*, names
*croiser*; `vingt-et-unième`, an ordinal filed under the numeral *vingt*, reads nothing). The 60,000
lemmas hold 3,339 verbs, 3,285 of them with all 45 simple finite cells in their table; those tables
list 151,363 form–cell pairs, 53,080 of them (35.07 %) forms the table holds — the rest are forms
wordfreq never met (`vînmes`). 17,118 of the 17,128 ranked nouns carry a gender on their own form.

**M8's cost.** A dictionary noun whose own form the forms table reads as another word is no lemma of
the tables, so none of its readings is kept: 551 nouns, 296 of them forms of one of the 5,000
commonest lemmas (`fait`, `été`, `porte`, `demande`, `élève`). « La porte » opens *porter*'s card
with its five verb readings.

**The five-reading forms (M21).** `parle` carries five readings, each its own tag — present
indicative and present subjunctive, first and third persons singular, and the imperative's second
person singular —, all under *parler*: **2,141 forms of 2,110 verbs** (40 of the 1,000 commonest
lemmas, 350 of the 5,000), 180 of PUD's 3,282 verb words and 92 of GSD test's 1,180. The two
presents carry `Tense=Pres` and the imperative none, so the card, which already merges the persons of
one tense and number, is left three groups, and the moods' merge — the word card's, change 51 —
joins the two presents and leaves the imperative apart. No flag marks them: a five-reading form is
recognised from its readings. 6,145 forms read the indicative and the subjunctive of one tense,
person and number (the 2,141, 2,728 third persons plural, 1,080 second persons singular, 162 like
`finissent`, 33 like `finissions`, one other): whether change 51 merges those too is its call.

**The tag pool** (`../fr/tags.tsv`): the 79 tags the readings carry, in byte order, written once by a
person (`cut -f3 grammar.tsv | LC_ALL=C sort -u`) and kept by every reduction (`pack_sources.py
KEPT_INPUTS`); a tag a later reduction adds is appended after them, so that a pinned tag never
changes index. Every pack studying French lays its pool out as the pin, then any reading tag it
lacks, then the tags only its senses carry: fr-en's and fr-es's readings are stored alike whatever
their glosses' senses carry. The pin moves no byte today — fr-en built with the empty pin had the
same sha256. `crates/lingua-pack/tests/committed_tables.rs` holds it to the readings, naming a tag
missing or left over.

**Size.** `grammar.tsv` is 7,936,427 B (752,507 B gzipped); with `forms.tsv` and `freq.tsv`, French's
three tables hold 11,038,422 B (1,636,173 B gzipped, file by file). The readings add 158,222 B to the
pack, which is 1,460,253 B with the levels (1,399,955 B on the forms and ranks alone).

**Measured** on the held-out treebanks (`measure/fr-ud.sh` → `measure/fr_readings.py`, reported,
never gating): over the words whose form the tables map to the treebank's lemma, the share that carry
a reading of their own and the share of those whose treebank part of speech and features are among
them — an auxiliary read as a verb, the conditional's and the imperative's tense left aside, a
participle without a tense read either way.

| | Words | Read | Agree |
|---|---|---|---|
| PUD, finite verbs | 1,742 | 99.94 % | 99.60 % |
| PUD, participles | 973 | 100 % | 99.38 % |
| PUD, infinitives | 466 | 100 % | 100 % |
| PUD, nouns | 4,482 | 99.09 % | 98.85 % |
| PUD, adjectives | 1,473 | 98.85 % | 96.43 % |
| PUD, determiners | 3,587 | 61.36 % | 85.42 % |
| PUD, pronouns | 522 | 16.67 % | 27.59 % |
| GSD test, finite verbs | 637 | 100 % | 99.53 % |
| GSD test, participles | 302 | 100 % | 99.34 % |
| GSD test, infinitives | 190 | 99.47 % | 100 % |
| GSD test, nouns | 1,772 | 98.65 % | 98.68 % |
| GSD test, adjectives | 560 | 98.93 % | 95.67 % |
| GSD test, determiners | 1,350 | 57.85 % | 78.10 % |
| GSD test, pronouns | 292 | 20.89 % | 19.67 % |

A determiner's or pronoun's dictionary form names nothing, hence their « read » column; what they
read is mostly a dictionary noun's spelling — `un` (the noun « un »), `son` (« sound »), `nous` —,
whose readings are the noun's, hence their « agree » column. Among verbs, what disagrees is mostly
the treebank's own annotation (`prit` and `crée` as participles, `attirerait` as an imperfect).

**Each rule, switched off alone** (on these tables; PUD's figures move as shown):

| Rule switched off | Readings | Forms with one | `other` | Five-reading forms | On PUD |
|---|---|---|---|---|---|
| — (the rules) | 125,177 | 88,518 | 9,088 | 2,141 | — |
| a reading's part of speech its lemma's | 126,495 | 88,536 | 9,051 | 2,141 | adjectives agree 96.43 → 96.57 % |
| a form of a form | 117,038 | 81,649 | 8,124 | 2,141 | participles read 100 → 75.44 %, agree 99.38 → 98.09 % |
| both numbers on an invariable noun | 124,628 | 88,518 | 9,088 | 2,141 | nouns agree 98.85 → 97.79 % |
| spelling variants | 124,662 | 88,095 | 9,118 | 2,128 | nouns read 99.09 → 99.02 % |
| merged senses read as each | 125,097 | 88,503 | 9,088 | 2,130 | — |
| a pronominal verb without its pronoun | 125,127 | 88,484 | 9,079 | 2,141 | infinitives read 100 → 99.79 %; 37 fewer full paradigms |
| a participle sense without `past` | 125,162 | 88,507 | 9,087 | 2,141 | — |
| articles as determiners | 125,171 | 88,518 | 9,085 | 2,141 | determiners agree 85.42 → 9.59 % |

| Exclusion switched off | Readings | Forms with one | `other` | On PUD |
|---|---|---|---|---|
| capitalised headwords read | 126,684 | 88,766 | 9,123 | nouns agree 98.85 → 99.26 %, adjectives 96.43 → 96.23 %, determiners 85.42 → 83.93 % |
| alternative-only and neologism entries read | 125,874 | 88,958 | 9,189 | nouns agree 98.85 → 98.81 % |
| `other` toward a link-only or a regional word | 125,348 | 88,518 | 9,259 | — |
| a feminine noun's masculine row read | 125,323 | 88,524 | 9,228 | — |
| a composed reading over a direct one | 125,192 | 88,518 | 9,091 | — |
| a letter's plural read | 125,190 | 88,518 | 9,089 | — |
| a pronoun's form without a gender read | 125,185 | 88,520 | 9,093 | pronouns read 16.67 → 23.37 %, agree 27.59 → 48.36 % |
| a plural-headed table read | 125,179 | 88,518 | 9,090 | — |
| a regional sense of a form's entry read | 125,180 | 88,518 | 9,091 | — |
| a link an override row sets aside | 125,178 | 88,518 | 9,089 | — (`fatiguée` names *parler*) |

**Known data defects**, left as the source writes them: `supe` reads as the verb *super* (« to
sip »), one of the 40 five-reading forms among the 1,000 commonest lemmas; `meilleure` reads as the
noun *meilleur* only, the adjective being a form entry of *bon*; `quelques-unes`, « feminine of
quelques-uns » with no number, reads as a feminine singular of *quelqu'un*.

## The glosses (`gloss.tsv`, `senses.tsv`, `mwe.tsv`)

French glossed in English (add-lingua-pack-fr-en, change 48), from the **English Wiktionary's French
section alone** — the file the forms come from, at the same pinned snapshot: fr-en was reduced again
from change 43's pin, nothing fetched that it does not record, `../fr/` byte for byte but its
dictionary words.

**The rules** (`reduce-fr-en.py`, after the forms, ranks, readings and levels; no shared module
edited):
1. **The section, cut** (`native_fields`): each entry's word, part of speech and senses (glosses,
   tags, pointers); a typographic apostrophe in a headword read as `'`, as the forms are
   (`nombre d’oxydation`), its case kept.
2. **The English edition's pre-passes**, in es-en's order: a letter's entry and a sense naming a
   letter left out (`elle`, the letter L), and the words written under a single capital letter
   (`X` « X-frame stool », `C` « abbreviation of cavalier »); the senses read as meanings and in one
   English typography (`english.read_as_meanings`, refine-lingua-es-en-glosses); a word's
   etymologies merged as the edition's setting says (off).
3. **French's expressions** (`expression_senses`, `split_words`): in an expression, a sense that only
   names another spelling (« post-1990 spelling of »), an inverted form (« subject-inverted form
   of ») or the pieces it is written with (« que + elle ») is no gloss, a meaning written after it
   stays (« y a-t-il » « is there? are there? »); `à la` is left out by name (`LEFT_OUT`): its one
   sense, « in the style of », needs the word after it, and the section writes those uses as entries
   of their own (`à la carte`, `à la maison`). The single words French's tokenisation splits — an
   apostrophe or a hyphen, a French token, no form of `../fr/forms.tsv` and no name (`d'abord`,
   `c'est`, `allez-y`) — are offered as expressions, read with the shared rules' sense rules; the
   builder keys those that read as two tokens or more (add-lingua-french-expression-keys).
4. **The shared rules** (`reduce_common.native_tables`) over the 60,000 ranked lemmas: eight whole
   senses grouped by part of speech, a form-only lemma lending its base's senses, an acronym's entry
   sparing the common word; the expressions' three senses. Every glossed lemma is a ranked lemma its
   own form reads as (`crates/lingua-pack/tests/committed_tables.rs`): the builder files a gloss at
   the lemma's own place, and no gloss is lent to another word — the design's prototype, on change
   43's tables before its fix, glossed *venir* « coming, arrival », `venue`'s.

5. **fr-en's own rules** (refine-lingua-fr-en-glosses), in `reduce-fr-en.py` and, for the treebank's
   part of speech, `reduce_french_treebank.py` — a rule module of its own, in fr-en's digest, that
   fr-es is to read too (refine-lingua-fr-es-glosses D8) — the English edition's and the shared rules
   read es-en's rows and every pair's too, and are not edited, so only fr-en re-pins (D1):
   - `read_as_french`, a pre-pass after `read_as_meanings` and before the etymology merging:
     **a pointer that carries its meaning** is read as that meaning (D4) — the section's `extra`
     for its target, else its quoted text, else its text past a colon or a semicolon —, for a word
     only when it names a degree of comparison, a synonym, a plural or a contraction (« des »
     « some; of the, from the, some », « mieux » « better; best; … », « ouais » « yeah, yep… »),
     never a female equivalent (`directrice` keeps `directeur`'s « director; school principal »:
     the owner, 2026-10-10), an alternative form, a spelling, an ellipsis or a clipping; for an
     expression whatever it names (« il y a » « there is, there are; ago »); never in a name's or an
     acronym's entry, never a meaning in capitals only. **A function word's row** (`ADP`, `DET`,
     `PRON`, `CCONJ`, `SCONJ`, `PART`, `ADV`) **or a row the page opens on a name** opens on the
     part of speech UD French-GSD reads the word as at least ten times and twice as often as the
     page's first, never a proper noun's (D5: `pas` on its negation, `son` « his, her… », `leur`
     « their », `bien` « well », `quand`, `juste`, `pendant`, `outre`, `envers`; `marche`,
     `réunion`, `somme`, `restauration` no longer open on a place; with D4, `du`'s article before
     its contraction); a noun, verb or adjective ahead
     of another stays as the page writes it (`ferme` « firm », `mort` « dead », `devoir` « duty »:
     the owner). **No name under a function word** (D6: `le` « a surname from Vietnamese », `on`
     « a village in Luxembourg »). **The page's notes out** (D8: « see usage notes », « (all
     senses) », « in its various senses », « (Folk etymology: …) »), fr-en's description openers in
     lower case (« Substitutes », « Impersonal », « Followed »…), a quotation's citation cut, a
     source's sense number out.
   - Five expressions whose one sense needs a context their key does not hold are left out by name
     (`LEFT_OUT`, each with its reason counted in GSD's text: `et des`, `que de`, `sur ce`, `et
     si`, `un coup`), and six post-1990 spellings keyed apart from their traditional spelling lend
     its gloss (`traditional_spellings`: `à priori`, `à postériori`, `et cétéra`, `sur son
     trente-et-un` and two verbs on it) (D7).
   - « etc » takes its period back after the shared rules (`with_etc_period`, D8).
   - **French's dictionary words** (`dictionary_words`, D2): the glossed lemmas less the 3,581 every
     sense run of which is a proper noun's, written to `../fr/lexical.tsv`; the levels read them
     (*The levels*).

**fr-en's own rules, each alone and together** (the design's *Measured*, re-measured with the
reducer's own code on the committed tables, its rules switched off but one; with none, the tables
before them byte for byte):

| Rule | Rows / top 10,000 | First sense / top 10,000 | Lemmas gained | Expressions changed / gained / left out |
|---|---|---|---|---|
| D4, a pointer's meaning | 40 / 18 | 13 / 10 | 12 (4 of the top 10,000) | 15 / 166 / 0 |
| D5, the treebank's part of speech | 13 / 13 | 13 / 13 | — | — |
| D6, no name under a function word | 2 / 2 | 0 / 0 | — | — |
| D7, left out by name | — | — | — | 0 / 0 / 5 |
| D7, post-1990 spellings lent | — | — | — | 0 / 6 / 0 |
| D8, notes | 106 / 36 | 96 / 28 | — | 3 / 0 / 0 |
| D8, openers | 14 / 8 | 8 / 2 | — | 1 / 0 / 0 |
| D8, citations and sense numbers | 4 / 3 | 2 / 2 | — | — |
| D8, « etc. » | 117 / 42 | 59 / 10 | — | 7 / 0 / 0 |
| **Together** | **291 / 117** | **190 / 64** | **12, none lost** | **26 / 172 / 5** |

The 12 lemmas gained are words whose only senses were pointers of D4's wordings: `french`, `burger`,
`dev`, `ès` (four of change 48's levelled lemmas with no gloss, which keep their level), `ive`,
`because`, `blockchain`, `chui`, `sherry`, `axis`, `loix`, `broyeuse`. Every figure is the design's.
A few read oddly and are in the owner's sample (task 6.2): « matelas » gains « French tacos »,
« bercy » « drunkard », « j't'à » « the 't' is epenthetic ». D4 also makes expressions of the
section's Louisiana spellings `à le`, `à les` « to the » and `de le`, `de les` « “of the”, some »:
every « au » and « aux », which the pre-pass reads as `à` + `le`/`les`, meets the first two, and a
pronoun after « de » the last two (« décidé de le faire » shows `de le` « “of the”, some ») — found
while implementing, in the owner's sample, not left out here (the design's figures hold them).

**No translation table** (D3). The French Wiktionary's English translations and the English
Wiktionary's French translations read backwards were measured on the design's prototype and
declined: they would add 1,988 lemmas, 1,895 of them words the section has no entry for — English
words in French text (« in », « end » « NDE, NDI, NDT »), names, initialisms, unaccented misspellings,
1,193 listing the word itself as its translation — and fr-en, French's reference pair, would make
them French's dictionary words, counted by every French reader's vocabulary estimate and kept as
words by the names rule. Read backwards, an English entry makes French's commonest bigrams
expressions (« il est » "he's", 782 times in UD French's 425,111 words). A word or an expression the
section does not gloss has no gloss; the two catalogue files change 38 registered for this pair are
no longer derived. Whether the French Wiktionary's English translations should gloss expressions in
a later update is the owner's question (design, Open Question 1).

**What they hold**, on the 2026-10-09 tables:

| | |
|---|---|
| Glossed lemmas | **30,067**, every one from the section's own entries |
| French's dictionary words (`../fr/lexical.tsv`) | **26,486**: the glossed lemmas less the 3,581 glossed by a proper noun's senses alone (359 / 912 / 1,763 of the 5,000 / 10,000 / 20,000 commonest) |
| Of the 5,000 / 10,000 / 20,000 commonest lemmas | **4,679 / 8,688 / 15,260 — 93.6 / 86.9 / 76.3 %**; floor 91.9 / 85.1 / 74.4 (`gloss_coverage.py FLOORS`, the study's 93.9 / 87.1 / 76.4 less two points, held by the reduce job; settled by the owner) |
| Expressions | **17,646**: 15,659 headwords with a space and 1,987 words the tokenisation splits |
| `gloss.tsv`, `senses.tsv`, `mwe.tsv` | 1,616,576, 512,267 and 834,225 B (625,308, 125,636 and 326,607 B gzipped); `../fr/lexical.tsv` 253,742 B |
| The pack | **2,537,587 B**: change 48's 2,527,222 B, fr-en's own rules +2,853 B (the glosses −96, their runs +315, the expressions +2,634), French's lexical table +7,512 B |

No figure is published: the site's figures list the shipped pairs alone, and fr-en ships with change
52.

**French's expressions** (D11), against the shared rules alone: of their 15,526 headwords with a
space, 18 are left out — `à la` and 17 post-1990 spellings whose one sense points at the
traditional one, which stands alone (`crème fraiche`, `s'il vous plait`, `boite à gants`, `cout
d'opportunité`, `plateforme de forage`, `être sur son trente-et-un`…) — and one gloss changes,
`y a-t-il`'s. Of 2,005 words the tokenisation splits that the section glosses, 34 are left out, every
sense of theirs a pointer: `qu'elle`, `qu'on`, `qu'un`… (« que + elle »), `s'est`, `s'en`, `jusqu'au`,
`jusqu'aux`, `qu'au`, `m'en`, `n'en`, `d'avoir`, and post-1990 spellings (`maitre-nageur`,
`casse-croute`, `bloc-note`); « jusqu'au soir » meets `jusqu'à` « until », not « jusque + au ».
`m'a` keeps « “I'm going” » and loses « me + a ». `gloss.tsv` and `senses.tsv` are byte for byte
the same without these rules: they read no lemma's entry.

**23b's rules on fr-en** (`read_as_meanings`), each alone and together, against the section without
them:

| Rule | Rows / top 10,000 | First sense | Expressions | Examples |
|---|---|---|---|---|
| D2, nested senses | 21 / 13 | 12 / 7 | 1 changed, 1 gained | « chambre » « a chamber in its various senses, including » → « a room; a hotel room; a bedroom; a house of a parliament »; « nous » « the plural personal pronoun in the first person » → « we; us, to us; … »; « scène », « champ », « ours », « concevoir », « parrain » |
| D3, shortened and case forms | 8 / 4; 2 lemmas gained (`eine`, `mam'zelle`) | 3 / 0 | 1 gained | « elle » gains « her, it, à elle = hers, its », « elles », « pus », « fac » |
| D4, a function word spelled like a place | 0 | 0 | 0 | — |
| D5, one typography | 243 / 110 | 93 / 30 | 141 | « du » « Forms the partitive article » → « forms the partitive article », « de », « en », « que », « pas » |
| **Together** | **269 / 125** | **107 / 37** | **142 changed, 2 gained** | no gloss lost |

Before them, 8 rows (6 of the top 10,000) held an ellipsis written `...` and 46 (28) straight
quotes; after them, none.

**M20 on fr-en** (the English edition's two settings, the owner's, committed at 0 and off): the
long-parenthesis bound at 40 would change 2,342 rows (902 of the top 10,000), the etymology merging
304 (168); neither loses a gloss. Either setting re-pins es-en and fr-en together.

**Against the design's figures** (the prototype, on change 43's tables before its fixes): 30,059
glossed lemmas → 30,055 (`revenue` and `subordonnée`, whose own form reads as a verb, are no ranks
of these tables; `curial` is) and 86.9 → 86.8 % at 10,000, the design's re-run on 43's first fix
included; 17,480 expressions → 17,479 (`maitre-nageuse`: what its post-1990 pointer writes after
the target, « female equivalent of maitre-nageur », is itself a pointer by the English edition's
rules, so no sense is left); the 23b and M20 figures to the row. The pack, 2,421,321 B in the design
(on changes 45's and 46's prototype tables), is 2,527,222 B: measured on the committed tables and
with change 44's builder, which keys French's expressions through the core's reading and names each
by its headword where its key differs.

**Names glossed by the section are no dictionary words** (refine-lingua-fr-en-glosses D2, the
owner's decision of 2026-10-10). 3,581 glossed lemmas (912 of the top 10,000) are glossed by a
proper noun's senses alone — every run of their `senses.tsv` row is `PROPN`: `france`, `paris`,
`québec`, `françois`, `lyon`, `durand` « a surname »; 3,586 before fr-en's own rules. They keep
their gloss — a reader who opens `Paris` at a sentence's head, or a lowercase `lyon`, still reads it
— and leave French's dictionary words: the vocabulary estimate's universe goes from 30,095 to 26,486
words, and change 41's names rule sets them aside as written: on the baseline's `noms` page `Durand`,
`Lefèvre`, `Jean-Pierre`, `Saint-Étienne`, `Rhône`, `Garonne`, `Paris`, `Renault`, `François`, `Lyon`
and `Grenoble` with `Myriel` (70 → 59 counted words), on `proust` `François` and `Charles`. A word
with a common sense beside a name's stays one (`lot`, `aube`, `nice`, `marche`). Left unglossed, the
names would take fr-en under its floor at every cut (86.4 / 77.8 / 67.5 %). English's and Spanish's
dictionary words keep their names: a change of its own (the owner, Q1). fr-en's pack therefore
carries a lexical table, and `crates/lingua-pack`'s check of a studied folder accepts a reference's
glossed lemmas or all of them but its names alone, nothing in between.

**No level without a gloss** (D3, the owner's decision): of change 48's 40 levelled lemmas fr-en did
not gloss, four are glossed now (`french`, `burger`, `dev`, `ès`) and keep their level; the 36 others
and the 9 levelled lemmas glossed as names alone lose theirs, their slots going to the next lemmas in
rank order (*The levels*). `crates/lingua-pack/tests/committed_tables.rs` checks that every levelled
lemma is a dictionary word with a gloss.

**What still reads wrong**. Change 48's list (its D7) was the input of refine-lingua-fr-en-glosses,
which fixed or left each class (its design's D9), the owner settling Q1–Q5 on 2026-10-10:

| Class | Then (rows / top 10,000) | Now |
|---|---|---|
| A borrowed gloss wrong for the word | « des » « of the; some, the feminine partitive article »; « ca » « board of directors » | `des` reads its own pointers (D4). `ca` keeps « board of directors »: wordfreq's `ca` is mostly an unaccented `ça`, to be read as its spelling in a forms-table change of its own (Q4) |
| A pointer's own meaning left out | « il y a », `mieux`, `moins`, `ouais` | read as meanings (D4), on a closed list of a word's wordings |
| An expression whose sense needs a context its key does not hold | `et des`, `que de`, `sur ce`, `et si`, `un coup` | left out by name (D7) |
| A post-1990 spelling keyed apart from its traditional one | `à priori`, `à postériori`, `et cétéra`, `sur son trente-et-un` | lend the traditional spelling's gloss (D7); `crème brulée` already met `crème brûlée` |
| A name glossed and a dictionary word | 3,586 / 914 | no dictionary word, still glossed (D2) |
| A proper noun's run in a common word's row | first 218 / 108; after another run 484 / 260 | the 13 function words' and names-first rows D5 moves, `le` and `on` (D6); 217 rows (104 of the top 10,000: `louis`, `midi`, `belgique`, `noël`, `bordeaux`) still open on a proper noun and 487 / 263 end on one — accepted until a card reads the token's capital (23b's Q3; Q5) |
| The page's own notes | « see usage notes » 5 / 3; « (all senses) » 98 / 30; « in its various senses » 1 / 1; « (Folk etymology: …) » 2 / 2 | out (D8) |
| A description in a capital outside 23b's list | 14 / 8 | fr-en's openers in lower case (D8); the definitions the page writes in sentence case (« Military rank equivalent to corporal ») stay |
| A citation inside a sense | 2 / 2 | cut (D8) |
| A source's numbered sense in another shape | 2 / 1 | out (D8) |
| « etc » without its period | 117 / 42 | « etc. » in fr-en (D8); en-fr's, es-fr's, es-en's and en-es's « etc » is the shared fix 23b named (its D6) |
| Labels left out | « or », « monde »: an obsolete or archaic sense | 23b's Q1 |
| The part of speech a row opens on | 2,897 / 1,264 rows hold two or more | 13 move (D5); a noun, verb or adjective ahead of another stays (Q2) |
| An acronym's pointers lending to a word with an entry of its own | `ca`, `svp`, `jsp`, `cv` | left (D9): dropping them would cost `svp` and `jsp` their gloss |

The rules fr-en adds would read es-en's rows right too (« (all senses) » in 15 es-en rows, its
« etc » in 35): a later refinement of the English edition, which re-pins es-en and fr-en together.

**The French baseline** (`crates/lingua-wasm/tests/french_baseline.rs`) runs over the pack these
tables build from this change on: a pull request that moves `../fr/` or this folder and the French
golden with them re-blesses `fr-en.golden` and says so.

## What the later changes add

| Change | Adds |
|---|---|
| 40 tokenisation | French's pre-pass, which these tables serve; its version bump re-reduces fr-en (`manifest.json` alone moves, and the pack and pin with it). The check over these tables (`crates/lingua-pack/tests/committed_tables.rs`) then reads its lists from lingua-core |
| 41 analysis | the cascade, designed and measured on these tables; its version bump re-reduces fr-en |
| 44 expression keys | the plain words left out here because they begin with a piece (`d'abord`, `c'est`, `l'on`) |
| 48 fr-en | done: the English glosses, expressions and senses (*The glosses*); `../fr/lexical.tsv` holds the glossed lemmas, and the French invariance baseline runs over these tables |
| 48b refine-lingua-fr-en-glosses | done: fr-en's own rules (*The glosses*); French's dictionary words leave out the names fr-en glosses alone, and only they take a level |
| 49 fr-es | a reader of `../fr/` as committed, capped at the same 60,000 |

## Licences

The repository is Apache-2.0; **these files are not**, nor French's in `../fr/`. They are derived
from the sources above and carry their licences:
- `../fr/forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `../fr/freq.tsv`: CC BY-SA 4.0 (wordfreq, and GSD's counts for the compounds);
- `../fr/grammar.tsv`: CC BY-SA 4.0 and the GFDL (kaikki);
- `../fr/level.tsv`: CC BY-SA 4.0 (derived from `freq.tsv`) and CC BY-SA 4.0 and the GFDL (kaikki,
  which says which lemmas take a level);
- `gloss.tsv`, `senses.tsv`, `mwe.tsv` and `../fr/lexical.tsv` (derived from `gloss.tsv` and
  `senses.tsv`): CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts, which order a
  function word's senses).

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand — except `../fr/tags.tsv` and `../fr/studied.json`, which no reducer writes.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=fr-en` and `mode=update`.
  It reads the English edition's dump once (about 3 GB gzipped), derives the French section from
  it, keeps its bytes as the release `lingua-pack-sources-fr-en-<snapshot>` (about 26 MB
  compressed), records GSD's two sections at their commit, reduces, and pushes the branch
  `lingua-pack/fr-en/<snapshot>`. Its runs are one at a time in the `fr` group, so an update of fr-en
  and of fr-es never propose `../fr/` on two branches.
- **After editing the reduction rules** — `reduce-fr-en.py`, the override, elision and `LEFT_OUT`
  tables included, `reduce_common.py`, which every pair shares, or `reduce_edition_en.py`, the
  English Wiktionary's rules, which es-en reads too (`pin.json` lists the three under
  `reducer.files`): the check lane fails until the tables are reduced again from the pinned sources.
  Run `scripts/lingua-data/build.sh --reduce fr-en <out>` (Python 3.12, `requirements-reduce.txt`),
  or `lingua-pack-update` with `mode=reduce`. A bump of `FRENCH_ANALYZER_VERSION` asks the same.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.
- **When a reduction's readings carry a tag `../fr/tags.tsv` does not pin, or no longer carry a
  pinned one**, `crates/lingua-pack/tests/committed_tables.rs` fails naming it: append the new tag
  after the pinned ones (a pinned tag never changes index), or remove the one left over, in the same
  pull request.

The monthly dry run of the update checks this pair as it checks the others.
