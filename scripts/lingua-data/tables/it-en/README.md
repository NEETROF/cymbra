# The it→en dictionary tables

The reduced tables Cymbra Lingua's Italian→English data pack is built from: Italian glossed in
English, for English speakers studying Italian — the first reader pair of Italian
(add-lingua-pack-it-en, change 22 of `docs/lingua/italian-programme.md`). They are committed so that
every build makes the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it. No extension package carries the pack: `apps/lingua-extension/packs.json`
does not list it-en until Italian is enabled for English speakers (change 25), after its refinement
(change 23, `refine-lingua-it-en-glosses`).

This folder holds what belongs to it-en alone: its English glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. Italian's own tables — its forms, ranks,
levels, readings, tag pool, dictionary words and acronym rows — are kept once, in `../it/`, and
written by it-fr's reduction alone (`../it/studied.json` names it-fr, Italian's reference pair).
it-en's reduction reads them as committed and computes nothing of them: its lemmas and their ranks
are `../it/forms.tsv` and `../it/freq.tsv`, its readings, levels and dictionary words it-fr's.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → English gloss | the English Wiktionary's Italian entries; else the English translations the Italian Wiktionary's Italian entries list (both CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same; a noun's gender is not here, the builder reads it from `../it/grammar.tsv`'s readings |
| `mwe.tsv` | expression → English gloss | the same sources, for headwords with a space and for the words Italian's pre-pass splits (`d'accordo`) |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources, and the French Wiktionary, whose Italian section (it-fr's glosses) decides Italian's dictionary words and which lemmas take a level | — |
| `manifest.json` | the pack's metadata: Italian glossed in English, the Italian analyser's version, `levels_estimated`, and `pack_version` (it-en's snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | it-en's raw sources, the pack these tables build with `../it/`, its base and layer, the rules that reduced them, and the sha256 of each of `../it/`'s six tables they were built on (`studied`) | — |

## What is in them

<!-- The figures below are the implementation's prediction (2026-10-25), reduced locally from the bytes
the first update reads; the update's pull request writes the committed ones in their place. -->

- **English glosses for 33,209 lemmas** of Italian's 60,000 — 33,014 from the English Wiktionary's
  Italian section, 194 from the Italian Wiktionary's translations, and `USA`, glossed through its
  synonyms (`America`, `Stati Uniti`) — and **9,831 expressions**: 8,542 headwords with a space from
  the section, 1,194 from the Italian Wiktionary, and 95 words Italian's pre-pass splits (93 from the
  section — `d'accordo`, `all'improvviso`, `anch'io` « me too (subject) » —, 2 from the Italian
  Wiktionary). Up to eight senses grouped by part of speech, as every pair's, in the lower case the
  English Wiktionary writes a foreign word's senses in.
- The share of the commonest lemmas glossed, which the `reduce` job holds to the floor fixed before
  the first committed measurement (`gloss_coverage.py --pair it-en`, `FLOORS["it-en"]`, the study's
  figures on change 9's committed ranks less two points):

  | Lemmas | English Wiktionary | with the translations | the floor |
  |---|---|---|---|
  | top 5,000 | 90.9 % | 91.3 % | 89.3 % |
  | top 10,000 | 83.8 % | 84.2 % | 82.2 % |
  | top 20,000 | 75.3 % | 75.7 % | 73.7 % |
  | all 60,000 | — | 55.3 % | — |

  Of the 8,423 lemmas glossed among the 10,000 commonest, 0.5 % (45) come from the Italian Wiktionary.
- **Dictionary words are it-fr's**: 9,627 lemmas it-en glosses are no dictionary word of Italian (4,524
  of them glossed by a proper noun's senses alone), and 634 dictionary words have no English gloss, so
  the pack carries a lexical section, and the vocabulary estimate counts the same words as it-fr's.
- **The pack is 2,467,844 B**, 47.1 % of the 5 MiB budget: Italian's base, 1,551,534 B, byte for byte
  it-fr's, and it-en's layer, 916,550 B.

## Its sources

- **The English Wiktionary's Italian section** (`kaikki-Italian.jsonl`) — the file it-fr reads for
  Italian's forms, derived at an update from the English Wiktionary's dump (`../../SOURCES.md`, *The
  editions' dumps*) — read here for its senses. it-en pins its own fetch, published under its own
  release (`lingua-pack-sources-it-en-<snapshot>`); when it-fr's update brings it-en along, it-en is
  reduced from its own pin and nothing of it is published.
- **The Italian Wiktionary's English translations** (`kaikki-it-traductions-en.jsonl`): the English
  words its Italian entries list, derived from kaikki's dump of the whole edition — the dump it-fr
  derives its French translations from — and kept as an asset of it-en's own release. They gloss what
  the English Wiktionary leaves out, at most three words per part of speech.
- No table read backwards: the English Wiktionary's English entries' Italian translations would add
  24 rows among the 5,000 commonest, about 6 of them wrong (`etc` « cub », `cv` « hp »). No pivot, no
  machine translation: a word neither source glosses has no gloss.

## How the section is read

Through the English edition's pre-passes, as es-en and fr-en read theirs (`reduce_edition_en.py`: its
letters out, its senses read as meanings), then it-en's own (`../../reduce_italian_en.py`, D4), which
edits neither the English edition's rules nor the shared ones:

- **A meaning a pointer carries**, by a reviewed list of wordings: Italian's own shapes wherever they
  stand — a form for euphony (`ed` « and », `ad` « to, at, in »), the article's form (`gli` « the »),
  a pronoun's clitic case form (`mi` « me; (to) me ») —, and a contraction's, a plural's or a degree's
  only for a word whose lowercase entries hold nothing else (`del` « some, of the, from the », `della`
  « of the, from the »; not `colla`, whose noun is « glue »); the `di` family's contraction wherever it
  stands (`dei` « of the, from the; some », beside the plural of *dio*).
- **Acronyms off common words** (`ad` without `AD`'s « CEO »; `pil`, whose entry points at `PIL`, keeps
  « gross domestic product ») and **names off function words** (`io` without the moon, `meno` without
  the river Main); a name beside a common word's senses stays (`natale` « Christmas »).
- **No word glossed with a letter's name**: the Latin letters by the English edition's rules, a Greek
  letter's sense beside a common word's meaning (`mi` without « mu », `chi`, `coppa`, `ni`); a word that
  is only the letter keeps it (`eta`, `ro`, `psi`).
- **The page's notes out**: « (all senses) », « (in all senses) », « (all meanings) », « See
  Category:… », « Same as entry above ».

After the shared rules, a sense repeating an earlier one of the same word word for word is shown once
(`ex` « ex, former »); a name's sense written with a capital stays beside a common word's (`cancro`
« Cancer; cancer »).

## A sample of 100 glosses, marked by source

The reduction draws them from the glossed lemmas of the 20,000 commonest with `random.Random(20261024)`
(`measures.json`, which the update's report shows); « section » is the English Wiktionary's Italian
section, « it wikt » the Italian Wiktionary's translations. Long glosses are cut here.

| Rank | Lemma | Source | Gloss |
|---:|---|---|---|
| 47 | `degli` | section | of the, from the |
| 213 | `dare` | section | to give (to transfer the possession/holding of something to someone else); to yield, to bear, to produce, to … |
| 312 | `sociale` | section | social, societal; partnership, company, form, corporate; society, social issues (social issues); social activ… |
| 558 | `chiedere` | section | to ask (for); to ask, request; to ask [with di ‘about (someone)’] |
| 612 | `signore` | section | lord (ruler); Mister or Sir; master; Lord, God; used to address a lover: beloved; used to address a man: sir … |
| 725 | `santa` | section | Saint; saint; Saint (before a name of a saint or in place names, as Santa ) |
| 806 | `disposizione` | section | disposal; disposition, arrangement, placing; order, direction, instruction, regulation, provision; gift, (nat… |
| 946 | `fenomeno` | section | phenomenon |
| 956 | `specialmente` | section | especially, particularly, mainly |
| 1015 | `frattempo` | section | meantime |
| 1032 | `inferiore` | section | inferior |
| 1245 | `esistenza` | section | existence; life |
| 1285 | `pianeta` | section | planet; chasuble, vestment |
| 1290 | `sensazione` | section | sensation, feeling |
| 1359 | `biglietto` | section | ticket; card; note, slip; billet; note, banknote, bill; a surname |
| 1459 | `elemento` | section | element |
| 1466 | `ossia` | section | that is, to be precise (used to introduce a clarification or explanation); or rather |
| 1988 | `probabilità` | section | probability, likelihood, odds, chance |
| 2269 | `destro` | section | clever, able, dexterous; right, right-hand; right, just; favourable/favorable; right; right foot; members of … |
| 2409 | `home` | section | home (initial position of various computing objects) |
| 2757 | `fallo` | section | fault, error; phallus; any member of the Phallus taxonomic genus; foul; compound of fa', the second-person si… |
| 2799 | `vienna` | section | Vienna (the capital city of Austria); Vienna (a state of Austria) |
| 3018 | `visibilità` | section | visibility |
| 3062 | `sconosciuto` | section | unknown, unfamiliar, unidentified, unexplored, obscure; unknown person, stranger, random |
| 3237 | `cosiddetto` | section | so-called |
| 3338 | `accadere` | section | to happen |
| 3393 | `uovo` | section | egg |
| 4242 | `domestica` | section | maid, servant |
| 4327 | `impulso` | section | impulse, drive, urge, boost; impulse, pulse, also archaic for momentum |
| 4516 | `tranquillità` | section | calm, peace, stillness, tranquillity, calmness |
| 4778 | `studioso` | section | studious; scholar, scientist |
| 4832 | `ironico` | section | ironic, ironical |
| 5135 | `indifferenza` | section | indifference |
| 5189 | `baseball` | section | baseball |
| 5603 | `adeguatamente` | section | adequately; suitably |
| 5647 | `lavorativo` | section | work, working; tillable, arable |
| 5663 | `oliva` | section | olive (fruit); olive (color) |
| 5778 | `vacca` | section | cow; whore, slut |
| 6152 | `telecomunicazioni` | section | telecommunications |
| 6217 | `pecora` | section | sheep (animal); ewe; its meat (raw or cooked), its skin or its fur; tame, meek animal; sheep (shy, gullible p… |
| 6390 | `etiopia` | section | Ethiopia (a country in East Africa) |
| 6520 | `irregolarità` | section | irregularity |
| 6833 | `lirica` | section | lyric; lyric poetry; opera |
| 6854 | `reporter` | section | reporter (journalist) |
| 7461 | `diamine` | section | an expression of wonder, impatience or irritation |
| 7557 | `balletto` | section | ballet; ballet music; dance troupe, corps de ballet |
| 7691 | `ergo` | section | ergo |
| 7906 | `soda` | section | parts of certain plants high in mineral salts the ashes in particular of which were used in glassmaking; soda… |
| 8239 | `insicurezza` | section | insecurity; uncertainty |
| 8731 | `clamoroso` | section | resounding, sensational |
| 9066 | `purgatorio` | section | purgatory; Purgatory |
| 9380 | `abisso` | section | abyss, gulf; a surname |
| 9492 | `tosto` | section | at once, immediately; swift, rapid, hurried; hard, tough, firm, hard-boiled, hardheaded, resistant, persisten… |
| 9565 | `into` | section | inside |
| 9645 | `zii` | section | uncles; uncle and aunt or aunt and uncle; uncles and aunts |
| 9732 | `nuca` | section | nape, back of the neck |
| 9736 | `pensionato` | section | hostel; pensioner |
| 9744 | `processato` | section | processed |
| 9954 | `concessionario` | section | concessionary; agent; dealer |
| 10234 | `bingo` | section | bingo (game); bingo! |
| 10257 | `dogma` | section | dogma |
| 10464 | `movente` | section | motive (That which incites to action, especially a crime) |
| 10918 | `otranto` | section | a town in the province of Lecce, Apulia, Italy; a surname transferred from the place name |
| 10997 | `ambulante` | section | travelling/traveling, itinerant; mobile (library); peddler |
| 11038 | `duraturo` | section | lasting, abiding, enduring, durable |
| 11450 | `siri` | section | a surname originating as a patronymic |
| 11547 | `interezza` | section | entirety, wholeness, oneness |
| 11669 | `certosa` | section | Carthusian monastery, charterhouse |
| 12249 | `rimorchio` | section | tow, towage, towing; trailer (of a car) |
| 12642 | `posare` | section | to put or lay down; to direct, to turn; to calm down, to placate; to lay or rest; to pose; to rest |
| 13130 | `frascati` | section | Frascati (wine); a town in Rome, Italy |
| 13186 | `picchio` | section | woodpecker; rap, tap, knock; continuous, frequent beating or banging; rapping, tapping, knocking; a surname |
| 13255 | `attivismo` | section | activism |
| 13438 | `virile` | section | virile |
| 13804 | `urgentemente` | section | urgently |
| 14298 | `lao` | section | a river in Calabria |
| 14364 | `sigismondo` | section | a male given name, equivalent to English Sigismund |
| 14926 | `plutarco` | section | Plutarch |
| 15006 | `battente` | section | heavy (of rain); beating, palpitating (of a heart); shutter (of a window); sash (of a window); leaf (panel of… |
| 15016 | `brusca` | section | scrubbing brush; straw; scorch (desiccation of the apex of a leaf or shoot); stubble or dried-up twig; a flex… |
| 15390 | `vallone` | section | Walloon; Walloon (person); Walloon (language); deep valley; a surname |
| 15749 | `olmo` | section | elm (Ulmus); field elm (Ulmus minor); wood from those trees |
| 16026 | `sudcoreano` | section | South Korean |
| 16179 | `olivo` | section | olive tree, olive |
| 16440 | `scotta` | section | sheet; whey |
| 17593 | `soccombere` | section | to succumb, to give way |
| 17604 | `sterco` | section | dung |
| 17667 | `cassone` | section | cofferdam, caisson, pontoon; skip (for waste), dumpster; truck dumping / tipping body |
| 17905 | `confiscato` | section | confiscated |
| 18153 | `elettromagnetico` | section | electromagnetic |
| 18223 | `montella` | section | a town in the province of Avellino, Campania, Italy; a surname transferred from the place name |
| 18298 | `sonetto` | section | sonnet |
| 18386 | `concistoro` | section | consistory |
| 18504 | `pietanza` | section | piety, a feeling of compassion; dish, course (of a meal) |
| 18670 | `gastrica` | section | gastric |
| 18717 | `messinese` | section | of, from or relating to Messina; native or inhabitant of Messina |
| 18788 | `sapientemente` | section | learnedly |
| 19070 | `sunto` | section | summary, abstract, resumé epitome |
| 19294 | `razionalizzare` | section | to rationalize |
| 19683 | `discriminante` | section | decisive; discriminating; extenuating; extenuating circumstance |

## Known defects, for change 23

`refine-lingua-it-en-glosses`, after these tables: pointer wordings the English edition does not know
(« proscribed spelling of » in `che`, rank 3; « Old Italian form of »; « compound of … with … » in
`aprile`, `fallo`); a sense order the page's etymologies set (`auto` « auto (genre of dramatic
literature) » before the car; `testo`, `polizia`, `università`, `piano` opening on an old or rare
sense); names' and communes' runs after a common word's (1,121 rows of the 20,000 hold « a surname »);
bracketed construction and auxiliary notes (82 rows); « etc » without its period (31); the species
among the expressions; a contraction's or a plural's carried meaning beside a word's own senses
(`guai` « trouble », `mura` « wall », left unread by the narrowing above).

## How it changes

An update (`lingua-pack-update`, mode `update`, pair `it-en`) reads today's dumps and proposes the
folder again on its own branch; a change to it-en's rules or to the English edition's re-reduces it from
its pin (mode `reduce`), as does a change to Italian's studied tables, which it-fr's update brings
along. Every pull request rebuilds the pack from these tables and checks it against `pin.json`, and the
`reduce` job reduces it again from its pinned sources and holds it to its floor.
