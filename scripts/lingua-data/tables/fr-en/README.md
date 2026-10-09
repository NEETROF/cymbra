# The fr→en dictionary tables

The reduced tables of Cymbra Lingua's French→English data pack, committed so that every build makes
the same pack with no download, and so that a change to the dictionary is a pull request whose diff
shows it (add-lingua-french-forms-tables, change 43 of `docs/lingua/language-matrix-programme.md`).
No extension package carries the pack: `packs.json` does not list it, and no reader holds a French
pack before `enable-lingua-french` (change 52).

They are in two folders (split-lingua-pack-tables-by-language). fr-en is French's reference pair
(`../fr/studied.json`): its reduction writes French's own tables in `../fr/`, which every pair
studying French reads as committed (fr-es, change 49), and this folder's `pin.json` records the
sources of both. Today fr-en reduces French's forms, ranks and estimated levels: it glosses nothing
yet.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → English gloss: **empty** until add-lingua-pack-fr-en (change 48) | — |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata: French glossed in English, French's analyser version (`FRENCH_ANALYZER_VERSION`, read from lingua-core), `levels_estimated` (the levels are estimated, not a CEFR list's), and `pack_version` (the snapshot, and the rules that reduced it) | — |
| `pin.json` | the raw sources these tables and `../fr/` came from, and the pack they build | — |

In `../fr/`, French's tables, written by fr-en's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org, the English Wiktionary's French section (CC BY-SA 4.0 + GFDL), with UD French-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0); a hyphenated word by GSD's own frequency too |
| `level.tsv` | lemma → estimated CEFR level (*The levels* below) | derived from `freq.tsv` and the English Wiktionary's French section (no source of its own) |
| `lexical.tsv` | French's dictionary words, the lemmas fr-en glosses: **empty** until change 48 | derived from `gloss.tsv` by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | French's pinned tag pool: **empty** until add-lingua-french-grammar-tables (change 45) pins it; written by no reducer | committed by hand |
| `studied.json` | the pair whose reduction writes `../fr/`: fr-en | committed by hand |

`grammar.tsv` (the readings) comes with change 45.

## The sources

- **kaikki.org, the English Wiktionary's French section** (`kaikki-French.jsonl`, 403,269 entries,
  510,058,226 B), derived from the English edition's dump (`pack_sources.py EDITIONS`, `DUMPS["fr-en"]`):
  the dump regenerated on 2026-10-03 08:24, read on 2026-10-09, decompressed sha256 `93b79aac…`.
  Its inflections are tagged, a lemma's table lists them and a form's own entry points at what it is
  a form of. A verb's table lists the masculine singular past participle (`dirigé`) and none of its
  agreed forms: those hang under the participle's own entry.
- **UD French-GSD**, its training and development sections at `94d5b68e185fc22a9ef292040e84f476d36d9b0e`
  (`pack_sources.py PINNED["fr-en"]`), read for one thing: how often a form stands for each lemma, and
  how often a hyphenated lemma occurs (390,368 words). Its test section is never read: the
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
tables build 1,308,123 B. With the levels it is 1,302,031 B (*The levels*). The builder holds it
under 5 MiB; the glosses and readings are measured against that budget by the changes that add them.

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
forms, never a pair's glosses: the table does not wait for fr-en's glosses (change 48) and does not
move when they land. Within the levelled span (ranks 1–10,762), 2,460 ranked lemmas take no level:

| Rule | Left out | For example |
|---|---|---|
| 1. the section gives it no sense that is not a form of another word… | 1,389 | `the`, `etc`, `in`, `km`, `http` |
| …or only a name's | 1,022 | `france`, `paris`, `québec`, `facebook` |
| 2. a single character the section gives no word's sense — a letter's name, a symbol, an abbreviation | 16 | `p`, `i`, `h`, `e`, `b`; `à` (a preposition), `y` (a pronoun), `x` (a stool, X-rated) and `ô` (a vocative) keep their place |
| 3. every sense it is given, a name's aside, only spells another word — an obsolete, archaic, rare, dated or alternative spelling, a letter-case form, a misspelling, a pronunciation spelling | 33 | `etat`, `etre`, `etait`, `parceque`, `orient`, `zombie`, `lys` |
| 4. its own form reads as another lemma in `forms.tsv` | 0 | none: the ranks keep no such lemma; the rule stays as a guard, since the builder keys a level by looking the lemma up as a form (`donnée`, read as *donner*, would give *donner* its level) |

`du` and `des`, words of their own (M21), are A1 whatever their senses say; `au` and `aux` are no
words. Every word the pre-pass writes has a level: the elided pieces' words, `à`, `le` and the
inversion's pronouns are A1, `jusque` A2 and `quoique` B1. M8's cost carries over: `porte` is no
lemma, so the door has no level and « il porte » reads as *porter* (A1).

| Level | Lemmas | Ranks | First words |
|---|---|---|---|
| A1 | 1,020 | 1–1,080 | de, le, et, à, en, des, un, que |
| A2 | 1,158 | 1,081–2,374 | certainement, clairement, collection, conscience, content, croissance |
| B1 | 2,015 | 2,375–4,809 | trompe, val, alimentaire, annuel, apparence, autrefois |
| B2 | 2,347 | 4,810–8,081 | rébellion, rédacteur, réserver, simultanément, slogan, sonde |
| C1 | 886 | 8,082–9,412 | conjoncture, consolidation, consolider, contradictoire, corner, croquis |
| C2 | 876 | 9,413–10,762 | clandestin, commentateur, contrefaçon, convaincant, convenablement, croate |

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

**Against the design's figures**, measured on change 43's prototype ranks: the counts per rule and
every proxy hold to the tenth but two, and the spans end a few ranks earlier — A2 at 2,374 (2,376),
B1 at 4,809 (4,814), B2 at 8,081 (8,086), C1 at 9,412 (9,418), C2 at 10,762 (10,768). Seven lemmas
the prototype ranked within the span are no ranks of these tables: `venue`, `donnée`, `saisie`,
`tranchée` and `revenue`, whose own form reads as a verb, and `ç` and `jusqu`, wordfreq's elision
stems. So 2,460 lemmas are left out instead of 2,466 — rule 4 finds none of its four, the letters lose
`ç`, the unknown words `jusqu` —, the proxy covers 5,949 lemmas instead of 5,950, and four levels
differ from the prototype's table: `revenue` (B2) is no lemma, so `confidentialité` moves up to B2,
`cheveu` to C1, and `sous-préfecture` takes C2's last place.

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
A2). C2 ends inside a block of 326 ranked lemmas sharing Zipf 3.41, most of them compounds GSD meets
once: 209 of C2's 876 lemmas are in it, 125 of them compounds, and its 27 lemmas past the edge have
no level. A C2 word is presumed known by no declared level, so only the C2 ladder and C2 seeding see
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

## What the later changes add

| Change | Adds |
|---|---|
| 40 tokenisation | French's pre-pass, which these tables serve; its version bump re-reduces fr-en (`manifest.json` alone moves, and the pack and pin with it). The check over these tables (`crates/lingua-pack/tests/committed_tables.rs`) then reads its lists from lingua-core |
| 41 analysis | the cascade, designed and measured on these tables; its version bump re-reduces fr-en |
| 44 expression keys | the plain words left out here because they begin with a piece (`d'abord`, `c'est`, `l'on`) |
| 45 grammar | the readings of these forms (`../fr/grammar.tsv`), and French's tag pool (`../fr/tags.tsv`) |
| 48 fr-en | the English glosses, expressions and senses; `../fr/lexical.tsv` then holds the glossed lemmas, and the French invariance baseline moves from its fixture to these tables |
| 49 fr-es | a reader of `../fr/` as committed, capped at the same 60,000 |

## Licences

The repository is Apache-2.0; **these files are not**, nor French's in `../fr/`. They are derived
from the sources above and carry their licences:
- `../fr/forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `../fr/freq.tsv`: CC BY-SA 4.0 (wordfreq, and GSD's counts for the compounds);
- `../fr/level.tsv`: CC BY-SA 4.0 (derived from `freq.tsv`) and CC BY-SA 4.0 and the GFDL (kaikki,
  which says which lemmas take a level).

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand — except `../fr/tags.tsv` and `../fr/studied.json`, which no reducer writes.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=fr-en` and `mode=update`.
  It reads the English edition's dump once (about 3 GB gzipped), derives the French section from
  it, keeps its bytes as the release `lingua-pack-sources-fr-en-<snapshot>` (about 26 MB
  compressed), records GSD's two sections at their commit, reduces, and pushes the branch
  `lingua-pack/fr-en/<snapshot>`. Its runs are one at a time in the `fr` group, so an update of fr-en
  and of fr-es never propose `../fr/` on two branches.
- **After editing the reduction rules** — `reduce-fr-en.py`, the override and elision tables
  included, or `reduce_common.py`, which every pair shares (`pin.json` lists both under
  `reducer.files`): the check lane fails until the tables are reduced again from the pinned sources.
  Run `scripts/lingua-data/build.sh --reduce fr-en <out>` (Python 3.12, `requirements-reduce.txt`),
  or `lingua-pack-update` with `mode=reduce`. A bump of `FRENCH_ANALYZER_VERSION` asks the same.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks the others.
