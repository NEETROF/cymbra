# The it→en dictionary tables

The reduced tables Cymbra Lingua's Italian→English data pack is built from: Italian glossed in
English, for English speakers studying Italian — the first reader pair of Italian
(add-lingua-pack-it-en, change 22 of `docs/lingua/italian-programme.md`). They are committed so that
every build makes the same pack with no download, and so that a change to the dictionary is a pull
request whose diff shows it. Extension packages carry the pack since `enable-lingua-italian-for-english-speakers` (change 25) lists
it-en in `apps/lingua-extension/packs.json`, which it does because these tables stand at or above their
coverage floor (change 22) and were refined by change 23. Its glosses are refined by
change 23 (`refine-lingua-it-en-glosses`), so that a row reads an Italian word's meanings in the order
Italian uses them.

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

<!-- Measured on the tables reduced again from the pinned sources of the update of 2026-10-25 (run
40183685031) with change 23's rules (refine-lingua-it-en-glosses), after change 17
(refine-lingua-it-fr-glosses) moved Italian's studied tables. -->

- **English glosses for 33,199 lemmas** of Italian's 60,000 — 33,004 from the English Wiktionary's
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
  | top 20,000 | 75.3 % | 75.8 % | 73.7 % |
  | all 60,000 | — | 55.3 % | — |

  Of the 8,425 lemmas glossed among the 10,000 commonest, 0.5 % (45) come from the Italian Wiktionary.
- **Dictionary words are it-fr's**: 9,568 lemmas it-en glosses are no dictionary word of Italian, 4,490
  lemmas are glossed by a proper noun's senses alone, and 636 dictionary words have no English gloss, so
  the pack carries a lexical section, and the vocabulary estimate counts the same words as it-fr's.
- **The pack is 2,461,617 B**, 47.0 % of the 5 MiB budget: Italian's base, 1,551,196 B, byte for byte
  it-fr's, and it-en's layer, 910,661 B.
- **Refined by change 23** (*How the section is read*): 1,536 rows read otherwise — 132 of the 1,000
  commonest, 298 of the next 4,000, 583 of the next 15,000, 523 beyond —, 341 of them in their first
  sense; 19 expressions; no lemma gains or loses a gloss, and the coverage does not move.
- **Reduced again after change 17**, whose forms moved Italian's studied tables: 17 lemmas left the
  ranks — `vado`, `sto`, `sapete`, `signor`, `od` among them, now forms of their words — and 7 joined
  (`dati`, `lega`, `prova`, and four past the 59,970th); no other row moved.

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
- **UD Italian-MarkIT's training and development sections**, the files it-fr's pin records, at the same
  commit and sha256s (CC BY 4.0): read for the part of speech a function word's row opens on, and for
  nothing else (change 23's D1). Its test section and UD Italian-PUD are never read.
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

Then change 23's rules (`refine-lingua-it-en-glosses`, in the same module), reading a headword's entries
together — every case of it —, each measured alone on change 22's tables (rows changed by band of the
1,000 / 5,000 / 20,000 commonest and beyond):

- **The row's order**: a function word's row opens on the part of speech MarkIT reads it as — fr-en's
  rule, as it-fr reads it — (16 rows, all of the 1,000: `su` « on, upon… », `solo` « only, just »,
  `spesso` « often »); a row opening on a meaning the section marks archaic, obsolete, dated or
  historical opens on its first meaning that is not (173 rows: `polizia` « police; polity »,
  `università`, `ufficio`, `prodotto`); and a reviewed table, `FIRST_RUN`, opens 40 rows on the sense it
  names (`bene` « well », `gente` « people », `testo` « text (writing) », `piano` « floor, storey » with
  « plan » back, `parlamento` « parliament », `ce` « us », `parmigiano` « parmesan cheese », and, on an
  Italian specialist's review, `tanto` « so much, so many », `costa` « coast, shoreline », `mica` « not »,
  `tratta` « trafficking, trade » …; `come` held at « how »).
- **Pointers**: the wordings the English edition does not know are pointers (`che` without « proscribed
  spelling of ché », `aprile` « April », `itria` « otter »); a pointer to a word of one or two letters
  lends that word's meanings after the row's own, within the eight-sense bound (`de` → *di*, `el` « he;
  the », `eo` « I », `se` gaining *si*'s reflexive, `ne` *in*'s « in ») — 26 rows.
- **Carried meanings**: a meaning nested under a degree's wording (31 rows: `minimo` « minimum, (the)
  least… », `minore` « less; shorter… »), a degree's and a clipping's wherever they stand (`più` « more;
  anymore… », `auto` « car; bus; … »), a plural's beside another part of speech (`guai` « woe!;
  trouble »).
- **Names' notes**: a surname's note goes from a row whose word holds a common meaning (`musica`
  « music », `marzo` « March (the month) »; `garibaldi` and `mattarella`, names alone, keep theirs), a given name's
  from a row a common word opens (`delfino`; `alice` and `francesco`, which open on the name, keep it), a
  town's from a common word unless the row opens on it, the place is a city or a capital itself, or a
  town and province (`noto`, `russo`, `piombino`; `cervia`, `andria`, `fermo` keep theirs); the reviewed
  list `TOWNS_KEPT` keeps Massa, and, on the specialist's review, Sassuolo, Orvieto, Frascati, Cassino and
  Monopoli. No note goes from a row whose every lowercase meaning the section tags archaic, obsolete,
  dated, historical, rare, poetic, literary, regional or dialectal (`esposito` « a common surname;
  foundling », `sora`, `italo`).
- **The page's notes and typography**: `lo` « the; him; it, this or that thing », « etc » with its period
  (71 rows, 13 expressions), a list marker out, `mar` « sea (used in poetry…) ».
- **`CORRECTIONS`**, by name: `all'antica` « ancient, antique », without the Italian Wiktionary's jocular
  « anshent »; and, the owner's, `amica` « friend; girlfriend », `bambina` « girl, little girl; babe »,
  `compagna` « companion, partner; … », `spagnola` « Spaniard; … », the person's meaning their « female
  equivalent of » pointer does not lend; `tratta` « trafficking, trade; …; stretch, section (of a
  route) », on the specialist's review.

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
| 213 | `piano` | section | floor, storey (of a building); plan, tactic, stratagem, scheme, plot; plane; of any of the popes named Pius, … |
| 312 | `regione` | section | region, area, zone, district; region (Italian administrative unit and local government); region |
| 558 | `chiedere` | section | to ask (for); to ask, request; to ask [with di ‘about (someone)’] |
| 612 | `signore` | section | Mister or Sir; Lord, God; lord (ruler); master; used to address a lover: beloved; used to address a man: sir … |
| 725 | `provare` | section | to try, to try out, to test; to try, to attempt; to feel, to experience; to prove, to demonstrate, to show; t… |
| 806 | `discussione` | section | discussion; argument; quarrel |
| 946 | `europeo` | section | European |
| 956 | `signora` | section | lady; Mrs, Lady; a title of address for women, madam (ladies is used for the plural); madam (ladies is used f… |
| 1015 | `elettorale` | section | election, electoral; ballot, polling |
| 1032 | `inferiore` | section | inferior |
| 1245 | `esistenza` | section | existence; life |
| 1285 | `pianeta` | section | planet; chasuble, vestment |
| 1290 | `sensazione` | section | sensation, feeling |
| 1359 | `biglietto` | section | ticket; card; note, slip; billet; note, banknote, bill |
| 1459 | `ferma` | section | service, draft; ferme générale (a system of outsourced tax collection in France prior to the French Revolutio… |
| 1466 | `paga` | section | pay; payer; wages; reward, recompense; bounty |
| 1987 | `ricchezza` | section | richness, wealth |
| 2269 | `essenziale` | section | essential, fundamental, main, crucial, basic; essential; terse (of style etc.); the essential thing, the kern… |
| 2409 | `leggenda` | section | legend (myth, and explanatory inscription); old wives' tale |
| 2756 | `giornalismo` | section | journalism |
| 2799 | `apparire` | section | to appear; to seem, to look; to grow |
| 3019 | `alimentazione` | section | nutrition, nourishment, diet, feeding; feed, fuel, power, supply |
| 3062 | `sottolineare` | section | to underline, underscore; to emphasize, point out, underline, stress, remark |
| 3236 | `difficilmente` | section | hardly, with difficulty, unlikely (adjective) |
| 3337 | `ancona` | section | Ancona (a province of Marche, Italy); Ancona (a port city and comune, the capital of the province of Ancona a… |
| 3392 | `volante` | section | flying; loose (of a page); freelance; steering wheel (of a car); Italian police car |
| 4240 | `faro` | section | lighthouse; headlight, headlamp; Capo Peloro, Faro Point (the historical demarcation between the Two Sicilies… |
| 4329 | `lombardo` | section | Lombard |
| 4514 | `versato` | section | versed, practiced |
| 4777 | `terrorista` | section | terrorist, terroristic (relating to or engaging in terrorism); terrorist (one who uses violence to further po… |
| 4831 | `mattarella` | section | a surname |
| 5134 | `litigare` | section | to quarrel, to fight, to argue [with con ‘with someone’]; to fight [with per ‘over a legal case, an inheritan… |
| 5187 | `concorrente` | section | concurrent; competing; competitor; contestant |
| 5602 | `bonifica` | section | reclamation |
| 5647 | `meritato` | section | deserved, merited |
| 5662 | `pontefice` | section | pontiff |
| 5777 | `allontanare` | section | to move away, pull away, push away; to spread out, to scatter; to send away, take away; to avert, avoid; to d… |
| 6152 | `vecchiaia` | section | old age |
| 6215 | `rancore` | section | rancour, resentment, grudge |
| 6387 | `grosseto` | section | Grosseto (a town and province of Italy) |
| 6518 | `manipolazione` | section | manipulation; rigging (of an election etc.) |
| 6831 | `melissa` | section | lemon balm |
| 6850 | `scavare` | section | to dig; to excavate; to bore; to hollow out |
| 7458 | `fata` | section | fairy (mythical creature) |
| 7554 | `cavità` | section | hollow, cavity; cavity |
| 7687 | `fida` | section | a tax on cultivated land; safe-conduct |
| 7902 | `tagliando` | section | coupon, voucher; service |
| 8241 | `naufragio` | section | a shipwreck, a wreck; a wreck, a failure |
| 8728 | `contadina` | section | the name of a particular dance, danced in a circle by only men; the music that accompanies that dance |
| 9064 | `rubinetto` | section | tap (UK), faucet (US); any similar device used to regulate the flow of a fluid |
| 9375 | `alterazione` | section | alteration, modification, change (especially for the worse); degradation |
| 9488 | `affettuoso` | section | affectionate, fond, loving |
| 9565 | `mancia` | section | tip (in a restaurant, etc.) |
| 9640 | `adottando` | section | who is to be adopted |
| 9728 | `potenziare` | section | to strengthen, develop, improve, enhance, empower; to potentiate |
| 9732 | `premiare` | section | to give or award a prize (to); to reward |
| 9739 | `richiedente` | section | applicant; suitor; wooer |
| 9949 | `distrarre` | section | to pull apart; to turn away, to divert (e.g. one's gaze); to distract (a person, one's attention, one's mind,… |
| 10228 | `castelnuovo` | section | a surname |
| 10262 | `governante` | section | governing, ruling; governess; government leader; housekeeper |
| 10458 | `osseo` | section | bone, bony |
| 10913 | `pipa` | section | pipe; a wank |
| 10991 | `armonica` | section | harmonica, mouth organ; armonica, glass harmonica |
| 11035 | `fruizione` | section | fruition, use, enjoyment; account |
| 11445 | `stucco` | section | stucco; putty; bas-relief |
| 11545 | `lucifero` | section | Lucifer; bearing or giving off light; name for the planet Venus as the morningstar, daystar |
| 11664 | `conegliano` | section | Conegliano (a town in Veneto, Italy) |
| 12244 | `sassonia` | section | Saxony (a state of modern Germany, located in the east, far from historical Saxon lands); Saxony (a historica… |
| 12639 | `ruotare` | section | to rotate, to revolve; to circle (of a bird or airplane) |
| 13126 | `guarnigione` | section | garrison (troops stationed at a permanent military post); the act of decorating or adorning; garrison (perman… |
| 13181 | `punteggiatura` | section | punctuation; dotting, speckling, dots |
| 13252 | `boicottaggio` | section | boycott |
| 13437 | `amalia` | section | a female given name |
| 13800 | `zambia` | section | Zambia (a country in Southern Africa) |
| 14295 | `messinscena` | section | mise en scène; dramatization, stunt |
| 14363 | `titanio` | section | of or pertaining to the Titans, Titanic; titanium |
| 14926 | `riluttanza` | section | reluctance, unwillingness |
| 15007 | `cameraman` | section | cameraman |
| 15012 | `centrifuga` | section | centrifuge; spin dryer |
| 15391 | `alpe` | section | alp |
| 15744 | `parvenza` | section | semblance; opinion |
| 16023 | `trentenne` | section | thirty-year-old |
| 16176 | `perforazione` | section | perforation; drilling |
| 16445 | `trambusto` | section | turmoil, fuss, ado |
| 17588 | `sterco` | section | dung |
| 17600 | `trepidazione` | section | anxiety, trepidation |
| 17663 | `connettore` | section | connector |
| 17900 | `depurazione` | section | purification |
| 18150 | `fiuto` | section | smell (the sense of smell); nose (intuition) |
| 18221 | `oligarchia` | section | oligarchy |
| 18303 | `tonico` | section | tonic; toner (to stimulate the skin) |
| 18382 | `debora` | section | Deborah (biblical character); a female given name |
| 18498 | `purificare` | section | to purify |
| 18674 | `intransigente` | section | intransigent, strict |
| 18718 | `orio` | section | a surname transferred from the place name |
| 18783 | `sgombrare` | section | to vacate or evacuate; to clear the way; to move out (of a residence) |
| 19066 | `trentuno` | section | thirty-one |
| 19293 | `schienale` | section | back, backrest (of a seat); saddle (of meat) |
| 19683 | `frattanto` | section | meanwhile; in the meantime |

## Known defects

Fixed by change 23 (`refine-lingua-it-en-glosses`): the pointer wordings the English edition does not
know (`che`, `de`, `el`, `aprile`), the order the page's entries and etymologies set (`auto`, `testo`,
`polizia`, `università`, `piano`), names' and towns' notes after a common word's senses, « etc »
without its period, `lo`'s usage note, a plural's carried meaning beside another part of speech
(`guai`).

Left, each with its reason (the change's design, D9):

- **Bracketed constructions and auxiliary notes** (`chiedere` « to ask [with di ‘about (someone)’] »,
  154 ranked rows): they say which preposition or auxiliary a verb's meaning takes, which the card says
  nowhere else.
- **`USA`** opens on the supercontinent, by the shared acronym table's order: reordering it edits
  `reduce_italian_acronyms.py`, which it-fr imports too (the programme's backlog).
- **A description heading nested senses** (`grande` « of greater physical dimensions or numerosity »),
  most of them a meaning (`fare` « to make »).
- **Offensive senses unlabelled** (`negro`): the card shows no label.
- **Names opening a row** (`svizzera` « Switzerland; hamburger », `francesco`, `alice`), accepted until a
  card reads the token's capital.
- **A « female equivalent of » beside another sense** lends nothing (106 ranked rows): lending the
  masculine's senses would read `signora` « lord (ruler) »; the owner added the person's meaning to four
  by name.
- **`the`**, the English article in Italian text, unglossed; **`mei`** and **`meo`** read *meare*'s « to
  filter through ».
- **Species among the expressions**: a rule cannot tell a rare sense from the plant or animal a word
  names.

The owner's (2026-10-25): the 40 rows of `FIRST_RUN`; the six towns of `TOWNS_KEPT`, and the other towns a
common word precedes going (`piombino`, `faenza`; `cassino` kept by the list on the specialist's review,
`sora` by the rule for marked rows); the corrections of `amica`, `bambina`, `compagna`, `spagnola` and
`tratta`.

**Read from the section's tags, the rule for marked rows keeps a name beside a few common words the
page tags oddly**: `ferro` « iron » is tagged rare and keeps « a surname », `eco` « echo » is tagged
literary and keeps « a surname », `babbo` « dad » is tagged regional and keeps « a surname »; `lucio`
(tagged Tuscany, colloquial) and `caruso` (« baby » untagged) lose theirs.

## How it changes

An update (`lingua-pack-update`, mode `update`, pair `it-en`) reads today's dumps and proposes the
folder again on its own branch; a change to it-en's rules or to the English edition's re-reduces it from
its pin (mode `reduce`), as does a change to Italian's studied tables, which it-fr's update brings
along. Every pull request rebuilds the pack from these tables and checks it against `pin.json`, and the
`reduce` job reduces it again from its pinned sources and holds it to its floor.
